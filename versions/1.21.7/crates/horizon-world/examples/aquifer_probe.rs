//! Aquifer probe: regenerate a chunk at the Test 1 seed through terrain +
//! decoration and count water, to reproduce the aquifer deficit (vanilla has
//! 745 water in chunk (-22,-18), 354 in underground-only columns; the port's
//! saved world has 43). Also dumps water positions, scheduled fluid ticks
//! and postprocessing marks for the center chunk.
//!
//! aquifer_probe <client resource dir> [--seed S] [--chunk CX CZ]
//!
//! Temporary diagnostic; not part of the shipped test suite.
use horizon_core::position::ChunkPos;
use horizon_world::{
    DATA_VERSION,
    fluid::FluidKind,
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
    // 3x3 region, row-major: index (dz+1)*3 + (dx+1) around the center.
    let mut region: [horizon_world::worldgen::generator::ProtoChunk; 9] =
        std::array::from_fn(|i| {
            let dx = (i % 3) as i32 - 1;
            let dz = (i / 3) as i32 - 1;
            worker
                .generate_proto(ChunkPos::new(pos.x + dx, pos.z + dz))
                .unwrap_or_else(|e| panic!("chunk {},{}: {e}", pos.x + dx, pos.z + dz))
        });
    let stats = worker.decorate(&mut region)?;
    println!("decorate stats: {stats:?}");
    let chunk = &region[4];
    let min_y = chunk.terrain.min_y();
    let height = chunk.terrain.height();
    let mut total_water = 0usize;
    let mut underground_water = 0usize;
    let mut total_lava = 0usize;
    let mut water_positions: Vec<(i32, i32, i32)> = Vec::new();
    for y in min_y..min_y + height {
        for z in 0..16 {
            for x in 0..16 {
                match chunk.block_name(x, y, z) {
                    "minecraft:water" => {
                        total_water += 1;
                        water_positions.push((x, y, z));
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
        "chunk {pos:?}: post-decorate water total {total_water}, underground (y<62) {underground_water}, lava {total_lava}"
    );
    println!("water positions (local x,y,z): {water_positions:?}");
    let fluid_ticks = chunk.terrain.fluid_ticks().pack(0);
    println!("scheduled fluid ticks: {}", fluid_ticks.len());
    for t in &fluid_ticks {
        let kind = FluidKind::from_index(t.kind.0 as usize)
            .map(|k| format!("{k:?}"))
            .unwrap_or_else(|| format!("kind {}", t.kind.0));
        println!(
            "  {kind} at ({},{},{}) delay {}",
            t.pos.x, t.pos.y, t.pos.z, t.delay
        );
    }
    let marks: usize = chunk
        .terrain
        .postprocessing()
        .iter()
        .map(|m| m.len())
        .sum();
    println!("postprocessing marks: {marks}");
    let block_ticks = chunk.terrain.block_ticks().pack(0);
    println!("scheduled block ticks: {}", block_ticks.len());
    for t in &block_ticks {
        println!(
            "  block kind {} at ({},{},{}) delay {}",
            t.kind.0, t.pos.x, t.pos.y, t.pos.z, t.delay
        );
    }
    Ok(())
}
