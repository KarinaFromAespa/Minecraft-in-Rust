use horizon_core::geometry::VoxelShape;
use horizon_world::{
    block_collision::{BlockCollisionRegistry, DynamicCollisionWorld, ShulkerCollisionState},
    collision::CachedCollisionRegistry,
    dynamic_support::DynamicSupportWorld,
    nbt::{self, Limits},
    registry::{BlockRegistry, BlockStateId},
    suffocation::{SuffocationDefinition, SuffocationRegistry},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Write},
    path::Path,
    sync::Arc,
};
struct World<'a, W> {
    out: &'a mut W,
    calls: i32,
    fail: i32,
    entity: &'a str,
}
impl<W: Write> World<'_, W> {
    fn query(&mut self, p: [i32; 3]) -> io::Result<()> {
        writeln!(self.out, "entity {} {} {}", p[0], p[1], p[2])?;
        let count = self.calls;
        self.calls += 1;
        if count == self.fail {
            Err(io::Error::other("injected suffocation query failure"))
        } else {
            Ok(())
        }
    }
}
impl<W: Write> DynamicSupportWorld for World<'_, W> {
    type Error = io::Error;
    fn shulker_is_closed(&mut self, p: [i32; 3]) -> io::Result<Option<bool>> {
        self.query(p)?;
        Ok(match self.entity {
            "none" | "other" => None,
            "closed" => Some(true),
            _ => Some(false),
        })
    }
    fn moving_piston_shape(&mut self, p: [i32; 3]) -> io::Result<Option<VoxelShape>> {
        self.query(p)?;
        Ok(None)
    }
}
impl<W: Write> DynamicCollisionWorld for World<'_, W> {
    fn shulker_collision_state(
        &mut self,
        p: [i32; 3],
    ) -> io::Result<Option<ShulkerCollisionState>> {
        self.query(p)?;
        Ok(None)
    }
}
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = crate::args().nth(1).expect("private fixture directory");
    let dir = Path::new(&arg);
    let read = |name| -> Result<_, Box<dyn std::error::Error>> {
        Ok(
            nbt::read_named(&fs::read(dir.join(name))?, Limits::default())?
                .0
                .tag,
        )
    };
    let registry = BlockRegistry::from_nbt(&read("block-registry.nbt")?)?;
    let cached = Arc::new(CachedCollisionRegistry::from_nbt(
        &read("cached-collision.nbt")?,
        &registry,
    )?);
    let collisions = Arc::new(BlockCollisionRegistry::new(cached)?);
    let definitions: BTreeMap<String, String> =
        serde_json::from_str(&fs::read_to_string(dir.join("definitions.json"))?)?;
    let definitions = definitions
        .into_iter()
        .map(|(name, kind)| {
            Ok((
                name,
                match kind.as_str() {
                    "motion_full" => SuffocationDefinition::MotionFullCollision,
                    "always" => SuffocationDefinition::Always,
                    "never" => SuffocationDefinition::Never,
                    "not_extended" => SuffocationDefinition::NotExtendedPiston,
                    "closed_shulker" => SuffocationDefinition::ClosedShulker,
                    _ => return Err(io::Error::other("unknown suffocation rule")),
                },
            ))
        })
        .collect::<io::Result<BTreeMap<_, _>>>()?;
    let motion: Vec<String> =
        serde_json::from_str(&fs::read_to_string(dir.join("motion.json"))?)?;
    let motion: BTreeSet<String> = motion.into_iter().collect();
    let binding = SuffocationRegistry::new(collisions.clone(), &definitions, &motion)?;
    let mut out = io::BufWriter::new(fs::File::create(dir.join("rust.txt"))?);
    for line in fs::read_to_string(dir.join("inputs.txt"))?.lines() {
        let f: Value = serde_json::from_str(line)?;
        let state = BlockStateId(f["state"].as_u64().unwrap() as u32);
        let p = std::array::from_fn(|i| f["position"][i].as_i64().unwrap() as i32);
        let block = collisions.collision_block(state)?;
        writeln!(
            out,
            "metadata {} {} {}",
            state.0, block.large_collision_shape, block.moving_piston
        )?;
        let mut world = World {
            out: &mut out,
            calls: 0,
            fail: f["fail"].as_i64().unwrap() as i32,
            entity: f["entity"].as_str().unwrap(),
        };
        match binding.is_suffocating(&mut world, state, p) {
            Ok(v) => writeln!(world.out, "suffocating {v}")?,
            Err(_) => writeln!(world.out, "suffocating error")?,
        }
        writeln!(world.out, "queries {}", world.calls)?;
    }
    Ok(())
}
