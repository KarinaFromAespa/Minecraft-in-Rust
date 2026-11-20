//! The data components that tune an item's behaviour (`food`, `tool`,
//! `attribute_modifiers`, ...), from their stream form to the codec form.
//! See [`super::raw`] for the rest. The pin predates `consumable`,
//! `use_remainder`, `use_cooldown`, `damage_resistant`, `equippable`,
//! `repairable`, `glider` and `death_protection`, so they have no arm here.
use super::*;
use crate::buf::Reader;
use horizon_world::snbt_text::to_snbt;

/// `EquipmentSlotGroup` by id.
const SLOT_GROUP: [&str; 11] = [
    "any", "mainhand", "offhand", "hand", "feet", "legs", "chest", "head", "armor", "body",
    "saddle",
];
const OPERATION: [&str; 3] = ["add_value", "add_multiplied_base", "add_multiplied_total"];

fn int(r: &mut Reader) -> Option<i32> {
    r.varint().ok()
}
fn float(r: &mut Reader) -> Option<f32> {
    r.f32().ok()
}
fn boolean(r: &mut Reader) -> Option<bool> {
    r.bool().ok()
}
fn ident(r: &mut Reader) -> Option<HashCode> {
    Some(string(&r.string(32767).ok()?))
}
/// `Codec.xxx.optionalFieldOf(name, default)` for a float.
fn float_or(rec: RecordHash, name: &str, v: f32, default: f32) -> RecordHash {
    or_default(
        rec,
        name,
        v.to_bits(),
        default.to_bits(),
        HashOps::create_f32(v),
    )
}
fn int_or(rec: RecordHash, name: &str, v: i32, default: i32) -> RecordHash {
    or_default(rec, name, v, default, HashOps::create_int(v))
}
fn bool_or(rec: RecordHash, name: &str, v: bool, default: bool) -> RecordHash {
    or_default(rec, name, v, default, HashOps::create_boolean(v))
}
fn optional<T>(
    r: &mut Reader,
    element: impl FnOnce(&mut Reader) -> Option<T>,
) -> Option<Option<T>> {
    if boolean(r)? {
        Some(Some(element(r)?))
    } else {
        Some(None)
    }
}
fn list<T>(r: &mut Reader, mut element: impl FnMut(&mut Reader) -> Option<T>) -> Option<Vec<T>> {
    let n = r.count(r.remaining()).ok()?;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(element(r)?);
    }
    Some(out)
}

/// `SoundEvent.STREAM_CODEC`: 0 and the event itself (`DIRECT_CODEC`: `sound_id`
/// and an optional `range`), or the registry id plus one.
pub(super) fn sound(r: &mut Reader, names: &dyn Names) -> Option<HashCode> {
    match int(r)? {
        0 => {
            let id = ident(r)?;
            let range = optional(r, float)?;
            Some(
                RecordHash::new()
                    .field("sound_id", id)
                    .optional("range", range.map(HashOps::create_f32))
                    .build(),
            )
        }
        id => Some(string(&names.name("minecraft:sound_event", id - 1)?)),
    }
}
/// `ByteBufCodecs.holderSet`: 0 and a tag, or the size plus one and the ids.
/// `HolderSetCodec` writes `#tag`, a lone entry as itself, else a list.
fn holder_set(registry: &str, names: &dyn Names, r: &mut Reader) -> Option<HashCode> {
    match int(r)? {
        0 => Some(string(&format!("#{}", r.string(32767).ok()?))),
        n => {
            let n = usize::try_from(n - 1)
                .ok()
                .filter(|n| *n <= r.remaining())?;
            let mut ids = Vec::with_capacity(n);
            for _ in 0..n {
                ids.push(entry(names, registry, int(r)?)?);
            }
            Some(if let [only] = ids[..] {
                only
            } else {
                HashOps::create_list(ids)
            })
        }
    }
}
/// `ByteBufCodecs.holderRegistry`.
fn holder(registry: &str, names: &dyn Names, r: &mut Reader) -> Option<HashCode> {
    entry(names, registry, int(r)?)
}

/// `BlockPredicate.STREAM_CODEC` as `BlockPredicate.CODEC` writes it. The
/// pin's predicates end at the nbt predicate: the component matchers
/// (exact, partial) arrived in 1.21.5, so a newer server's are dropped on
/// read and no hash of them exists here.
fn block_predicate(r: &mut Reader, names: &dyn Names) -> Option<HashCode> {
    let blocks = optional(r, |r| holder_set("minecraft:block", names, r))?;
    // StatePropertiesPredicate: a name and an exact value or a (min, max) range.
    let state = optional(r, |r| {
        let props = list(r, |r| {
            let name = string(&r.string(32767).ok()?);
            let matcher = if boolean(r)? {
                string(&r.string(32767).ok()?)
            } else {
                let min = optional(r, |r| Some(string(&r.string(32767).ok()?)))?;
                let max = optional(r, |r| Some(string(&r.string(32767).ok()?)))?;
                RecordHash::new()
                    .optional("min", min)
                    .optional("max", max)
                    .build()
            };
            Some((name, matcher))
        })?;
        Some(HashOps::create_map(props))
    })?;
    // NbtPredicate: `TagParser.LENIENT_CODEC` encodes with its flattened codec,
    // the tag's SNBT string.
    let nbt = optional(r, |r| match r.nbt().ok()? {
        tag @ horizon_world::nbt::Tag::Compound(_) => Some(string(&to_snbt(&tag))),
        _ => None,
    })?;
    Some(
        RecordHash::new()
            .optional("blocks", blocks)
            .optional("state", state)
            .optional("nbt", nbt)
            .build(),
    )
}

/// The hash of component `kind` from its stream, or `None`.
pub(super) fn code(kind: i32, r: &mut Reader, names: &dyn Names) -> Option<HashCode> {
    Some(match kind {
        // ItemAttributeModifiers: attribute, modifier, slot, display.
        12 => HashOps::create_list(list(r, |r| {
            let attribute = holder("minecraft:attribute", names, r)?;
            let id = ident(r)?;
            let amount = r.f64().ok()?;
            let operation = pick(&OPERATION, int(r)?)?;
            let slot = int(r)?;
            let rec = RecordHash::new()
                .field("type", attribute)
                .field("id", id)
                .field("amount", HashOps::create_double(amount.to_bits()))
                .field("operation", operation);
            let rec = or_default(rec, "slot", slot, 0, pick(&SLOT_GROUP, slot)?);
            Some(
                match int(r)? {
                    0 => rec,
                    1 => rec.field(
                        "display",
                        RecordHash::new().field("type", string("hidden")).build(),
                    ),
                    2 => rec.field(
                        "display",
                        RecordHash::new()
                            .field("type", string("override"))
                            .field("value", text(&r.nbt().ok()?))
                            .build(),
                    ),
                    _ => return None,
                }
                .build(),
            )
        })?),
        // FoodProperties.DIRECT_CODEC: nutrition, saturation, canAlwaysEat,
        // eatSeconds, usingConvertsTo, effects. The pin's food is the combined
        // form (1.21.2 split the last three fields into `consumable`); a newer
        // server's food arrives here expanded with the codec defaults, so it
        // hashes the same as the split form's food.
        20 => {
            let nutrition = int(r)?;
            let saturation = float(r)?;
            let can_always_eat = boolean(r)?;
            let seconds = float(r)?;
            let converts = optional(r, |r| super::raw::template_code(r, names))?;
            let effects = list(r, |r| {
                let effect = super::raw::mob_effect(r, names)?;
                let probability = float(r)?;
                Some(
                    float_or(
                        RecordHash::new().field("effect", effect),
                        "probability",
                        probability,
                        1.0,
                    )
                    .build(),
                )
            })?;
            let rec = RecordHash::new()
                .field("nutrition", HashOps::create_int(nutrition))
                .field("saturation", HashOps::create_f32(saturation));
            let rec = bool_or(rec, "can_always_eat", can_always_eat, false);
            let rec = float_or(rec, "eat_seconds", seconds, 1.6);
            let rec = rec.optional("using_converts_to", converts);
            if effects.is_empty() {
                rec
            } else {
                rec.field("effects", HashOps::create_list(effects))
            }
            .build()
        }
        // Tool: rules, defaultMiningSpeed, damagePerBlock. The pin predates
        // canDestroyBlocksInCreative (1.21.5); a newer server's tool arrives
        // here with the flag stripped, so only a default-true flag hashes
        // the same as the newer form.
        22 => {
            let rules = list(r, |r| {
                let blocks = holder_set("minecraft:block", names, r)?;
                let speed = optional(r, float)?;
                let drops = optional(r, boolean)?;
                Some(
                    RecordHash::new()
                        .field("blocks", blocks)
                        .optional("speed", speed.map(HashOps::create_f32))
                        .optional("correct_for_drops", drops.map(HashOps::create_boolean))
                        .build(),
                )
            })?;
            let (mining, damage) = (float(r)?, int(r)?);
            let rec = RecordHash::new().field("rules", HashOps::create_list(rules));
            let rec = float_or(rec, "default_mining_speed", mining, 1.0);
            int_or(rec, "damage_per_block", damage, 1).build()
        }
        // `JukeboxSong.CODEC` is a `RegistryFixedCodec`: a direct song cannot be
        // encoded (vanilla's client would throw), so only a reference hashes.
        42 => match int(r)? {
            0 => return None,
            id => entry(names, "minecraft:jukebox_song", id - 1)?,
        },
        // AdventureModePredicate (can_place_on, can_break): `compactListCodec`
        // writes one predicate as itself and `nonEmptyList` refuses none.
        10 | 11 => {
            let mut predicates = list(r, |r| block_predicate(r, names))?;
            match predicates.len() {
                0 => return None,
                1 => predicates.remove(0),
                _ => HashOps::create_list(predicates),
            }
        }
        _ => return None,
    })
}
