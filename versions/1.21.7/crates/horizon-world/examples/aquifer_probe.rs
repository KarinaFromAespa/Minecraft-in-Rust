//! Aquifer probe: regenerate chunk (-22,-18) at the Test 1 seed and count
//! underground water, to reproduce the aquifer deficit (vanilla has 745
//! water in the chunk, 354 in underground-only columns; the port's saved
//! world has 43).
//!
//! aquifer_probe <client resource dir> [--seed S] [--chunk CX CZ]
//!
//! Temporary diagnostic; not part of the shipped test suite.
use horizon_core::position::ChunkPos;
use horizon_world::{
    DATA_VERSION,
    nbt::Tag,
    registry::BlockRegistry,
    worldgen::generator::{GeneratorData, OverworldGenerator},
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

type Error = Box<dyn std::error::Error + Send + Sync>;

fn tag(v: &Value) -> Tag {
    match v {
        Value::String(s) => Tag::String(s.as_str().into()),
        Value::Number(n) => Tag::Int(n.as_i64().unwrap() as i32),
        Value::Array(a) => Tag::List(a.iter().map(tag).collect()),
        Value::Object(o) => {
            Tag::Compound(o.iter().map(|(k, v)| (k.as_str().into(), tag(v))).collect())
        }
        _ => panic!("invalid registry metadata"),
    }
}

fn registry(resources: &Path) -> Result<&'static BlockRegistry, Error> {
    let facts: Value = serde_json::from_str(&fs::read_to_string(resources.join("facts.json"))?)?;
    let registry = BlockRegistry::from_nbt(&Tag::compound([
        ("DataVersion", Tag::Int(DATA_VERSION)),
        (
            "states",
            Tag::List(
                facts["states"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| tag(&v[4]))
                    .collect(),
            ),
        ),
        ("defaults", tag(&facts["defaults"])),
    ]))?;
    Ok(Box::leak(Box::new(registry)))
}

fn main() -> Result<(), Error> {
    let mut args = std::env::args().skip(1);
    let resources = PathBuf::from(args.next().expect("client resource directory"));
    let (mut seed, mut center) = (8257987075890829262i64, (-22, -18));
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => seed = args.next().expect("seed value").parse()?,
            "--chunk" => {
                center = (
                    args.next().expect("chunk x").parse()?,
                    args.next().expect("chunk z").parse()?,
                )
            }
            _ => return Err(format!("unknown argument {arg}").into()),
        }
    }
    let registry = registry(&resources)?;
    let data = GeneratorData::load(&resources.join("worldgen"), registry)?;
    let generator = OverworldGenerator::new(data.clone(), seed)?;
    let mut worker = generator.worker();
    let pos = ChunkPos::new(center.0, center.1);
    let chunk = worker.generate_proto(pos)?;
    let min_y = chunk.terrain.min_y();
    let height = chunk.terrain.height();
    let mut total_water = 0usize;
    let mut underground_water = 0usize;
    let mut total_lava = 0usize;
    for y in min_y..min_y + height {
        for z in 0..16 {
            for x in 0..16 {
                match chunk.block_name(x, y, z) {
                    "minecraft:water" => {
                        total_water += 1;
                        if y < 62 {
                            underground_water += 1;
                        }
                    }
                    "minecraft:lava" => total_lava += 1,
                    _ => {}
                }
            }
        }
    }
    println!(
        "chunk {pos:?}: terrain-stage water total {total_water}, underground (y<62) {underground_water}, lava {total_lava}"
    );
    println!("vanilla post-decorate: water total 745, underground 354+");
    for (lx, lz) in [(13, 5), (5, 11)] {
        let mut runs: Vec<(i32, i32, &str)> = Vec::new();
        let mut cur: Option<(i32, &str)> = None;
        for y in min_y..min_y + height {
            let name = chunk.block_name(lx, y, lz);
            let cls = if name == "minecraft:water" {
                "water"
            } else if matches!(
                name,
                "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
            ) {
                "air"
            } else if name == "minecraft:lava" {
                "lava"
            } else {
                "solid"
            };
            match cur {
                Some((start, c)) if c == cls => {
                    let _ = start;
                }
                Some((start, c)) => {
                    runs.push((start, y - 1, c));
                    cur = Some((y, cls));
                }
                None => cur = Some((y, cls)),
            }
        }
        if let Some((start, c)) = cur {
            runs.push((start, min_y + height - 1, c));
        }
        let interesting: Vec<_> = runs
            .into_iter()
            .filter(|(s, e, _)| *e >= 0 && *s <= 70)
            .collect();
        println!("column ({lx},{lz}) runs y0..70: {interesting:?}");
    }
    Ok(())
}
