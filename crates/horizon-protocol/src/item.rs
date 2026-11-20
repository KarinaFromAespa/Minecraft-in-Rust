//! Item stacks on the wire (ItemStack.OPTIONAL_STREAM_CODEC): a count, the
//! item's registry id and a data component patch, plus the hashed form the
//! client sends back in container clicks (HashedStack).
use crate::{
    Result,
    adapter::ids,
    buf::{Codec, Reader, Writer, pack_block_pos, unpack_block_pos},
};
use horizon_world::nbt::Tag;
use std::sync::Arc;

#[doc(hidden)]
pub mod custom;
mod hash;
mod skip;
mod trim;
pub(crate) mod v1_20_3;
pub(crate) mod v1_20_5;
mod v1_21_1;
mod v1_21_4;
mod v1_21_5;
pub use trim::{TrimHolder, TrimMaterial, TrimPattern};

pub use hash::{Names, NoNames};

/// How deep values may sit inside one another while an item is decoded: stacks in a stack's components (a
/// bundle in a bundle, a use_remainder's use_remainder) and a mob effect's hidden effects. Those codecs
/// call each other with no limit of their own. In vanilla a payload nested past the stack ends in a
/// StackOverflowError, which the decoder turns into a disconnect; in Rust a stack overflow aborts the
/// process, so a single packet from a peer would end the game, or a whole server and all its players.
/// Real items stay far below it (a survival bundle holds bundles about 16 deep at most).
const MAX_NESTING: u32 = 64;

thread_local! {
    static NESTING: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// One level of [`MAX_NESTING`] while it lives. Counted per thread rather than in the [`Reader`], because a
/// delimited or rewritten component is read through a reader of its own.
struct Nested;
impl Nested {
    fn enter() -> Result<Self> {
        NESTING.with(|n| {
            let depth = n.get();
            if depth >= MAX_NESTING {
                return Err(crate::Error::new("item data nested too deeply"));
            }
            n.set(depth + 1);
            Ok(Nested)
        })
    }
}
impl Drop for Nested {
    fn drop(&mut self) {
        NESTING.with(|n| n.set(n.get() - 1));
    }
}

/// minecraft:data_component_type registry ids (1.21.4).
pub mod component {
    /// DataComponents' registry order at the pinned 1.20.6 (56 entries).
    /// These are PIN ids: they shift whenever the registry does, so a port
    /// across a registry boundary re-pins them from the pin's reports (the
    /// 1.21 boundary added jukebox_playable at 42, shifting every later id
    /// up by one, which is why 1.20.6 is the first pin where these differ
    /// from 1.21.1's).
    pub const CUSTOM_DATA: i32 = 0;
    pub const MAX_STACK_SIZE: i32 = 1;
    pub const MAX_DAMAGE: i32 = 2;
    pub const DAMAGE: i32 = 3;
    pub const UNBREAKABLE: i32 = 4;
    pub const CUSTOM_NAME: i32 = 5;
    pub const ITEM_NAME: i32 = 6;
    /// Not a pinned component: 1.21.1 has no `item_model` (a 1.21.2
    /// addition). The negative value is an internal marker only, so the
    /// arms that name it can never match a canonical id here.
    pub const ITEM_MODEL: i32 = -1004;
    pub const LORE: i32 = 7;
    pub const RARITY: i32 = 8;
    pub const ENCHANTMENTS: i32 = 9;
    /// `AdventureModePredicate`: the blocks an adventure player may place on.
    pub const CAN_PLACE_ON: i32 = 10;
    /// `AdventureModePredicate`: the blocks an adventure player may break.
    pub const CAN_BREAK: i32 = 11;
    /// `ItemAttributeModifiers`: a list of entries, each with a display from 1.21.6 on.
    pub const ATTRIBUTE_MODIFIERS: i32 = 12;
    pub const CUSTOM_MODEL_DATA: i32 = 13;
    /// 1.21.1's two tooltip booleans, which the pin codec folds into the
    /// synthesized `TOOLTIP_DISPLAY` below (1.21.5 folded them into one
    /// component; the pin predates it).
    pub const HIDE_ADDITIONAL_TOOLTIP: i32 = 14;
    pub const HIDE_TOOLTIP: i32 = 15;
    pub const REPAIR_COST: i32 = 16;
    pub const CREATIVE_SLOT_LOCK: i32 = 17;
    pub const ENCHANTMENT_GLINT_OVERRIDE: i32 = 18;
    /// `FoodProperties.STREAM_CODEC`: the full pre-1.21.2 form (nutrition,
    /// saturation, canAlwaysEat, eatSeconds, an optional remainder, the
    /// possible effects) — 1.21.2 split the tail into consumable and
    /// use_remainder, which this pin predates.
    pub const FOOD: i32 = 20;
    /// `Unit.STREAM_CODEC`: the pre-1.21.2 fire flag (1.21.2 replaced it
    /// with damage_resistant, a holder set).
    pub const FIRE_RESISTANT: i32 = 21;
    /// `Tool.STREAM_CODEC`: rules, defaultMiningSpeed, damagePerBlock
    /// (1.21.5 added canDestroyBlocksInCreative after the damage).
    pub const TOOL: i32 = 22;
    /// Not pinned components: 1.20.6 has no `consumable`, `equippable`,
    /// `glider`, `death_protection`, `use_remainder` or `damage_resistant`
    /// (all 1.21.2 additions). Negative internal markers, as for ITEM_MODEL.
    pub const CONSUMABLE: i32 = -1005;
    pub const EQUIPPABLE: i32 = -1006;
    pub const GLIDER: i32 = -1007;
    /// `DeathProtection`: the `death_effects` a stack runs when it saves its holder.
    pub const DEATH_PROTECTION: i32 = -1008;
    pub const USE_REMAINDER: i32 = -1009;
    pub const DAMAGE_RESISTANT: i32 = -1010;
    pub const STORED_ENCHANTMENTS: i32 = 23;
    pub const PROFILE: i32 = 46;
    pub const BANNER_PATTERNS: i32 = 48;
    pub const BASE_COLOR: i32 = 49;
    pub const DYED_COLOR: i32 = 24;
    pub const MAP_ID: i32 = 26;
    pub const MAP_POST_PROCESSING: i32 = 28;
    pub const CHARGED_PROJECTILES: i32 = 29;
    pub const BUNDLE_CONTENTS: i32 = 30;
    pub const POTION_CONTENTS: i32 = 31;
    pub const SUSPICIOUS_STEW_EFFECTS: i32 = 32;
    pub const WRITABLE_BOOK_CONTENT: i32 = 33;
    pub const WRITTEN_BOOK_CONTENT: i32 = 34;
    pub const TRIM: i32 = 35;
    pub const INSTRUMENT: i32 = 40;
    pub const LODESTONE_TRACKER: i32 = 43;
    pub const FIREWORK_EXPLOSION: i32 = 44;
    pub const FIREWORKS: i32 = 45;
    pub const BLOCK_ENTITY_DATA: i32 = 39;
    pub const OMINOUS_BOTTLE_AMPLIFIER: i32 = 41;
    pub const POT_DECORATIONS: i32 = 50;
    /// Not a pinned component: 1.20.6 has no `jukebox_playable` (a 1.21
    /// addition). The negative value is an internal marker only, so the
    /// arms that name it can never match a canonical id here.
    pub const JUKEBOX_PLAYABLE: i32 = -1011;
    pub const CONTAINER: i32 = 51;
    /// Not a pinned component: 1.21.4 has no `tooltip_display` (a 1.21.5
    /// addition). The negative value is an internal marker only, so the
    /// synthesized form the pin codec builds from the two hide booleans
    /// can never match a canonical id here.
    pub const TOOLTIP_DISPLAY: i32 = -1003;
    /// Not a pinned component: 1.21.4 has no `sulfur_cube_content` (a 26.3
    /// addition). The negative value is an internal marker only, so the
    /// 26.2-facing arms that name it can never match a canonical id here.
    pub const SULFUR_CUBE_CONTENT: i32 = -1000;
    /// Not a pinned component: 1.21.4 has no `painting/variant` (a 1.21.5
    /// addition). Kept for the client's internal painting stacks, which
    /// carry the variant as a component; the value is out of the pin's
    /// registry range, so it can never match a canonical id here.
    pub const PAINTING_VARIANT: i32 = 89;
    /// `SignText.STREAM_CODEC`: a sign item's text, front and back. Also not
    /// pinned components (26.3 additions); internal markers, as above.
    pub const SIGN_TEXT_FRONT: i32 = -1001;
    pub const SIGN_TEXT_BACK: i32 = -1002;
}

/// A written book's content (`WrittenBookContent.STREAM_CODEC`). The raw title
/// and pages are what a book shows; the filtered variants (`Filterable.filtered`,
/// which a server sends when chat filtering is on) are kept only so that the
/// component hashes as the server holds it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WrittenBook {
    pub title: String,
    pub title_filtered: Option<String>,
    pub author: String,
    pub generation: i32,
    /// Each page's text component.
    pub pages: Vec<Tag>,
    /// Each page's filtered variant: empty (none filtered) or one per page.
    pub pages_filtered: Vec<Option<Tag>>,
    pub resolved: bool,
}

/// The value of one patched data component. Components whose stream codec
/// Horizon does not implement make the whole stack (and packet) undecodable.
#[derive(Clone, Debug, PartialEq)]
pub enum Component {
    Int(i32),
    Unit,
    Bool(bool),
    Text(Tag),
    Nbt(Tag),
    Ident(String),
    Lore(Vec<Tag>),
    Enchantments(Vec<(i32, i32)>),
    AttackRange([f32; 6]),
    /// `PotionContents`: the potion's `minecraft:potion` id and the custom
    /// colour; the custom effects and name stay their encoded bytes.
    Potion {
        potion: Option<i32>,
        color: Option<i32>,
        rest: Vec<u8>,
    },
    Profile(Profile),
    CustomModelData {
        floats: Vec<f32>,
        flags: Vec<bool>,
        strings: Vec<String>,
        colors: Vec<i32>,
    },
    TooltipDisplay {
        hide: bool,
        hidden: Vec<i32>,
    },
    /// `WritableBookContent`: the pages (raw text) and their filtered variants
    /// (empty, or one per page).
    WritableBook {
        pages: Vec<String>,
        filtered: Vec<Option<String>>,
    },
    WrittenBook(WrittenBook),
    /// `ChargedProjectiles`: a loaded crossbow's ammunition, as templates.
    Stacks(Vec<Stack>),
    /// `Fireworks`: flight duration and its explosions.
    Fireworks {
        flight: i32,
        explosions: Vec<FireworkExplosion>,
    },
    /// `FireworkExplosion` (a firework star).
    FireworkExplosion(FireworkExplosion),
    /// `LodestoneTracker`: the target's dimension and block, and whether it
    /// follows the lodestone.
    LodestoneTracker {
        target: Option<(String, [i32; 3])>,
        tracked: bool,
    },
    /// `ArmorTrim` naming its material and pattern by registry id (`trim_material`,
    /// `trim_pattern`); direct holders use [`Component::TrimDirect`].
    Trim {
        material: i32,
        pattern: i32,
    },
    /// ArmorTrim with a direct holder. Named-only trims keep `Trim` unchanged.
    TrimDirect {
        material: TrimHolder<TrimMaterial>,
        pattern: TrimHolder<TrimPattern>,
    },
    /// `SuspiciousStewEffects`: each entry's `minecraft:mob_effect` id and
    /// duration in ticks.
    StewEffects(Vec<(i32, i32)>),
    /// `InstrumentComponent` naming its instrument by `minecraft:instrument`
    /// registry id; a direct instrument stays [`Component::Raw`].
    Instrument(i32),
    /// `BannerPatternLayers`: each layer's `banner_pattern` registry id and
    /// its dye id (`DyeColor.getId`). A layer with a direct pattern has no
    /// registry name: that value stays [`Component::Raw`].
    BannerPatterns(Vec<(i32, i32)>),
    /// `painting/variant` naming its `painting_variant` by registry id; a
    /// direct variant stays [`Component::Raw`].
    PaintingVariant(i32),
    /// A component Horizon reads past without interpreting it: its encoded
    /// bytes, written back as they came.
    Raw(Vec<u8>),
}

/// `FireworkExplosion.STREAM_CODEC`: shape id, colors, fade colors, trail
/// and twinkle (FireworkExplosion.Shape's BY_ID order).
#[derive(Clone, Debug, PartialEq)]
pub struct FireworkExplosion {
    pub shape: i32,
    pub colors: Vec<i32>,
    pub fade: Vec<i32>,
    pub trail: bool,
    pub twinkle: bool,
}
impl FireworkExplosion {
    fn read(r: &mut Reader) -> Result<Self> {
        let ints = |r: &mut Reader| -> Result<Vec<i32>> {
            let n = r.count(1024)?;
            (0..n).map(|_| r.i32()).collect()
        };
        Ok(Self {
            shape: r.varint()?,
            colors: ints(r)?,
            fade: ints(r)?,
            trail: r.bool()?,
            twinkle: r.bool()?,
        })
    }
    fn write(&self, w: &mut Writer) {
        w.varint(self.shape);
        w.varint(self.colors.len() as i32);
        self.colors.iter().for_each(|c| w.i32(*c));
        w.varint(self.fade.len() as i32);
        self.fade.iter().for_each(|c| w.i32(*c));
        w.bool(self.trail);
        w.bool(self.twinkle);
    }
}

/// `Filterable.streamCodec`: the raw value, then an optional filtered one.
fn filterable<T>(
    r: &mut Reader,
    value: impl Fn(&mut Reader) -> Result<T>,
) -> Result<(T, Option<T>)> {
    let raw = value(r)?;
    let filtered = if r.bool()? { Some(value(r)?) } else { None };
    Ok((raw, filtered))
}
/// The filtered variants of a list of pages, or none when no page has one.
fn filtered_variants<T>(pages: &mut Vec<(T, Option<T>)>) -> (Vec<T>, Vec<Option<T>>) {
    let any = pages.iter().any(|(_, f)| f.is_some());
    let (raw, filtered): (Vec<_>, Vec<_>) = pages.drain(..).unzip();
    (raw, if any { filtered } else { Vec::new() })
}

/// `ResolvableProfile`: an optional name and UUID and the profile's property
/// list, followed by `PlayerSkin.Patch`'s four optional textures.
#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub name: Option<String>,
    pub id: Option<u128>,
    pub properties: Vec<(String, String, Option<String>)>,
    pub skin: SkinPatch,
}
/// `PlayerSkin.Patch`: the optional body, cape and elytra texture ids and the
/// model type (`true` for slim).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkinPatch {
    pub body: Option<String>,
    pub cape: Option<String>,
    pub elytra: Option<String>,
    pub slim: Option<bool>,
}
impl SkinPatch {
    fn read(r: &mut Reader) -> Result<Self> {
        let texture = |r: &mut Reader| -> Result<Option<String>> {
            if r.bool()? {
                Ok(Some(r.string(32767)?))
            } else {
                Ok(None)
            }
        };
        let (body, cape, elytra) = (texture(r)?, texture(r)?, texture(r)?);
        let slim = if r.bool()? { Some(r.bool()?) } else { None };
        Ok(Self {
            body,
            cape,
            elytra,
            slim,
        })
    }
    fn write(&self, w: &mut Writer) {
        for texture in [&self.body, &self.cape, &self.elytra] {
            w.bool(texture.is_some());
            if let Some(texture) = texture {
                w.string(texture);
            }
        }
        w.bool(self.slim.is_some());
        if let Some(slim) = self.slim {
            w.bool(slim);
        }
    }
}
impl Component {
    pub(crate) fn read(kind: i32, r: &mut Reader) -> Result<Self> {
        if kind == component::CONTAINER && ids::quirks().count_first_templates {
            return Ok(Self::Raw(read_legacy_container(r)?));
        }
        if kind == component::PROFILE && ids::quirks().optional_profile_fields {
            return read_optional_profile(r).map(Self::Profile);
        }
        // Before 1.21 attribute modifiers carried a UUID and a name, not an
        // identifier: the 1.21.1 step that converted them is inert at this
        // pin, so the conversion happens here.
        if kind == component::ATTRIBUTE_MODIFIERS && v1_20_5::active() {
            return Ok(Self::Raw(v1_20_5::read_attributes(r)?));
        }
        if let Some(bytes) = v1_21_5::read(kind, r)? {
            return Ok(Self::Raw(bytes));
        }
        use component::*;
        Ok(match kind {
            MAX_STACK_SIZE
            | MAX_DAMAGE
            | DAMAGE
            | RARITY
            | REPAIR_COST
            | MAP_ID
            | MAP_POST_PROCESSING
            | OMINOUS_BOTTLE_AMPLIFIER => Self::Int(r.varint()?),
            WRITABLE_BOOK_CONTENT => {
                let n = r.count(100)?;
                let mut pages = (0..n)
                    .map(|_| filterable(r, |r| r.string(1024)))
                    .collect::<Result<Vec<_>>>()?;
                let (pages, filtered) = filtered_variants(&mut pages);
                Self::WritableBook { pages, filtered }
            }
            WRITTEN_BOOK_CONTENT => {
                let (title, title_filtered) = filterable(r, |r| r.string(32))?;
                let author = r.string(32767)?;
                let generation = r.varint()?;
                let n = r.count(r.remaining())?;
                let mut pages = (0..n)
                    .map(|_| filterable(r, |r| r.nbt()))
                    .collect::<Result<Vec<_>>>()?;
                let (pages, pages_filtered) = filtered_variants(&mut pages);
                Self::WrittenBook(WrittenBook {
                    title,
                    title_filtered,
                    author,
                    generation,
                    pages,
                    pages_filtered,
                    resolved: r.bool()?,
                })
            }
            CHARGED_PROJECTILES | BUNDLE_CONTENTS => {
                let n = r.count(1024)?;
                Self::Stacks((0..n).map(|_| Template::read(r)).collect::<Result<_>>()?)
            }
            DYED_COLOR => Self::Int(r.i32()?),
            UNBREAKABLE
            | CREATIVE_SLOT_LOCK
            | HIDE_TOOLTIP
            | HIDE_ADDITIONAL_TOOLTIP
            | FIRE_RESISTANT => Self::Unit,
            ENCHANTMENT_GLINT_OVERRIDE => Self::Bool(r.bool()?),
            CUSTOM_NAME | ITEM_NAME => Self::Text(r.nbt()?),
            CUSTOM_DATA => Self::Nbt(r.nbt()?),
            ITEM_MODEL => Self::Ident(r.string(32767)?),
            LORE => {
                let n = r.count(256)?;
                Self::Lore((0..n).map(|_| r.nbt()).collect::<Result<_>>()?)
            }
            // SuspiciousStewEffects.Entry.STREAM_CODEC: MobEffect.STREAM_CODEC
            // (the registry id), then the duration, both varints.
            SUSPICIOUS_STEW_EFFECTS => {
                let n = r.count(r.remaining())?;
                Self::StewEffects(
                    (0..n)
                        .map(|_| Ok((r.varint()?, r.varint()?)))
                        .collect::<Result<_>>()?,
                )
            }
            ENCHANTMENTS | STORED_ENCHANTMENTS => {
                let n = r.count(r.remaining())?;
                Self::Enchantments(
                    (0..n)
                        .map(|_| Ok((r.varint()?, r.varint()?)))
                        .collect::<Result<_>>()?,
                )
            }
            POTION_CONTENTS => {
                let potion = if r.bool()? { Some(r.varint()?) } else { None };
                let color = if r.bool()? { Some(r.i32()?) } else { None };
                let start = r.position();
                // The custom name arrived in 1.21.2; the pin's tail is the
                // effects alone. The stored tail always carries the name
                // field (a false bool when the wire had none), so the write
                // side can strip it for a pre-1.21.2 target.
                let rest = if ids::quirks().components_before_1_21_2 {
                    skip::potion_effects(r)?;
                    let mut rest = r.since(start).to_vec();
                    rest.push(0);
                    rest
                } else {
                    skip::potion_effects_and_name(r)?;
                    r.since(start).to_vec()
                };
                // No custom effects and no name: an empty tail.
                let rest = if rest == [0, 0] { Vec::new() } else { rest };
                Self::Potion {
                    potion,
                    color,
                    rest,
                }
            }
            PROFILE => {
                fn properties(r: &mut Reader) -> Result<Vec<(String, String, Option<String>)>> {
                    let n = r.count(1024)?;
                    let mut out = Vec::with_capacity(n);
                    for _ in 0..n {
                        let name = r.string(32767)?;
                        let value = r.string(32767)?;
                        let signature = if r.bool()? {
                            Some(r.string(32767)?)
                        } else {
                            None
                        };
                        out.push((name, value, signature));
                    }
                    Ok(out)
                }
                // Either<GameProfile, Partial>: the partial form carries
                // optional fields, the resolved one a UUID, a name and the
                // properties.
                let (name, id, properties) = if r.u8()? == 0 {
                    let name = if r.bool()? { Some(r.string(64)?) } else { None };
                    let id = if r.bool()? { Some(r.uuid()?) } else { None };
                    (name, id, properties(r)?)
                } else {
                    let id = Some(r.uuid()?);
                    let name = Some(r.string(64)?);
                    (name, id, properties(r)?)
                };
                // PlayerSkin.Patch: three texture ids and the model type.
                Self::Profile(Profile {
                    name,
                    id,
                    properties,
                    skin: SkinPatch::read(r)?,
                })
            }
            CUSTOM_MODEL_DATA => {
                if ids::quirks().integer_custom_model_data {
                    return Ok(Self::CustomModelData {
                        floats: vec![r.varint()? as f32],
                        flags: Vec::new(),
                        strings: Vec::new(),
                        colors: Vec::new(),
                    });
                }
                let n = r.count(r.remaining())?;
                let floats = (0..n).map(|_| r.f32()).collect::<Result<_>>()?;
                let n = r.count(r.remaining())?;
                let flags = (0..n).map(|_| r.bool()).collect::<Result<_>>()?;
                let n = r.count(r.remaining())?;
                let strings = (0..n).map(|_| r.string(32767)).collect::<Result<_>>()?;
                let n = r.count(r.remaining())?;
                let colors = (0..n).map(|_| r.i32()).collect::<Result<_>>()?;
                Self::CustomModelData {
                    floats,
                    flags,
                    strings,
                    colors,
                }
            }
            FIREWORKS => {
                let flight = r.varint()?;
                let n = r.count(256)?;
                Self::Fireworks {
                    flight,
                    explosions: (0..n)
                        .map(|_| FireworkExplosion::read(r))
                        .collect::<Result<_>>()?,
                }
            }
            FIREWORK_EXPLOSION => Self::FireworkExplosion(FireworkExplosion::read(r)?),
            LODESTONE_TRACKER => {
                let target = if r.bool()? {
                    Some((r.string(32767)?, unpack_block_pos(r.i64()?)))
                } else {
                    None
                };
                Self::LodestoneTracker {
                    target,
                    tracked: r.bool()?,
                }
            }
            TRIM => {
                let material = TrimHolder::read(r, TrimMaterial::read)?;
                let pattern = TrimHolder::read(r, TrimPattern::read)?;
                match (material, pattern) {
                    (TrimHolder::Id(material), TrimHolder::Id(pattern)) => {
                        Self::Trim { material, pattern }
                    }
                    (material, pattern) => Self::TrimDirect { material, pattern },
                }
            }
            // InstrumentComponent.STREAM_CODEC: a holder, a reference's id plus one.
            // A direct instrument has no registry name: read past.
            INSTRUMENT => {
                let start = r.position();
                skip::component(INSTRUMENT, r)?;
                let bytes = r.since(start);
                let mut ids = Reader::new(bytes);
                match ids.varint() {
                    Ok(id @ 1..) if ids.remaining() == 0 => Self::Instrument(id - 1),
                    _ => Self::Raw(bytes.to_vec()),
                }
            }
            // Holder<PaintingVariant>: a reference's id plus one, or 0 and
            // the variant itself (no registry name: read past).
            PAINTING_VARIANT => {
                let start = r.position();
                skip::component(PAINTING_VARIANT, r)?;
                let bytes = r.since(start);
                let mut ids = Reader::new(bytes);
                match ids.varint() {
                    Ok(id @ 1..) if ids.remaining() == 0 => Self::PaintingVariant(id - 1),
                    _ => Self::Raw(bytes.to_vec()),
                }
            }
            // DyeColor.STREAM_CODEC: its id.
            BASE_COLOR => Self::Int(r.varint()?),
            // BannerPatternLayers.STREAM_CODEC: a list of a pattern holder (a
            // reference's id plus one, or 0 and the pattern itself) and a dye.
            BANNER_PATTERNS => {
                let start = r.position();
                skip::component(BANNER_PATTERNS, r)?;
                let bytes = r.since(start);
                match Self::banner_layers(bytes) {
                    Some(layers) => Self::BannerPatterns(layers),
                    None => Self::Raw(bytes.to_vec()),
                }
            }
            other if custom::contains(other) => Self::Nbt(r.nbt()?),
            other => {
                let start = r.position();
                skip::component(other, r)?;
                Self::Raw(r.since(start).to_vec())
            }
        })
    }
    /// The layers of encoded `BannerPatternLayers` whose patterns are all
    /// registry references (`None` when one is direct).
    fn banner_layers(bytes: &[u8]) -> Option<Vec<(i32, i32)>> {
        let mut r = Reader::new(bytes);
        let n = r.count(r.remaining() / 2).ok()?;
        let mut layers = Vec::with_capacity(n);
        for _ in 0..n {
            let pattern = r.varint().ok()?;
            let color = r.varint().ok()?;
            if pattern < 1 {
                return None;
            }
            layers.push((pattern - 1, color));
        }
        (r.remaining() == 0).then_some(layers)
    }
    pub(crate) fn write(&self, kind: i32, w: &mut Writer) {
        match self {
            Self::Raw(bytes) => {
                let legacy = if kind == component::CONTAINER
                    && ids::quirks_written().count_first_templates
                {
                    write_legacy_container(bytes).ok()
                } else if kind == component::POT_DECORATIONS
                    && ids::quirks_written().pot_decorations_are_items
                {
                    write_pot_decorations(bytes).ok()
                } else if kind == component::DAMAGE_RESISTANT
                    && ids::quirks_written().either_holders_and_tag_resistance
                {
                    write_damage_resistant(bytes).ok()
                } else if kind == component::TOOL
                    && !ids::quirks_written().components_before_1_21_5
                {
                    // A 1.21.5+ target takes the canDestroyBlocksInCreative
                    // bool the pin lacks (true: the vanilla default).
                    let mut bytes = bytes.to_vec();
                    bytes.push(1);
                    Some(bytes)
                } else if matches!(kind, component::CAN_PLACE_ON | component::CAN_BREAK)
                    && !ids::quirks_written().components_before_1_21_5
                {
                    append_predicate_matchers(bytes)
                } else if kind == component::FOOD
                    && !ids::quirks_written().components_before_1_21_2
                {
                    truncate_food(bytes)
                } else if kind == component::FOOD
                    && ids::quirks_written().components_before_1_21
                {
                    strip_food_remainder(bytes)
                } else if kind == component::ATTRIBUTE_MODIFIERS
                    && ids::quirks_written().components_before_1_21
                {
                    v1_20_5::write_attributes(bytes).ok()
                } else if kind == component::INSTRUMENT
                    && !ids::quirks_written().components_before_1_21_2
                {
                    expand_instrument(bytes)
                } else {
                    v1_21_5::write(kind, bytes)
                };
                // Before 26.1 these components were EitherHolders: a flag
                // set for the holder, which the read consumed. A key was
                // dropped on read, so what is stored is always a holder.
                // The instrument's wrapper only started in 1.21.5, so
                // earlier versions send the holder bare (as the read side's
                // pre-1.21.5 bypass does).
                let quirks = ids::quirks_written();
                let either_holder = quirks.either_holders_and_tag_resistance
                    && (kind != component::INSTRUMENT
                        || !quirks.components_before_1_21_5);
                if matches!(
                    kind,
                    component::INSTRUMENT | component::JUKEBOX_PLAYABLE
                ) && either_holder
                {
                    w.bool(true);
                }
                w.bytes(legacy.as_deref().unwrap_or(bytes));
            }
            Self::BannerPatterns(layers) => {
                w.varint(layers.len() as i32);
                for (pattern, color) in layers {
                    w.varint(pattern + 1);
                    w.varint(*color);
                }
            }
            Self::Trim { material, pattern } => {
                w.varint(material + 1);
                w.varint(pattern + 1);
            }
            Self::TrimDirect { material, pattern } => {
                material.write(w, TrimMaterial::write);
                pattern.write(w, TrimPattern::write);
            }
            Self::Instrument(id) => {
                // The EitherHolder wrapper started in 1.21.5; earlier
                // versions send the holder bare.
                let quirks = ids::quirks_written();
                if quirks.either_holders_and_tag_resistance
                    && !quirks.components_before_1_21_5
                {
                    w.bool(true);
                }
                w.varint(id + 1)
            }
            Self::PaintingVariant(id) => w.varint(id + 1),
            Self::Int(v) if kind == component::DYED_COLOR => w.i32(*v),
            Self::Int(v) => w.varint(*v),
            Self::Unit => {}
            Self::Bool(v) => w.bool(*v),
            Self::Text(t) | Self::Nbt(t) => w.nbt(t),
            Self::Ident(s) => w.string(s),
            Self::Lore(lines) => {
                w.varint(lines.len() as i32);
                lines.iter().for_each(|l| w.nbt(l));
            }
            Self::Enchantments(list) | Self::StewEffects(list) => {
                w.varint(list.len() as i32);
                for (e, level) in list {
                    w.varint(*e);
                    w.varint(*level);
                }
            }
            Self::AttackRange(values) => values.iter().for_each(|v| w.f32(*v)),
            Self::Potion {
                potion,
                color,
                rest,
            } => {
                w.bool(potion.is_some());
                if let Some(id) = potion {
                    w.varint(*id);
                }
                w.bool(color.is_some());
                if let Some(color) = color {
                    w.i32(*color);
                }
                // An empty tail is no custom effects and no name. A
                // pre-1.21.2 target takes the effects alone (no name field);
                // a newer one takes the stored tail as it is.
                if rest.is_empty() {
                    w.varint(0);
                    if !ids::quirks_written().components_before_1_21_2 {
                        w.bool(false);
                    }
                } else if ids::quirks_written().components_before_1_21_2 {
                    w.bytes(&strip_potion_name(rest));
                } else {
                    w.bytes(rest);
                }
            }
            Self::Profile(p) => {
                if ids::quirks_written().optional_profile_fields
                {
                    write_optional_profile(p, w);
                    return;
                }
                w.u8(0);
                w.bool(p.name.is_some());
                if let Some(name) = &p.name {
                    w.string(name);
                }
                w.bool(p.id.is_some());
                if let Some(id) = p.id {
                    w.uuid(id);
                }
                w.varint(p.properties.len() as i32);
                for (name, value, signature) in &p.properties {
                    w.string(name);
                    w.string(value);
                    w.bool(signature.is_some());
                    if let Some(signature) = signature {
                        w.string(signature);
                    }
                }
                p.skin.write(w);
            }
            Self::CustomModelData {
                floats,
                flags,
                strings,
                colors,
            } => {
                // The target's layout: the pinned 1.21.3 writes one VarInt, as
                // does any older target; 1.21.4+ targets get the four lists.
                if ids::quirks_written().integer_custom_model_data {
                    w.varint(floats.first().copied().unwrap_or(0.0) as i32);
                    return;
                }
                w.varint(floats.len() as i32);
                floats.iter().for_each(|f| w.f32(*f));
                w.varint(flags.len() as i32);
                flags.iter().for_each(|f| w.bool(*f));
                w.varint(strings.len() as i32);
                strings.iter().for_each(|s| w.string(s));
                w.varint(colors.len() as i32);
                colors.iter().for_each(|c| w.i32(*c));
            }
            // The 1.21.5+ wire form (a bool, then the hidden ids). The pin has
            // no tooltip_display: `write_patch` routes around this arm and
            // emits the two hide booleans instead, and `v1_21_4::write_stack`
            // splits it the same way for older versions. The arm stays so the
            // match is exhaustive — and so a newer-target synthesis can use it.
            Self::TooltipDisplay { hide, hidden } => {
                w.bool(*hide);
                // Without the components an older client being written for lacks.
                let kept = || {
                    hidden
                        .iter()
                        .map(|h| ids::component(*h))
                        .filter(|h| *h >= 0)
                };
                w.varint(kept().count() as i32);
                kept().for_each(|h| w.varint(h));
            }
            Self::Stacks(stacks) => {
                w.varint(stacks.len() as i32);
                stacks.iter().for_each(|s| Template::write(s, w));
            }
            Self::Fireworks { flight, explosions } => {
                w.varint(*flight);
                w.varint(explosions.len() as i32);
                explosions.iter().for_each(|e| e.write(w));
            }
            Self::FireworkExplosion(e) => e.write(w),
            Self::LodestoneTracker { target, tracked } => {
                w.bool(target.is_some());
                if let Some((dimension, pos)) = target {
                    w.string(dimension);
                    w.i64(pack_block_pos(*pos));
                }
                w.bool(*tracked);
            }
            Self::WritableBook { pages, filtered } => {
                w.varint(pages.len() as i32);
                for (i, page) in pages.iter().enumerate() {
                    w.string(page);
                    match filtered.get(i).and_then(Option::as_ref) {
                        Some(f) => {
                            w.bool(true);
                            w.string(f);
                        }
                        None => w.bool(false),
                    }
                }
            }
            Self::WrittenBook(b) => {
                w.string(&b.title);
                match &b.title_filtered {
                    Some(f) => {
                        w.bool(true);
                        w.string(f);
                    }
                    None => w.bool(false),
                }
                w.string(&b.author);
                w.varint(b.generation);
                w.varint(b.pages.len() as i32);
                for (i, page) in b.pages.iter().enumerate() {
                    w.nbt(page);
                    match b.pages_filtered.get(i).and_then(Option::as_ref) {
                        Some(f) => {
                            w.bool(true);
                            w.nbt(f);
                        }
                        None => w.bool(false),
                    }
                }
                w.bool(b.resolved);
            }
        }
    }
}

fn read_optional_profile(r: &mut Reader) -> Result<Profile> {
    let name = if r.bool()? { Some(r.string(16)?) } else { None };
    let id = if r.bool()? { Some(r.uuid()?) } else { None };
    let n = r.count(16)?;
    let mut properties = Vec::with_capacity(n);
    for _ in 0..n {
        let name = r.string(64)?;
        let value = r.string(32767)?;
        let signature = if r.bool()? {
            Some(r.string(32767)?)
        } else {
            None
        };
        properties.push((name, value, signature));
    }
    Ok(Profile {
        name,
        id,
        properties,
        skin: SkinPatch::default(),
    })
}
fn write_optional_profile(p: &Profile, w: &mut Writer) {
    w.bool(p.name.is_some());
    if let Some(name) = &p.name {
        w.string(name);
    }
    w.bool(p.id.is_some());
    if let Some(id) = p.id {
        w.uuid(id);
    }
    w.varint(p.properties.len() as i32);
    for (name, value, signature) in &p.properties {
        w.string(name);
        w.string(value);
        w.bool(signature.is_some());
        if let Some(signature) = signature {
            w.string(signature);
        }
    }
}
fn read_legacy_container(r: &mut Reader) -> Result<Vec<u8>> {
    let n = r.count(256)?;
    let mut w = Writer::new();
    w.varint(n as i32);
    for _ in 0..n {
        let stack = Slot::read(r)?;
        w.bool(stack.is_some());
        if let Some(stack) = stack {
            // Raw values stored in a Stack always use the pinned layout — the
            // template form — so it is written out here, not via
            // Template::write, which follows the target's quirks and would
            // write a count-first slot for the pinned 1.21.8.
            let _pinned = ids::unrenumbered();
            w.varint(ids::item(stack.item));
            w.varint(stack.count);
            write_patch(&stack, &mut w);
        }
    }
    Ok(w.data)
}
fn write_legacy_container(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(bytes);
    let mut w = Writer::new();
    let n = r.count(256)?;
    w.varint(n as i32);
    for _ in 0..n {
        let stack = if r.bool()? {
            Some({
                let _pinned = ids::unrenumbered();
                // The held bytes are the internal template form, not the
                // wire form: read_template, not Template::read, which
                // would read a count-first slot for the pinned 1.21.8.
                read_template(&mut r)?
            })
        } else {
            None
        };
        write_slot(stack.as_ref(), &mut w);
    }
    Ok(w.data)
}

/// A damage-resistant tag in the bare-tag form from the internal HolderSet
/// form: the tag behind a `0`, without it. An explicit set has no tag
/// form — an empty tag keeps the stream aligned where the list would not.
fn write_damage_resistant(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(bytes);
    let mut w = Writer::new();
    if r.varint()? == 0 {
        w.string(&r.string(32767)?);
    } else {
        w.string("");
    }
    Ok(w.data)
}

/// A potion tail without its 1.21.2+ custom name: the effect count and
/// effects, dropping the trailing name field. Unparseable tails (which
/// cannot come from `Component::read`) come back empty.
fn strip_potion_name(rest: &[u8]) -> Vec<u8> {
    let mut r = Reader::new(rest);
    let Ok(n) = r.count(65536) else {
        return Vec::new();
    };
    for _ in 0..n {
        if skip::mob_effect_skip(&mut r).is_err() {
            return Vec::new();
        }
    }
    rest[..r.position()].to_vec()
}

/// A pin-form food (nutrition, saturation, canAlwaysEat, eatSeconds, an
/// optional remainder, the possible effects) as a 1.21.2+ server reads it:
/// the first three fields only. The tail has no field there — the
/// consumable and use_remainder components carry it, and this write cannot
/// emit them (see PORT-NOTES).
fn truncate_food(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut r = Reader::new(bytes);
    let mut w = Writer::new();
    w.varint(r.varint().ok()?);
    w.f32(r.f32().ok()?);
    w.bool(r.bool().ok()?);
    Some(w.data)
}

/// A pin-form food (nutrition, saturation, canAlwaysEat, eatSeconds, an
/// optional remainder, effects) as a pre-1.21 server reads it: no remainder
/// field — a present remainder is dropped, the target cannot hold it.
fn strip_food_remainder(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut r = Reader::new(bytes);
    let mut w = Writer::new();
    w.varint(r.varint().ok()?);
    w.f32(r.f32().ok()?);
    w.bool(r.bool().ok()?);
    w.f32(r.f32().ok()?);
    if r.bool().ok()? {
        let _pinned = ids::unrenumbered();
        Template::read(&mut r).ok()?;
    }
    let n = r.count(65536).ok()?;
    w.varint(n as i32);
    for _ in 0..n {
        v1_21_1::effect(&mut r, &mut w, false).ok()?;
        w.f32(r.f32().ok()?);
    }
    (r.remaining() == 0).then_some(w.data)
}

/// A pin-form direct instrument (sound, useDuration in ticks, range) as a
/// 1.21.2+ server reads it: the duration in float seconds, then
/// durabilityDamage 0 and an empty description. A reference passes through.
fn expand_instrument(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut r = Reader::new(bytes);
    let mut w = Writer::new();
    let holder = r.varint().ok()?;
    w.varint(holder);
    if holder == 0 {
        copy_sound_holder(&mut r, &mut w).ok()?;
        w.f32(r.varint().ok()? as f32 / 20.0);
        w.f32(r.f32().ok()?);
        w.varint(0);
        w.nbt(&Tag::String("".into()));
    }
    (r.remaining() == 0).then_some(w.data)
}

/// Pin-form block predicates (blocks, properties, nbt) as a 1.21.5+ server
/// reads them: an empty DataComponentMatchers pair (exact, partial) after
/// each predicate's nbt.
fn append_predicate_matchers(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut r = Reader::new(bytes);
    let mut w = Writer::new();
    let n = r.count(65536).ok()?;
    w.varint(n as i32);
    for _ in 0..n {
        let start = r.position();
        if r.bool().ok()? {
            skip::holder_set(&mut r).ok()?;
        }
        skip::block_properties_and_nbt(&mut r).ok()?;
        w.bytes(r.since(start));
        w.varint(0);
        w.varint(0);
    }
    (r.remaining() == 0).then_some(w.data)
}
fn write_pot_decorations(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(bytes);
    let mut w = Writer::new();
    w.varint(4);
    for _ in 0..4 {
        let item = if r.bool()? {
            let _pinned = ids::unrenumbered();
            read_template(&mut r)?.item
        } else {
            BRICK
        };
        w.varint(ids::item(item));
    }
    Ok(w.data)
}

/// A component patch as `DelimitedSlot` writes it (both counts, then each
/// value behind its length), kept from a stack that was read so one nobody
/// edited goes back byte for byte. It is no part of a stack's equality.
#[derive(Clone, Debug, Default)]
pub struct WirePatch(pub Option<Arc<[u8]>>);
impl WirePatch {
    /// Whether a patch in this form sets component `kind`.
    pub fn adds(patch: &[u8], kind: i32) -> bool {
        let mut r = Reader::new(patch);
        let Ok(positive) = r.count(65536) else {
            return false;
        };
        if r.count(65536).is_err() {
            return false;
        }
        (0..positive).any(|_| {
            let Ok(k) = r.varint() else {
                return false;
            };
            let Ok(len) = r.count(r.remaining()) else {
                return false;
            };
            r.bytes(len).is_ok() && k == kind
        })
    }
}
/// The `negative` removed component types of a patch: an older server's component the pinned version
/// lacks ([`ids::dropped_component`]) is left out.
fn read_removed(r: &mut Reader, negative: usize) -> Result<Vec<i32>> {
    let mut removed = Vec::with_capacity(negative.min(64));
    for _ in 0..negative {
        let raw = r.varint()?;
        if ids::dropped_component(raw).is_none() {
            let kind = ids::component(raw);
            // A pre-1.21.5 stack removes the tooltip booleans; the pin codec
            // holds them as the synthesized display, so the removal names it.
            let kind = if ids::quirks().components_before_1_21_5
                && matches!(
                    kind,
                    component::HIDE_ADDITIONAL_TOOLTIP | component::HIDE_TOOLTIP
                ) {
                component::TOOLTIP_DISPLAY
            } else {
                kind
            };
            if !removed.contains(&kind) {
                removed.push(kind);
            }
        }
    }
    Ok(removed)
}

/// The tooltip state a pre-1.21.5 stack reader accumulates: the two hide
/// booleans and the tooltip-flagged components shown false, folded into one
/// synthesized `TooltipDisplay` when the patch is complete (the pin has no
/// tooltip_display component; 1.21.5 folded these into one).
struct Tooltip {
    hide: bool,
    additional: bool,
    hidden: Vec<i32>,
}

impl Tooltip {
    fn new() -> Self {
        Self {
            hide: false,
            additional: false,
            hidden: Vec::new(),
        }
    }

    /// An added component of a pre-1.21.5 stack: the hide booleans are
    /// swallowed (`false`: not an added component), anything else passes
    /// through, and a hidden flag joins the hidden list.
    fn added(&mut self, kind: i32, shown: Option<bool>) -> bool {
        use component::*;
        match kind {
            HIDE_ADDITIONAL_TOOLTIP => {
                self.additional = true;
                false
            }
            HIDE_TOOLTIP => {
                self.hide = true;
                false
            }
            _ => {
                if shown == Some(false) {
                    self.hidden.push(kind);
                }
                true
            }
        }
    }

    /// The synthesized component, if the stack hides anything. The additional
    /// flag hides the item-specific providers present on the stack.
    fn finish(mut self, added: &[(i32, Component)]) -> Option<(i32, Component)> {
        if self.additional {
            self.hidden.extend(
                added
                    .iter()
                    .map(|(kind, _)| *kind)
                    .filter(|kind| v1_21_4::ADDITIONAL.contains(kind)),
            );
        }
        if self.hide || !self.hidden.is_empty() {
            self.hidden.sort_unstable();
            self.hidden.dedup();
            Some((
                component::TOOLTIP_DISPLAY,
                Component::TooltipDisplay {
                    hide: self.hide,
                    hidden: self.hidden,
                },
            ))
        } else {
            None
        }
    }
}

/// An added component `raw` (the sender's id) and its value from `r`: its pinned id, the value, and the
/// value's bytes in the pinned layout, plus the show flag for a tooltip-flagged component of a pre-1.21.5
/// stack (`None` otherwise). An older server's component the pinned version lacks is read past (`None`),
/// and one whose layout changed since that server's version is read in its layout.
fn read_added<'a>(
    raw: i32,
    r: &mut Reader<'a>,
) -> Result<Option<(i32, Component, std::borrow::Cow<'a, [u8]>, Option<bool>)>> {
    // Every way a stack holds another passes through here.
    let _nested = Nested::enter()?;
    if let Some(name) = ids::dropped_component(raw) {
        skip_dropped(name, r)?;
        return Ok(None);
    }
    let kind = ids::component(raw);
    match older_layout(kind, r)? {
        Older::Dropped => Ok(None),
        Older::Rewritten(bytes) => {
            // Already in the pinned numbering.
            let _pinned = ids::pinned();
            // A rewritten container is already in the internal form
            // (bool + template per slot); Component::read would re-apply
            // the legacy-wire conversion to it.
            let component = if kind == component::CONTAINER {
                Component::Raw(bytes.clone())
            } else {
                Component::read(kind, &mut Reader::new(&bytes))?
            };
            Ok(Some((kind, component, bytes.into(), None)))
        }
        Older::Pinned => {
            let start = r.position();
            let component = Component::read(kind, r)?;
            // A pre-1.21.5 stack carries a show flag after each
            // tooltip-flagged component's value; it rides in the value
            // bytes, and the stack reader folds it into the synthesized
            // TooltipDisplay.
            let shown = if ids::quirks().components_before_1_21_5
                && v1_21_4::TOOLTIP_FLAGS.contains(&kind)
            {
                Some(r.bool()?)
            } else {
                None
            };
            Ok(Some((kind, component, r.since(start).into(), shown)))
        }
    }
}

/// How [`older_layout`] found a component of an older server.
enum Older {
    /// What is left of it reads as the pinned version's.
    Pinned,
    /// Read past: the pinned version can't hold it.
    Dropped,
    /// Read, and written again in the pinned layout.
    Rewritten(Vec<u8>),
}

/// `minecraft:brick` (1.21.1): an older pot's undecorated side.
const BRICK: i32 = 921;

/// Reads the start of a component `kind` (a pinned id) whose layout changed since the version of the
/// server whose packet is read on this thread ([`ids::quirks`]).
fn older_layout(kind: i32, r: &mut Reader) -> Result<Older> {
    use component::*;
    let quirks = ids::quirks();
    Ok(match kind {
        // EitherHolder.streamCodec: ByteBufCodecs.either(the holder, its ResourceKey), a flag set for the
        // holder, which then reads as the pinned one does. A key names an entry of the server's registry
        // that the pinned holder can't point to without its numbering. The jukebox's wrapper predates
        // 1.21.5 (the instrument's started there; see its own arm below).
        JUKEBOX_PLAYABLE if quirks.either_holders_and_tag_resistance => {
            if r.bool()? {
                Older::Pinned
            } else {
                r.string(32767)?;
                Older::Dropped
            }
        }
        // InstrumentComponent gained durabilityDamage and a description in 1.21.2, its use duration went
        // from int ticks to float seconds, and 1.21.5 wrapped the holder in an EitherHolder flag. A newer
        // server's direct instrument is rewritten to the pin's three-field form (sound, ticks, range); a
        // reference passes through (its id is the server's — see `Component::read`).
        INSTRUMENT if !quirks.components_before_1_21_2 => {
            if quirks.either_holders_and_tag_resistance && !quirks.components_before_1_21_5 {
                if !r.bool()? {
                    r.string(32767)?;
                    return Ok(Older::Dropped);
                }
            }
            let holder = r.varint()?;
            let mut w = Writer::new();
            if holder == 0 {
                w.varint(0);
                copy_sound_holder(r, &mut w)?;
                w.varint((r.f32()? * 20.0) as i32);
                w.f32(r.f32()?);
                r.varint()?; // durabilityDamage: the pin has no such field.
                r.nbt()?; // description: likewise.
            } else {
                w.varint(holder);
            }
            Older::Rewritten(w.data)
        }
        // Before 1.21 FoodProperties had no usingConvertsTo remainder: the
        // pin's combined form gains an absent remainder between eatSeconds
        // and the effects (the effects' ids are renumbered as usual).
        FOOD if quirks.components_before_1_21 => {
            let mut w = Writer::new();
            w.varint(r.varint()?);
            w.f32(r.f32()?);
            w.bool(r.bool()?);
            w.f32(r.f32()?);
            w.bool(false);
            let n = r.count(65536)?;
            w.varint(n as i32);
            for _ in 0..n {
                v1_21_1::effect(r, &mut w, true)?;
                w.f32(r.f32()?);
            }
            Older::Rewritten(w.data)
        }
        // FoodProperties was split in 1.21.2: a newer server's food is just nutrition, saturation and
        // canAlwaysEat, with the rest in consumable and use_remainder (both dropped at this pin). The
        // pin's food is the combined form, so the tail is rebuilt with vanilla's defaults: 1.6s to eat,
        // no remainder, no effects. A newer server's consume effects are lost here; merging them into
        // the pin's food needs the whole stack (see PORT-NOTES).
        FOOD if !quirks.components_before_1_21_2 => {
            let mut w = Writer::new();
            w.varint(r.varint()?);
            w.f32(r.f32()?);
            w.bool(r.bool()?);
            w.f32(1.6);
            w.bool(false);
            w.varint(0);
            Older::Rewritten(w.data)
        }
        // Tool gained canDestroyBlocksInCreative in 1.21.5; the pin's form ends at damagePerBlock. A
        // 1.21.5+ server's trailing bool is dropped (its rules' block ids are renumbered, as ever).
        TOOL if !quirks.components_before_1_21_5 => {
            let mut w = Writer::new();
            let n = r.count(65536)?;
            w.varint(n as i32);
            for _ in 0..n {
                v1_21_4::block_set(r, &mut w)?;
                let speed = r.bool()?;
                w.bool(speed);
                if speed {
                    w.f32(r.f32()?);
                }
                let drops = r.bool()?;
                w.bool(drops);
                if drops {
                    w.bool(r.bool()?);
                }
            }
            w.f32(r.f32()?);
            w.varint(r.varint()?);
            r.bool()?; // canDestroyBlocksInCreative: the pin has no such field.
            Older::Rewritten(w.data)
        }
        // BlockPredicate gained the DataComponentMatchers (exact, partial) in 1.21.5; the pin's form
        // ends at the nbt predicate. A 1.21.5+ server's matchers are dropped.
        CAN_PLACE_ON | CAN_BREAK if !quirks.components_before_1_21_5 => {
            let mut w = Writer::new();
            let n = r.count(65536)?;
            w.varint(n as i32);
            for _ in 0..n {
                let blocks = r.bool()?;
                w.bool(blocks);
                if blocks {
                    v1_21_4::block_set(r, &mut w)?;
                }
                let start = r.position();
                skip::block_properties_and_nbt(r)?;
                w.bytes(r.since(start));
                skip::component_matchers(r)?;
            }
            Older::Rewritten(w.data)
        }
        // TagKey.streamCodec: the tag's id. A HolderSet names a tag behind a 0.
        DAMAGE_RESISTANT if quirks.either_holders_and_tag_resistance => {
            let tag = r.string(32767)?;
            let mut w = Writer::new();
            w.varint(0);
            w.string(&tag);
            Older::Rewritten(w.data)
        }
        // ByteBufCodecs.registry(ITEM).apply(list(4)): back, left, right and front, a brick for none;
        // the internal form is four optional stack templates of one item and no components (the pin wire
        // itself is four item ids, as the older form is). The value is kept as these bytes and sent on as
        // they are, so its items take their pinned ids here.
        POT_DECORATIONS if quirks.pot_decorations_are_items => {
            let n = r.count(4)?;
            let items = (0..n)
                .map(|_| r.varint().map(ids::item))
                .collect::<Result<Vec<_>>>()?;
            let mut w = Writer::new();
            for side in 0..4 {
                match items.get(side).filter(|&&item| item != BRICK) {
                    Some(&item) => {
                        w.bool(true);
                        w.varint(item);
                        w.varint(1);
                        w.varint(0);
                        w.varint(0);
                    }
                    None => w.bool(false),
                }
            }
            Older::Rewritten(w.data)
        }
        // A value holding item stacks is kept as its bytes and sent on as they are: written again, its
        // items take their pinned ids and layout.
        USE_REMAINDER | SULFUR_CUBE_CONTENT if ids::reading_older() => {
            let stack = if quirks.stacks_in_components {
                Slot::read(r)?.ok_or_else(|| crate::Error::new("an empty use_remainder"))?
            } else {
                Template::read(r)?
            };
            let mut w = Writer::new();
            let _pinned = ids::pinned();
            // The internal form is the template form, not the wire form:
            // Template::write would write a count-first slot for the
            // pinned 1.21.8.
            write_template(&stack, &mut w);
            Older::Rewritten(w.data)
        }
        // ItemContainerContents: up to 256 slots, each an optional template (ItemStack.OPTIONAL_STREAM_CODEC
        // before 26.1).
        CONTAINER if ids::reading_older() => {
            let n = r.count(256)?;
            let slots = (0..n)
                .map(|_| {
                    if quirks.stacks_in_components {
                        Slot::read(r)
                    } else if r.bool()? {
                        Template::read(r).map(Some)
                    } else {
                        Ok(None)
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            let mut w = Writer::new();
            let _pinned = ids::pinned();
            w.varint(n as i32);
            for slot in &slots {
                w.bool(slot.is_some());
                if let Some(stack) = slot {
                    write_template(stack, &mut w);
                }
            }
            Older::Rewritten(w.data)
        }
        _ => Older::Pinned,
    })
}

/// Reads past the value of `name`, a data component an older server sent that the pinned version lacks
/// ([`ids::dropped_component`]), as that server writes it.
/// An optional sound holder (PiercingWeapon/KineticWeapon): a flag, then the
/// holder — 0 and the direct event (a name, an optional fixed range), or the
/// registry id plus one. `SoundEvent.STREAM_CODEC` in the 1.21.11 jar.
fn skip_sound(r: &mut Reader) -> Result<()> {
    if !r.bool()? {
        return Ok(());
    }
    if r.varint()? == 0 {
        r.string(32767)?;
        if r.bool()? {
            r.f32()?;
        }
    }
    Ok(())
}

/// A sound holder copied from one numbering to another: a direct event (0,
/// a name, an optional fixed range) passes through, a reference is
/// renumbered.
fn copy_sound_holder(r: &mut Reader, w: &mut Writer) -> Result<()> {
    let holder = r.varint()?;
    if holder == 0 {
        w.varint(0);
        w.string(&r.string(32767)?);
        let fixed = r.bool()?;
        w.bool(fixed);
        if fixed {
            w.f32(r.f32()?);
        }
    } else {
        w.varint(ids::sound_event(holder - 1) + 1);
    }
    Ok(())
}

fn skip_dropped(name: &str, r: &mut Reader) -> Result<()> {
    match name {
        // Unit.STREAM_CODEC: fire_resistant and the two hide flags are pin
        // components at 1.21.1, so no server's drop list names them; the arms
        // stay so a stray one still reads past.
        "minecraft:fire_resistant"
        | "minecraft:hide_tooltip"
        | "minecraft:hide_additional_tooltip" => Ok(()),
        // The 1.21.2 components this pin predates, with their 1.21.2+
        // stream codecs (DataComponents in the 1.21.11 jar; the shapes
        // carried into 26.x unchanged). ItemModel: a resource location.
        "minecraft:item_model" | "minecraft:tooltip_style" => {
            r.string(32767).map(drop)
        }
        // JukeboxPlayable (a 1.21 addition this pin predates): an
        // EitherHolder — a flag, then a holder id or a key.
        "minecraft:jukebox_playable" => {
            if r.bool()? {
                r.varint().map(drop)
            } else {
                r.string(32767).map(drop)
            }
        }
        // Consumable: consumeSeconds, animation, sound,
        // hasConsumeParticles, onConsumeEffects.
        "minecraft:consumable" => skip::consumable(r),
        // UseRemainder: one item stack template.
        "minecraft:use_remainder" => Template::read(r).map(drop),
        // UseCooldown: seconds, cooldownGroup.
        "minecraft:use_cooldown" => skip::use_cooldown(r),
        // DamageResistant is a TagKey, a bare string — not a holder set
        // (reading it as one eats the following components). Repairable
        // is a holder set; Enchantable a varint (the enchantment value).
        "minecraft:damage_resistant" => r.string(32767).map(drop),
        "minecraft:repairable" => skip::holder_set(r),
        "minecraft:enchantable" => r.varint().map(drop),
        // Equippable: slot, equipSound, assetId, cameraOverlay,
        // allowedEntities, dispensable, swappable, damageOnHurt,
        // equipOnInteract, canBeSheared, shearingSound. Before 1.21.6
        // there is no canBeSheared and no shearingSound, and before
        // 1.21.5 no equipOnInteract either.
        "minecraft:equippable" => {
            if ids::quirks().components_before_1_21_6 {
                skip::equippable_before_1_21_6(r, ids::quirks().components_before_1_21_5)
            } else {
                skip::equippable(r)
            }
        }
        // Glider: Unit.
        "minecraft:glider" => Ok(()),
        // DeathProtection: deathEffects.
        "minecraft:death_protection" => skip::death_protection(r),
        // TooltipDisplay.STREAM_CODEC (1.21.5+): hide, then the hidden ids.
        "minecraft:tooltip_display" => {
            r.bool()?;
            let n = r.count(r.remaining())?;
            for _ in 0..n {
                r.varint()?;
            }
            Ok(())
        }
        // Weapon.STREAM_CODEC (1.21.5+): itemDamagePerAttack, disableBlockingForSeconds.
        "minecraft:weapon" => {
            r.varint()?;
            r.f32().map(drop)
        }
        // BlocksAttacks.STREAM_CODEC (1.21.5+).
        "minecraft:blocks_attacks" => skip::blocks_attacks(r),
        // FLOAT: potion_duration_scale (1.21.5+).
        "minecraft:potion_duration_scale" => r.f32().map(drop),
        // EitherHolder (1.21.5+): a flag, then a holder id or a key.
        "minecraft:provides_trim_material" | "minecraft:chicken/variant" => {
            if r.bool()? {
                r.varint().map(drop)
            } else {
                r.string(32767).map(drop)
            }
        }
        // A holder set: provides_banner_patterns (1.21.5+).
        "minecraft:provides_banner_patterns" => skip::holder_set(r),
        // SoundEvent.STREAM_CODEC: break_sound (1.21.5+).
        "minecraft:break_sound" => skip::sound(r),
        // Holder<PaintingVariant> (1.21.5+): a reference's id plus one, or 0
        // and the variant itself.
        "minecraft:painting/variant" => skip::component(component::PAINTING_VARIANT, r),
        // A VarInt (a registry or holder id): the 1.21.5 mob variants and
        // colours (except painting/variant, which has its own holder arm above).
        "minecraft:villager/variant"
        | "minecraft:wolf/variant"
        | "minecraft:wolf/sound_variant"
        | "minecraft:wolf/collar"
        | "minecraft:fox/variant"
        | "minecraft:salmon/size"
        | "minecraft:parrot/variant"
        | "minecraft:tropical_fish/base_color"
        | "minecraft:tropical_fish/pattern"
        | "minecraft:tropical_fish/pattern_color"
        | "minecraft:mooshroom/variant"
        | "minecraft:rabbit/variant"
        | "minecraft:pig/variant"
        | "minecraft:cow/variant"
        | "minecraft:frog/variant"
        | "minecraft:horse/variant"
        | "minecraft:llama/variant"
        | "minecraft:axolotl/variant"
        | "minecraft:cat/variant"
        | "minecraft:cat/collar"
        | "minecraft:sheep/color"
        | "minecraft:shulker/color" => r.varint().map(drop),
        // SulfurCubeContent (a 26.2 component 1.21.10 predates): one item stack
        // template, read and dropped.
        "minecraft:sulfur_cube_content" => Template::read(r).map(drop),
        // VarInt (a registry or holder id): the 26.1 components 1.21.10
        // predates — additional_trade_cost, dye and the sound variants.
        "minecraft:additional_trade_cost"
        | "minecraft:dye"
        | "minecraft:pig/sound_variant"
        | "minecraft:cow/sound_variant"
        | "minecraft:chicken/sound_variant"
        | "minecraft:cat/sound_variant" => r.varint().map(drop),
        // The 1.21.11 components 1.21.10 predates, with their 1.21.11 stream
        // codecs (DataComponents in the 1.21.11 jar; the shapes carried into
        // 26.x unchanged). EitherHolder: a flag, then a holder id or a key.
        "minecraft:damage_type" | "minecraft:zombie_nautilus/variant" => {
            if r.bool()? {
                r.varint().map(drop)
            } else {
                r.string(32767).map(drop)
            }
        }
        // AttackRange.STREAM_CODEC: six floats.
        "minecraft:attack_range" => {
            for _ in 0..6 {
                r.f32()?;
            }
            Ok(())
        }
        // UseEffects.STREAM_CODEC: two flags and a float.
        "minecraft:use_effects" => {
            r.bool()?;
            r.bool()?;
            r.f32().map(drop)
        }
        // ByteBufCodecs.FLOAT.
        "minecraft:minimum_attack_charge" => r.f32().map(drop),
        // SwingAnimation.STREAM_CODEC: the type id, then the duration.
        "minecraft:swing_animation" => {
            r.varint()?;
            r.varint().map(drop)
        }
        // PiercingWeapon.STREAM_CODEC: two flags, then two optional sound
        // holders (a flag, then the holder: 0 and the direct event — a name
        // and an optional fixed range — or the registry id plus one).
        "minecraft:piercing_weapon" => {
            r.bool()?;
            r.bool()?;
            skip_sound(r)?;
            skip_sound(r)
        }
        // KineticWeapon.STREAM_CODEC: two counts, three optional conditions
        // (a count and two floats), two floats, then two optional sound
        // holders.
        "minecraft:kinetic_weapon" => {
            r.varint()?;
            r.varint()?;
            for _ in 0..3 {
                if r.bool()? {
                    r.varint()?;
                    r.f32()?;
                    r.f32()?;
                }
            }
            r.f32()?;
            r.f32()?;
            skip_sound(r)?;
            skip_sound(r)
        }
        _ => Err(crate::Error::new(format!(
            "{name}: a component the pinned version no longer has"
        ))),
    }
}

impl WirePatch {
    /// The components and removals a patch in this form holds, read as
    /// [`DelimitedSlot`] does.
    pub fn decode(patch: &[u8]) -> Result<(Vec<(i32, Component)>, Vec<i32>)> {
        let mut r = Reader::new(patch);
        let positive = r.count(65536)?;
        let negative = r.count(65536)?;
        let mut added = Vec::with_capacity(positive.min(64));
        let mut tooltip = Tooltip::new();
        for _ in 0..positive {
            let raw = r.varint()?;
            let len = r.count(r.remaining())?;
            if let Some((kind, component, _, shown)) =
                read_added(raw, &mut Reader::new(r.bytes(len)?))?
            {
                if tooltip.added(kind, shown) {
                    added.push((kind, component));
                }
            }
        }
        if let Some(display) = tooltip.finish(&added) {
            added.push(display);
        }
        Ok((added, read_removed(&mut r, negative)?))
    }
}
impl PartialEq for WirePatch {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// A non-empty item stack.
#[derive(Clone, Debug, PartialEq)]
pub struct Stack {
    pub item: i32,
    pub count: i32,
    pub added: Vec<(i32, Component)>,
    pub removed: Vec<i32>,
    /// The patch as it was read; [`DelimitedSlot`] writes it in place of
    /// `added` and `removed` when set. Only for a stack nobody has edited.
    pub wire: WirePatch,
}
impl Stack {
    pub fn new(item: i32, count: i32) -> Self {
        Self {
            item,
            count,
            added: Vec::new(),
            removed: Vec::new(),
            wire: WirePatch::default(),
        }
    }
    pub fn component(&self, kind: i32) -> Option<&Component> {
        self.added.iter().find(|(k, _)| *k == kind).map(|(_, c)| c)
    }
    pub fn damage(&self) -> i32 {
        match self.component(component::DAMAGE) {
            Some(Component::Int(d)) => *d,
            _ => 0,
        }
    }
}
/// ItemStack.OPTIONAL_STREAM_CODEC: `None` is the empty stack (count 0).
pub struct Slot;
impl Codec<Option<Stack>> for Slot {
    fn read(r: &mut Reader) -> Result<Option<Stack>> {
        read_stack(r, false)
    }
    fn write(v: &Option<Stack>, w: &mut Writer) {
        write_slot(v.as_ref(), w);
    }
}
fn write_slot(v: Option<&Stack>, w: &mut Writer) {
    if v1_20_3::writing() {
        v1_20_3::write_stack(v, w);
        return;
    }
    if v1_21_1::writing() {
        v1_21_1::write_stack(v, w);
        return;
    }
    if v1_21_4::writing() {
        v1_21_4::write_stack(v, w);
        return;
    }
    // An item an older client has no stand-in for reaches it as nothing.
    let item = v.map_or(-1, |s| ids::item(s.item));
    let Some(s) = v.filter(|_| item >= 0) else {
        w.varint(0);
        return;
    };
    w.varint(s.count);
    w.varint(item);
    write_patch(s, w);
}

/// A component value as the version being written for reads it. Values kept as their pinned bytes that hold
/// items (a pot's sides, a container's slots, a use remainder) are read again and written with that
/// version's item ids, and a pot's sides as 26.2's item ids when it reads them so
/// ([`ids::Quirks::pot_decorations_are_items`]). Written as they are when nothing is renumbered.
fn write_value(kind: i32, c: &Component, w: &mut Writer) {
    use component::*;
    let quirks = ids::quirks_written();
    let renumbered = matches!(ids::active(), Some((_, ids::Way::Down)));
    let held = match kind {
        POT_DECORATIONS | USE_REMAINDER | SULFUR_CUBE_CONTENT => true,
        // Before 26.1 these hold ItemStacks, written by another codec.
        CONTAINER => !quirks.stacks_in_components,
        _ => false,
    };
    if !renumbered || !held {
        return c.write(kind, w);
    }
    // The held bytes are already in the pinned (internal) form: a Raw
    // value is stored that way, and writing it through Component::write
    // would convert it back to the wire form (a container's legacy
    // slots), which value_down cannot read.
    let pinned = match c {
        Component::Raw(bytes) => bytes.clone(),
        _ => {
            let _pinned = ids::pinned();
            let mut b = Writer::new();
            c.write(kind, &mut b);
            b.data
        }
    };
    match value_down(kind, quirks, &pinned) {
        Ok(bytes) => w.bytes(&bytes),
        Err(_) => w.bytes(&pinned),
    }
}

/// A stack template as the pinned version wrote it: the internal form Raw
/// values hold (item, count, patch), whatever the wire layout is. Template
/// is the wire codec and dispatches on the quirks; this is the form the
/// held bytes are actually in.
fn read_template(r: &mut Reader) -> Result<Stack> {
    let item = ids::item(r.varint()?);
    let count = r.varint()?;
    let positive = r.count(65536)?;
    let negative = r.count(65536)?;
    let mut added = Vec::with_capacity(positive.min(64));
    let mut tooltip = Tooltip::new();
    for _ in 0..positive {
        let raw = r.varint()?;
        if let Some((kind, component, _, shown)) = read_added(raw, r)? {
            if tooltip.added(kind, shown) {
                added.push((kind, component));
            }
        }
    }
    if let Some(display) = tooltip.finish(&added) {
        added.push(display);
    }
    let removed = read_removed(r, negative)?;
    Ok(Stack {
        item,
        count,
        added,
        removed,
        wire: WirePatch::default(),
    })
}

/// A stack template in the pinned layout (item, count, patch): the form Raw
/// values hold, whatever the wire layout is.
fn write_template(s: &Stack, w: &mut Writer) {
    w.varint(ids::item(s.item));
    w.varint(s.count);
    write_patch(s, w);
}

/// A stack template as the pinned version wrote it.
fn read_pinned(r: &mut Reader) -> Result<Stack> {
    let _pinned = ids::pinned();
    read_template(r)
}

/// The pinned bytes of an item-holding component (see [`write_value`]) in the numbering written now.
fn value_down(kind: i32, quirks: ids::Quirks, pinned: &[u8]) -> Result<Vec<u8>> {
    use component::*;
    let mut r = Reader::new(pinned);
    let mut w = Writer::new();
    match kind {
        // Back, left, right, front: each an optional template of one item.
        POT_DECORATIONS => {
            let mut sides = Vec::with_capacity(4);
            for _ in 0..4 {
                sides.push(if r.bool()? {
                    Some(read_pinned(&mut r)?)
                } else {
                    None
                });
            }
            if quirks.pot_decorations_are_items {
                // ByteBufCodecs.registry(ITEM).apply(list(4)): a brick for an undecorated side.
                w.varint(4);
                for side in &sides {
                    w.varint(ids::item(side.as_ref().map_or(BRICK, |s| s.item)));
                }
            } else {
                for side in &sides {
                    w.bool(side.is_some());
                    if let Some(s) = side {
                        Template::write(s, &mut w);
                    }
                }
            }
        }
        CONTAINER => {
            let n = r.count(256)?;
            w.varint(n as i32);
            for _ in 0..n {
                let slot = r.bool()?;
                w.bool(slot);
                if slot {
                    Template::write(&read_pinned(&mut r)?, &mut w);
                }
            }
        }
        _ => {
            let stack = read_pinned(&mut r)?;
            if quirks.stacks_in_components {
                Slot::write(&Some(stack), &mut w);
            } else {
                Template::write(&stack, &mut w);
            }
        }
    }
    Ok(w.data)
}

/// A stack's components, without those an older client being written for lacks.
/// A pre-1.21.5 target reads the pin's tooltip as the two hide booleans
/// (1.21.4 has no tooltip_display), with a show flag after every
/// tooltip-flagged component's value; a 1.21.5+ target gets no tooltip at all
/// (the synthesized display has no newer-side form without a name lookup).
fn write_patch(s: &Stack, w: &mut Writer) {
    let has = |kind: i32| ids::component(kind) >= 0;
    let pin_tooltip = ids::quirks_written().components_before_1_21_5;
    let display = s
        .added
        .iter()
        .find(|(k, _)| *k == component::TOOLTIP_DISPLAY)
        .and_then(|(_, c)| match c {
            Component::TooltipDisplay { hide, hidden } => Some((*hide, hidden.as_slice())),
            _ => None,
        });
    let (hide, hidden) = display.unwrap_or((false, &[]));
    // The additional flag hides the item-specific providers; it shows in the
    // hidden list exactly when one of them is hidden (the flagged kinds are
    // disjoint from the additional set).
    let additional =
        pin_tooltip && hidden.iter().any(|k| v1_21_4::ADDITIONAL.contains(k));
    let booleans = pin_tooltip && display.is_some() && (hide || additional);
    let added = s
        .added
        .iter()
        .filter(|(k, _)| *k != component::TOOLTIP_DISPLAY && has(*k));
    let added_count = added.clone().count()
        + usize::from(booleans && additional)
        + usize::from(booleans && hide);
    let removed_marker = pin_tooltip && s.removed.contains(&component::TOOLTIP_DISPLAY);
    let removed_count = s
        .removed
        .iter()
        .filter(|k| **k != component::TOOLTIP_DISPLAY && has(**k))
        .count()
        + usize::from(removed_marker) * 2;
    w.varint(added_count as i32);
    w.varint(removed_count as i32);
    for (kind, c) in s.added.iter() {
        if *kind == component::TOOLTIP_DISPLAY {
            if booleans {
                // Unit components: the kind only, no value bytes.
                if additional {
                    w.varint(ids::component(component::HIDE_ADDITIONAL_TOOLTIP));
                }
                if hide {
                    w.varint(ids::component(component::HIDE_TOOLTIP));
                }
            }
            continue;
        }
        if !has(*kind) {
            continue;
        }
        w.varint(ids::component(*kind));
        write_value(*kind, c, w);
        if pin_tooltip && v1_21_4::TOOLTIP_FLAGS.contains(kind) {
            w.bool(!hidden.contains(kind));
        }
    }
    for kind in s
        .removed
        .iter()
        .filter(|k| **k != component::TOOLTIP_DISPLAY && has(**k))
    {
        w.varint(ids::component(*kind));
    }
    if removed_marker {
        w.varint(ids::component(component::HIDE_ADDITIONAL_TOOLTIP));
        w.varint(ids::component(component::HIDE_TOOLTIP));
    }
}

/// A stack of either optional codec: with `delimited` each component value
/// has a length in front (DataComponentPatch.DELIMITED_STREAM_CODEC). The
/// patch is also kept, in the delimited form, for [`Stack::wire`].
fn read_stack(r: &mut Reader, delimited: bool) -> Result<Option<Stack>> {
    if v1_20_3::reading() {
        return v1_20_3::read_stack(r);
    }
    if v1_21_1::reading() {
        return v1_21_1::read_stack(r);
    }
    if v1_21_4::reading() {
        return v1_21_4::read_stack(r);
    }
    let count = r.varint()?;
    if count <= 0 {
        return Ok(None);
    }
    let item = ids::item(r.varint()?);
    let positive = r.count(65536)?;
    let negative = r.count(65536)?;
    let mut added = Vec::with_capacity(positive.min(64));
    let mut values = Vec::with_capacity(positive.min(64));
    let mut tooltip = Tooltip::new();
    for _ in 0..positive {
        let raw = r.varint()?;
        let read = if delimited {
            let len = r.count(r.remaining())?;
            let mut component_reader = Reader::new(r.bytes(len)?);
            let read = read_added(raw, &mut component_reader)?;
            if read
                .as_ref()
                .is_some_and(|(kind, ..)| custom::contains(*kind))
                && component_reader.remaining() != 0
            {
                return Err(crate::Error::new("trailing custom component data"));
            }
            read
        } else {
            read_added(raw, r)?
        };
        if let Some((kind, component, value, shown)) = read {
            if tooltip.added(kind, shown) {
                values.push(value);
                added.push((kind, component));
            }
        }
    }
    let removed = read_removed(r, negative)?;
    // A synthesized display has no wire bytes of its own, so the patch is
    // kept only when nothing was synthesized: an echo then rebuilds from the
    // values (which carry the show flags) instead of replaying a short patch.
    let synthesized = match tooltip.finish(&added) {
        Some(display) => {
            added.push(display);
            true
        }
        None => false,
    };
    let wire = (!synthesized && (!added.is_empty() || !removed.is_empty())).then(|| {
        let mut patch = Writer::new();
        patch.varint(added.len() as i32);
        patch.varint(removed.len() as i32);
        for ((kind, _), value) in added.iter().zip(&values) {
            patch.varint(*kind);
            patch.byte_array(value);
        }
        removed.iter().for_each(|kind| patch.varint(*kind));
        patch.data.into()
    });
    Ok(Some(Stack {
        item,
        count,
        added,
        removed,
        // Adapter bytes may still contain older ids; rebuild from normalized values.
        wire: WirePatch(wire.filter(|_| ids::active().is_none())),
    }))
}

/// ItemStack.OPTIONAL_UNTRUSTED_STREAM_CODEC (`SetCreativeModeSlot`): as [`Slot`],
/// but every component value goes behind its byte length
/// (DataComponentPatch.DELIMITED_STREAM_CODEC), so a server skips what it can't read.
pub struct DelimitedSlot;
impl Codec<Option<Stack>> for DelimitedSlot {
    fn read(r: &mut Reader) -> Result<Option<Stack>> {
        read_stack(r, true)
    }
    fn write(v: &Option<Stack>, w: &mut Writer) {
        if v1_20_3::writing() {
            v1_20_3::write_stack(v.as_ref(), w);
            return;
        }
        if v1_21_1::writing() {
            v1_21_1::write_stack(v.as_ref(), w);
            return;
        }
        if v1_21_4::writing() {
            v1_21_4::write_stack(v.as_ref(), w);
            return;
        }
        let Some(s) = v else {
            w.varint(0);
            return;
        };
        w.varint(s.count);
        w.varint(ids::item(s.item));
        // The patch as read holds the pinned version's component ids: written again from the values when
        // they are being renumbered.
        if let Some(patch) = &s.wire.0
            && ids::active().is_none()
        {
            w.bytes(patch);
            return;
        }
        // The delimited form carries the same tooltip synthesis as the plain
        // patch: the marker has no wire id of its own, so it becomes the two
        // hide booleans at its position, and each flagged component's show
        // flag rides inside its length-prefixed value.
        let has = |kind: i32| ids::component(kind) >= 0;
        let pin_tooltip = ids::quirks_written().components_before_1_21_5;
        let display = s
            .added
            .iter()
            .find(|(k, _)| *k == component::TOOLTIP_DISPLAY)
            .and_then(|(_, c)| match c {
                Component::TooltipDisplay { hide, hidden } => Some((*hide, hidden.as_slice())),
                _ => None,
            });
        let (hide, hidden) = display.unwrap_or((false, &[]));
        let additional =
            pin_tooltip && hidden.iter().any(|k| v1_21_4::ADDITIONAL.contains(k));
        let booleans = pin_tooltip && display.is_some() && (hide || additional);
        let added_count = s
            .added
            .iter()
            .filter(|(k, _)| *k != component::TOOLTIP_DISPLAY && has(*k))
            .count()
            + usize::from(booleans && additional)
            + usize::from(booleans && hide);
        let removed_marker = pin_tooltip && s.removed.contains(&component::TOOLTIP_DISPLAY);
        let removed_count = s
            .removed
            .iter()
            .filter(|k| **k != component::TOOLTIP_DISPLAY && has(**k))
            .count()
            + usize::from(removed_marker) * 2;
        w.varint(added_count as i32);
        w.varint(removed_count as i32);
        for (kind, c) in &s.added {
            if *kind == component::TOOLTIP_DISPLAY {
                if booleans {
                    if additional {
                        w.varint(ids::component(component::HIDE_ADDITIONAL_TOOLTIP));
                        w.byte_array(&[]);
                    }
                    if hide {
                        w.varint(ids::component(component::HIDE_TOOLTIP));
                        w.byte_array(&[]);
                    }
                }
                continue;
            }
            if !has(*kind) {
                continue;
            }
            w.varint(ids::component(*kind));
            let mut value = Writer::new();
            write_value(*kind, c, &mut value);
            if pin_tooltip && v1_21_4::TOOLTIP_FLAGS.contains(kind) {
                value.bool(!hidden.contains(kind));
            }
            w.byte_array(&value.data);
        }
        for kind in s
            .removed
            .iter()
            .filter(|k| **k != component::TOOLTIP_DISPLAY && has(**k))
        {
            w.varint(ids::component(*kind));
        }
        if removed_marker {
            w.varint(ids::component(component::HIDE_ADDITIONAL_TOOLTIP));
            w.varint(ids::component(component::HIDE_TOOLTIP));
        }
    }
}

/// ItemStackTemplate.STREAM_CODEC: the item, the count, then the patch.
pub struct Template;
impl Codec<Stack> for Template {
    fn read(r: &mut Reader) -> Result<Stack> {
        if ids::quirks().count_first_templates {
            return read_stack(r, false)?.ok_or_else(|| crate::Error::new("empty item template"));
        }
        read_template(r)
    }
    fn write(s: &Stack, w: &mut Writer) {
        if ids::quirks_written().count_first_templates {
            write_slot(Some(s), w);
            return;
        }
        write_template(s, w);
    }
}

/// HashedStack: what the client believes a slot holds. Component values are
/// sent as hashes ([`Hashed::create`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Hashed {
    pub item: i32,
    pub count: i32,
    pub added: Vec<(i32, i32)>,
    pub removed: Vec<i32>,
}
impl Hashed {
    /// The stack hashed without registry names: only components that hold no
    /// registry id get a hash (see [`Hashed::create`]).
    pub fn of(stack: &Stack) -> Self {
        Self::create(stack, &NoNames)
    }
}
pub struct HashedSlot;
impl Codec<Option<Hashed>> for HashedSlot {
    fn read(r: &mut Reader) -> Result<Option<Hashed>> {
        if !r.bool()? {
            return Ok(None);
        }
        let item = r.varint()?;
        let count = r.varint()?;
        let n = r.count(256)?;
        let added = (0..n)
            .map(|_| Ok((r.varint()?, r.i32()?)))
            .collect::<Result<_>>()?;
        let n = r.count(256)?;
        let removed = (0..n).map(|_| r.varint()).collect::<Result<_>>()?;
        Ok(Some(Hashed {
            item,
            count,
            added,
            removed,
        }))
    }
    fn write(v: &Option<Hashed>, w: &mut Writer) {
        w.bool(v.is_some());
        if let Some(h) = v {
            w.varint(ids::item(h.item));
            w.varint(h.count);
            w.varint(h.added.len() as i32);
            for (k, hash) in &h.added {
                w.varint(ids::component(*k));
                w.i32(*hash);
            }
            w.varint(h.removed.len() as i32);
            for k in &h.removed {
                w.varint(ids::component(*k));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::literal;

    fn round<T: PartialEq + std::fmt::Debug, C: Codec<T>>(v: T) {
        let mut w = Writer::new();
        C::write(&v, &mut w);
        let mut r = Reader::new(&w.data);
        assert_eq!(C::read(&mut r).unwrap(), v);
        assert_eq!(r.remaining(), 0);
    }

    /// A stack holding bundles `depth` deep, each the only stack in the one around it.
    fn nested_bundles(depth: usize) -> Vec<u8> {
        // count (or item) 1, item (or count) 1, one added component, none removed: bundle_contents, one stack.
        let level = [1, 1, 1, 0, component::BUNDLE_CONTENTS as u8, 1];
        let mut bytes = level.repeat(depth);
        bytes.extend([1, 1, 0, 0]);
        bytes
    }

    /// Decodes `bytes` as a slot on a thread with the 2 MiB stack network threads get.
    fn read_on_network_stack(bytes: Vec<u8>) -> Result<Option<Stack>> {
        std::thread::Builder::new()
            .stack_size(2 << 20)
            .spawn(move || Slot::read(&mut Reader::new(&bytes)))
            .unwrap()
            .join()
            .unwrap()
    }

    #[test]
    fn items_nested_without_end_are_refused_instead_of_overflowing_the_stack() {
        // About 1.2 MB, under the 2 MiB frame cap: without a limit this aborts the process.
        assert!(read_on_network_stack(nested_bundles(200_000)).is_err());
        // A potion whose effect hides an effect that hides another, 200,000 times over.
        let mut potion = vec![1, 1, 1, 0, component::POTION_CONTENTS as u8, 0, 0, 1, 5];
        potion.extend([0, 1, 0, 0, 0, 1].repeat(200_000));
        potion.extend([0, 1, 0, 0, 0, 0, 0]);
        assert!(read_on_network_stack(potion).is_err());
        // (A third case nested can_break predicates through their exact
        // component matchers; the pin predates matchers, so it has no form
        // here. The bundle and potion cases still bound the nesting.)
    }

    #[test]
    fn deeply_nested_bundles_still_decode() {
        let bytes = nested_bundles(MAX_NESTING as usize);
        let mut stack = read_on_network_stack(bytes.clone()).unwrap().unwrap();
        let mut depth = 0;
        while let Some(Component::Stacks(inner)) = stack.component(component::BUNDLE_CONTENTS) {
            stack = inner[0].clone();
            depth += 1;
        }
        assert_eq!(depth, MAX_NESTING);
        // A refused stack leaves nothing counted behind on its thread.
        assert!(Slot::read(&mut Reader::new(&nested_bundles(MAX_NESTING as usize + 1))).is_err());
        assert!(Slot::read(&mut Reader::new(&bytes)).unwrap().is_some());
    }

    #[test]
    fn a_wire_patch_tells_what_it_adds() {
        // Two added components (damage 3, then can_break's bytes as read)
        // and one removed.
        let mut w = Writer::new();
        w.varint(2);
        w.varint(1);
        w.varint(component::DAMAGE);
        w.byte_array(&[3]);
        w.varint(component::CAN_BREAK);
        w.byte_array(&[1, 0, 0, 0, 0, 0]);
        w.varint(component::ENCHANTMENTS);
        assert!(WirePatch::adds(&w.data, component::DAMAGE));
        assert!(WirePatch::adds(&w.data, component::CAN_BREAK));
        assert!(!WirePatch::adds(&w.data, component::ENCHANTMENTS));
        assert!(!WirePatch::adds(&[], component::CAN_BREAK));
    }

    #[test]
    fn potion_contents_round_trip() {
        let mut arrow = Stack::new(7, 3);
        arrow.added.push((
            component::POTION_CONTENTS,
            Component::Potion {
                potion: Some(31),
                color: None,
                rest: Vec::new(),
            },
        ));
        round::<Option<Stack>, Slot>(Some(arrow));
        // A custom colour, one custom effect (speed II for 600 ticks, shown,
        // no hidden effect) survive as they came. The pin's potion has no
        // custom name (a 1.21.2 addition); a newer server's name is covered
        // below, read under its own numbering and stripped on the way back.
        let mut w = Writer::new();
        w.bool(false);
        w.bool(true);
        w.i32(0x00FF_8800);
        w.varint(1);
        for v in [1, 1, 600] {
            w.varint(v);
        }
        for b in [false, true, true, false] {
            w.bool(b);
        }
        let mut r = Reader::new(&w.data);
        let read = Component::read(component::POTION_CONTENTS, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        let Component::Potion {
            potion,
            color,
            rest,
        } = &read
        else {
            panic!("{read:?}");
        };
        assert_eq!((*potion, *color), (None, Some(0x00FF_8800)));
        assert!(!rest.is_empty());
        let mut again = Writer::new();
        read.write(component::POTION_CONTENTS, &mut again);
        assert_eq!(again.data, w.data);
        // The same potion as a 1.21.11 server sends it, with a custom name:
        // the name survives in the tail, and writing it for the pin drops it.
        let mut named = w.data.clone();
        named.push(1);
        named.extend_from_slice(&[3, b'T', b'e', b'a']);
        let read = {
            let _up = ids::renumbering(
                crate::adapter::ServerVersion::V1_21_11.ids().unwrap(),
                ids::Way::Up,
            );
            let mut r = Reader::new(&named);
            Component::read(component::POTION_CONTENTS, &mut r).unwrap()
        };
        let mut pin = Writer::new();
        read.write(component::POTION_CONTENTS, &mut pin);
        assert_eq!(pin.data, w.data);
        let mut back = Writer::new();
        {
            let _down = ids::renumbering(
                crate::adapter::ServerVersion::V1_21_11.ids().unwrap(),
                ids::Way::Down,
            );
            read.write(component::POTION_CONTENTS, &mut back);
        }
        assert_eq!(back.data, named);
    }

    #[test]
    fn stacks_round_trip() {
        round::<Option<Stack>, Slot>(None);
        round::<Option<Stack>, Slot>(Some(Stack::new(42, 64)));
        let mut sword = Stack::new(900, 1);
        sword.added.push((component::DAMAGE, Component::Int(17)));
        sword.added.push((
            component::CUSTOM_NAME,
            Component::Text(literal("Excalibur")),
        ));
        sword.added.push((
            component::ENCHANTMENTS,
            Component::Enchantments(vec![(5, 3), (12, 1)]),
        ));
        sword
            .added
            .push((component::DYED_COLOR, Component::Int(-6265536)));
        sword.added.push((
            component::LORE,
            Component::Lore(vec![literal("a"), literal("b")]),
        ));
        sword.removed.push(component::REPAIR_COST);
        let mut book = Stack::new(1100, 1);
        book.added.push((
            component::WRITTEN_BOOK_CONTENT,
            Component::WrittenBook(WrittenBook {
                title: "Notes".into(),
                author: "Guest".into(),
                generation: 1,
                pages: vec![literal("one"), literal("two")],
                pages_filtered: vec![None, Some(literal("2"))],
                title_filtered: Some("N".into()),
                resolved: true,
            }),
        ));
        book.added
            .push((component::MAP_POST_PROCESSING, Component::Int(1)));
        round::<Option<Stack>, Slot>(Some(book));
        // OminousBottleAmplifier.STREAM_CODEC: a VarInt.
        let mut bottle = Stack::new(1200, 1);
        bottle
            .added
            .push((component::OMINOUS_BOTTLE_AMPLIFIER, Component::Int(3)));
        let mut w = Writer::new();
        Slot::write(&Some(bottle.clone()), &mut w);
        assert_eq!(w.data, [1, 0xb0, 0x09, 1, 0, 41, 3]);
        round::<Option<Stack>, Slot>(Some(bottle));
        let mut quill = Stack::new(1099, 1);
        quill.added.push((
            component::WRITABLE_BOOK_CONTENT,
            Component::WritableBook {
                pages: vec!["a".into(), String::new()],
                filtered: vec![None, Some("b".into())],
            },
        ));
        round::<Option<Stack>, Slot>(Some(quill));
        assert_eq!(sword.damage(), 17);
        round::<Option<Stack>, Slot>(Some(sword.clone()));
        round::<Option<Hashed>, HashedSlot>(Some(Hashed::of(&sword)));
        round::<Option<Hashed>, HashedSlot>(None);
    }

    #[test]
    fn fireworks_components_round_trip() {
        let mut rocket = Stack::new(42, 1);
        rocket.added.push((
            component::FIREWORKS,
            Component::Fireworks {
                flight: 3,
                explosions: vec![
                    FireworkExplosion {
                        shape: 4,
                        colors: vec![0xff0000, 0x00ff00],
                        fade: vec![0x0000ff],
                        trail: true,
                        twinkle: false,
                    },
                    FireworkExplosion {
                        shape: 0,
                        colors: vec![],
                        fade: vec![],
                        trail: false,
                        twinkle: true,
                    },
                ],
            },
        ));
        rocket.added.push((
            component::FIREWORK_EXPLOSION,
            Component::FireworkExplosion(FireworkExplosion {
                shape: 2,
                colors: vec![7],
                fade: vec![],
                trail: false,
                twinkle: false,
            }),
        ));
        round::<Option<Stack>, Slot>(Some(rocket));
    }

    #[test]
    fn charged_projectiles_are_item_stack_templates() {
        // A crossbow (id 7) loaded with one arrow (id 9): ChargedProjectiles
        // is a list of ItemStackTemplate, count-first in 1.21.1 (the item
        // before the count arrived in 1.21.9).
        let bytes = [1, 7, 1, 0, 29, 1, 1, 9, 0, 0];
        let stack = Slot::read(&mut Reader::new(&bytes)).unwrap().unwrap();
        assert_eq!(
            stack.component(component::CHARGED_PROJECTILES),
            Some(&Component::Stacks(vec![Stack::new(9, 1)]))
        );
        let mut w = Writer::new();
        Slot::write(&Some(stack), &mut w);
        assert_eq!(w.data, bytes);
    }

    #[test]
    fn bundle_contents_are_item_stack_templates() {
        // A bundle (id 7) holding 32 of item 9: BundleContents is a list of
        // ItemStackTemplate, count-first in 1.21.1, as ChargedProjectiles is.
        let bytes = [1, 7, 1, 0, 30, 1, 32, 9, 0, 0];
        let stack = Slot::read(&mut Reader::new(&bytes)).unwrap().unwrap();
        assert_eq!(
            stack.component(component::BUNDLE_CONTENTS),
            Some(&Component::Stacks(vec![Stack::new(9, 32)]))
        );
        let mut w = Writer::new();
        Slot::write(&Some(stack), &mut w);
        assert_eq!(w.data, bytes);
    }

    #[test]
    fn filtered_book_pages_are_read_and_kept() {
        // A writable book with one page "hi" and its filtered "**".
        let bytes = [1, 7, 1, 0, 33, 1, 2, b'h', b'i', 1, 2, b'*', b'*'];
        let stack = Slot::read(&mut Reader::new(&bytes)).unwrap().unwrap();
        assert_eq!(
            stack.component(component::WRITABLE_BOOK_CONTENT),
            Some(&Component::WritableBook {
                pages: vec!["hi".into()],
                filtered: vec![Some("**".into())],
            })
        );
    }

    #[test]
    fn empty_stack_is_a_zero_count() {
        let mut w = Writer::new();
        Slot::write(&None, &mut w);
        assert_eq!(w.data, [0]);
        let mut w = Writer::new();
        Slot::write(&Some(Stack::new(1, 2)), &mut w);
        assert_eq!(w.data, [2, 1, 0, 0]);
    }

    #[test]
    fn unknown_components_are_errors() {
        // 96 is past DataComponents' last id (container_loot, 66).
        let bytes = [1, 5, 1, 0, 96];
        assert!(Slot::read(&mut Reader::new(&bytes)).is_err());
    }

    #[test]
    fn profiles_round_trip() {
        // 1.21.4's profile carries no skin patch on the wire (it arrived in
        // 1.21.11), so the round-tripped profile has the default skin.
        let mut sword = Stack::new(900, 1);
        sword.added.push((
            component::PROFILE,
            Component::Profile(Profile {
                name: Some("Player53470".into()),
                id: Some(0x7a5d_6a1a_30d7_4187_88c5_efe9_1201_8922),
                properties: vec![("textures".into(), "dGV4dHVyZXM=".into(), None)],
                skin: SkinPatch::default(),
            }),
        ));
        round::<Option<Stack>, Slot>(Some(sword));
    }

    #[test]
    fn a_partial_profile_decodes_from_the_wire() {
        // count 1, item 1, one added component of kind 57 (profile): a partial
        // profile with an empty name, a UUID and one unsigned texture
        // property. 1.21.4's profile is optional fields only — no Either
        // tag, no skin patch (those arrived in 1.21.11).
        let uuid = [
            0x7a, 0x5d, 0x6a, 0x1a, 0x30, 0xd7, 0x41, 0x87, 0x88, 0xc5, 0xef, 0xe9, 0x12, 0x01,
            0x89, 0x22,
        ];
        let mut bytes = vec![1, 1, 1, 0, 47, 1, 0, 1];
        bytes.extend_from_slice(&uuid);
        bytes.extend_from_slice(&[1, 8]);
        bytes.extend_from_slice(b"textures");
        bytes.extend_from_slice(&[1, b'x', 0]);
        let mut r = Reader::new(&bytes);
        let stack = Slot::read(&mut r).unwrap().unwrap();
        assert_eq!(r.remaining(), 0);
        let Some(Component::Profile(profile)) = stack.component(component::PROFILE) else {
            panic!("the profile component did not decode");
        };
        assert_eq!(profile.name.as_deref(), Some(""));
        assert_eq!(profile.id, Some(0x7a5d_6a1a_30d7_4187_88c5_efe9_1201_8922));
        assert_eq!(
            profile.properties,
            [("textures".to_owned(), "x".to_owned(), None)]
        );
    }

    #[test]
    fn direct_trim_decodes_complete_official_inventory_stack_payload() {
        // Slot portion of our actual Java26.3 ClientboundContainerSetSlotPacket,
        // re-based to the pinned 1.21.1 component numbering (the trim kind byte)
        // with the trim's trailing show flag, which the codec carries;
        // complete packet and official client decode retained with H284 proof.
        let hex = "01bd080200000a0300046b6565700000011c0023000e6d696e6563726166743a676f6c6408000f496e6c696e65206d6174657269616c000f6d696e6563726166743a636f61737408000e496e6c696e65207061747465726e0001";
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let mut reader = Reader::new(&bytes);
        let stack = Slot::read(&mut reader).unwrap();
        assert_eq!(reader.remaining(), 0);
        let component = stack.as_ref().unwrap().component(component::TRIM).unwrap();
        assert_eq!(
            component,
            &Component::TrimDirect {
                material: TrimHolder::Direct(TrimMaterial {
                    palette_id: "minecraft:gold".into(),
                    description: Tag::String("Inline material".into()),
                }),
                pattern: TrimHolder::Direct(TrimPattern {
                    asset_id: "minecraft:coast".into(),
                    description: Tag::String("Inline pattern".into()),
                    decal: false,
                }),
            }
        );
        let mut writer = Writer::new();
        Slot::write(&stack, &mut writer);
        assert_eq!(writer.data, bytes);
        // Existing direct HashOps path is reused unchanged.
        let mut raw = Writer::new();
        component.write(component::TRIM, &mut raw);
        let code = component.code(component::TRIM, &NoNames);
        assert!(code.is_some());
        assert_eq!(
            code,
            Component::Raw(raw.data).code(component::TRIM, &NoNames)
        );
    }

    #[test]
    fn components_read_past_keep_their_bytes() {
        // Vanilla 26.3's encoding (tools/test_item_wire_parity.py), re-based to the
        // pinned 1.21.4 numbering, of a chestplate's
        // armour trim (decoded: both holders are references), a banner's patterns
        // (one direct), a pot's sherds and a stew's effects: one server's
        // inventory, recipe book and advancements had them. The pot's value is
        // in 1.21.4's own form (four item ids: brick, angler, brick, skull).
        // The trim carries its show flag (a tooltip-flagged component).
        for (hex, kind) in [
            ("01bd08010023050201", 35),
            ("01890b01003102200e000b6d696e6563726166743a7801790b", 49),
            ("01880301003304ba07bd0aba07d20a", 51),
            ("01a408010020020f6412a001", 32),
            // A goat horn's instrument by reference (InstrumentComponent's holder).
            // 1.21.1's instrument sends its holder bare (the EitherHolder
            // wrapper started in 1.21.5).
            ("01bd0801002802", 40),
        ] {
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let mut r = Reader::new(&bytes);
            let stack = Slot::read(&mut r).unwrap();
            assert_eq!(r.remaining(), 0, "component {kind}");
            let expected = |c: &Component| match kind {
                35 => {
                    *c == Component::Trim {
                        material: 4,
                        pattern: 1,
                    }
                }
                // night_vision for 100 ticks, poison for the default 160.
                32 => *c == Component::StewEffects(vec![(15, 100), (18, 160)]),
                40 => *c == Component::Instrument(1),
                _ => matches!(c, Component::Raw(_)),
            };
            assert!(
                stack
                    .as_ref()
                    .and_then(|s| s.component(kind))
                    .is_some_and(expected)
            );
            let mut w = Writer::new();
            Slot::write(&stack, &mut w);
            assert_eq!(w.data, bytes, "component {kind}");
        }
    }

    #[test]
    fn negative_holder_ids_and_holder_set_sizes_are_refused() {
        // Vanilla 26.3's ByteBufCodecs.holder looks a nonzero id up with
        // `byIdOrThrow(id - 1)`, and holderSet sizes its list with `new ArrayList(size)`:
        // neither takes a negative value. `1` is a valid value at the same place.
        let slot = |kind: i32, component: &[u8]| {
            // Count 1, item 1, one added component, none removed.
            let mut w = Writer::new();
            for v in [1, 1, 1, 0, kind] {
                w.varint(v);
            }
            w.data.extend_from_slice(component);
            let mut r = Reader::new(&w.data);
            Slot::read(&mut r).map(|stack| (stack, r.remaining()))
        };
        for (value, valid) in [(1, true), (-1, false), (i32::MIN, false)] {
            let mut v = Writer::new();
            v.varint(value);
            // instrument (40): a bare holder (`value`).
            // can_break: one predicate whose blocks are a holder set (`value`, an empty
            // list when 1), with no properties or NBT, and its trailing show
            // flag (a tooltip-flagged component). The pin's predicates carry
            // no component matchers (a 1.21.5 addition).
            let mut can_break = vec![1, 1];
            can_break.extend_from_slice(&v.data);
            can_break.extend([0, 0, 1]);
            for (kind, bytes) in [(component::INSTRUMENT, v.data.clone()), (component::CAN_BREAK, can_break)] {
                let read = slot(kind, &bytes);
                if valid {
                    assert!(
                        matches!(read, Ok((Some(_), 0))),
                        "component {kind}: {read:?}"
                    );
                } else {
                    assert!(read.is_err(), "component {kind} with {value}: {read:?}");
                }
            }
        }
    }

    #[test]
    fn painting_variants_decode_by_reference() {
        // Holder<PaintingVariant>: a reference (its id plus one) is typed; a
        // direct variant (0, width, height, asset id, no title, no author)
        // is read past. Both encode back to the same bytes.
        let asset = b"minecraft:x";
        let mut direct = vec![1u8, 7, 1, 0, 89, 0, 2, 1, asset.len() as u8];
        direct.extend_from_slice(asset);
        direct.extend_from_slice(&[0, 0]);
        for (bytes, expected) in [
            (
                vec![1u8, 7, 1, 0, 89, 5],
                Some(Component::PaintingVariant(4)),
            ),
            (direct, None),
        ] {
            let mut r = Reader::new(&bytes);
            let stack = Slot::read(&mut r).unwrap();
            assert_eq!(r.remaining(), 0);
            let c = stack
                .as_ref()
                .and_then(|s| s.component(component::PAINTING_VARIANT))
                .unwrap();
            match &expected {
                Some(e) => assert_eq!(c, e),
                None => assert!(matches!(c, Component::Raw(_))),
            }
            let mut w = Writer::new();
            Slot::write(&stack, &mut w);
            assert_eq!(w.data, bytes);
        }
    }
}
