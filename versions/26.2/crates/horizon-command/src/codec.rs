//! ResourceOrIdArgument and the data-fixer-upper decoding of inline values
//! commands accept (loot predicates, item modifiers, slot sources,
//! dialogs), reproducing the codec error messages vanilla reports.
use super::args::{Registries, is_id_char, parse_identifier, slot_ranges};
use super::{CmdResult, CommandError, Reader};
use horizon_world::nbt::{Compound, JavaString, Tag};

pub fn snbt(t: &Tag) -> String {
    horizon_client_core::snbt::to_snbt(t)
}
pub fn not_map(t: &Tag) -> String {
    format!("Not a map: {}", snbt(t))
}
pub fn not_list(t: &Tag) -> String {
    format!("Not a list: {}", snbt(t))
}
pub fn no_key(key: &str, t: &Tag) -> String {
    format!("No key {key} in MapLike[{}]", snbt(t))
}
pub fn either(a: String, b: String) -> String {
    format!("Failed to parse either. First: {a}; Second: {b}")
}
pub fn get<'t>(map: &'t Compound, key: &str) -> Option<&'t Tag> {
    map.get(&JavaString::from(key))
}
pub fn is_number(t: &Tag) -> bool {
    matches!(
        t,
        Tag::Byte(_) | Tag::Short(_) | Tag::Int(_) | Tag::Long(_) | Tag::Float(_) | Tag::Double(_)
    )
}
pub fn as_string(t: &Tag) -> Option<String> {
    match t {
        Tag::String(s) => Some(s.to_string_lossy()),
        _ => None,
    }
}

/// KeyDispatchCodec over a type registry: the map, its dispatch key and the
/// type's own fields (`fields` checks them). 26.2 dispatches loot conditions
/// on `condition` and loot functions on `function` (LootItemCondition's
/// TYPED_CODEC and LootItemFunctions' TYPED_CODEC in the 26.2 jar); 26.3
/// unified both onto `type`, so the key is a parameter here.
fn dispatch(
    t: &Tag,
    reg: &Registries,
    type_registry: &str,
    key: &str,
    fields: &dyn Fn(&str, &Compound, &Tag) -> Result<(), String>,
) -> Result<(), String> {
    let Tag::Compound(map) = t else {
        return Err(not_map(t));
    };
    let Some(ty) = get(map, key) else {
        return Err(no_key(key, t));
    };
    let Some(name) = as_string(ty) else {
        return Err("Not a string".into());
    };
    let Some(id) = parse_identifier(&name) else {
        return Err(format!("Not a valid resource location: {name}"));
    };
    let known = reg.ids(type_registry);
    if !known.is_empty() && !known.contains(&id) {
        return Err(format!(
            "Unknown registry key in ResourceKey[minecraft:root / {type_registry}]: {id}"
        ));
    }
    fields(&id, map, t)
}
/// Required fields, reported in vanilla's order when missing.
fn require(map: &Compound, t: &Tag, keys: &[&str]) -> Result<(), String> {
    let missing: Vec<String> = keys
        .iter()
        .filter(|k| get(map, k).is_none())
        .map(|k| no_key(k, t))
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing.join("; "))
    }
}

/// `Codec.withAlternative(TYPED_CODEC, INLINE_CODEC)`: a single typed element
/// or a list of them, with the either error 26.2 reports when both fail.
/// 26.2 has no holder registry for these codecs, so `#tag` references are
/// not admitted (26.3's holder-set form is).
fn with_alternative(t: &Tag, element: &dyn Fn(&Tag) -> Result<(), String>) -> Result<(), String> {
    let first = match element(t) {
        Ok(()) => return Ok(()),
        Err(e) => e,
    };
    let list = match t {
        Tag::List(items) => {
            let errors: Vec<String> = items.iter().filter_map(|i| element(i).err()).collect();
            if errors.is_empty() {
                return Ok(());
            }
            errors.join("; ")
        }
        _ => not_list(t),
    };
    Err(either(first, list))
}

/// CompositeLootItemCondition.createCodec / InvertedLootItemCondition:
/// required inline combinator children must decode before command execution.
fn predicate_fields(id: &str, map: &Compound, t: &Tag, reg: &Registries) -> Result<(), String> {
    match id.trim_start_matches("minecraft:") {
        "all_of" | "any_of" => {
            require(map, t, &["terms"])?;
            let terms = get(map, "terms").expect("required field");
            let Tag::List(terms) = terms else {
                return Err(not_list(terms));
            };
            for term in terms {
                decode("minecraft:predicate", term, reg)?;
            }
            Ok(())
        }
        "inverted" => {
            require(map, t, &["term"])?;
            decode(
                "minecraft:predicate",
                get(map, "term").expect("required field"),
                reg,
            )
        }
        // Other predicate families retain their existing codec/evaluator paths.
        _ => Ok(()),
    }
}

/// Slot-source fields: `contents` needs a component holding items,
/// `slot_range` needs its slots and (in 26.2) its source.
fn slot_source_fields(id: &str, map: &Compound, t: &Tag, reg: &Registries) -> Result<(), String> {
    if id == "minecraft:contents" {
        require(map, t, &["component"])?;
        let value = get(map, "component").expect("required component");
        let name = as_string(value).ok_or_else(|| "Not a string".to_owned())?;
        let component = parse_identifier(&name)
            .ok_or_else(|| format!("Not a valid resource location: {name}"))?;
        let known = reg.ids("minecraft:data_component_type");
        if !known.is_empty() && !known.contains(&component) {
            return Err(format!(
                "Unknown registry key in ResourceKey[minecraft:root / minecraft:data_component_type]: {component}"
            ));
        }
        if !matches!(
            component.as_str(),
            "minecraft:container" | "minecraft:bundle_contents" | "minecraft:charged_projectiles"
        ) {
            return Err("No items in component".into());
        }
    } else if id == "minecraft:slot_range" {
        // RangeSlotSource.MAP_CODEC uses the global SlotRanges.CODEC,
        // independently of the eventual target inventory's size. 26.2's
        // codec requires `source` (`LootContextArg.ENTITY_OR_BLOCK`: an
        // entity or block-entity target); 26.3 made it optional,
        // defaulting to the container.
        require(map, t, &["slots", "source"])?;
        let source = get(map, "source").expect("required source");
        let source = as_string(source).ok_or_else(|| "Not a string".to_owned())?;
        if !matches!(
            source.as_str(),
            "this"
                | "attacker"
                | "direct_attacker"
                | "attacking_player"
                | "target_entity"
                | "interacting_entity"
                | "block_entity"
        ) {
            return Err(format!("Unknown element id: {source}"));
        }
        let value = get(map, "slots").expect("required slots");
        let name = as_string(value).ok_or_else(|| "Not a string".to_owned())?;
        if !slot_ranges().iter().any(|(known, _)| known == &name) {
            return Err(format!("Unknown element name:{name}"));
        }
    }
    Ok(())
}

/// Decode an inline value of a ResourceOrIdArgument registry.
pub fn decode(registry: &str, t: &Tag, reg: &Registries) -> Result<(), String> {
    match registry {
        // LootItemCondition.DIRECT_CODEC: withAlternative(TYPED_CODEC,
        // AllOfCondition.INLINE_CODEC) — a single typed condition or a list
        // of them, dispatched on `condition`.
        "minecraft:predicate" => with_alternative(t, &|x| {
            dispatch(
                x,
                reg,
                "minecraft:loot_condition_type",
                "condition",
                &|id, map, t| predicate_fields(id, map, t, reg),
            )
        }),
        // LootItemFunctions.ROOT_CODEC: withAlternative(TYPED_CODEC,
        // SequenceFunction.INLINE_CODEC), dispatched on `function`.
        "minecraft:item_modifier" => with_alternative(t, &|x| {
            dispatch(
                x,
                reg,
                "minecraft:loot_function_type",
                "function",
                &|_, _, _| Ok(()),
            )
        }),
        "minecraft:slot_source" => with_alternative(t, &|x| {
            dispatch(
                x,
                reg,
                "minecraft:slot_source_type",
                "type",
                &|id, map, t| slot_source_fields(id, map, t, reg),
            )
        }),
        // Dialog types: CommonDialogData's title, then each type's own.
        "minecraft:dialog" => dispatch(t, reg, "minecraft:dialog_type", "type", &|id, map, t| {
            let own: &[&str] = match id {
                "minecraft:confirmation" => &["yes", "no"],
                "minecraft:multi_action" => &["actions"],
                "minecraft:dialog_list" => &["dialogs"],
                _ => &[],
            };
            let keys: Vec<&str> = std::iter::once("title")
                .chain(own.iter().copied())
                .collect();
            require(map, t, &keys)
        }),
        "minecraft:loot_table" => match t {
            Tag::Compound(_) => loot_table_references(t, reg),
            _ => Err(not_map(t)),
        },
        _ => Ok(()),
    }
}

/// A partial check of `LootTable.DIRECT_CODEC`: only the holders of its
/// `minecraft:loot_table` entries, in the codec's own structure: `pools`,
/// each pool's `entries`, a composite entry's `children` (alternatives, group,
/// sequence), and a `NestedLootTable`'s `value`. Every other field, entry type
/// and codec error is not checked here and stays accepted.
fn loot_table_references(t: &Tag, reg: &Registries) -> Result<(), String> {
    let Tag::Compound(map) = t else {
        return Ok(());
    };
    let Some(Tag::List(pools)) = get(map, "pools") else {
        return Ok(());
    };
    for pool in pools {
        let Tag::Compound(pool) = pool else {
            continue;
        };
        if let Some(Tag::List(entries)) = get(pool, "entries") {
            for entry in entries {
                loot_entry_references(entry, reg)?;
            }
        }
    }
    Ok(())
}
/// `LootPoolEntries.CODEC`, dispatched on the entry's `type`.
fn loot_entry_references(t: &Tag, reg: &Registries) -> Result<(), String> {
    let Tag::Compound(map) = t else {
        return Ok(());
    };
    let Some(ty) = get(map, "type")
        .and_then(as_string)
        .and_then(|name| parse_identifier(&name))
    else {
        return Ok(());
    };
    match ty.as_str() {
        "minecraft:alternatives" | "minecraft:group" | "minecraft:sequence" => {
            if let Some(Tag::List(children)) = get(map, "children") {
                for child in children {
                    loot_entry_references(child, reg)?;
                }
            }
            Ok(())
        }
        "minecraft:loot_table" => match get(map, "value") {
            Some(value) => loot_table_holder_set(value, reg),
            None => Ok(()),
        },
        _ => Ok(()),
    }
}
/// `HolderSetCodec`: either a `TagKey.hashedCodec` or `compactListCodec`,
/// itself either a list or one `RegistryFileCodec` holder: an identifier the
/// registry holds, or an inline table.
fn loot_table_holder_set(t: &Tag, reg: &Registries) -> Result<(), String> {
    match t {
        Tag::String(_) => {
            let name = as_string(t).unwrap_or_default();
            // A tag's contents are not checked here.
            if name.starts_with('#') {
                return Ok(());
            }
            // A string that is not an identifier would decode inline: not checked here.
            let Some(id) = parse_identifier(&name) else {
                return Ok(());
            };
            let known = reg.ids("minecraft:loot_table");
            if known.is_empty() || known.contains(&id) {
                return Ok(());
            }
            Err(either(
                "Not a tag id".into(),
                either(
                    not_list(t),
                    format!("Failed to get element {id}"),
                ),
            ))
        }
        // The inline holder's error is the partial result both eithers keep.
        Tag::Compound(_) => loot_table_references(t, reg),
        // A list of holders is not checked here.
        _ => Ok(()),
    }
}

/// Identifier.readNonEmpty (None when empty or invalid).
fn read_non_empty(reader: &mut Reader) -> Option<String> {
    let start = reader.cursor;
    while reader.can_read() && is_id_char(reader.peek()) {
        reader.skip();
    }
    let raw = &reader.string[start..reader.cursor];
    let id = if raw.is_empty() {
        None
    } else {
        parse_identifier(raw)
    };
    if id.is_none() {
        reader.cursor = start;
    }
    id
}

/// ResourceOrIdArgument.parse: a registry id (checked) or an inline SNBT
/// value decoded with the registry's codec.
pub fn resource_or_id(reader: &mut Reader, reg: &Registries, registry: &str) -> CmdResult<()> {
    let start = reader.cursor;
    let mut r = reader.clone();
    r.skip_whitespace();
    if let Some(id) = read_non_empty(&mut r) {
        reader.cursor = r.cursor;
        if !reg.ids(registry).contains(&id) {
            return Err(CommandError::tr_at(
                "argument.resource_or_id.no_such_element",
                &[&id, &registry],
                reader,
            ));
        }
        return Ok(());
    }
    reader.cursor = start;
    let tag = super::snbt::parse_tag_argument(reader)?;
    decode(registry, &tag, reg)
        .map_err(|m| CommandError::tr_at("argument.resource_or_id.failed_to_parse", &[&m], reader))
}

#[cfg(test)]
mod predicate_tests {
    use super::*;
    use crate::args::RegistryIds;

    #[test]
    fn inline_constant_combinators_require_decodable_children() {
        let reg = Registries::default();
        let decode_text = |text| {
            let (tag, _) = horizon_world::nbt::parse_snbt(text).unwrap();
            decode("minecraft:predicate", &tag, &reg)
        };
        assert!(decode_text(r#"{condition:"minecraft:all_of",terms:[{condition:"minecraft:inverted",term:{condition:"minecraft:any_of",terms:[]}}]}"#).is_ok());
        // 26.2's DIRECT_CODEC is withAlternative(TYPED, INLINE): a missing
        // field fails the typed branch and the list branch alike.
        assert_eq!(
            decode_text(r#"{condition:"minecraft:all_of"}"#),
            Err(either(
                r#"No key terms in MapLike[{condition:"minecraft:all_of"}]"#.into(),
                r#"Not a list: {condition:"minecraft:all_of"}"#.into(),
            ))
        );
        assert_eq!(
            decode_text(r#"{condition:"minecraft:inverted"}"#),
            Err(either(
                r#"No key term in MapLike[{condition:"minecraft:inverted"}]"#.into(),
                r#"Not a list: {condition:"minecraft:inverted"}"#.into(),
            ))
        );
        // A bad nested term fails with the nested either error, wrapped in
        // the outer either (terms decode with DIRECT_CODEC).
        let nested = either(
            "No key condition in MapLike[{}]".into(),
            "Not a list: {}".into(),
        );
        for (bad, outer) in [
            (
                r#"{condition:"minecraft:all_of",terms:[{}]}"#,
                r#"{condition:"minecraft:all_of",terms:[{}]}"#,
            ),
            (
                r#"{condition:"minecraft:inverted",term:{}}"#,
                r#"{condition:"minecraft:inverted",term:{}}"#,
            ),
        ] {
            assert_eq!(
                decode_text(bad),
                Err(either(nested.clone(), format!("Not a list: {outer}")))
            );
        }
    }

    #[test]
    fn predicates_dispatch_on_condition_and_functions_on_function() {
        let mut reg = Registries::default();
        reg.by_registry.insert(
            "minecraft:loot_function_type".to_string(),
            RegistryIds {
                ids: vec!["minecraft:set_count".to_string()],
                tags: vec![],
            },
        );
        let decode_text = |registry: &str, text: &str| {
            let (tag, _) = horizon_world::nbt::parse_snbt(text).unwrap();
            decode(registry, &tag, &reg)
        };
        assert_eq!(
            decode_text("minecraft:predicate", "{}"),
            Err(either(
                "No key condition in MapLike[{}]".into(),
                "Not a list: {}".into(),
            ))
        );
        assert_eq!(
            decode_text(
                "minecraft:predicate",
                r#"{type:"minecraft:chest",pools:[]}"#
            ),
            Err(either(
                r#"No key condition in MapLike[{pools:[],type:"minecraft:chest"}]"#.into(),
                r#"Not a list: {pools:[],type:"minecraft:chest"}"#.into(),
            ))
        );
        assert_eq!(
            decode_text("minecraft:item_modifier", "{}"),
            Err(either(
                "No key function in MapLike[{}]".into(),
                "Not a list: {}".into(),
            ))
        );
        assert!(
            decode_text(
                "minecraft:item_modifier",
                r#"{"function":"minecraft:set_count","count":1}"#
            )
            .is_ok()
        );
        assert_eq!(
            decode_text("minecraft:item_modifier", r#"{"function":"nope"}"#),
            Err(either(
                "Unknown registry key in ResourceKey[minecraft:root / minecraft:loot_function_type]: minecraft:nope".into(),
                r#"Not a list: {function:"nope"}"#.into(),
            ))
        );
    }
}

#[cfg(test)]
mod contents_component_tests {
    use super::*;

    #[test]
    fn only_component_collections_are_admitted_for_contents_sources() {
        let reg = Registries::default();
        for component in [
            "container",
            "bundle_contents",
            "charged_projectiles",
            "damage",
        ] {
            let raw = Tag::compound([
                ("type", Tag::String("contents".into())),
                ("component", Tag::String(component.into())),
                (
                    "slot_source",
                    Tag::compound([
                        ("type", Tag::String("slot_range".into())),
                        ("slots", Tag::String("container.0".into())),
                    ]),
                ),
            ]);
            let result = decode("minecraft:slot_source", &raw, &reg);
            if component == "damage" {
                // 26.2's withAlternative error: either(typed, inline-list).
                assert_eq!(result, Err(r#"Failed to parse either. First: No items in component; Second: Not a list: {component:"damage",slot_source:{slots:"container.0",type:"slot_range"},type:"contents"}"#.into()));
            } else {
                assert_eq!(result, Ok(()));
            }
        }
    }
}

#[cfg(test)]
mod slot_range_tests {
    use super::*;

    #[test]
    fn inline_range_names_use_the_global_slot_registry() {
        let reg = Registries::default();
        for name in ["container.0", "container.53", "container.*", "hotbar.8"] {
            let input = format!(r#"{{type:"minecraft:slot_range",source:"this",slots:"{name}"}}"#);
            let mut reader = Reader::new(&input);
            assert!(
                resource_or_id(&mut reader, &reg, "minecraft:slot_source").is_ok(),
                "{name}"
            );
            assert_eq!(reader.cursor, input.len());
        }
        for name in ["container.54", "container.99"] {
            let input = format!(r#"{{type:"minecraft:slot_range",source:"block_entity",slots:"{name}"}}"#);
            let mut reader = Reader::new(&input);
            assert!(
                resource_or_id(&mut reader, &reg, "minecraft:slot_source").is_err(),
                "{name} crossed argument parsing"
            );
            let (raw, _) = horizon_world::nbt::parse_snbt(&input).unwrap();
            let message = format!("Unknown element name:{name}");
            assert_eq!(
                decode("minecraft:slot_source", &raw, &reg),
                Err(either(message.clone(), not_list(&raw)))
            );
        }
    }

    #[test]
    fn a_slot_range_needs_its_source_in_26_2() {
        let reg = Registries::default();
        let decode_text = |text: &str| {
            let (raw, _) = horizon_world::nbt::parse_snbt(text).unwrap();
            decode("minecraft:slot_source", &raw, &reg)
        };
        // 26.2's RangeSlotSource requires `source`; 26.3 made it optional.
        assert_eq!(
            decode_text(r#"{type:"minecraft:slot_range",slots:"container.0"}"#),
            Err(either(
                r#"No key source in MapLike[{slots:"container.0",type:"minecraft:slot_range"}]"#.into(),
                r#"Not a list: {slots:"container.0",type:"minecraft:slot_range"}"#.into(),
            )),
        );
        // An unknown source is the id-resolver codec's error.
        assert_eq!(
            decode_text(r#"{type:"minecraft:slot_range",source:"container",slots:"container.0"}"#),
            Err(either(
                "Unknown element id: container".into(),
                r#"Not a list: {slots:"container.0",source:"container",type:"minecraft:slot_range"}"#.into(),
            )),
        );
    }
}
