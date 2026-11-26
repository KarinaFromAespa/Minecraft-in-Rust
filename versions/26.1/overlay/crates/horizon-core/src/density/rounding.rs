use super::*;

#[derive(Clone, Copy, Debug)]
pub enum RoundMode {
    Floor,
    Round,
    Ceil,
    Truncate,
}
impl RoundMode {
    pub(super) fn integer(self, value: f64) -> f64 {
        match self {
            Self::Floor => value.floor(),
            Self::Ceil => value.ceil(),
            Self::Truncate => {
                if value > 0.0 {
                    value.floor()
                } else {
                    value.ceil()
                }
            }
            Self::Round => {
                // Math.round(double): ties go toward positive infinity, NaNs
                // become zero, and the result saturates to Java's long domain.
                let bits = value.to_bits() as i64;
                let shift = 1074 - ((bits >> 52) & 0x7ff);
                let rounded = if (0..64).contains(&shift) {
                    let mut significand = (bits & 0xF_FFFF_FFFF_FFFF) | 0x10_0000_0000_0000;
                    if bits < 0 {
                        significand = -significand;
                    }
                    ((significand >> shift) + 1) >> 1
                } else {
                    value as i64
                };
                rounded as f64
            }
        }
    }
    fn apply(self, input: f64, multiple: f64) -> f64 {
        if multiple == 0.0 {
            input
        } else {
            self.integer(input / multiple) * multiple
        }
    }
}

/// A missing multiple selects vanilla's literal-one specialization. A supplied
/// sampler is evaluated even when its value is zero or always one.
#[derive(Clone, Debug)]
pub struct RoundSampler {
    pub mode: RoundMode,
    pub input: Arc<dyn DensitySampler>,
    pub multiple: Option<Arc<dyn DensitySampler>>,
}
impl DensitySampler for RoundSampler {
    fn sample_value(
        &self,
        context: &mut SamplerContext,
        position: [i32; 3],
    ) -> Result<f64, SampleError> {
        let input = self.input.sample_value(context, position)?;
        Ok(if let Some(multiple) = &self.multiple {
            self.mode
                .apply(input, multiple.sample_value(context, position)?)
        } else {
            self.mode.integer(input)
        })
    }
    fn sample_volume(
        &self,
        context: &mut SamplerContext,
        buffer: &mut [f64],
        volume: DensityVolume,
    ) -> Result<(), SampleError> {
        if self.multiple.is_some() {
            check_exact_buffer(buffer, volume)?;
        } else {
            check_buffer(buffer, volume)?;
        }
        self.input.sample_volume(context, buffer, volume)?;
        if let Some(multiple) = &self.multiple {
            context.with_buffer(volume.size(), |context, scratch| {
                multiple.sample_volume(context, scratch, volume)?;
                for (input, multiple) in buffer.iter_mut().zip(scratch) {
                    *input = self.mode.apply(*input, *multiple);
                }
                Ok(())
            })
        } else {
            for value in buffer {
                *value = self.mode.integer(*value);
            }
            Ok(())
        }
    }
}
