//! Vanilla's `WeirdScaledSampler` density function (`minecraft:weird_scaled_sampler`,
//! 1.21.7-era; 26.x dropped the type). The input is mapped through a rarity
//! curve to a scale factor `d`, the noise is sampled at coordinates divided by
//! `d`, and the result is `d * |noise|` — always non-negative. The rarity
//! curves are `NoiseRouterData$QuantizedSpaghettiRarity` (verified against the
//! 1.21.11 named jar's bytecode): type_1 is the 3D spaghetti rarity, type_2
//! the 2D one. The earlier clamp-to-[0,1] stand-in sampled the noise at raw
//! coordinates and could go negative, which collapsed `overworld/caves/entrances`
//! to about -0.08 wherever the rarity noise was negative and turned whole
//! mountain columns into air.
use super::{DensitySampler, SampleError, check_buffer, visit};
use crate::noise::{DensityVolume, Noise};
use std::sync::Arc;

/// Vanilla's `RarityValueMapper`: the step curves quantizing the rarity
/// noise into a noise scale. NaN falls through every comparison to the
/// last step, as Java's `dcmpg` does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RarityValueMapper {
    Type1,
    Type2,
}
impl RarityValueMapper {
    pub fn map(self, value: f64) -> f64 {
        match self {
            Self::Type1 => {
                if value < -0.5 {
                    0.75
                } else if value < 0.0 {
                    1.0
                } else if value < 0.5 {
                    1.5
                } else {
                    2.0
                }
            }
            Self::Type2 => {
                if value < -0.75 {
                    0.5
                } else if value < -0.5 {
                    0.75
                } else if value < 0.5 {
                    1.0
                } else if value < 0.75 {
                    2.0
                } else {
                    3.0
                }
            }
        }
    }
    /// The mapper's largest output: the range bound (`maxRarity`).
    pub fn max_rarity(self) -> f64 {
        match self {
            Self::Type1 => 2.0,
            Self::Type2 => 3.0,
        }
    }
}

/// `d * |noise(x/d, y/d, z/d)|`, with `d` the mapped input at the same
/// position. The scale and the final product are doubles, as in vanilla;
/// only the noise value itself is the pipeline's `f32`.
#[derive(Clone, Debug)]
pub struct WeirdScaledSampler {
    pub input: Arc<dyn DensitySampler>,
    pub noise: Arc<dyn Noise>,
    pub mapper: RarityValueMapper,
}
impl WeirdScaledSampler {
    fn compute(&self, input: f32, x: i32, y: i32, z: i32) -> f32 {
        let d = self.mapper.map(f64::from(input));
        (d * (self.noise.get3(x as f64 / d, y as f64 / d, z as f64 / d) as f64).abs()) as f32
    }
}
impl DensitySampler for WeirdScaledSampler {
    fn pointwise_inputs(&self) -> Option<Vec<&Arc<dyn DensitySampler>>> {
        Some(vec![&self.input])
    }
    fn sample_value(
        &self,
        context: &mut super::SamplerContext,
        position: [i32; 3],
    ) -> Result<f32, SampleError> {
        let input = self.input.sample_value(context, position)?;
        Ok(self.compute(input, position[0], position[1], position[2]))
    }
    fn sample_volume(
        &self,
        context: &mut super::SamplerContext,
        buffer: &mut [f32],
        volume: DensityVolume,
    ) -> Result<(), SampleError> {
        check_buffer(buffer, volume)?;
        self.input.sample_volume(context, buffer, volume)?;
        let noise = &self.noise;
        let mapper = self.mapper;
        visit(volume, |index, [x, y, z]| {
            let d = mapper.map(f64::from(buffer[index]));
            buffer[index] =
                (d * (noise.get3(x as f64 / d, y as f64 / d, z as f64 / d) as f64).abs()) as f32;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::density::{ConstantSampler, SamplerContext};
    use crate::noise::NanRange;

    /// A noise returning a fixed value, remembering the coordinates it was
    /// asked for: the scale division is observable through them.
    #[derive(Debug)]
    struct FixedNoise {
        value: f32,
        seen: std::sync::Mutex<Vec<[f64; 3]>>,
    }
    impl Noise for FixedNoise {
        fn range(&self) -> Result<[f32; 2], NanRange> {
            Ok([self.value, self.value])
        }
        fn get2(&self, _x: f64, _z: f64) -> f32 {
            self.value
        }
        fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
            self.seen.lock().unwrap().push([x, y, z]);
            self.value
        }
    }

    #[test]
    fn rarity_mappers_match_vanillas_quantized_steps() {
        // Type 1 (3D): -0.5 -> 0.75, 0 -> 1.0, 0.5 -> 1.5, else 2.0.
        let one = RarityValueMapper::Type1;
        assert_eq!(one.map(-1.0), 0.75);
        assert_eq!(one.map(-0.5), 1.0);
        assert_eq!(one.map(-0.25), 1.0);
        assert_eq!(one.map(0.0), 1.5);
        assert_eq!(one.map(0.49), 1.5);
        assert_eq!(one.map(0.5), 2.0);
        assert_eq!(one.map(f64::NAN), 2.0);
        assert_eq!(one.max_rarity(), 2.0);
        // Type 2 (2D): -0.75 -> 0.5, -0.5 -> 0.75, 0.5 -> 1.0, 0.75 -> 2.0,
        // else 3.0.
        let two = RarityValueMapper::Type2;
        assert_eq!(two.map(-1.0), 0.5);
        assert_eq!(two.map(-0.75), 0.75);
        assert_eq!(two.map(-0.5), 1.0);
        assert_eq!(two.map(0.0), 1.0);
        assert_eq!(two.map(0.5), 2.0);
        assert_eq!(two.map(0.75), 3.0);
        assert_eq!(two.map(f64::NAN), 3.0);
        assert_eq!(two.max_rarity(), 3.0);
    }

    #[test]
    fn sampler_scales_coordinates_and_takes_the_absolute_value() {
        let noise = Arc::new(FixedNoise {
            value: -0.5,
            seen: std::sync::Mutex::new(Vec::new()),
        });
        // Input 0.25 maps to 1.5 (type 1): coordinates divide by 1.5, the
        // result is 1.5 * |-0.5| = 0.75, never negative.
        let sampler = WeirdScaledSampler {
            input: Arc::new(ConstantSampler(0.25)),
            noise: noise.clone(),
            mapper: RarityValueMapper::Type1,
        };
        let mut context = SamplerContext::default();
        let value = sampler.sample_value(&mut context, [30, 15, -45]).unwrap();
        assert_eq!(value, 0.75);
        {
            let seen = noise.seen.lock().unwrap();
            assert_eq!(seen.len(), 1);
            assert_eq!(seen[0], [20.0, 10.0, -30.0]);
        } // Release the guard: the next sample locks the same mutex.
        // A negative input still maps to a positive scale (0.75), so the
        // output stays non-negative where the old clamp gave zero.
        let sampler = WeirdScaledSampler {
            input: Arc::new(ConstantSampler(-1.0)),
            noise: noise.clone(),
            mapper: RarityValueMapper::Type1,
        };
        assert_eq!(
            sampler.sample_value(&mut context, [0, 0, 0]).unwrap(),
            0.375
        );
    }

    #[test]
    fn bulk_sampling_matches_scalar_sampling() {
        let noise = Arc::new(FixedNoise {
            value: 0.25,
            seen: std::sync::Mutex::new(Vec::new()),
        });
        let sampler = WeirdScaledSampler {
            input: Arc::new(ConstantSampler(1.0)),
            noise,
            mapper: RarityValueMapper::Type2,
        };
        // Input 1.0 maps to 3.0 (type 2): every cell is 3.0 * |0.25|.
        let volume = DensityVolume::new([2, 3, 2], [0, 0, 0], [1, 1, 1]).unwrap();
        let mut buffer = vec![f32::NAN; volume.size()];
        let mut context = SamplerContext::default();
        sampler
            .sample_volume(&mut context, &mut buffer, volume)
            .unwrap();
        assert!(buffer.iter().all(|v| *v == 0.75));
    }
}
