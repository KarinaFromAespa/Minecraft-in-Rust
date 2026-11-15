//! Vanilla 26.2's `world.level.levelgen.synth` package, in double precision.
//!
//! The sibling modules in this directory are Horizon's own float sampler
//! engine (the 26.3 `GradientNoise`/`NoiseStack` model): octave structures
//! and seed consumption match 26.2, but sampling runs in `f32`. Vanilla 26.2
//! computes noise in `f64` throughout (`ImprovedNoise.noise`,
//! `PerlinNoise.getValue`, `NormalNoise.getValue`, `SimplexNoise.getValue`),
//! and the difference is observable wherever a sampled value feeds a
//! discrete decision — `NoiseBasedCountPlacement` takes
//! `ceil((noise + offset) * ratio)`, so an `f32`-rounded noise flips counts
//! at extreme ratios (792 placement fixtures).
//!
//! This module is the faithful 26.2 port: `ImprovedNoise`, `PerlinNoise`
//! (octave FBM, new and legacy initialization), `SimplexNoise` (whose
//! `getValue` does NOT apply the offsets — callers add them, as
//! `PerlinSimplexNoise` does when `useNoiseStart` is set),
//! `PerlinSimplexNoise`, `NormalNoise` with its `NoiseParameters`
//! (`{firstOctave, amplitudes}`), `BlendedNoise`, and `NoiseUtils`.
//! Operation order mirrors the decompiled 26.2 sources exactly; the noise
//! parity harness compares these types against the 26.2 jars bit-for-bit.
//!
//! The float engine stays for the density graph, whose bulk samplers are
//! `f32` end to end — converting that engine to `f64` is programme-level
//! work (see PORT-NOTES-26.2.md, item 115 follow-up), not this module's job.

use super::{GRADIENTS, NoiseBuildError};
use crate::{
    geometry::floor_i32,
    jmath,
    random::{LegacyRandom, PositionalRandom, RandomSource, XoroshiroRandom},
};
use std::fmt::Write as _;

/// `Mth.lerp`, double form: `p0 + a * (p1 - p0)`.
fn lerp(a: f64, p0: f64, p1: f64) -> f64 {
    p0 + a * (p1 - p0)
}

/// `Mth.lerp2`, double form.
fn lerp2(a1: f64, a2: f64, x00: f64, x10: f64, x01: f64, x11: f64) -> f64 {
    lerp(a2, lerp(a1, x00, x10), lerp(a1, x01, x11))
}

/// `Mth.lerp3`, double form.
#[allow(clippy::too_many_arguments)]
fn lerp3(
    a1: f64,
    a2: f64,
    a3: f64,
    x000: f64,
    x100: f64,
    x010: f64,
    x110: f64,
    x001: f64,
    x101: f64,
    x011: f64,
    x111: f64,
) -> f64 {
    lerp(
        a3,
        lerp2(a1, a2, x000, x100, x010, x110),
        lerp2(a1, a2, x001, x101, x011, x111),
    )
}

/// `Mth.smoothstep`, double form.
fn smoothstep(x: f64) -> f64 {
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

/// `Mth.smoothstepDerivative`, double form.
fn smoothstep_derivative(x: f64) -> f64 {
    30.0 * x * x * (x - 1.0) * (x - 1.0)
}

/// `Mth.clampedLerp`, double form.
fn clamped_lerp(factor: f64, min: f64, max: f64) -> f64 {
    if factor < 0.0 {
        min
    } else if factor > 1.0 {
        max
    } else {
        lerp(factor, min, max)
    }
}

/// `Mth.lfloor`: `(long) Math.floor(v)`. Mirrors [`floor_i32`] with `i64`
/// saturation: NaN is 0, out-of-range values saturate.
fn lfloor_i64(value: f64) -> i64 {
    let truncated = value as i64;
    if value < truncated as f64 {
        truncated.saturating_sub(1)
    } else {
        truncated
    }
}

/// `Math.pow(2, exponent)` exactly, without a platform libm approximation —
/// the same construction as the float engine's `power_of_two`, including
/// exact subnormal powers and saturation.
fn power_of_two(exponent: i32) -> f64 {
    match exponent {
        ..=-1075 => 0.0,
        -1074..=-1023 => f64::from_bits(1_u64 << (exponent + 1074)),
        -1022..=1023 => f64::from_bits(((exponent + 1023) as u64) << 52),
        _ => f64::INFINITY,
    }
}

/// `String.format(Locale.ROOT, "%.<precision>f", value)` as Java renders it:
/// non-finite values spell out (`NaN`, `Infinity`, `-Infinity`); finite
/// values round to nearest, and an exact halfway tie is impossible for a
/// binary float at a decimal rounding point (the tie would need a factor of
/// 5 in the denominator), so Rust's `{:.prec$}` agrees with Java's HALF_UP.
fn java_format_fixed(value: f64, precision: usize) -> String {
    if value.is_nan() {
        "NaN".to_owned()
    } else if value == f64::INFINITY {
        "Infinity".to_owned()
    } else if value == f64::NEG_INFINITY {
        "-Infinity".to_owned()
    } else {
        format!("{value:.precision$}")
    }
}

fn dot_grad(g: [i32; 3], x: f64, y: f64, z: f64) -> f64 {
    g[0] as f64 * x + g[1] as f64 * y + g[2] as f64 * z
}

/// What vanilla throws while building noise: `IllegalArgumentException`
/// (empty octave set, too few octaves, positive legacy octaves) and
/// `IllegalStateException` (the legacy initializer built the wrong number
/// of levels). The parity probe reports every variant as one `error` line,
/// matching the oracle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SynthError {
    EmptyOctaves,
    TooFewOctaves,
    PositiveLegacyOctaves,
    MissingNoiseLevels,
}
impl std::fmt::Display for SynthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::EmptyOctaves => "Need some octaves!",
            Self::TooFewOctaves => "Total number of octaves needs to be >= 1",
            Self::PositiveLegacyOctaves => "Positive octaves are temporarily disabled",
            Self::MissingNoiseLevels => {
                "Failed to create correct number of noise levels for given non-zero amplitudes"
            }
        })
    }
}
impl std::error::Error for SynthError {}
impl From<SynthError> for NoiseBuildError {
    fn from(_: SynthError) -> Self {
        Self::MissingNoiseLevels
    }
}

/// Vanilla 26.2 `ImprovedNoise`: one gradient-noise octave in doubles.
/// The permutation table is `byte[256]` in Java; the bytes are read with
/// `& 0xFF`, so `u8` holds the same values (the parity config prints `p0`
/// and `p255` as *signed* bytes — see [`Self::parity_config_string`]).
#[derive(Clone, Debug)]
pub struct ImprovedNoise {
    p: [u8; 256],
    pub xo: f64,
    pub yo: f64,
    pub zo: f64,
}

impl ImprovedNoise {
    pub fn new(random: &mut impl RandomSource) -> Self {
        let xo = random.next_double() * 256.0;
        let yo = random.next_double() * 256.0;
        let zo = random.next_double() * 256.0;
        let mut p = [0u8; 256];
        for (i, slot) in p.iter_mut().enumerate() {
            *slot = i as u8;
        }
        for i in 0..256usize {
            // Bound is always positive (256 - i >= 1).
            let offset = random.next_int_bound(256 - i as i32).unwrap() as usize;
            p.swap(i, i + offset);
        }
        Self { p, xo, yo, zo }
    }

    pub fn permutations(&self) -> &[u8; 256] {
        &self.p
    }

    fn permute(&self, index: i32) -> i32 {
        i32::from(self.p[(index & 255) as usize])
    }

    pub fn noise(&self, x: f64, y: f64, z: f64) -> f64 {
        self.noise_smeared(x, y, z, 0.0, 0.0)
    }

    /// `noise(x, y, z, yScale, yFudge)`: the smear form `PerlinNoise` and
    /// `BlendedNoise` call. `SHIFT_UP_EPSILON` is the `float` literal
    /// `1.0E-7F`, widened to double — `1e-7f32 as f64`, not `1e-7f64`.
    pub fn noise_smeared(
        &self,
        x_in: f64,
        y_in: f64,
        z_in: f64,
        y_scale: f64,
        y_fudge: f64,
    ) -> f64 {
        let x = x_in + self.xo;
        let y = y_in + self.yo;
        let z = z_in + self.zo;
        let xf = floor_i32(x);
        let yf = floor_i32(y);
        let zf = floor_i32(z);
        let xr = x - f64::from(xf);
        let yr = y - f64::from(yf);
        let zr = z - f64::from(zf);
        let yr_fudge = if y_scale != 0.0 {
            let fudge_limit = if y_fudge >= 0.0 && y_fudge < yr {
                y_fudge
            } else {
                yr
            };
            f64::from(floor_i32(fudge_limit / y_scale + f64::from(1e-7_f32))) * y_scale
        } else {
            0.0
        };
        self.sample_and_lerp(xf, yf, zf, xr, yr - yr_fudge, zr, yr)
    }

    /// `noiseWithDerivative`: returns the value and *adds* the gradient into
    /// `derivative_out`, exactly as vanilla does.
    pub fn noise_with_derivative(
        &self,
        x_in: f64,
        y_in: f64,
        z_in: f64,
        derivative_out: &mut [f64; 3],
    ) -> f64 {
        let x = x_in + self.xo;
        let y = y_in + self.yo;
        let z = z_in + self.zo;
        let xf = floor_i32(x);
        let yf = floor_i32(y);
        let zf = floor_i32(z);
        let xr = x - f64::from(xf);
        let yr = y - f64::from(yf);
        let zr = z - f64::from(zf);
        self.sample_with_derivative(xf, yf, zf, xr, yr, zr, derivative_out)
    }

    #[allow(clippy::too_many_arguments)]
    fn sample_and_lerp(
        &self,
        x: i32,
        y: i32,
        z: i32,
        xr: f64,
        yr: f64,
        zr: f64,
        yr_original: f64,
    ) -> f64 {
        let x0 = self.permute(x);
        let x1 = self.permute(x.wrapping_add(1));
        let xy00 = self.permute(x0.wrapping_add(y));
        let xy01 = self.permute(x0.wrapping_add(y).wrapping_add(1));
        let xy10 = self.permute(x1.wrapping_add(y));
        let xy11 = self.permute(x1.wrapping_add(y).wrapping_add(1));
        let d000 = dot_grad(
            GRADIENTS[(self.permute(xy00.wrapping_add(z)) & 15) as usize],
            xr,
            yr,
            zr,
        );
        let d100 = dot_grad(
            GRADIENTS[(self.permute(xy10.wrapping_add(z)) & 15) as usize],
            xr - 1.0,
            yr,
            zr,
        );
        let d010 = dot_grad(
            GRADIENTS[(self.permute(xy01.wrapping_add(z)) & 15) as usize],
            xr,
            yr - 1.0,
            zr,
        );
        let d110 = dot_grad(
            GRADIENTS[(self.permute(xy11.wrapping_add(z)) & 15) as usize],
            xr - 1.0,
            yr - 1.0,
            zr,
        );
        let d001 = dot_grad(
            GRADIENTS[(self.permute(xy00.wrapping_add(z).wrapping_add(1)) & 15) as usize],
            xr,
            yr,
            zr - 1.0,
        );
        let d101 = dot_grad(
            GRADIENTS[(self.permute(xy10.wrapping_add(z).wrapping_add(1)) & 15) as usize],
            xr - 1.0,
            yr,
            zr - 1.0,
        );
        let d011 = dot_grad(
            GRADIENTS[(self.permute(xy01.wrapping_add(z).wrapping_add(1)) & 15) as usize],
            xr,
            yr - 1.0,
            zr - 1.0,
        );
        let d111 = dot_grad(
            GRADIENTS[(self.permute(xy11.wrapping_add(z).wrapping_add(1)) & 15) as usize],
            xr - 1.0,
            yr - 1.0,
            zr - 1.0,
        );
        let x_alpha = smoothstep(xr);
        let y_alpha = smoothstep(yr_original);
        let z_alpha = smoothstep(zr);
        lerp3(
            x_alpha, y_alpha, z_alpha, d000, d100, d010, d110, d001, d101, d011, d111,
        )
    }

    fn sample_with_derivative(
        &self,
        x: i32,
        y: i32,
        z: i32,
        xr: f64,
        yr: f64,
        zr: f64,
        derivative_out: &mut [f64; 3],
    ) -> f64 {
        let x0 = self.permute(x);
        let x1 = self.permute(x.wrapping_add(1));
        let xy00 = self.permute(x0.wrapping_add(y));
        let xy01 = self.permute(x0.wrapping_add(y).wrapping_add(1));
        let xy10 = self.permute(x1.wrapping_add(y));
        let xy11 = self.permute(x1.wrapping_add(y).wrapping_add(1));
        let p000 = self.permute(xy00.wrapping_add(z));
        let p100 = self.permute(xy10.wrapping_add(z));
        let p010 = self.permute(xy01.wrapping_add(z));
        let p110 = self.permute(xy11.wrapping_add(z));
        let p001 = self.permute(xy00.wrapping_add(z).wrapping_add(1));
        let p101 = self.permute(xy10.wrapping_add(z).wrapping_add(1));
        let p011 = self.permute(xy01.wrapping_add(z).wrapping_add(1));
        let p111 = self.permute(xy11.wrapping_add(z).wrapping_add(1));
        let g000 = GRADIENTS[(p000 & 15) as usize];
        let g100 = GRADIENTS[(p100 & 15) as usize];
        let g010 = GRADIENTS[(p010 & 15) as usize];
        let g110 = GRADIENTS[(p110 & 15) as usize];
        let g001 = GRADIENTS[(p001 & 15) as usize];
        let g101 = GRADIENTS[(p101 & 15) as usize];
        let g011 = GRADIENTS[(p011 & 15) as usize];
        let g111 = GRADIENTS[(p111 & 15) as usize];
        let d000 = dot_grad(g000, xr, yr, zr);
        let d100 = dot_grad(g100, xr - 1.0, yr, zr);
        let d010 = dot_grad(g010, xr, yr - 1.0, zr);
        let d110 = dot_grad(g110, xr - 1.0, yr - 1.0, zr);
        let d001 = dot_grad(g001, xr, yr, zr - 1.0);
        let d101 = dot_grad(g101, xr - 1.0, yr, zr - 1.0);
        let d011 = dot_grad(g011, xr, yr - 1.0, zr - 1.0);
        let d111 = dot_grad(g111, xr - 1.0, yr - 1.0, zr - 1.0);
        let x_alpha = smoothstep(xr);
        let y_alpha = smoothstep(yr);
        let z_alpha = smoothstep(zr);
        let grad = |axis: usize| {
            lerp3(
                x_alpha,
                y_alpha,
                z_alpha,
                g000[axis] as f64,
                g100[axis] as f64,
                g010[axis] as f64,
                g110[axis] as f64,
                g001[axis] as f64,
                g101[axis] as f64,
                g011[axis] as f64,
                g111[axis] as f64,
            )
        };
        let d1x = grad(0);
        let d1y = grad(1);
        let d1z = grad(2);
        let d2x = lerp2(
            y_alpha,
            z_alpha,
            d100 - d000,
            d110 - d010,
            d101 - d001,
            d111 - d011,
        );
        let d2y = lerp2(
            z_alpha,
            x_alpha,
            d010 - d000,
            d011 - d001,
            d110 - d100,
            d111 - d101,
        );
        let d2z = lerp2(
            x_alpha,
            y_alpha,
            d001 - d000,
            d101 - d100,
            d011 - d010,
            d111 - d110,
        );
        derivative_out[0] += d1x + smoothstep_derivative(xr) * d2x;
        derivative_out[1] += d1y + smoothstep_derivative(yr) * d2y;
        derivative_out[2] += d1z + smoothstep_derivative(zr) * d2z;
        lerp3(
            x_alpha, y_alpha, z_alpha, d000, d100, d010, d110, d001, d101, d011, d111,
        )
    }

    /// `NoiseUtils.parityNoiseOctaveConfigString` for this octave: offsets
    /// as `%.3f` of the `float` casts, `p0`/`p255` as *signed* bytes.
    pub fn parity_config_string(&self) -> String {
        format!(
            "xo={}, yo={}, zo={}, p0={}, p255={}",
            java_format_fixed(f64::from(self.xo as f32), 3),
            java_format_fixed(f64::from(self.yo as f32), 3),
            java_format_fixed(f64::from(self.zo as f32), 3),
            self.p[0] as i8,
            self.p[255] as i8,
        )
    }
}

/// Vanilla 26.2 `PerlinNoise`: a stack of [`ImprovedNoise`] octaves.
/// New initialization seeds each octave positionally (`"octave_N"`);
/// legacy initialization draws sequentially from the caller's stream,
/// skipping 262 draws per absent octave.
#[derive(Clone, Debug)]
pub struct PerlinNoise {
    noise_levels: Vec<Option<ImprovedNoise>>,
    first_octave: i32,
    amplitudes: Vec<f64>,
    lowest_freq_value_factor: f64,
    lowest_freq_input_factor: f64,
    max_value: f64,
}

impl PerlinNoise {
    /// `PerlinNoise.create(random, firstOctave, amplitudes)` — new init.
    pub fn create(
        random: &mut impl RandomSource,
        first_octave: i32,
        amplitudes: &[f64],
    ) -> Result<Self, SynthError> {
        Self::new(random, first_octave, amplitudes.to_vec(), true)
    }

    /// `PerlinNoise.create(random, octaveSet)` — amplitudes of 1.0 on the
    /// given octaves, 0.0 elsewhere, new init.
    pub fn create_from_octaves(
        random: &mut impl RandomSource,
        octave_set: &[i32],
    ) -> Result<Self, SynthError> {
        let (first_octave, amplitudes) = make_amplitudes(octave_set)?;
        Self::new(random, first_octave, amplitudes, true)
    }

    /// `PerlinNoise.createLegacyForBlendedNoise(random, octaves)`.
    pub fn create_legacy_for_blended_noise(
        random: &mut impl RandomSource,
        octave_set: &[i32],
    ) -> Result<Self, SynthError> {
        let (first_octave, amplitudes) = make_amplitudes(octave_set)?;
        Self::new(random, first_octave, amplitudes, false)
    }

    /// `PerlinNoise.createLegacyForLegacyNetherBiome(random, firstOctave,
    /// amplitudes)`.
    pub fn create_legacy_for_legacy_nether_biome(
        random: &mut impl RandomSource,
        first_octave: i32,
        amplitudes: &[f64],
    ) -> Result<Self, SynthError> {
        Self::new(random, first_octave, amplitudes.to_vec(), false)
    }

    fn new(
        random: &mut impl RandomSource,
        first_octave: i32,
        amplitudes: Vec<f64>,
        use_new_initialization: bool,
    ) -> Result<Self, SynthError> {
        let octaves = amplitudes.len() as i32;
        let zero_octave_index = first_octave.wrapping_neg();
        let mut noise_levels: Vec<Option<ImprovedNoise>> =
            (0..amplitudes.len()).map(|_| None).collect();
        if use_new_initialization {
            let positional = random.fork_positional();
            for (i, level) in noise_levels.iter_mut().enumerate() {
                if amplitudes[i] != 0.0 {
                    let octave = first_octave.wrapping_add(i as i32);
                    *level = Some(ImprovedNoise::new(
                        &mut positional.from_hash_of(&format!("octave_{octave}")),
                    ));
                }
            }
        } else {
            let zero_octave = ImprovedNoise::new(random);
            if zero_octave_index >= 0
                && zero_octave_index < octaves
                && amplitudes[zero_octave_index as usize] != 0.0
            {
                noise_levels[zero_octave_index as usize] = Some(zero_octave);
            }
            // `for (int i = zeroOctaveIndex - 1; i >= 0; i--)`, with Java's
            // wrapping int arithmetic.
            let mut i = zero_octave_index.wrapping_sub(1);
            while i >= 0 {
                if i < octaves {
                    if amplitudes[i as usize] != 0.0 {
                        noise_levels[i as usize] = Some(ImprovedNoise::new(random));
                    } else {
                        skip_octave(random);
                    }
                } else {
                    skip_octave(random);
                }
                i = i.wrapping_sub(1);
            }
            if noise_levels.iter().filter(|v| v.is_some()).count()
                != amplitudes.iter().filter(|v| **v != 0.0).count()
            {
                return Err(SynthError::MissingNoiseLevels);
            }
            if zero_octave_index < octaves.wrapping_sub(1) {
                return Err(SynthError::PositiveLegacyOctaves);
            }
        }
        let lowest_freq_input_factor = power_of_two(zero_octave_index.wrapping_neg());
        let lowest_freq_value_factor =
            power_of_two(octaves.wrapping_sub(1)) / (power_of_two(octaves) - 1.0);
        let mut noise = Self {
            noise_levels,
            first_octave,
            amplitudes,
            lowest_freq_value_factor,
            lowest_freq_input_factor,
            max_value: 0.0,
        };
        noise.max_value = noise.edge_value(2.0);
        Ok(noise)
    }

    pub fn first_octave(&self) -> i32 {
        self.first_octave
    }

    pub fn amplitudes(&self) -> &[f64] {
        &self.amplitudes
    }

    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    pub fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        self.get_value_smeared(x, y, z, 0.0, 0.0)
    }

    /// `getValue(x, y, z, yScale, yFudge)`: each octave samples the wrapped
    /// scaled coordinates with the scaled smear.
    pub fn get_value_smeared(&self, x: f64, y: f64, z: f64, y_scale: f64, y_fudge: f64) -> f64 {
        let mut value = 0.0;
        let mut factor = self.lowest_freq_input_factor;
        let mut value_factor = self.lowest_freq_value_factor;
        for (i, level) in self.noise_levels.iter().enumerate() {
            if let Some(noise) = level {
                let noise_val = noise.noise_smeared(
                    wrap(x * factor),
                    wrap(y * factor),
                    wrap(z * factor),
                    y_scale * factor,
                    y_fudge * factor,
                );
                value += self.amplitudes[i] * noise_val * value_factor;
            }
            factor *= 2.0;
            value_factor /= 2.0;
        }
        value
    }

    pub fn max_broken_value(&self, y_scale: f64) -> f64 {
        self.edge_value(y_scale + 2.0)
    }

    fn edge_value(&self, noise_value: f64) -> f64 {
        let mut value = 0.0;
        let mut value_factor = self.lowest_freq_value_factor;
        for (i, level) in self.noise_levels.iter().enumerate() {
            if level.is_some() {
                value += self.amplitudes[i] * noise_value * value_factor;
            }
            value_factor /= 2.0;
        }
        value
    }

    /// `getOctaveNoise(i)`: level `len - 1 - i` (octave 0 first). `None`
    /// when `i` is out of range or that octave is absent.
    pub fn get_octave_noise(&self, i: usize) -> Option<&ImprovedNoise> {
        let len = self.noise_levels.len();
        if i >= len {
            return None;
        }
        self.noise_levels[len - 1 - i].as_ref()
    }

    pub fn parity_config_string(&self) -> String {
        let mut out = String::from("PerlinNoise{");
        let amplitudes: Vec<String> = self
            .amplitudes
            .iter()
            .map(|a| java_format_fixed(*a, 2))
            .collect();
        let _ = write!(
            out,
            "first octave: {}, amplitudes: [{}], noise levels: [",
            self.first_octave,
            amplitudes.join(", ")
        );
        for (i, level) in self.noise_levels.iter().enumerate() {
            let _ = write!(out, "{i}: ");
            match level {
                None => out.push_str("null"),
                Some(noise) => out.push_str(&noise.parity_config_string()),
            }
            out.push_str(", ");
        }
        out.push_str("]}");
        out
    }
}

/// `PerlinNoise.wrap`: `x - lfloor(x / 33554432 + 0.5) * 33554432`.
/// The `lfloor` is a *long* floor — for magnitudes past `i64` range it
/// saturates, unlike the float engine's `wrap`, which floors in `f64`.
pub fn wrap(x: f64) -> f64 {
    x - lfloor_i64(x / 33_554_432.0 + 0.5) as f64 * 33_554_432.0
}

fn skip_octave(random: &mut impl RandomSource) {
    random.consume_count(262);
}

/// `PerlinNoise.makeAmplitudes`: the sorted octave set becomes
/// `(firstOctave, amplitudes)` with 1.0 on the set's octaves.
fn make_amplitudes(octave_set: &[i32]) -> Result<(i32, Vec<f64>), SynthError> {
    let mut sorted: Vec<i32> = octave_set.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    if sorted.is_empty() {
        return Err(SynthError::EmptyOctaves);
    }
    let low_freq_octaves = sorted[0].wrapping_neg();
    let high_freq_octaves = sorted[sorted.len() - 1];
    let octaves = low_freq_octaves
        .wrapping_add(high_freq_octaves)
        .wrapping_add(1);
    if octaves < 1 {
        return Err(SynthError::TooFewOctaves);
    }
    let mut amplitudes = vec![0.0; octaves as usize];
    for octave in sorted {
        amplitudes[(octave.wrapping_add(low_freq_octaves)) as usize] = 1.0;
    }
    Ok((low_freq_octaves.wrapping_neg(), amplitudes))
}

/// Vanilla 26.2 `SimplexNoise`: Gustavson simplex in doubles. Unlike the
/// float engine's `SimplexNoise`, `getValue` does NOT apply the offsets —
/// they are public fields for callers (`PerlinSimplexNoise` adds them when
/// `useNoiseStart` is set). The permutation table is `int[512]` in Java
/// with only the first 256 entries filled; `p(x)` masks to `0xFF`, so a
/// 256-entry table holds the same values.
#[derive(Clone, Debug)]
pub struct SimplexNoise {
    p: [u8; 256],
    pub xo: f64,
    pub yo: f64,
    pub zo: f64,
}

impl SimplexNoise {
    pub fn new(random: &mut impl RandomSource) -> Self {
        let xo = random.next_double() * 256.0;
        let yo = random.next_double() * 256.0;
        let zo = random.next_double() * 256.0;
        let mut p = [0u8; 256];
        for (i, slot) in p.iter_mut().enumerate() {
            *slot = i as u8;
        }
        for i in 0..256usize {
            let offset = random.next_int_bound(256 - i as i32).unwrap() as usize;
            p.swap(i, i + offset);
        }
        Self { p, xo, yo, zo }
    }

    pub fn permutations(&self) -> &[u8; 256] {
        &self.p
    }

    fn permute(&self, index: i32) -> i32 {
        i32::from(self.p[(index & 255) as usize])
    }

    fn corner(gradient: usize, point: [f64; 3], base: f64) -> f64 {
        let t0 = base - point[0] * point[0] - point[1] * point[1] - point[2] * point[2];
        if t0 < 0.0 {
            0.0
        } else {
            let squared = t0 * t0;
            squared * squared * dot_grad(GRADIENTS[gradient], point[0], point[1], point[2])
        }
    }

    pub fn get_value(&self, xin: f64, yin: f64) -> f64 {
        let sqrt_3 = 3.0_f64.sqrt();
        let f2 = 0.5 * (sqrt_3 - 1.0);
        let g2 = (3.0 - sqrt_3) / 6.0;
        let s = (xin + yin) * f2;
        let i = floor_i32(xin + s);
        let j = floor_i32(yin + s);
        let t = f64::from(i.wrapping_add(j)) * g2;
        let x0 = xin - (f64::from(i) - t);
        let y0 = yin - (f64::from(j) - t);
        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
        let x1 = x0 - f64::from(i1) + g2;
        let y1 = y0 - f64::from(j1) + g2;
        let x2 = x0 - 1.0 + 2.0 * g2;
        let y2 = y0 - 1.0 + 2.0 * g2;
        let ii = i & 255;
        let jj = j & 255;
        let gi0 = self.permute(ii + self.permute(jj)) % 12;
        let gi1 = self.permute(ii + i1 + self.permute(jj + j1)) % 12;
        let gi2 = self.permute(ii + 1 + self.permute(jj + 1)) % 12;
        let n0 = Self::corner(gi0 as usize, [x0, y0, 0.0], 0.5);
        let n1 = Self::corner(gi1 as usize, [x1, y1, 0.0], 0.5);
        let n2 = Self::corner(gi2 as usize, [x2, y2, 0.0], 0.5);
        70.0 * (n0 + n1 + n2)
    }

    pub fn get_value_3d(&self, xin: f64, yin: f64, zin: f64) -> f64 {
        let s = (xin + yin + zin) * 0.3333333333333333;
        let i = floor_i32(xin + s);
        let j = floor_i32(yin + s);
        let k = floor_i32(zin + s);
        let t = f64::from(i.wrapping_add(j).wrapping_add(k)) * 0.16666666666666666;
        let x0 = xin - (f64::from(i) - t);
        let y0 = yin - (f64::from(j) - t);
        let z0 = zin - (f64::from(k) - t);
        let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
            if y0 >= z0 {
                (1, 0, 0, 1, 1, 0)
            } else if x0 >= z0 {
                (1, 0, 0, 1, 0, 1)
            } else {
                (0, 0, 1, 1, 0, 1)
            }
        } else if y0 < z0 {
            (0, 0, 1, 0, 1, 1)
        } else if x0 < z0 {
            (0, 1, 0, 0, 1, 1)
        } else {
            (0, 1, 0, 1, 1, 0)
        };
        let x1 = x0 - f64::from(i1) + 0.16666666666666666;
        let y1 = y0 - f64::from(j1) + 0.16666666666666666;
        let z1 = z0 - f64::from(k1) + 0.16666666666666666;
        let x2 = x0 - f64::from(i2) + 0.3333333333333333;
        let y2 = y0 - f64::from(j2) + 0.3333333333333333;
        let z2 = z0 - f64::from(k2) + 0.3333333333333333;
        let x3 = x0 - 1.0 + 0.5;
        let y3 = y0 - 1.0 + 0.5;
        let z3 = z0 - 1.0 + 0.5;
        let ii = i & 255;
        let jj = j & 255;
        let kk = k & 255;
        let gi0 = self.permute(ii + self.permute(jj + self.permute(kk))) % 12;
        let gi1 = self.permute(ii + i1 + self.permute(jj + j1 + self.permute(kk + k1))) % 12;
        let gi2 = self.permute(ii + i2 + self.permute(jj + j2 + self.permute(kk + k2))) % 12;
        let gi3 = self.permute(ii + 1 + self.permute(jj + 1 + self.permute(kk + 1))) % 12;
        let n0 = Self::corner(gi0 as usize, [x0, y0, z0], 0.6);
        let n1 = Self::corner(gi1 as usize, [x1, y1, z1], 0.6);
        let n2 = Self::corner(gi2 as usize, [x2, y2, z2], 0.6);
        let n3 = Self::corner(gi3 as usize, [x3, y3, z3], 0.6);
        32.0 * (n0 + n1 + n2 + n3)
    }

    /// `NoiseUtils.parityNoiseOctaveConfigString` for the `int[]` table:
    /// `p0`/`p255` are non-negative here.
    pub fn parity_config_string(&self) -> String {
        format!(
            "xo={}, yo={}, zo={}, p0={}, p255={}",
            java_format_fixed(f64::from(self.xo as f32), 3),
            java_format_fixed(f64::from(self.yo as f32), 3),
            java_format_fixed(f64::from(self.zo as f32), 3),
            self.p[0],
            self.p[255],
        )
    }
}

/// Vanilla 26.2 `PerlinSimplexNoise`: a stack of [`SimplexNoise`] octaves
/// for 2D sampling. The zero octave is drawn first; lower octaves follow
/// from the caller's stream; positive octaves come from a fresh
/// `LegacyRandom` seeded by the zero octave's own value at its offsets.
/// (`WorldgenRandom` wraps that source in vanilla, but it delegates
/// `next(bits)` straight through, so the stream is the plain legacy one.)
#[derive(Clone, Debug)]
pub struct PerlinSimplexNoise {
    noise_levels: Vec<Option<SimplexNoise>>,
    highest_freq_value_factor: f64,
    highest_freq_input_factor: f64,
}

impl PerlinSimplexNoise {
    pub fn new(random: &mut impl RandomSource, octave_set: &[i32]) -> Result<Self, SynthError> {
        let mut sorted: Vec<i32> = octave_set.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        if sorted.is_empty() {
            return Err(SynthError::EmptyOctaves);
        }
        let low_freq_octaves = sorted[0].wrapping_neg();
        let high_freq_octaves = sorted[sorted.len() - 1];
        let octaves = low_freq_octaves
            .wrapping_add(high_freq_octaves)
            .wrapping_add(1);
        if octaves < 1 {
            return Err(SynthError::TooFewOctaves);
        }
        let zero_octave = SimplexNoise::new(random);
        let zero_octave_index = high_freq_octaves;
        let mut noise_levels: Vec<Option<SimplexNoise>> =
            (0..octaves as usize).map(|_| None).collect();
        if zero_octave_index >= 0 && zero_octave_index < octaves && sorted.contains(&0) {
            noise_levels[zero_octave_index as usize] = Some(zero_octave.clone());
        }
        // `for (int i = zeroOctaveIndex + 1; i < octaves; i++)`, wrapping.
        let mut i = zero_octave_index.wrapping_add(1);
        while i < octaves {
            if i >= 0 && sorted.contains(&zero_octave_index.wrapping_sub(i)) {
                noise_levels[i as usize] = Some(SimplexNoise::new(random));
            } else {
                random.consume_count(262);
            }
            i = i.wrapping_add(1);
        }
        if high_freq_octaves > 0 {
            // `9.223372E18F` is a *float* literal, widened to double.
            let positive_octave_seed =
                (zero_octave.get_value_3d(zero_octave.xo, zero_octave.yo, zero_octave.zo)
                    * f64::from(9.223372e18_f32)) as i64;
            let mut high_freq_random = LegacyRandom::new(positive_octave_seed);
            let mut i = zero_octave_index.wrapping_sub(1);
            while i >= 0 {
                if i < octaves && sorted.contains(&zero_octave_index.wrapping_sub(i)) {
                    noise_levels[i as usize] = Some(SimplexNoise::new(&mut high_freq_random));
                } else {
                    high_freq_random.consume_count(262);
                }
                i = i.wrapping_sub(1);
            }
        }
        Ok(Self {
            noise_levels,
            highest_freq_input_factor: power_of_two(high_freq_octaves),
            highest_freq_value_factor: 1.0 / (power_of_two(octaves) - 1.0),
        })
    }

    /// The octave levels, for the parity probe (vanilla exposes them only
    /// through reflection).
    pub fn levels(&self) -> &[Option<SimplexNoise>] {
        &self.noise_levels
    }

    pub fn highest_freq_input_factor(&self) -> f64 {
        self.highest_freq_input_factor
    }

    pub fn highest_freq_value_factor(&self) -> f64 {
        self.highest_freq_value_factor
    }

    pub fn get_value(&self, x: f64, y: f64, use_noise_start: bool) -> f64 {
        let mut value = 0.0;
        let mut factor = self.highest_freq_input_factor;
        let mut value_factor = self.highest_freq_value_factor;
        for level in &self.noise_levels {
            if let Some(noise) = level {
                value += noise.get_value(
                    x * factor + if use_noise_start { noise.xo } else { 0.0 },
                    y * factor + if use_noise_start { noise.yo } else { 0.0 },
                ) * value_factor;
            }
            factor /= 2.0;
            value_factor *= 2.0;
        }
        value
    }
}

/// Vanilla 26.2 `NormalNoise.NoiseParameters`: `{firstOctave, amplitudes}`,
/// the `worldgen/noise` registry JSON shape.
#[derive(Clone, Debug, PartialEq)]
pub struct NoiseParameters {
    pub first_octave: i32,
    pub amplitudes: Vec<f64>,
}

/// Vanilla 26.2 `NormalNoise`: two [`PerlinNoise`] stacks, the second
/// sampled at `INPUT_FACTOR`-scaled coordinates, normalized by
/// `valueFactor = (1/6) / expectedDeviation(octaveSpan)`.
#[derive(Clone, Debug)]
pub struct NormalNoise {
    value_factor: f64,
    first: PerlinNoise,
    second: PerlinNoise,
    max_value: f64,
    parameters: NoiseParameters,
}

impl NormalNoise {
    /// `NormalNoise.create(random, firstOctave, amplitudes...)`.
    pub fn create(
        random: &mut impl RandomSource,
        first_octave: i32,
        amplitudes: &[f64],
    ) -> Result<Self, SynthError> {
        Self::create_with_params(
            random,
            NoiseParameters {
                first_octave,
                amplitudes: amplitudes.to_vec(),
            },
            true,
        )
    }

    /// `NormalNoise.create(random, parameters)`.
    pub fn create_with_params(
        random: &mut impl RandomSource,
        parameters: NoiseParameters,
        use_new_initialization: bool,
    ) -> Result<Self, SynthError> {
        Self::new(random, parameters, use_new_initialization)
    }

    /// `NormalNoise.createLegacyNetherBiome(random, parameters)`.
    pub fn create_legacy_nether_biome(
        random: &mut impl RandomSource,
        parameters: NoiseParameters,
    ) -> Result<Self, SynthError> {
        Self::new(random, parameters, false)
    }

    fn new(
        random: &mut impl RandomSource,
        parameters: NoiseParameters,
        use_new_initialization: bool,
    ) -> Result<Self, SynthError> {
        let first_octave = parameters.first_octave;
        let amplitudes = parameters.amplitudes.clone();
        let (first, second) = if use_new_initialization {
            (
                PerlinNoise::create(random, first_octave, &amplitudes)?,
                PerlinNoise::create(random, first_octave, &amplitudes)?,
            )
        } else {
            (
                PerlinNoise::create_legacy_for_legacy_nether_biome(
                    random,
                    first_octave,
                    &amplitudes,
                )?,
                PerlinNoise::create_legacy_for_legacy_nether_biome(
                    random,
                    first_octave,
                    &amplitudes,
                )?,
            )
        };
        let mut min_octave = i32::MAX;
        let mut max_octave = i32::MIN;
        for (i, amplitude) in amplitudes.iter().enumerate() {
            if *amplitude != 0.0 {
                min_octave = min_octave.min(i as i32);
                max_octave = max_octave.max(i as i32);
            }
        }
        // `maxOctave - minOctave` wraps when no amplitude is nonzero
        // (MIN - MAX == 1), exactly as in Java.
        let value_factor =
            0.16666666666666666 / expected_deviation(max_octave.wrapping_sub(min_octave));
        let max_value = (first.max_value() + second.max_value()) * value_factor;
        Ok(Self {
            value_factor,
            first,
            second,
            max_value,
            parameters,
        })
    }

    pub fn value_factor(&self) -> f64 {
        self.value_factor
    }

    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    pub fn parameters(&self) -> &NoiseParameters {
        &self.parameters
    }

    pub fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        let x2 = x * 1.0181268882175227;
        let y2 = y * 1.0181268882175227;
        let z2 = z * 1.0181268882175227;
        (self.first.get_value(x, y, z) + self.second.get_value(x2, y2, z2)) * self.value_factor
    }

    pub fn parity_config_string(&self) -> String {
        format!(
            "NormalNoise {{first: {}, second: {}}}",
            self.first.parity_config_string(),
            self.second.parity_config_string()
        )
    }
}

fn expected_deviation(octave_span: i32) -> f64 {
    0.1 * (1.0 + 1.0 / f64::from(octave_span.wrapping_add(1)))
}

/// Vanilla 26.2 `BlendedNoise`: the old blended terrain density function.
/// Three legacy-initialized [`PerlinNoise`] stacks (min/max over octaves
/// -15..0, main over -7..0); `compute` lerps the limit noises by the main
/// noise, all in doubles.
#[derive(Clone, Debug)]
pub struct BlendedNoise {
    min_limit_noise: PerlinNoise,
    max_limit_noise: PerlinNoise,
    main_noise: PerlinNoise,
    xz_multiplier: f64,
    y_multiplier: f64,
    xz_factor: f64,
    y_factor: f64,
    smear_scale_multiplier: f64,
    max_value: f64,
    xz_scale: f64,
    y_scale: f64,
}

impl BlendedNoise {
    /// `BlendedNoise.createUnseeded(...)`: seeded with
    /// `XoroshiroRandomSource(0)`, as vanilla does.
    pub fn create_unseeded(
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale_multiplier: f64,
    ) -> Result<Self, SynthError> {
        let mut random = XoroshiroRandom::new(0);
        Self::new(
            &mut random,
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale_multiplier,
        )
    }

    /// The seeded constructor (`@VisibleForTesting` in vanilla).
    pub fn new(
        random: &mut impl RandomSource,
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale_multiplier: f64,
    ) -> Result<Self, SynthError> {
        let min_limit =
            PerlinNoise::create_legacy_for_blended_noise(random, &(-15..=0).collect::<Vec<_>>())?;
        let max_limit =
            PerlinNoise::create_legacy_for_blended_noise(random, &(-15..=0).collect::<Vec<_>>())?;
        let main =
            PerlinNoise::create_legacy_for_blended_noise(random, &(-7..=0).collect::<Vec<_>>())?;
        Self::from_stacks(
            min_limit,
            max_limit,
            main,
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale_multiplier,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_stacks(
        min_limit_noise: PerlinNoise,
        max_limit_noise: PerlinNoise,
        main_noise: PerlinNoise,
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale_multiplier: f64,
    ) -> Result<Self, SynthError> {
        let xz_multiplier = 684.412 * xz_scale;
        let y_multiplier = 684.412 * y_scale;
        let max_value = min_limit_noise.max_broken_value(y_multiplier);
        Ok(Self {
            min_limit_noise,
            max_limit_noise,
            main_noise,
            xz_multiplier,
            y_multiplier,
            xz_factor,
            y_factor,
            smear_scale_multiplier,
            max_value,
            xz_scale,
            y_scale,
        })
    }

    /// `withNewRandom`: rebuild with a fresh stream, same scales.
    pub fn with_new_random(&self, random: &mut impl RandomSource) -> Result<Self, SynthError> {
        Self::new(
            random,
            self.xz_scale,
            self.y_scale,
            self.xz_factor,
            self.y_factor,
            self.smear_scale_multiplier,
        )
    }

    /// `compute(FunctionContext)`, taking the block coordinates directly.
    pub fn compute(&self, block_x: i32, block_y: i32, block_z: i32) -> f64 {
        let limit_x = f64::from(block_x) * self.xz_multiplier;
        let limit_y = f64::from(block_y) * self.y_multiplier;
        let limit_z = f64::from(block_z) * self.xz_multiplier;
        let main_x = limit_x / self.xz_factor;
        let main_y = limit_y / self.y_factor;
        let main_z = limit_z / self.xz_factor;
        let limit_smear = self.y_multiplier * self.smear_scale_multiplier;
        let main_smear = limit_smear / self.y_factor;
        let mut blend_min = 0.0;
        let mut blend_max = 0.0;
        let mut main_noise_value = 0.0;
        let mut pow = 1.0;
        for i in 0..8 {
            if let Some(noise) = self.main_noise.get_octave_noise(i) {
                main_noise_value += noise.noise_smeared(
                    wrap(main_x * pow),
                    wrap(main_y * pow),
                    wrap(main_z * pow),
                    main_smear * pow,
                    main_y * pow,
                ) / pow;
            }
            pow /= 2.0;
        }
        let factor = (main_noise_value / 10.0 + 1.0) / 2.0;
        let is_max = factor >= 1.0;
        let is_min = factor <= 0.0;
        pow = 1.0;
        for i in 0..16 {
            let wx = wrap(limit_x * pow);
            let wy = wrap(limit_y * pow);
            let wz = wrap(limit_z * pow);
            let y_scale_pow = limit_smear * pow;
            if !is_max && let Some(noise) = self.min_limit_noise.get_octave_noise(i) {
                blend_min += noise.noise_smeared(wx, wy, wz, y_scale_pow, limit_y * pow) / pow;
            }
            if !is_min && let Some(noise) = self.max_limit_noise.get_octave_noise(i) {
                blend_max += noise.noise_smeared(wx, wy, wz, y_scale_pow, limit_y * pow) / pow;
            }
            pow /= 2.0;
        }
        clamped_lerp(factor, blend_min / 512.0, blend_max / 512.0) / 128.0
    }

    pub fn min_value(&self) -> f64 {
        -self.max_value()
    }

    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    /// `BlendedNoise.parityConfigString`, including vanilla's hardcoded
    /// scale display (684.412, 8.55515, 4.277575) — a diagnostic quirk,
    /// kept faithful.
    pub fn parity_config_string(&self) -> String {
        format!(
            "BlendedNoise{{minLimitNoise={}, maxLimitNoise={}, mainNoise={}, xzScale={}, yScale={}, xzMainScale={}, yMainScale={}, cellWidth=4, cellHeight=8}}",
            self.min_limit_noise.parity_config_string(),
            self.max_limit_noise.parity_config_string(),
            self.main_noise.parity_config_string(),
            java_format_fixed(684.412, 3),
            java_format_fixed(684.412, 3),
            java_format_fixed(8.555150000000001, 3),
            java_format_fixed(4.277575000000001, 3),
        )
    }
}

/// Vanilla 26.2 `NoiseUtils`.
pub struct NoiseUtils;

impl NoiseUtils {
    /// `biasTowardsExtreme`: `noise + sin(PI * noise) * factor / PI`,
    /// with HotSpot's `Math.sin` via [`jmath`].
    pub fn bias_towards_extreme(noise: f64, factor: f64) -> f64 {
        noise + jmath::sin(std::f64::consts::PI * noise) * factor / std::f64::consts::PI
    }
}

#[cfg(test)]
mod tests {
    use super::super::PerlinNoise as FloatPerlin;
    use super::*;
    use crate::random::LegacyRandom;

    #[test]
    fn improved_noise_shares_the_gradient_tables_seed_draws() {
        // ImprovedNoise and the float engine's PerlinNoise consume the
        // stream identically: three doubles scaled by 256, then a
        // 256-swap Fisher-Yates shuffle.
        let improved = ImprovedNoise::new(&mut LegacyRandom::new(0));
        let float = FloatPerlin::new(&mut LegacyRandom::new(0));
        assert_eq!(
            [improved.xo, improved.yo, improved.zo],
            *float.table().offsets()
        );
        assert_eq!(improved.permutations(), float.table().permutations());
        let mut random = LegacyRandom::new(0);
        let _ = ImprovedNoise::new(&mut random);
        assert_eq!(random.next_long(), -6_819_959_017_335_327_829);
    }

    #[test]
    fn wrap_matches_perlin_noise_wrap_including_huge_magnitudes() {
        assert_eq!(wrap(0.0), 0.0);
        assert_eq!(wrap(33_554_432.0), 0.0);
        assert_eq!(wrap(-33_554_432.0), 0.0);
        // 2^24 / 2^25 + 0.5 == 1.0, whose floor is 1: wraps to -2^24.
        assert_eq!(wrap(16_777_216.0), -16_777_216.0);
        // -2^24 / 2^25 + 0.5 == 0.0, whose floor is 0: unchanged.
        assert_eq!(wrap(-16_777_216.0), -16_777_216.0);
        assert_eq!(wrap(33_554_433.0), 1.0);
        // Past i64 range the long floor saturates, so the value is
        // unchanged up to double rounding — unlike the float engine's
        // f64-floor wrap, which would reduce it near zero.
        let huge = wrap(1e100);
        assert!(huge.is_finite());
        assert_eq!(huge, 1e100 - i64::MAX as f64 * 33_554_432.0);
        assert!(wrap(f64::NAN).is_nan());
    }

    #[test]
    fn single_octave_perlin_simplex_is_its_simplex_level() {
        let simplex = PerlinSimplexNoise::new(&mut LegacyRandom::new(2345), &[0]).unwrap();
        let plain = SimplexNoise::new(&mut LegacyRandom::new(2345));
        for (x, y) in [(0.0, 0.0), (0.25, 0.75), (-17.5, 64.125), (1e6, -1e6)] {
            assert_eq!(
                simplex.get_value(x, y, false).to_bits(),
                plain.get_value(x, y).to_bits(),
                "({x}, {y})"
            );
        }
        // With the noise start, the level's offsets are added.
        assert_eq!(
            simplex.get_value(0.0, 0.0, true).to_bits(),
            plain.get_value(plain.xo, plain.yo).to_bits()
        );
    }

    #[test]
    fn normal_noise_value_factor_matches_the_old_formula() {
        // Amplitudes [1, 1, 1, 0]: span 2, expected deviation
        // 0.1 * (1 + 1/3), value factor (1/6) / that = 1.25.
        let noise =
            NormalNoise::create(&mut LegacyRandom::new(0), -3, &[1.0, 1.0, 1.0, 0.0]).unwrap();
        assert!((noise.value_factor() - 1.25).abs() < 1e-12);
        assert_eq!(noise.parameters().first_octave, -3);
        assert!(noise.max_value() > 0.0);
        assert!(noise.get_value(0.0, 0.0, 0.0).is_finite());
    }

    #[test]
    fn blended_noise_builds_and_computes_finite_values() {
        let noise = BlendedNoise::create_unseeded(1.0, 1.0, 80.0, 160.0, 1.0).unwrap();
        for (x, y, z) in [(0, 0, 0), (-17, 64, 32), (100, -60, -100)] {
            assert!(noise.compute(x, y, z).is_finite(), "({x}, {y}, {z})");
        }
        assert!(noise.max_value() > 0.0);
        assert_eq!(noise.min_value(), -noise.max_value());
    }

    #[test]
    fn legacy_perlin_rejects_positive_octaves_after_drawing() {
        // firstOctave 1 with a nonzero amplitude: the zero octave is
        // created and discarded, then the count check fails (0 levels
        // built vs 1 nonzero amplitude) before the positive-octave
        // check — matching vanilla's throw order.
        let mut random = LegacyRandom::new(0);
        let err =
            PerlinNoise::create_legacy_for_legacy_nether_biome(&mut random, 1, &[1.0]).unwrap_err();
        assert_eq!(err, SynthError::MissingNoiseLevels);
        // All-zero amplitudes pass the count check and hit the
        // positive-octave error instead.
        let err = PerlinNoise::create_legacy_for_legacy_nether_biome(
            &mut LegacyRandom::new(0),
            1,
            &[0.0],
        )
        .unwrap_err();
        assert_eq!(err, SynthError::PositiveLegacyOctaves);
        assert_eq!(
            PerlinNoise::create_from_octaves(&mut LegacyRandom::new(0), &[]).unwrap_err(),
            SynthError::EmptyOctaves
        );
    }
}
