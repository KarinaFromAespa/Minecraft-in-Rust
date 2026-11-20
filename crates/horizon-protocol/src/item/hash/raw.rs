//! The components Horizon keeps as their stream bytes ([`Component::Raw`]),
//! decoded into the `Codec` form that vanilla's `HashOps` sees. Each function
//! reads exactly what the component's `STREAM_CODEC` writes (the readers in
//! `item/skip.rs` step over the same bytes) and builds what its `CODEC`
//! encodes: a component without a `networkSynchronized` stream codec goes as
//! its codec's NBT, so that NBT converts to hashes as it is (`NbtOps.convertTo`).
use super::*;
use crate::buf::Reader;
use crate::item::Profile;
use horizon_world::nbt::Tag;

/// `DyeColor` names by id.
fn dye(id: i32) -> Option<HashCode> {
    pick(&DYE, id)
}
/// The `StringRepresentable` names of an enum whose stream codec is an `idMapper`
/// of the ids `0..`.
fn named(table: &[&str], r: &mut Reader) -> Option<HashCode> {
    pick(table, r.varint().ok()?)
}
/// `ByteBufCodecs.holderRegistry`: the registry id itself; the codec writes the
/// entry's identifier.
fn registry_entry(registry: &str, names: &dyn Names, r: &mut Reader) -> Option<HashCode> {
    entry(names, registry, r.varint().ok()?)
}

const FOX: [&str; 2] = ["red", "snow"];
const SALMON: [&str; 3] = ["small", "medium", "large"];
const PARROT: [&str; 5] = ["red_blue", "blue", "green", "yellow_blue", "gray"];
const MOOSHROOM: [&str; 2] = ["red", "brown"];
const HORSE: [&str; 7] = [
    "white",
    "creamy",
    "chestnut",
    "brown",
    "black",
    "gray",
    "dark_brown",
];
const LLAMA: [&str; 4] = ["creamy", "white", "brown", "gray"];
const AXOLOTL: [&str; 5] = ["lucy", "wild", "gold", "cyan", "blue"];
const RABBIT: [&str; 6] = ["brown", "white", "black", "white_splotched", "gold", "salt"];
/// `TropicalFish.Pattern` by packed id (`base | index << 8`).
const TROPICAL: [(i32, &str); 12] = [
    (0, "kob"),
    (256, "sunstreak"),
    (512, "snooper"),
    (768, "dasher"),
    (1024, "brinely"),
    (1280, "spotty"),
    (1, "flopper"),
    (257, "stripey"),
    (513, "glitter"),
    (769, "blockfish"),
    (1025, "betty"),
    (1281, "clayfish"),
];

/// `TypedEntityData.codec`: the NBT with the type's identifier as `id`.
fn typed_data(registry: &str, names: &dyn Names, r: &mut Reader) -> Option<HashCode> {
    let id = registry_entry(registry, names, r)?;
    let Tag::Compound(mut tag) = r.nbt().ok()? else {
        return None;
    };
    // `stripId` leaves the data without its own id; the codec writes the type's.
    tag.remove(&"id".into());
    Some(typed_with_id(id, &tag))
}
fn typed_with_id(id: HashCode, tag: &horizon_world::nbt::Compound) -> HashCode {
    let mut entries: Vec<(HashCode, HashCode)> = tag
        .iter()
        .map(|(k, v)| {
            (
                HashOps::create_string_units(&k.0),
                HashOps::from_nbt(v, &[]),
            )
        })
        .collect();
    entries.push((string("id"), id));
    HashOps::create_map(entries)
}
/// A list written as a count and its elements, hashed as a list.
fn list(
    r: &mut Reader,
    mut element: impl FnMut(&mut Reader) -> Option<HashCode>,
) -> Option<HashCode> {
    let n = r.count(r.remaining()).ok()?;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(element(r)?);
    }
    Some(HashOps::create_list(out))
}
fn optional(
    r: &mut Reader,
    element: impl FnOnce(&mut Reader) -> Option<HashCode>,
) -> Option<Option<HashCode>> {
    if r.bool().ok()? {
        Some(Some(element(r)?))
    } else {
        Some(None)
    }
}
pub(super) fn template_code(r: &mut Reader, names: &dyn Names) -> Option<HashCode> {
    // The held bytes are the internal template form, not the wire form:
    // read_template, not Template::read, which would read a count-first
    // slot for the pinned 1.21.8.
    let stack = super::super::read_template(r).ok()?;
    template(&stack, names)
}
/// `MobEffectInstance.Details.MAP_CODEC` (the fields of a record, flattened into
/// `rec`) from `STREAM_CODEC`: amplifier, duration, ambient, show_particles,
/// show_icon, then the hidden effect's details.
fn effect_details(rec: RecordHash, r: &mut Reader) -> Option<RecordHash> {
    let amplifier = r.varint().ok()?;
    let duration = r.varint().ok()?;
    let ambient = r.bool().ok()?;
    let particles = r.bool().ok()?;
    let icon = r.bool().ok()?;
    // `ExtraCodecs.UNSIGNED_BYTE` is a byte tag.
    let rec = or_default(
        rec,
        "amplifier",
        amplifier,
        0,
        HashOps::create_byte(amplifier as i8),
    );
    let rec = or_default(rec, "duration", duration, 0, HashOps::create_int(duration));
    let rec = or_default(
        rec,
        "ambient",
        ambient,
        false,
        HashOps::create_boolean(ambient),
    );
    let rec = or_default(
        rec,
        "show_particles",
        particles,
        true,
        HashOps::create_boolean(particles),
    );
    // `show_icon` is always written.
    let rec = rec.field("show_icon", HashOps::create_boolean(icon));
    Some(if r.bool().ok()? {
        let hidden = effect_details(RecordHash::new(), r)?.build();
        rec.field("hidden_effect", hidden)
    } else {
        rec
    })
}
/// `MobEffectInstance.CODEC`: the effect's identifier and its details.
pub(super) fn mob_effect(r: &mut Reader, names: &dyn Names) -> Option<HashCode> {
    let rec = RecordHash::new().field("id", registry_entry("minecraft:mob_effect", names, r)?);
    Some(effect_details(rec, r)?.build())
}

/// `PotionContents` after its potion and colour: the custom effects and name
/// (`custom_effects` is absent when empty).
pub(super) fn potion_tail(rec: RecordHash, rest: &[u8], names: &dyn Names) -> Option<RecordHash> {
    let mut r = Reader::new(rest);
    let n = r.count(r.remaining()).ok()?;
    let mut effects = Vec::with_capacity(n);
    for _ in 0..n {
        effects.push(mob_effect(&mut r, names)?);
    }
    let name = if r.bool().ok()? {
        Some(string(&r.string(32767).ok()?))
    } else {
        None
    };
    (r.remaining() == 0).then_some(())?;
    let rec = if effects.is_empty() {
        rec
    } else {
        rec.field("custom_effects", HashOps::create_list(effects))
    };
    Some(rec.optional("custom_name", name))
}

/// `ResolvableProfile.CODEC` (`FULL_CODEC`): the game profile's or the partial's
/// fields, then the skin patch's. Both forms write `id`, `name` and a
/// `properties` list that is absent when empty.
pub(super) fn profile(p: &Profile) -> HashCode {
    let rec = RecordHash::new()
        .optional("name", p.name.as_deref().map(string))
        .optional(
            "id",
            // `UUIDUtil.CODEC`: an int array.
            p.id.map(|id| {
                HashOps::create_int_list(&[
                    (id >> 96) as i32,
                    (id >> 64) as i32,
                    (id >> 32) as i32,
                    id as i32,
                ])
            }),
        );
    let rec = if p.properties.is_empty() {
        rec
    } else {
        rec.field(
            "properties",
            HashOps::create_list(p.properties.iter().map(|(name, value, signature)| {
                RecordHash::new()
                    .field("name", string(name))
                    .field("value", string(value))
                    .optional("signature", signature.as_deref().map(string))
                    .build()
            })),
        )
    };
    rec.optional("texture", p.skin.body.as_deref().map(string))
        .optional("cape", p.skin.cape.as_deref().map(string))
        .optional("elytra", p.skin.elytra.as_deref().map(string))
        .optional(
            "model",
            p.skin
                .slim
                .map(|slim| string(if slim { "slim" } else { "wide" })),
        )
        .build()
}

/// `ByteBufCodecs.holder(key, direct)` as `RegistryCodecs.holder(key, directCodec)`
/// writes it: a reference by identifier, a direct value as its record.
fn holder_or(
    registry: &str,
    names: &dyn Names,
    r: &mut Reader,
    direct: impl FnOnce(&mut Reader) -> Option<HashCode>,
) -> Option<HashCode> {
    match r.varint().ok()? {
        0 => direct(r),
        id => entry(names, registry, id - 1),
    }
}
/// `TrimMaterial.DIRECT_CODEC`.
pub(super) fn trim_material(r: &mut Reader) -> Option<HashCode> {
    let palette = string(&r.string(32767).ok()?);
    let description = text(&r.nbt().ok()?);
    Some(
        RecordHash::new()
            .field("palette_id", palette)
            .field("description", description)
            .build(),
    )
}
/// `TrimPattern.DIRECT_CODEC` (`decal` is always written).
fn trim_pattern(r: &mut Reader) -> Option<HashCode> {
    let asset = string(&r.string(32767).ok()?);
    let description = text(&r.nbt().ok()?);
    let decal = r.bool().ok()?;
    Some(
        RecordHash::new()
            .field("asset_id", asset)
            .field("description", description)
            .field("decal", HashOps::create_boolean(decal))
            .build(),
    )
}
/// `Instrument.DIRECT_CODEC`.
fn instrument(r: &mut Reader, names: &dyn Names) -> Option<HashCode> {
    let sound = super::gameplay::sound(r, names)?;
    // The pin's instrument is (holder, useDuration in ticks, range), and
    // its codec writes the duration as ticks too (`POSITIVE_INT`); 1.21.2
    // moved it to seconds and added durability and a description, which
    // the pin's codec has no field for.
    let (duration, range) = (r.varint().ok()?, r.f32().ok()?);
    Some(
        RecordHash::new()
            .field("sound_event", sound)
            .field("use_duration", HashOps::create_int(duration))
            .field("range", HashOps::create_f32(range))
            .build(),
    )
}
/// `BannerPattern.DIRECT_CODEC`.
fn banner_pattern(r: &mut Reader) -> Option<HashCode> {
    let asset = string(&r.string(32767).ok()?);
    let key = string(&r.string(32767).ok()?);
    Some(
        RecordHash::new()
            .field("asset_id", asset)
            .field("translation_key", key)
            .build(),
    )
}

/// The hash of component `kind` from its stream bytes, or `None` when Horizon
/// has no codec form for it (no persistent codec in vanilla, a direct holder, an
/// id with no name, a component still to be ported).
pub(super) fn code(kind: i32, bytes: &[u8], names: &dyn Names) -> Option<HashCode> {
    let mut r = Reader::new(bytes);
    let r = &mut r;
    let code = match kind {
        // `Unit.CODEC`: an empty map. The pin's unit components (unbreakable,
        // hide_additional_tooltip, hide_tooltip, fire_resistant).
        4 | 14 | 15 | 21 => HashOps::empty_map(),
        // Persistent codecs only: the stream form is the codec's NBT.
        // intangible_projectile, map_decorations, debug_stick_state, recipes,
        // lock (an `ItemPredicate`) and container_loot.
        19 | 27 | 36 | 43 | 55 | 56 => HashOps::from_nbt(&r.nbt().ok()?, &[]),
        // `Identifier.CODEC`: note_block_sound.
        48 => string(&r.string(32767).ok()?),
        // The entity-variant components (wolf_variant, pig_variant, ...):
        // 26.2 ids, which cannot occur in a 1.21.1-numbered stack; kept so
        // the arms stay readable against the newer trees.
        // `DyeColor.CODEC`: wolf/collar, tropical_fish/base_color and
        // pattern_color, cat/collar, sheep/color, shulker/color.
        75 | 80 | 81 | 93 | 94 | 95 => dye(r.varint().ok()?)?,
        72 => registry_entry("minecraft:villager_type", names, r)?,
        73 => registry_entry("minecraft:wolf_variant", names, r)?,
        74 => registry_entry("minecraft:wolf_sound_variant", names, r)?,
        84 => registry_entry("minecraft:pig_variant", names, r)?,
        85 => registry_entry("minecraft:cow_variant", names, r)?,
        86 => registry_entry("minecraft:chicken_variant", names, r)?,
        87 => registry_entry("minecraft:frog_variant", names, r)?,
        92 => registry_entry("minecraft:cat_variant", names, r)?,
        // The mob variants that are enums.
        76 => named(&FOX, r)?,
        77 => named(&SALMON, r)?,
        78 => named(&PARROT, r)?,
        79 => {
            let id = r.varint().ok()?;
            string(TROPICAL.iter().find(|(packed, _)| *packed == id)?.1)
        }
        82 => named(&MOOSHROOM, r)?,
        83 => match r.varint().ok()? {
            99 => string("evil"),
            id => pick(&RABBIT, id)?,
        },
        88 => named(&HORSE, r)?,
        90 => named(&LLAMA, r)?,
        91 => named(&AXOLOTL, r)?,
        // `TypedEntityData` (entity_data, block_entity_data) and `CustomData`.
        37 => typed_data("minecraft:entity_type", names, r)?,
        39 => typed_data("minecraft:block_entity_type", names, r)?,
        38 => HashOps::from_nbt(&r.nbt().ok()?, &[]),
        // `BlockItemStateProperties`: a string map.
        53 => {
            let n = r.count(r.remaining()).ok()?;
            let mut entries = Vec::with_capacity(n);
            for _ in 0..n {
                let k = string(&r.string(32767).ok()?);
                entries.push((k, string(&r.string(32767).ok()?)));
            }
            HashOps::create_map(entries)
        }
        // `ArmorTrim`, `InstrumentComponent` and `BannerPatternLayers`
        // holders (a reference stays typed in Horizon but a direct one
        // comes here).
        35 => {
            let material = holder_or("minecraft:trim_material", names, r, trim_material)?;
            let pattern = holder_or("minecraft:trim_pattern", names, r, trim_pattern)?;
            RecordHash::new()
                .field("material", material)
                .field("pattern", pattern)
                .build()
        }
        40 => holder_or("minecraft:instrument", names, r, |r| instrument(r, names))?,
        49 => list(r, |r| {
            let pattern = holder_or("minecraft:banner_pattern", names, r, banner_pattern)?;
            let color = dye(r.varint().ok()?)?;
            Some(
                RecordHash::new()
                    .field("pattern", pattern)
                    .field("color", color)
                    .build(),
            )
        })?,
        // `PotDecorations`: back, left, right, front.
        51 => {
            let mut rec = RecordHash::new();
            for side in ["back", "left", "right", "front"] {
                rec = rec.optional(side, optional(r, |r| template_code(r, names))?);
            }
            rec.build()
        }
        // `ItemContainerContents`: the occupied slots as `{slot, item}`.
        52 => {
            let n = r.count(256).ok()?;
            let mut slots = Vec::new();
            for slot in 0..n {
                if let Some(item) = optional(r, |r| template_code(r, names))? {
                    slots.push(
                        RecordHash::new()
                            .field("slot", HashOps::create_int(slot as i32))
                            .field("item", item)
                            .build(),
                    );
                }
            }
            HashOps::create_list(slots)
        }
        // `Bees`: occupants.
        54 => list(r, |r| {
            let data = typed_data("minecraft:entity_type", names, r)?;
            let ticks = r.varint().ok()?;
            let min = r.varint().ok()?;
            Some(
                RecordHash::new()
                    .field("entity_data", data)
                    .field("ticks_in_hive", HashOps::create_int(ticks))
                    .field("min_ticks_in_hive", HashOps::create_int(min))
                    .build(),
            )
        })?,
        _ => super::gameplay::code(kind, r, names)?,
    };
    (r.remaining() == 0).then_some(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Component;

    /// The ids vanilla 1.21.11 gave the names the cases use.
    struct Pinned;
    impl Names for Pinned {
        fn name(&self, registry: &str, id: i32) -> Option<String> {
            let name = match (registry, id) {
                ("minecraft:item", 1) => "minecraft:stone",
                ("minecraft:item", 895) => "minecraft:arrow",
                ("minecraft:item", 946) => "minecraft:stick",
                ("minecraft:item", 1025) => "minecraft:brick",
                ("minecraft:mob_effect", 0) => "minecraft:speed",
                ("minecraft:mob_effect", 1) => "minecraft:slowness",
                ("minecraft:potion", 13) => "minecraft:swiftness",
                ("minecraft:entity_type", 11) => "minecraft:bee",
                ("minecraft:entity_type", 29) => "minecraft:cow",
                ("minecraft:block_entity_type", 1) => "minecraft:chest",
                ("minecraft:data_component_type", 1) => "minecraft:max_stack_size",
                ("minecraft:data_component_type", 2) => "minecraft:max_damage",
                ("minecraft:data_component_type", 3) => "minecraft:damage",
                ("minecraft:data_component_type", 9) => "minecraft:rarity",
                ("minecraft:villager_type", 0) => "minecraft:desert",
                ("minecraft:data_component_predicate_type", 0) => "minecraft:damage",
                ("minecraft:data_component_predicate_type", 1) => "minecraft:enchantments",
                ("minecraft:data_component_predicate_type", 2) => "minecraft:stored_enchantments",
                ("minecraft:data_component_predicate_type", 3) => "minecraft:potion_contents",
                ("minecraft:data_component_predicate_type", 4) => "minecraft:custom_data",
                ("minecraft:data_component_predicate_type", 5) => "minecraft:container",
                ("minecraft:data_component_predicate_type", 6) => "minecraft:bundle_contents",
                ("minecraft:data_component_predicate_type", 7) => "minecraft:firework_explosion",
                ("minecraft:data_component_predicate_type", 8) => "minecraft:fireworks",
                ("minecraft:data_component_predicate_type", 9) => "minecraft:writable_book_content",
                ("minecraft:data_component_predicate_type", 10) => "minecraft:written_book_content",
                ("minecraft:data_component_predicate_type", 11) => "minecraft:attribute_modifiers",
                ("minecraft:data_component_predicate_type", 12) => "minecraft:trim",
                ("minecraft:data_component_predicate_type", 13) => "minecraft:jukebox_playable",
                ("minecraft:attribute", 0) => "minecraft:armor",
                ("minecraft:attribute", 2) => "minecraft:attack_damage",
                ("minecraft:attribute", 19) => "minecraft:max_health",
                ("minecraft:sound_event", 50) => "minecraft:block.anvil.land",
                ("minecraft:sound_event", 70) => "minecraft:item.armor.equip_generic",
                ("minecraft:sound_event", 71) => "minecraft:item.armor.equip_gold",
                ("minecraft:sound_event", 645) => "minecraft:entity.generic.eat",
                ("minecraft:sound_event", 1204) => "minecraft:entity.player.burp",
                ("minecraft:sound_event", 1328) => "minecraft:item.shears.snip",
                ("minecraft:sound_event", 1340) => "minecraft:item.shield.block",
                ("minecraft:mob_effect", 18) => "minecraft:poison",
                ("minecraft:block", 1) => "minecraft:stone",
                ("minecraft:block", 9) => "minecraft:dirt",
                ("minecraft:item", 892) => "minecraft:bowl",
                ("minecraft:item", 904) => "minecraft:iron_ingot",
                ("minecraft:entity_type", 31) => "minecraft:creeper",
                ("minecraft:entity_type", 97) => "minecraft:pig",
                ("minecraft:entity_type", 147) => "minecraft:zombie",
                _ => return None,
            };
            Some(name.to_owned())
        }
    }

    fn unhex(s: &str) -> Vec<u8> {
        (0..s.len() / 2)
            .map(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap())
            .collect()
    }

    /// The stream bytes and hashes vanilla 1.21.11 printed for components built from
    /// their codecs (`STREAM_CODEC.encode`, then `TypedDataComponent.encodeValue`
    /// through `HashOps.CRC32C_INSTANCE`).
    #[test]
    fn components_read_past_hash_as_vanilla_does() {
        let cases: [(&str, i32, &str, i32); 23] = [
        (
        "potion_contents",
        31,
        "010d01000000ff020001c8010001000001000000010100010178",
        568571899,
        ),
        ("potion_contents", 31, "0000010000000100000000", -1716828889),
        ("potion_contents", 31, "00000001026e6d", -1256264264),
        (
        "profile",
        47,
        "010000000100000002000000030000000405537465766501087465787475726573036162630103736967010d6d696e6563726166743a666f6f010e6d696e6563726166743a63617065000101",
        1753620268,
        ),
        ("profile", 47, "0001054e6f746368000000000000", -1744970702),
        (
        "profile",
        47,
        "000001fffffffb000000060000000700000008000000000100",
        -196647545,
        ),
        (
        "profile",
        47,
        "010000000100000001000000010000000101610108746578747572657301760000000000",
        -1883525005,
        ),
        (
        "entity_data",
        37,
        "1d0a0100044e6f4149010300034167650000000500",
        1343601188,
        ),
        (
        "block_entity_data",
        39,
        "010a0800044c6f636b00017800",
        -1404731534,
        ),
        (
        "bucket_entity_data",
        38,
        "0a0500064865616c7468404000000100044e6f41490100",
        2142887662,
        ),
        (
        "container",
        52,
        "0601b2070300000000000001ff060101000304",
        92591498,
        ),
        ("container", 52, "00", -1978007022),
        (
        "pot_decorations",
        51,
        "01810801000000000101020000",
        1078935147,
        ),
        (
        "block_state",
        53,
        "02036c6974047472756506666163696e67056e6f727468",
        1069123732,
        ),
        ("bees", 54, "010b0a030003416765000000030003d804", -122152224),
        (
        "lock",
        55,
        "0a0800056974656d73000f6d696e6563726166743a737469636b00",
        791108361,
        ),
        (
        "container_loot",
        56,
        "0a04000473656564000000000000000508000a6c6f6f745f7461626c65001f6d696e6563726166743a6368657374732f73696d706c655f64756e67656f6e00",
        33155076,
        ),
        (
        "container_loot",
        56,
        "0a08000a6c6f6f745f7461626c65001f6d696e6563726166743a6368657374732f73696d706c655f64756e67656f6e00",
        -2040272502,
        ),
        (
        "map_decorations",
        27,
        "0a0a000161050008726f746174696f6e40400000060001783ff00000000000000600017a40000000000000000800047479706500106d696e6563726166743a706c617965720000",
        1757442925,
        ),
        (
        "debug_stick_state",
        36,
        "0a0800146d696e6563726166743a6f616b5f7374616972730006666163696e6700",
        -1281127558,
        ),
        (
        "recipes",
        43,
        "090800000001000f6d696e6563726166743a737469636b",
        -533729841,
        ),
        ("intangible_projectile", 19, "0a00", -982207288),
        (
        "note_block_sound",
        48,
        "106d696e6563726166743a637573746f6d",
        1404676026,
        ),
    ];
        for (name, kind, hex, expected) in cases {
            let bytes = unhex(hex);
            let mut r = Reader::new(&bytes);
            // The cases are vanilla 1.21.11's stream bytes: read them as
            // 1.21.11 data (its Either-form profiles and template-form
            // containers), not as the pinned 1.21.1's.
            let c = {
                let _up = crate::adapter::ids::renumbering(
                    crate::adapter::ServerVersion::V1_21_11.ids().unwrap(),
                    crate::adapter::ids::Way::Up,
                );
                Component::read(kind, &mut r).unwrap()
            };
            assert_eq!(r.remaining(), 0, "{name}");
            assert_eq!(c.hash(kind, &Pinned), Some(expected), "{name} {hex}");
        }
    }

    /// The same for the components that tune an item's behaviour. The bytes
    /// are vanilla 1.21.11's, converted to the pin's form (predicates without
    /// their matcher counts, food expanded with the codec defaults, the
    /// tool without its creative flag); the codec form omits those defaults,
    /// so the 1.21.11 hashes hold. Cases whose 1.21.11 form has no pin
    /// equivalent (a false creative flag, non-empty matchers, the
    /// 1.21.2+ components) are gone.
    #[test]
    fn gameplay_components_hash_as_vanilla_does() {
        let cases: [(&str, i32, &str, i32); 10] = [
        (
        "trim",
        35,
        "000b6d696e6563726166743a700800034d6174000b6d696e6563726166743a610a080005636f6c6f7200037265640800047465787400035061740001",
        58182709,
        ),
        (
        "banner_patterns",
        49,
        "02000b6d696e6563726166743a62016b0e000b6d696e6563726166743a63016c0b",
        -178464072,
        ),
        ("can_break", 11, "010102010000", -1161294463),
        (
        "can_place_on",
        10,
        "02010301090103036c6974000101300006666163696e6701056e6f7274680361676500010131010133000102090000",
        -279938577,
        ),
        ("can_break", 11, "01000000", -982207288),
        ("can_break", 11, "0100010000", 281125914),
        (
        "attribute_modifiers",
        12,
        "03020e6d696e6563726166743a746573744014000000000000000100130b6d696e6563726166743a78c004000000000000010001000b6d696e6563726166743a793ff00000000000000208020800024869",
        694631198,
        ),
        ("food", 20, "0440200000013fcccccd0000", 1478675052),
        ("food", 20, "0100000000003fcccccd0000", -72303299),
        ("tool", 22, "003f80000001", 937295970),
    ];
        for (name, kind, hex, expected) in cases {
            let bytes = unhex(hex);
            let mut r = Reader::new(&bytes);
            // The attributes case keeps its 1.21.11 wire form (with the
            // display field the pin's wire lacks), so it reads under a
            // 1.21.11 session; the rest are already in the pin's form.
            let c = if name == "attribute_modifiers" {
                let _up = crate::adapter::ids::renumbering(
                    crate::adapter::ServerVersion::V1_21_11.ids().unwrap(),
                    crate::adapter::ids::Way::Up,
                );
                Component::read(kind, &mut r).unwrap()
            } else {
                Component::read(kind, &mut r).unwrap()
            };
            assert_eq!(r.remaining(), 0, "{name}");
            assert_eq!(c.hash(kind, &Pinned), Some(expected), "{name} {hex}");
        }
    }

    #[test]
    fn what_vanilla_cannot_hash_or_horizon_cannot_name_stays_out() {
        // damage_type hashes through its registry name, which the table
        // cannot give (vanilla's client would throw on an unnamed entry).
        assert_eq!(Component::Raw(vec![3]).hash(8, &Pinned), None);
        // A registry id with no name.
        assert_eq!(Component::Raw(vec![9]).hash(79, &Pinned), None);
        // Bytes left over after the value.
        assert_eq!(Component::Raw(vec![0, 0]).hash(34, &Pinned), None);
    }

    /// Block predicates with an `nbt` predicate, and books with filtered
    /// text: stream bytes and hashes vanilla 1.21.11 printed (the same
    /// driver as above, SNBT through the component's codec). The predicate
    /// bytes are the pin's form, without the component matchers (1.21.5);
    /// the codec form omits empty matchers, so the 1.21.11 hashes hold.
    /// Cases with non-empty matchers have no pin form and are gone.
    #[test]
    fn predicates_and_filtered_books_hash_as_vanilla_does() {
        let cases: [(&str, i32, &str, Option<i32>); 5] = [
        ("stone_nbt", 10, "0101020100010a01000161010800016200017800", Some(-359417396)),
        ("nbt_all_types", 10, "010000010a08000361206200027122020001730007060001654202a05f20000000050001783fc000000600017940000000000000000b0001690000000200000001000000020900054974656d730a000000010800026964000e6d696e6563726166743a64697274010005436f756e7401000700017a0000000201020400016c00000000000000050c00014c0000000100000000000000030500016e0000000000", Some(18640168)),
        ("written_filtered", 34, "01540101460161010208000170010a08000474657874000171010004626f6c640100080005706c61696e0001", Some(1884774986)),
        ("written_plain", 34, "01540001610001080001700000", Some(100213250)),
        ("writable_filtered", 33, "03036f6e6501022a2a0374776f000133010133", Some(1192237218)),
    ];
        for (name, kind, hex, expected) in cases {
            let bytes = unhex(hex);
            let mut r = Reader::new(&bytes);
            let c = Component::read(kind, &mut r).unwrap();
            assert_eq!(r.remaining(), 0, "{name}");
            assert_eq!(c.hash(kind, &Pinned), expected, "{name} {hex}");
        }
    }

    // No exact-components-in-predicates test: the pin predates the component
    // matchers (1.21.5), so a newer server's matchers are dropped on read and
    // there is nothing of them to hash. The matcher-less predicate cases
    // above carry the predicate hash coverage.
}
