use super::{DensityError, DensityVolume, NanRange, PerlinNoise, SimplexNoise, SmearedPerlinNoise};
use std::sync::Arc;

/// Immutable noise sampling. Implementations may override the default scalar
/// volume traversal when vanilla has a distinct bulk arithmetic path.
pub trait Noise: std::fmt::Debug + Send + Sync {
    fn range(&self) -> Result<[f64; 2], NanRange>;
    fn get2(&self, x: f64, z: f64) -> f64;
    fn get3(&self, x: f64, y: f64, z: f64) -> f64;
    fn add_to_volume(
        &self,
        buffer: &mut [f64],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f64,
    ) -> Result<(), DensityError> {
        if buffer.len() < volume.size() {
            return Err(DensityError::BufferTooShort);
        }
        let [sx, sy, sz] = volume.sizes();
        let mut index = 0;
        for z in 0..sz {
            let z = f64::from(volume.coordinate(2, z)) * xz_scale;
            for x in 0..sx {
                let x = f64::from(volume.coordinate(0, x)) * xz_scale;
                for y in 0..sy {
                    let y = f64::from(volume.coordinate(1, y)) * y_scale;
                    buffer[index] += amplitude * self.get3(x, y, z);
                    index += 1;
                }
            }
        }
        Ok(())
    }
    /// [`Noise::add_to_volume`] for a caller that reads only the Ys set in `needed`: bit `y` of
    /// `needed[column]`, X/Z columns in buffer order. The other Ys may be left unchanged.
    fn add_to_needed(
        &self,
        buffer: &mut [f64],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f64,
        _needed: &[u64],
    ) -> Result<(), DensityError> {
        self.add_to_volume(buffer, volume, xz_scale, y_scale, amplitude)
    }
}

impl Noise for PerlinNoise {
    fn range(&self) -> Result<[f64; 2], NanRange> {
        let range = PerlinNoise::range(self);
        Ok([f64::from(range[0]), f64::from(range[1])])
    }
    fn get2(&self, x: f64, z: f64) -> f64 {
        f64::from(PerlinNoise::get2(self, x, z))
    }
    fn get3(&self, x: f64, y: f64, z: f64) -> f64 {
        f64::from(PerlinNoise::get3(self, x, y, z))
    }
    fn add_to_volume(
        &self,
        buffer: &mut [f64],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f64,
    ) -> Result<(), DensityError> {
        // The float engine's bulk path stays f32; widen its output into the f64 buffer.
        if buffer.len() < volume.size() {
            return Err(DensityError::BufferTooShort);
        }
        let mut tmp = vec![0.0f32; volume.size()];
        PerlinNoise::add_to_volume(self, &mut tmp, volume, xz_scale, y_scale, amplitude as f32)?;
        for (out, v) in buffer.iter_mut().zip(tmp.iter()) {
            *out += f64::from(*v);
        }
        Ok(())
    }
    fn add_to_needed(
        &self,
        buffer: &mut [f64],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f64,
        needed: &[u64],
    ) -> Result<(), DensityError> {
        if buffer.len() < volume.size() {
            return Err(DensityError::BufferTooShort);
        }
        let mut tmp = vec![0.0f32; volume.size()];
        PerlinNoise::add_to_needed(self, &mut tmp, volume, xz_scale, y_scale, amplitude as f32, needed)?;
        for (out, v) in buffer.iter_mut().zip(tmp.iter()) {
            *out += f64::from(*v);
        }
        Ok(())
    }
}
impl Noise for SmearedPerlinNoise {
    fn range(&self) -> Result<[f64; 2], NanRange> {
        let range = SmearedPerlinNoise::range(self)?;
        Ok([f64::from(range[0]), f64::from(range[1])])
    }
    fn get2(&self, x: f64, z: f64) -> f64 {
        f64::from(SmearedPerlinNoise::get2(self, x, z))
    }
    fn get3(&self, x: f64, y: f64, z: f64) -> f64 {
        f64::from(SmearedPerlinNoise::get3(self, x, y, z))
    }
    fn add_to_volume(
        &self,
        buffer: &mut [f64],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f64,
    ) -> Result<(), DensityError> {
        if buffer.len() < volume.size() {
            return Err(DensityError::BufferTooShort);
        }
        let mut tmp = vec![0.0f32; volume.size()];
        SmearedPerlinNoise::add_to_volume(self, &mut tmp, volume, xz_scale, y_scale, amplitude as f32)?;
        for (out, v) in buffer.iter_mut().zip(tmp.iter()) {
            *out += f64::from(*v);
        }
        Ok(())
    }
    fn add_to_needed(
        &self,
        buffer: &mut [f64],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f64,
        needed: &[u64],
    ) -> Result<(), DensityError> {
        if buffer.len() < volume.size() {
            return Err(DensityError::BufferTooShort);
        }
        let mut tmp = vec![0.0f32; volume.size()];
        SmearedPerlinNoise::add_to_needed(
            self, &mut tmp, volume, xz_scale, y_scale, amplitude as f32, needed,
        )?;
        for (out, v) in buffer.iter_mut().zip(tmp.iter()) {
            *out += f64::from(*v);
        }
        Ok(())
    }
}
impl Noise for SimplexNoise {
    fn range(&self) -> Result<[f64; 2], NanRange> {
        let range = SimplexNoise::range(self);
        Ok([f64::from(range[0]), f64::from(range[1])])
    }
    fn get2(&self, x: f64, z: f64) -> f64 {
        f64::from(SimplexNoise::get2(self, x, z))
    }
    fn get3(&self, x: f64, y: f64, z: f64) -> f64 {
        f64::from(SimplexNoise::get3(self, x, y, z))
    }
}

#[derive(Clone, Debug)]
pub struct NoiseLayer {
    pub noise: Arc<dyn Noise>,
    pub frequency: f64,
    pub amplitude: f64,
}

#[derive(Clone, Debug)]
pub struct NoiseStack {
    layers: Vec<NoiseLayer>,
    range: [f64; 2],
}

impl NoiseStack {
    pub fn builder() -> NoiseStackBuilder {
        NoiseStackBuilder::default()
    }
    pub fn layers(&self) -> &[NoiseLayer] {
        &self.layers
    }
    pub fn get_layer(&self, index: usize) -> Option<&Arc<dyn Noise>> {
        self.layers.get(index).map(|layer| &layer.noise)
    }
}
impl Noise for NoiseStack {
    fn range(&self) -> Result<[f64; 2], NanRange> {
        Ok(self.range)
    }
    fn get2(&self, x: f64, z: f64) -> f64 {
        let mut value = 0.0;
        for layer in &self.layers {
            value += layer.amplitude * layer.noise.get2(x * layer.frequency, z * layer.frequency);
        }
        value
    }
    fn get3(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut value = 0.0;
        for layer in &self.layers {
            value += layer.amplitude
                * layer.noise.get3(
                    x * layer.frequency,
                    y * layer.frequency,
                    z * layer.frequency,
                );
        }
        value
    }
    fn add_to_volume(
        &self,
        buffer: &mut [f64],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f64,
    ) -> Result<(), DensityError> {
        self.add_layers(buffer, volume, [xz_scale, y_scale], amplitude, None)
    }
    fn add_to_needed(
        &self,
        buffer: &mut [f64],
        volume: DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f64,
        needed: &[u64],
    ) -> Result<(), DensityError> {
        self.add_layers(buffer, volume, [xz_scale, y_scale], amplitude, Some(needed))
    }
}
impl NoiseStack {
    fn add_layers(
        &self,
        buffer: &mut [f64],
        volume: DensityVolume,
        [xz_scale, y_scale]: [f64; 2],
        amplitude: f64,
        needed: Option<&[u64]>,
    ) -> Result<(), DensityError> {
        if buffer.len() < volume.size() {
            return Err(DensityError::BufferTooShort);
        }
        for layer in &self.layers {
            let (xz_scale, y_scale) = (xz_scale * layer.frequency, y_scale * layer.frequency);
            let amplitude = amplitude * layer.amplitude;
            match needed {
                Some(needed) => layer
                    .noise
                    .add_to_needed(buffer, volume, xz_scale, y_scale, amplitude, needed)?,
                None => layer
                    .noise
                    .add_to_volume(buffer, volume, xz_scale, y_scale, amplitude)?,
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct NoiseStackBuilder {
    layers: Vec<NoiseLayer>,
}
impl NoiseStackBuilder {
    pub fn add(&mut self, noise: Arc<dyn Noise>, frequency: f64, amplitude: f64) -> &mut Self {
        self.layers.push(NoiseLayer {
            noise,
            frequency,
            amplitude,
        });
        self
    }
    /// Flattening rescales each existing layer before subsequent sampling.
    pub fn add_stack(&mut self, stack: &NoiseStack, frequency: f64, amplitude: f64) -> &mut Self {
        self.layers
            .extend(stack.layers.iter().map(|layer| NoiseLayer {
                noise: Arc::clone(&layer.noise),
                frequency: layer.frequency * frequency,
                amplitude: layer.amplitude * amplitude,
            }));
        self
    }
    pub fn build(&self) -> Result<NoiseStack, NanRange> {
        let mut range = [0.0; 2];
        for layer in &self.layers {
            let bounds = layer.noise.range()?;
            if layer.amplitude.is_nan() {
                return Err(NanRange);
            }
            let scaled = if bounds[0].is_nan() || bounds[1].is_nan() {
                [f64::NAN; 2]
            } else {
                // Interval.mulBound defines zero times infinity as positive zero.
                let product = |v: f64| {
                    if v == 0.0 || layer.amplitude == 0.0 {
                        0.0
                    } else {
                        v * layer.amplitude
                    }
                };
                let a = product(bounds[0]);
                let b = product(bounds[1]);
                if a < b {
                    [a, b]
                } else if a > b {
                    [b, a]
                } else if a == 0.0 {
                    [
                        f64::from_bits(a.to_bits() | b.to_bits()),
                        f64::from_bits(a.to_bits() & b.to_bits()),
                    ]
                } else {
                    [a, b]
                }
            };
            range = [range[0] + scaled[0], range[1] + scaled[1]];
            if range[0].is_nan() || range[1].is_nan() {
                range = [f64::NAN; 2];
            }
            // Interval.of interns [0,1], including a supplied negative zero.
            if range[0] == 0.0 && range[1] == 1.0 {
                range[0] = 0.0;
            }
        }
        Ok(NoiseStack {
            layers: self.layers.clone(),
            range,
        })
    }
}
