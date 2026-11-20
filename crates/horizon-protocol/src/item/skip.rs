//! Reading past the data components Horizon doesn't interpret: each
//! function consumes exactly what its vanilla StreamCodec writes
//! (DataComponents' `networkSynchronized` codecs; a component without one
//! goes as its persistent Codec's NBT, `ByteBufCodecs.fromCodecWithRegistries`).
//! Item data has no length prefix, so a component read short or long makes
//! everything after it unreadable.
use super::{Component, Nested, Template};
use crate::{
    Error, Result,
    buf::{Codec, Reader},
};

type Read = fn(&mut Reader) -> Result<()>;

/// DataComponents' registry order (1.21.1) for the ids `Component::read`
/// doesn't decode itself. The pin predates 1.21.2, so the shapes here are
/// the pre-split ones: food is the full combined form, the instrument is
/// sound + an int use duration + range, the tool has no
/// canDestroyBlocksInCreative bool, and block predicates have no component
/// matchers. The 1.21.2+ components (consumable, use_remainder,
/// use_cooldown, damage_resistant, repairable, equippable, glider,
/// death_protection, enchantable, item_model, tooltip_style) have no arm:
/// they never reach this function at this pin (`skip_dropped` reads past
/// them, and `older_layout` converts the ones with a pin counterpart).
pub(super) fn component(kind: i32, r: &mut Reader) -> Result<()> {
    match kind {
        // Components without a network codec go as their NBT: intangible_projectile,
        // map_decorations, debug_stick_state, recipes, lock, container_loot.
        19 | 27 | 36 | 43 | 55 | 56 => nbt(r),
        // A VarInt (VAR_INT, an idMapper, a registry or holderRegistry id):
        // ominous_bottle_amplifier, base_color.
        41 | 50 => varint(r),
        // Identifier.STREAM_CODEC: note_block_sound.
        48 => ident(r),
        // AdventureModePredicate: can_place_on, can_break.
        10..=11 => list(r, block_predicate),
        12 => list(r, attribute_modifier),
        // FoodProperties.STREAM_CODEC (before the 1.21.2 split): nutrition,
        // saturation, canAlwaysEat, eatSeconds, an optional remainder
        // template, then the possible effects (an effect and a probability).
        20 => {
            varint(r)?;
            f32(r)?;
            boolean(r)?;
            f32(r)?;
            optional(r, template)?;
            list(r, |r| {
                mob_effect(r)?;
                f32(r)
            })
        }
        22 => tool(r),
        // MapColor: a big-endian int.
        25 => i32(r),
        // BundleContents: templates.
        30 => list(r, template),
        // SuspiciousStewEffects: effect, duration.
        32 => list(r, |r| {
            varint(r)?;
            varint(r)
        }),
        // ArmorTrim: material, then Holder<TrimPattern> (assetId, description, decal).
        35 => {
            trim_material(r)?;
            holder(r, |r| {
                ident(r)?;
                nbt(r)?;
                boolean(r)
            })
        }
        // TypedEntityData (entity_data, block_entity_data): the type id, then a compound.
        37 | 39 => {
            varint(r)?;
            nbt(r)
        }
        // CustomData: bucket_entity_data.
        38 => nbt(r),
        // Instrument: Holder<Instrument> (soundEvent, useDuration in ticks,
        // range). 1.21.2 made the duration a float in seconds and added
        // durabilityDamage and a description; the pin predates both.
        40 => holder(r, |r| {
            sound(r)?;
            varint(r)?;
            f32(r)
        }),
        // JukeboxPlayable: Holder<JukeboxSong> (soundEvent, description, lengthInSeconds,
        // comparatorOutput).
        42 => holder(r, |r| {
            sound(r)?;
            nbt(r)?;
            f32(r)?;
            varint(r)
        }),
        // LodestoneTracker: target (GlobalPos: dimension, BlockPos), tracked.
        44 => {
            optional(r, |r| {
                ident(r)?;
                r.i64().map(drop)
            })?;
            boolean(r)
        }
        45 => firework_explosion(r),
        // Fireworks: flightDuration, explosions.
        46 => {
            varint(r)?;
            list(r, firework_explosion)
        }
        // BannerPatternLayers: Holder<BannerPattern> (assetId, translationKey), colour.
        49 => list(r, |r| {
            holder(r, |r| {
                ident(r)?;
                string(r)
            })?;
            varint(r)
        }),
        // PotDecorations: back, left, right, front. The internal form is
        // four optional templates: `older_layout` rewrites the pin wire's
        // four item ids into these before `Component::read` sees them.
        51 => (0..4).try_for_each(|_| optional(r, template)),
        // ItemContainerContents: optional templates.
        52 => list(r, |r| optional(r, template)),
        // BlockItemStateProperties: a string to string map.
        53 => list(r, |r| {
            string(r)?;
            string(r)
        }),
        // Bees: Occupant (entity type and data, ticksInHive, minTicksInHive).
        54 => list(r, |r| {
            varint(r)?;
            nbt(r)?;
            varint(r)?;
            varint(r)
        }),
        // Holder<PaintingVariant>: width, height, assetId, title, author.
        89 => holder(r, |r| {
            varint(r)?;
            varint(r)?;
            ident(r)?;
            optional(r, nbt)?;
            optional(r, nbt)
        }),
        other => Err(Error::new(format!("unsupported item component {other}"))),
    }
}

fn boolean(r: &mut Reader) -> Result<()> {
    r.bool().map(drop)
}
fn varint(r: &mut Reader) -> Result<()> {
    r.varint().map(drop)
}
fn f32(r: &mut Reader) -> Result<()> {
    r.f32().map(drop)
}
fn i32(r: &mut Reader) -> Result<()> {
    r.i32().map(drop)
}
/// ByteBufCodecs.STRING_UTF8.
fn string(r: &mut Reader) -> Result<()> {
    r.string(32767).map(drop)
}
/// Identifier.STREAM_CODEC, ResourceKey.streamCodec.
fn ident(r: &mut Reader) -> Result<()> {
    string(r)
}
/// A text component or compound (fromCodecWithRegistries, COMPOUND_TAG).
fn nbt(r: &mut Reader) -> Result<()> {
    r.nbt().map(drop)
}
fn template(r: &mut Reader) -> Result<()> {
    Template::read(r).map(drop)
}
/// ByteBufCodecs.optional: a flag, then the value.
fn optional(r: &mut Reader, value: Read) -> Result<()> {
    if r.bool()? { value(r) } else { Ok(()) }
}
/// ByteBufCodecs.list and map: a VarInt count, then the elements (a map's
/// key and value each). Every element is at least a byte, so the count
/// can't pass what is left.
fn list(r: &mut Reader, element: Read) -> Result<()> {
    let n = r.count(r.remaining())?;
    (0..n).try_for_each(|_| element(r))
}
/// ByteBufCodecs.holder: 0 and the value itself, or the registry id plus one.
/// Vanilla looks the id up with `byIdOrThrow(id - 1)`, which refuses a negative one.
fn holder(r: &mut Reader, direct: Read) -> Result<()> {
    match r.varint()? {
        0 => direct(r),
        ..0 => Err(Error::new("negative holder id")),
        _ => Ok(()),
    }
}
/// ByteBufCodecs.holderSet: 0 and a tag id, or the size plus one and the ids.
/// Vanilla's `new ArrayList(size)` refuses a negative size.
pub(super) fn holder_set(r: &mut Reader) -> Result<()> {
    match r.varint()? {
        0 => ident(r),
        n => {
            let n = usize::try_from(n).map_err(|_| Error::new("negative holder set size"))? - 1;
            if n > r.remaining() {
                return Err(Error::new("holder set past the end"));
            }
            (0..n).try_for_each(|_| varint(r))
        }
    }
}
/// SoundEvent.STREAM_CODEC: a holder of a location and an optional fixed range.
pub(super) fn sound(r: &mut Reader) -> Result<()> {
    holder(r, |r| {
        ident(r)?;
        optional(r, f32)
    })
}
/// TrimMaterial.STREAM_CODEC: a holder of a palette id and a description.
fn trim_material(r: &mut Reader) -> Result<()> {
    holder(r, |r| {
        ident(r)?;
        nbt(r)
    })
}
/// MobEffectInstance: the effect id, then its Details.
fn mob_effect(r: &mut Reader) -> Result<()> {
    varint(r)?;
    effect_details(r)
}
/// MobEffectInstance.Details: amplifier, duration, ambient, showParticles, showIcon, then the
/// hidden effect's Details, recursively.
fn effect_details(r: &mut Reader) -> Result<()> {
    let _nested = Nested::enter()?;
    varint(r)?;
    varint(r)?;
    boolean(r)?;
    boolean(r)?;
    boolean(r)?;
    optional(r, effect_details)
}
/// Consumable: consumeSeconds, animation, sound, hasConsumeParticles, onConsumeEffects.
/// Only `skip_dropped` reads this at the pin (consumable is a 1.21.2 addition).
pub(super) fn consumable(r: &mut Reader) -> Result<()> {
    f32(r)?;
    varint(r)?;
    sound(r)?;
    boolean(r)?;
    list(r, consume_effect)
}
/// UseCooldown: seconds, cooldownGroup. Only `skip_dropped` reads this at the pin.
pub(super) fn use_cooldown(r: &mut Reader) -> Result<()> {
    f32(r)?;
    optional(r, ident)
}
/// DeathProtection: deathEffects. Only `skip_dropped` reads this at the pin.
pub(super) fn death_protection(r: &mut Reader) -> Result<()> {
    list(r, consume_effect)
}
fn consume_effect(r: &mut Reader) -> Result<()> {
    match r.varint()? {
        // apply_effects: effects, probability.
        0 => {
            list(r, mob_effect)?;
            f32(r)
        }
        // remove_effects: a mob effect holder set.
        1 => holder_set(r),
        // clear_all_effects.
        2 => Ok(()),
        // teleport_randomly: diameter, directionalParticles.
        3 => {
            f32(r)?;
            boolean(r)
        }
        // play_sound.
        4 => sound(r),
        other => Err(Error::new(format!("unknown consume effect type {other}"))),
    }
}
/// BlockPredicate: blocks, properties, nbt. (1.21.5 added the
/// DataComponentMatchers after the nbt; the pin predates them, and
/// `older_layout` strips them from a newer server's predicates.)
fn block_predicate(r: &mut Reader) -> Result<()> {
    optional(r, holder_set)?;
    block_properties_and_nbt(r)
}
pub(super) fn block_properties_and_nbt(r: &mut Reader) -> Result<()> {
    // StatePropertiesPredicate: PropertyMatchers, each a name and an
    // Either<ExactMatcher (value), RangedMatcher (min, max)>.
    optional(r, |r| {
        list(r, |r| {
            string(r)?;
            if r.bool()? {
                string(r)
            } else {
                optional(r, string)?;
                optional(r, string)
            }
        })
    })?;
    optional(r, nbt)
}
pub(super) fn component_matchers(r: &mut Reader) -> Result<()> {
    // DataComponentExactPredicate: TypedDataComponents, a component type id and its value.
    list(r, |r| {
        let _nested = Nested::enter()?;
        let kind = r.varint()?;
        Component::read(kind, r).map(drop)
    })?;
    // DataComponentPredicate.STREAM_CODEC: an Either of a predicate type id or a component
    // type id, then the predicate as NBT (fromCodecWithRegistries).
    list(r, |r| {
        boolean(r)?;
        varint(r)?;
        nbt(r)
    })
}
/// ItemAttributeModifiers.Entry: attribute, modifier (id, amount, operation), slot, display.
fn attribute_modifier(r: &mut Reader) -> Result<()> {
    varint(r)?;
    ident(r)?;
    r.f64()?;
    varint(r)?;
    varint(r)?;
    // Display.Type: default, hidden, or override and its text.
    match r.varint()? {
        0 | 1 => Ok(()),
        2 => nbt(r),
        other => Err(Error::new(format!(
            "unknown attribute modifier display {other}"
        ))),
    }
}
/// Tool: rules (blocks, speed, correctForDrops), defaultMiningSpeed,
/// damagePerBlock. (1.21.5 added canDestroyBlocksInCreative after the
/// damage; the pin predates it, and `older_layout` strips it from a newer
/// server's tools.)
fn tool(r: &mut Reader) -> Result<()> {
    list(r, |r| {
        holder_set(r)?;
        optional(r, f32)?;
        optional(r, boolean)
    })?;
    f32(r)?;
    varint(r)
}
/// Equippable: slot, equipSound, assetId, cameraOverlay, allowedEntities, dispensable,
/// swappable, damageOnHurt, equipOnInteract, canBeSheared, shearingSound.
/// Only `skip_dropped` reads this at the pin (equippable is a 1.21.2 addition).
pub(super) fn equippable(r: &mut Reader) -> Result<()> {
    varint(r)?;
    sound(r)?;
    optional(r, ident)?;
    optional(r, ident)?;
    optional(r, holder_set)?;
    (0..5).try_for_each(|_| boolean(r))?;
    sound(r)
}
/// BlocksAttacks: blockDelaySeconds, disableCooldownScale, damageReductions (angle, type,
/// base, factor), itemDamage (threshold, base, factor), bypassedBy, blockSound, disableSound.
pub(super) fn blocks_attacks(r: &mut Reader) -> Result<()> {
    f32(r)?;
    f32(r)?;
    list(r, |r| {
        f32(r)?;
        optional(r, holder_set)?;
        f32(r)?;
        f32(r)
    })?;
    (0..3).try_for_each(|_| f32(r))?;
    optional(r, holder_set)?;
    optional(r, sound)?;
    optional(r, sound)
}
/// PotionContents' customEffects and customName (`Component::read` reads
/// the potion and colour before them). The custom name arrived in 1.21.2;
/// `potion_effects` reads the pre-1.21.2 tail (effects only).
pub(super) fn potion_effects_and_name(r: &mut Reader) -> Result<()> {
    list(r, mob_effect)?;
    optional(r, string)
}
/// PotionContents' customEffects without the custom name: the 1.21.1 tail.
pub(super) fn potion_effects(r: &mut Reader) -> Result<()> {
    list(r, mob_effect)
}
/// A MobEffectInstance: the effect id, duration, amplifier, ambient,
/// showParticles, showIcon, and an optional hidden effect (same shape).
pub(super) fn mob_effect_skip(r: &mut Reader) -> Result<()> {
    mob_effect(r)
}
/// FireworkExplosion: shape, colors, fadeColors, hasTrail, hasTwinkle.
fn firework_explosion(r: &mut Reader) -> Result<()> {
    varint(r)?;
    list(r, i32)?;
    list(r, i32)?;
    boolean(r)?;
    boolean(r)
}
