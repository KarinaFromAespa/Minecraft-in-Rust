//! Game-path round-trip for `minecraft:pot_decorations`: NBT -> model ->
//! NBT -> model, for the differential harness
//! `tools/test_pot_decorations_parity.py`.
//!
//! Each input line is `label\t<json list of item ids>`. Each output line is
//! `label\t<sides after decode>\t<re-encoded component>\t<sides after the
//! second decode>`, where sides are [back, left, right, front] item ids with
//! `-` for an undecorated side, and the re-encoded component is the JSON list
//! `encode_components` wrote — or `absent` when the model holds no component
//! (all four sides undecorated collapses to no component; the driver maps
//! that onto vanilla's four-brick list, which the decorated pot's default
//! component makes equivalent).
use horizon_items::item_nbt::{item_from_nbt, item_to_nbt};
use horizon_world::nbt::Tag;
use serde_json::Value;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

/// [back, left, right, front] item ids, `-` for an undecorated side.
fn sides(stack: &horizon_items::items::ItemStack) -> String {
    match stack
        .extra
        .as_ref()
        .and_then(|d| d.pot_decorations.as_ref())
    {
        Some(sides) => sides
            .iter()
            .map(|s| s.as_ref().map(|s| s.item.as_ref()).unwrap_or("-"))
            .collect::<Vec<_>>()
            .join(","),
        None => "-,-,-,-".to_string(),
    }
}

/// The re-encoded component as a JSON list of item ids, or `absent` when
/// the model holds no component.
fn encoded(stack: &horizon_items::items::ItemStack) -> String {
    let tag = item_to_nbt(stack);
    match tag
        .get("components")
        .and_then(|c| c.get("minecraft:pot_decorations"))
    {
        Some(Tag::List(list)) => {
            let ids: Vec<String> = list
                .iter()
                .map(|t| match t {
                    Tag::String(s) => format!("\"{}\"", s.to_string_lossy()),
                    _ => "\"?\"".to_string(),
                })
                .collect();
            format!("[{}]", ids.join(","))
        }
        Some(_) => "not-a-list".to_string(),
        None => "absent".to_string(),
    }
}

fn main() -> horizon_items::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let mut out = BufWriter::new(File::create(&args[2])?);
    for line in BufReader::new(File::open(&args[1])?).lines() {
        let line = line?;
        let (label, json) = line.split_once('\t').ok_or("fixture separator")?;
        let value: Value = serde_json::from_str(json).map_err(|_| "fixture json")?;
        let list = value.as_array().ok_or("fixture list")?;
        let decorations = Tag::List(
            list.iter()
                .map(|v| Tag::String(v.as_str().unwrap_or("").into()))
                .collect(),
        );
        let tag = Tag::compound([
            ("id", Tag::String("minecraft:decorated_pot".into())),
            ("count", Tag::Int(1)),
            (
                "components",
                Tag::compound([("minecraft:pot_decorations", decorations)]),
            ),
        ]);
        let stack = item_from_nbt(&tag).ok_or("decoded item")?;
        let first = sides(&stack);
        let saved = encoded(&stack);
        // Decode the saved form again: the round-trip must be stable.
        let stack2 = item_from_nbt(&item_to_nbt(&stack)).ok_or("re-decoded item")?;
        let second = sides(&stack2);
        writeln!(out, "{label}\t{first}\t{saved}\t{second}")?;
    }
    Ok(())
}
