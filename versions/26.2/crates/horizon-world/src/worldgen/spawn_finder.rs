//! `Climate$SpawnFinder`: where a new world's spawn search starts.
//!
//! 26.2 has no `NoiseSpawnFinder` (and no `ChunkGenerator.getOrigin`): the
//! spawn search lives in `MinecraftServer.setInitialSpawn`, which calls
//! `Climate.findSpawnPosition` with the noise settings' `spawn_target`
//! points (climate `ParameterPoint`s) and the `RandomState` climate sampler.
//! It samples the six climate parameters at the column's quart corner,
//! zeroes the depth, and keeps the block whose climate fits the targets
//! best, biased towards 0 0. The world spawn search then spirals out from
//! its chunk.
use super::climate::{ClimateSampler, ParameterPoint, TargetPoint};
use horizon_core::density::{SampleError, SamplerContext};
use horizon_core::jmath;

const MAX_RADIUS: f32 = 2048.0;

#[derive(Clone, Copy, Debug)]
struct Candidate {
    x: i32,
    z: i32,
    fitness: i64,
}

/// `Climate.findSpawnPosition`: the best block column (x, z) of a coarse
/// then a fine radial search around the best of the two. The caller passes
/// an empty target list through as 0 0 (`Climate.Sampler.findSpawnPosition`
/// returns `BlockPos.ZERO` when the spawn target is empty).
pub fn find_spawn_position(
    targets: &[ParameterPoint],
    sampler: &ClimateSampler,
    context: &mut SamplerContext,
) -> Result<[i32; 2], SampleError> {
    let mut best = fitness(targets, sampler, context, 0, 0)?;
    radial_search(targets, sampler, context, &mut best, MAX_RADIUS, 512.0)?;
    radial_search(targets, sampler, context, &mut best, 512.0, 32.0)?;
    Ok([best.x, best.z])
}

/// `SpawnFinder.radialSearch`: rings `increment` apart around the best
/// position so far, sampled every `increment` blocks along each ring.
fn radial_search(
    targets: &[ParameterPoint],
    sampler: &ClimateSampler,
    context: &mut SamplerContext,
    best: &mut Candidate,
    max_radius: f32,
    increment: f32,
) -> Result<(), SampleError> {
    let mut angle = 0f32;
    let mut radius = increment;
    let (origin_x, origin_z) = (best.x, best.z);
    while radius <= max_radius {
        let x = origin_x.wrapping_add((jmath::sin(f64::from(angle)) * f64::from(radius)) as i32);
        let z = origin_z.wrapping_add((jmath::cos(f64::from(angle)) * f64::from(radius)) as i32);
        let candidate = fitness(targets, sampler, context, x, z)?;
        if candidate.fitness < best.fitness {
            *best = candidate;
        }
        angle += increment / radius;
        if f64::from(angle) > std::f64::consts::PI * 2.0 {
            angle = 0.0;
            radius += increment;
        }
    }
    Ok(())
}

/// `SpawnFinder.getSpawnPositionAndFitness`: the best target's fitness at
/// the column's quart corner, scaled so the squared distance to 0 0 only
/// breaks ties between equally fitting columns. The climate sample's depth
/// is zeroed before the fitness, as vanilla does.
fn fitness(
    targets: &[ParameterPoint],
    sampler: &ClimateSampler,
    context: &mut SamplerContext,
    x: i32,
    z: i32,
) -> Result<Candidate, SampleError> {
    let sampled = sampler.sample(context, [x >> 2, 0, z >> 2])?;
    let target = TargetPoint {
        depth: 0,
        ..sampled
    };
    let mut best = i64::MAX;
    for point in targets {
        best = best.min(point.fitness(target));
    }
    let bias = i64::from(x) * i64::from(x) + i64::from(z) * i64::from(z);
    Ok(Candidate {
        x,
        z,
        fitness: best.wrapping_mul(2048 * 2048).wrapping_add(bias),
    })
}

#[cfg(test)]
mod tests {
    use super::super::climate::Parameter;
    use super::*;
    use horizon_core::density::{ConstantSampler, DensitySampler};
    use std::sync::Arc;

    #[derive(Debug)]
    struct Gradient;
    impl DensitySampler for Gradient {
        fn sample_value(&self, _: &mut SamplerContext, p: [i32; 3]) -> Result<f32, SampleError> {
            Ok(p[0] as f32 / 1000.0)
        }
    }

    #[test]
    fn the_closest_fitting_column_wins() {
        // x / 1000 must reach 0.1: every column from x = 100 on fits equally,
        // so the distance bias picks the fitting column nearest to 0 0.
        let sampler = ClimateSampler::new([
            Arc::new(Gradient),
            Arc::new(ConstantSampler(0.0)),
            Arc::new(ConstantSampler(0.0)),
            Arc::new(ConstantSampler(0.0)),
            Arc::new(ConstantSampler(0.0)),
            Arc::new(ConstantSampler(0.0)),
        ]);
        let wide = Parameter::new(-10_000, 10_000);
        let targets = [ParameterPoint {
            coordinates: [Parameter::new(1000, 10_000), wide, wide, wide, wide, wide],
            offset: 0,
        }];
        let [x, z] =
            find_spawn_position(&targets, &sampler, &mut SamplerContext::default()).unwrap();
        assert!(x >= 100, "{x} {z}");
        assert!(x * x + z * z < 160 * 160, "{x} {z}");
    }
}
