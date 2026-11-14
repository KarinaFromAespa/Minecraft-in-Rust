//! Seeded carver paths using pinned scalar providers and shared mask geometry.
//! 26.2 carvers are `ConfiguredWorldCarver`s: a `type` plus a nested `config`
//! (`CaveCarverConfiguration` / `CanyonCarverConfiguration`); the nether cave
//! is its own type with its own bound, thickness and y scale.
mod canyon;
mod config;
mod generation;
use super::{DecodeError, Object, anchor::WorldGenerationContext, density::identifier, required};
pub use canyon::CanyonCarver;
pub use config::{
    BlockSetRef, CanyonCarverConfiguration, CanyonShapeConfiguration, CarverConfiguration,
    CarverDebugSettings, CaveCarverConfiguration,
};
pub use generation::{
    CarverCatalog, MaskGenerationError, WorldCarver, generate_carving_mask, large_feature_random,
};
use horizon_core::{
    carving::{CarveError, CarverOutput, can_reach, carve_ellipsoid_with_budget},
    geometry::trig::{cos, sin},
    position::ChunkPos,
    random::{InvalidBound, LegacyRandom, RandomSource},
};

/// `CaveWorldCarver` against `NetherWorldCarver`: the cave bound (15/10),
/// the thickness formula (random sum, with the nether's doubling and no
/// weird-bias branch) and the tunnel y scale (1.0/5.0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaveKind {
    Cave,
    Nether,
}
impl CaveKind {
    fn bound(self) -> i32 {
        match self {
            Self::Cave => 15,
            Self::Nether => 10,
        }
    }
    fn y_scale(self) -> f64 {
        match self {
            Self::Cave => 1.0,
            Self::Nether => 5.0,
        }
    }
    fn thickness(self, random: &mut impl RandomSource) -> Result<f32, InvalidBound> {
        let mut thickness = random.next_float() * 2. + random.next_float();
        match self {
            Self::Cave => {
                if random.next_int_bound(10)? == 0 {
                    thickness *= random.next_float() * random.next_float() * 3. + 1.;
                }
            }
            Self::Nether => thickness *= 2.,
        }
        Ok(thickness)
    }
}

#[derive(Clone, Debug)]
pub struct CaveCarver {
    pub config: CaveCarverConfiguration,
    pub kind: CaveKind,
}
#[derive(Debug)]
pub enum CarverError<E> {
    Random(InvalidBound),
    Geometry(CarveError<E>),
    IterationLimit,
    InvalidBounds,
}
impl<E: std::fmt::Display> std::fmt::Display for CarverError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Random(e) => e.fmt(f),
            Self::Geometry(e) => e.fmt(f),
            Self::IterationLimit => f.write_str("carver work limit exceeded"),
            Self::InvalidBounds => f.write_str("carver output or generation bounds are invalid"),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for CarverError<E> {}
impl<E> From<InvalidBound> for CarverError<E> {
    fn from(e: InvalidBound) -> Self {
        Self::Random(e)
    }
}
impl<E> From<CarveError<E>> for CarverError<E> {
    fn from(e: CarveError<E>) -> Self {
        Self::Geometry(e)
    }
}
impl CaveCarver {
    pub fn from_json(json: &str) -> Result<Self, DecodeError> {
        let object: Object = serde_json::from_str(json)?;
        let kind = identifier(&serde_json::from_str::<String>(
            required(&object, "type")?.get(),
        )?)?;
        let kind = match kind.as_str() {
            "minecraft:cave" => CaveKind::Cave,
            "minecraft:nether_cave" => CaveKind::Nether,
            _ => return Err(DecodeError::Field("cave carver type")),
        };
        let config = config::config_object(json)?;
        Ok(Self {
            config: CaveCarverConfiguration::from_json(&config)?,
            kind,
        })
    }
    pub fn is_start_chunk(&self, random: &mut impl RandomSource) -> bool {
        random.next_float() <= self.config.base.probability
    }
    /// Carve one origin chunk's paths into the target output. The caller performs
    /// start-chunk selection separately. Local tunnel RNGs always use the legacy
    /// algorithm, regardless of the caller's RNG. At most 2^24 candidate cells
    /// are allowed; errors preserve random draws and prior output. The cave
    /// count is the triple-nested bound draw (`nextInt(nextInt(nextInt(15)+1)+1)`,
    /// bound 10 for the nether), so at most 14 starts.
    pub fn carve<O: CarverOutput>(
        &self,
        context: WorldGenerationContext,
        random: &mut impl RandomSource,
        target: ChunkPos,
        origin: ChunkPos,
        output: &mut O,
    ) -> Result<bool, CarverError<O::Error>> {
        let bound = self.kind.bound();
        let count = random.next_int_bound(bound)? + 1;
        let count = random.next_int_bound(count)? + 1;
        let count = random.next_int_bound(count)?;
        let mut budget = 1 << 24;
        for _ in 0..count {
            let x = f64::from(
                origin
                    .x
                    .wrapping_shl(4)
                    .wrapping_add(random.next_int_bound(16)?),
            );
            let y = f64::from(self.config.base.y.sample(random, context)?);
            let z = f64::from(
                origin
                    .z
                    .wrapping_shl(4)
                    .wrapping_add(random.next_int_bound(16)?),
            );
            let horizontal = f64::from(self.config.horizontal_radius_multiplier.sample(random));
            let vertical = f64::from(self.config.vertical_radius_multiplier.sample(random));
            let floor = f64::from(self.config.floor_level.sample(random));
            let mut run = CaveRun {
                target,
                output,
                budget: &mut budget,
                horizontal,
                vertical,
                floor,
            };
            let mut tunnels = 1;
            if random.next_int_bound(4)? == 0 {
                let multiplier = f64::from(self.config.base.y_scale.sample(random));
                let thickness = 1. + random.next_float() * 6.;
                let radius =
                    1.5 + f64::from(sin(f64::from(std::f32::consts::FRAC_PI_2)) * thickness);
                run.ellipsoid([x + 1., y, z], radius, radius * multiplier)?;
                tunnels += random.next_int_bound(4)?;
            }
            for _ in 0..tunnels {
                let yaw = random.next_float() * std::f32::consts::TAU;
                let pitch = (random.next_float() - 0.5) / 4.;
                let thickness = self.kind.thickness(random)?;
                let end = 112 - random.next_int_bound(28)?;
                run.tunnel(Tunnel {
                    seed: random.next_long(),
                    position: [x, y, z],
                    thickness,
                    yaw,
                    pitch,
                    start: 0,
                    end,
                    vertical_scale: self.kind.y_scale(),
                })?;
            }
        }
        Ok(true)
    }
}
struct CaveRun<'a, O: CarverOutput> {
    target: ChunkPos,
    output: &'a mut O,
    budget: &'a mut usize,
    horizontal: f64,
    vertical: f64,
    floor: f64,
}
#[derive(Clone, Copy)]
struct Tunnel {
    seed: i64,
    position: [f64; 3],
    thickness: f32,
    yaw: f32,
    pitch: f32,
    start: i32,
    end: i32,
    vertical_scale: f64,
}
impl<O: CarverOutput> CaveRun<'_, O> {
    fn ellipsoid(
        &mut self,
        position: [f64; 3],
        horizontal: f64,
        vertical: f64,
    ) -> Result<(), CarverError<O::Error>> {
        let floor = self.floor;
        Ok(carve_ellipsoid_with_budget(
            self.target,
            position,
            horizontal,
            vertical,
            self.output,
            |[x, y, z], _| y <= floor || x * x + y * y + z * z >= 1.,
            self.budget,
        )?)
    }
    fn tunnel(&mut self, mut path: Tunnel) -> Result<(), CarverError<O::Error>> {
        let mut random = LegacyRandom::new(path.seed);
        let split = random.next_int_bound(path.end / 2)? + path.end / 4;
        let gentle = random.next_int_bound(6)? == 0;
        let (mut yaw_change, mut pitch_change) = (0f32, 0f32);
        for step in path.start..path.end {
            let radius = 1.5
                + f64::from(
                    sin(f64::from(
                        std::f32::consts::PI * step as f32 / path.end as f32,
                    )) * path.thickness,
                );
            let vertical = radius * path.vertical_scale;
            let cos_pitch = cos(f64::from(path.pitch));
            path.position[0] += f64::from(cos(f64::from(path.yaw)) * cos_pitch);
            path.position[1] += f64::from(sin(f64::from(path.pitch)));
            path.position[2] += f64::from(sin(f64::from(path.yaw)) * cos_pitch);
            path.pitch *= if gentle { 0.92 } else { 0.7 };
            path.pitch += pitch_change * 0.1;
            path.yaw += yaw_change * 0.1;
            pitch_change *= 0.9;
            yaw_change *= 0.75;
            pitch_change += (random.next_float() - random.next_float()) * random.next_float() * 2.;
            yaw_change += (random.next_float() - random.next_float()) * random.next_float() * 4.;
            if step == split && path.thickness > 1. {
                let seed = random.next_long();
                let thickness = random.next_float() * 0.5 + 0.5;
                self.tunnel(Tunnel {
                    seed,
                    thickness,
                    yaw: path.yaw - std::f32::consts::FRAC_PI_2,
                    pitch: path.pitch / 3.,
                    start: step,
                    vertical_scale: 1.,
                    ..path
                })?;
                let seed = random.next_long();
                let thickness = random.next_float() * 0.5 + 0.5;
                self.tunnel(Tunnel {
                    seed,
                    thickness,
                    yaw: path.yaw + std::f32::consts::FRAC_PI_2,
                    pitch: path.pitch / 3.,
                    start: step,
                    vertical_scale: 1.,
                    ..path
                })?;
                return Ok(());
            }
            if random.next_int_bound(4)? == 0 {
                continue;
            }
            if !can_reach(
                self.target,
                path.position[0],
                path.position[2],
                step,
                path.end,
                path.thickness,
            ) {
                return Ok(());
            }
            self.ellipsoid(
                path.position,
                radius * self.horizontal,
                vertical * self.vertical,
            )?;
        }
        Ok(())
    }
}
