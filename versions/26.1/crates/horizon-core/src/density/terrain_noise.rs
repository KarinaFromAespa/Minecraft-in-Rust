use super::{graph::WeirdRarityMapper, *};
use crate::{
    noise::{synth, NoiseBuildError, NoiseStack, SmearedPerlinNoise},
    random::RandomSource,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiftNoiseKind {
    Shift,
    ShiftA,
    ShiftB,
}

/// 26.1's `WeirdScaledSampler.transform`: the input value maps to a rarity
/// through the mapper, the noise is sampled at coordinates divided by that
/// rarity, and the absolute value is scaled back up by the rarity:
/// `rarity * |noise(x/rarity, y/rarity, z/rarity)|`. The volume path is the
/// default pointwise loop: the rarity varies per point, so no bulk path
/// applies. 26.2 dropped the type; the overworld spaghetti caves need it.
#[derive(Debug)]
pub struct WeirdScaledNoiseSampler {
    pub input: Arc<dyn DensitySampler>,
    pub noise: Arc<dyn Noise>,
    pub mapper: WeirdRarityMapper,
}
impl DensitySampler for WeirdScaledNoiseSampler {
    fn pointwise_inputs(&self) -> Option<Vec<&Arc<dyn DensitySampler>>> {
        Some(vec![&self.input])
    }
    fn sample_value(
        &self,
        context: &mut SamplerContext,
        [x, y, z]: [i32; 3],
    ) -> Result<f64, SampleError> {
        let rarity = self
            .mapper
            .rarity(self.input.sample_value(context, [x, y, z])?);
        Ok(rarity
            * self
                .noise
                .get3(x as f64 / rarity, y as f64 / rarity, z as f64 / rarity)
                .abs())
    }
}

/// Compile the three offset-noise variants without changing their bulk paths.
pub fn compile_shift_noise(noise: Arc<dyn Noise>, kind: ShiftNoiseKind) -> Arc<dyn DensitySampler> {
    if kind == ShiftNoiseKind::ShiftB {
        return Arc::new(ShiftBSampler { noise });
    }
    Arc::new(UnarySampler {
        input: Arc::new(NoiseSampler {
            noise,
            xz_scale: 0.25,
            y_scale: if kind == ShiftNoiseKind::ShiftA {
                0.0
            } else {
                0.25
            },
        }),
        operation: UnaryOp::Multiply(4.0),
    })
}
#[derive(Clone, Debug)]
struct ShiftBSampler {
    noise: Arc<dyn Noise>,
}
impl DensitySampler for ShiftBSampler {
    fn sample_value(
        &self,
        _: &mut SamplerContext,
        [x, _, z]: [i32; 3],
    ) -> Result<f64, SampleError> {
        Ok(self.noise.get3(z as f64 * 0.25, x as f64 * 0.25, 0.0) * 4.0)
    }
    fn sample_volume(
        &self,
        context: &mut SamplerContext,
        buffer: &mut [f64],
        volume: DensityVolume,
    ) -> Result<(), SampleError> {
        check_buffer(buffer, volume)?;
        let [sx, sy, sz] = volume.sizes();
        let [mx, _, mz] = volume.minimum();
        let [dx, _, dz] = volume.steps();
        let transposed = DensityVolume::new([sz as i32, sx as i32, 1], [mz, mx, 0], [dz, dx, 1])?;
        context.with_buffer(transposed.size(), |_, plane| {
            plane.fill(0.0);
            self.noise
                .add_to_volume(plane, transposed, 0.25, 0.25, 4.0)?;
            for z in 0..sz {
                for x in 0..sx {
                    let start = (x + z * sx) * sy;
                    buffer[start..start + sy].fill(plane[x + z * sx]);
                }
            }
            Ok(())
        })
    }
}

/// Parameters for vanilla's old blended terrain density function. Serialized
/// codec limits and compile-context seed lookup are separate integration work.
#[derive(Clone, Copy, Debug)]
pub struct BlendedNoise {
    pub xz_scale: f64,
    pub y_scale: f64,
    pub xz_factor: f64,
    pub y_factor: f64,
    pub smear_scale_multiplier: f64,
}

/// A compiled old blended noise as a density sampler, backed by the
/// faithful f64 synth `BlendedNoise` — vanilla 26.1's `BlendedNoise` class,
/// which IS the `minecraft:old_blended_noise` density function
/// (`DensityFunction.SimpleFunction`, double throughout). The previous
/// float `SmearedPerlinNoise` stacks rounded every octave value to f32,
/// flipping near-zero densities the same way the float NormalNoise did.
/// Bulk sampling uses the trait's scalar traversal; vanilla computes this
/// function per block too.
#[derive(Debug)]
pub struct BlendedNoiseSampler {
    noise: synth::BlendedNoise,
}
impl DensitySampler for BlendedNoiseSampler {
    fn sample_value(
        &self,
        _: &mut SamplerContext,
        [x, y, z]: [i32; 3],
    ) -> Result<f64, SampleError> {
        Ok(self.noise.compute(x, y, z))
    }
}
impl BlendedNoise {
    /// Compile against the faithful f64 synth noise. The random stream
    /// matches vanilla's `BlendedNoise` constructor (min limit, max limit,
    /// then main, each built from the caller's stream in octave order).
    pub fn compile(
        &self,
        random: &mut impl RandomSource,
    ) -> Result<BlendedNoiseSampler, NoiseBuildError> {
        Ok(BlendedNoiseSampler {
            noise: synth::BlendedNoise::new(
                random,
                self.xz_scale,
                self.y_scale,
                self.xz_factor,
                self.y_factor,
                self.smear_scale_multiplier,
            )
            .map_err(NoiseBuildError::from)?,
        })
    }
    pub fn range(&self) -> Result<Interval, IntervalError> {
        let smear = (684.412 * self.y_scale) * self.smear_scale_multiplier;
        let mut range = Interval::exact(0.0)?;
        let mut frequency = 1.0;
        let mut amplitude = 0.9999847412109375 / 65535.0;
        for _ in 0..16 {
            let base = Interval::symmetric((smear * frequency).abs() + 2.0)?;
            range = range.add_range(base.multiply_range(Interval::exact(amplitude)?));
            frequency /= 2.0;
            amplitude *= 2.0;
        }
        Ok(range)
    }
}

/// Construct high-to-low frequency smeared octaves, consuming the caller's
/// stream directly. The public API caps work at 4096 layers before any draws.
pub fn create_blended_fbm(
    random: &mut impl RandomSource,
    first_octave: i32,
    smear: f64,
    amplitude: f64,
) -> Result<NoiseStack, NoiseBuildError> {
    if first_octave > 0 {
        return Err(NoiseBuildError::PositiveLegacyOctaves);
    }
    if first_octave < -4095 {
        return Err(NoiseBuildError::TooManyOctaves);
    }
    let count = 1 - first_octave;
    let power = if count > 1023 {
        f64::INFINITY
    } else {
        f64::from_bits(((count + 1023) as u64) << 52)
    };
    let mut amplitude = amplitude / (power - 1.0);
    let mut frequency = 1.0;
    let mut builder = NoiseStack::builder();
    for _ in 0..count {
        builder.add(
            Arc::new(SmearedPerlinNoise::new(random, smear * frequency)),
            frequency,
            amplitude,
        );
        frequency /= 2.0;
        amplitude *= 2.0;
    }
    Ok(builder.build()?)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DistanceMetric {
    Euclidean,
    EuclideanSquared,
    Manhattan,
    Chebyshev,
}
impl DistanceMetric {
    pub fn compute2(self, x: f64, y: f64) -> f64 {
        match self {
            Self::Euclidean => (x * x + y * y).sqrt(),
            Self::EuclideanSquared => x * x + y * y,
            Self::Manhattan => x.abs() + y.abs(),
            Self::Chebyshev => arithmetic::java_max(x.abs(), y.abs()),
        }
    }
    pub fn compute3(self, x: f64, y: f64, z: f64) -> f64 {
        match self {
            Self::Euclidean => (x * x + y * y + z * z).sqrt(),
            Self::EuclideanSquared => x * x + y * y + z * z,
            Self::Manhattan => x.abs() + y.abs() + z.abs(),
            Self::Chebyshev => {
                arithmetic::java_max(arithmetic::java_max(x.abs(), y.abs()), z.abs())
            }
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct DistanceToPointSampler {
    pub point: [i32; 3],
    pub metric: DistanceMetric,
}
impl DensitySampler for DistanceToPointSampler {
    fn sample_value(&self, _: &mut SamplerContext, position: [i32; 3]) -> Result<f64, SampleError> {
        let delta: [f64; 3] =
            std::array::from_fn(|i| self.point[i].wrapping_sub(position[i]) as f64);
        Ok(self.metric.compute3(delta[0], delta[1], delta[2]))
    }
}
