//! `MipmapGenerator`: the mip levels of a sprite. Each 2x2 block averages in
//! gamma-2.2 light (`alphaBlend` / `gammaBlend`); when the base has a fully
//! transparent pixel, transparent pixels are skipped (their colour does not
//! bleed into the mip) and a computed alpha below the cutout cutoff (96)
//! becomes fully transparent.

/// How a sprite's mips are made: whether its base has a fully transparent
/// pixel (which picks `alphaBlend`'s skip-transparent path) and its
/// original square frame edge.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Settings {
    /// Original square frame edge, before fitting the sprite into the array.
    /// Zero means the supplied image is already at its native size.
    pub native_size: u32,
    pub has_transparent: bool,
    /// Whether a high-resolution cutout's small levels are dithered ([`dither_cutout_levels`]).
    pub dither_cutouts: bool,
}

/// The `ditherCutoutMips` setting, for textures loaded from now on.
static DITHER_CUTOUTS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Sets `ditherCutoutMips`; true if that changes it, when the textures must load again.
pub fn set_dither_cutouts(on: bool) -> bool {
    DITHER_CUTOUTS.swap(on, std::sync::atomic::Ordering::Relaxed) != on
}

/// The `ditherCutoutMips` setting.
pub fn dither_cutouts() -> bool {
    DITHER_CUTOUTS.load(std::sync::atomic::Ordering::Relaxed)
}

/// A square or rectangular RGBA8 image, rows top to bottom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[u8; 4]>,
}
impl Image {
    fn resized(&self, size: u32) -> Self {
        Self {
            width: size,
            height: size,
            pixels: (0..size * size)
                .map(|i| self.get(i % size * self.width / size, i / size * self.height / size))
                .collect(),
        }
    }
    fn get(&self, x: u32, y: u32) -> [u8; 4] {
        self.pixels[(x + y * self.width) as usize]
    }
    fn set(&mut self, x: u32, y: u32, p: [u8; 4]) {
        self.pixels[(x + y * self.width) as usize] = p;
    }
}

/// `MipmapGenerator.POW22`: `(i / 255)^2.2` for every channel value, as
/// vanilla fills it (`(float) Math.pow((float) i / 255.0f, 2.2)`).
fn pow22() -> &'static [f32; 256] {
    static TABLE: std::sync::OnceLock<[f32; 256]> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        std::array::from_fn(|i| f64::from(i as f32 / 255.0).powf(2.2) as f32)
    })
}

/// `MipmapGenerator.ALPHA_CUTOUT_CUTOFF`: a computed alpha below this
/// becomes fully transparent (only on the skip-transparent path).
const ALPHA_CUTOUT_CUTOFF: u8 = 96;

/// `MipmapGenerator.alphaBlend`: the 2x2 block's mean in gamma-2.2 light.
/// When the base has a fully transparent pixel, pixels with alpha 0 are
/// skipped (their colour does not bleed into the mip) and every channel
/// sum is still divided by four; a computed alpha below the cutout cutoff
/// becomes fully transparent. Otherwise every pixel contributes, one
/// channel at a time (`gammaBlend`).
fn alpha_blend(c: [[u8; 4]; 4], has_transparent: bool) -> [u8; 4] {
    if has_transparent {
        let table = pow22();
        let mut total = [0.0f32; 4];
        for p in c.iter().filter(|p| p[3] != 0) {
            for k in 0..4 {
                total[k] += table[p[k] as usize];
            }
        }
        let mut out = [0u8; 4];
        for k in 0..4 {
            let mean = total[k] / 4.0;
            out[k] = (f64::from(mean).powf(0.454_545_454_545_454_53) * 255.0) as i32 as u8;
        }
        if out[3] < ALPHA_CUTOUT_CUTOFF {
            out[3] = 0;
        }
        out
    } else {
        [gamma_blend(c, 0), gamma_blend(c, 1), gamma_blend(c, 2), gamma_blend(c, 3)]
    }
}

/// `MipmapGenerator.gammaBlend`: one channel of a 2x2 block, all four
/// pixels contributing, in gamma-2.2 light. The mean is taken in double
/// (`sum * 0.25`), the power rounded back through float, then scaled to
/// 255 and truncated.
fn gamma_blend(c: [[u8; 4]; 4], k: usize) -> u8 {
    let table = pow22();
    let sum = table[c[0][k] as usize]
        + table[c[1][k] as usize]
        + table[c[2][k] as usize]
        + table[c[3][k] as usize];
    let linear = (f64::from(sum) * 0.25).powf(0.454_545_454_545_454_53) as f32;
    (f64::from(linear) * 255.0) as i32 as u8
}

/// `MipmapGenerator.generateMipLevels` from the base image (`levels` mips
/// below it). Array-only enlargement resizes the base to its native size
/// first, so the mip chain is made from the sprite as authored.
pub fn generate(base: Image, levels: u32, settings: Settings) -> Vec<Image> {
    let native = settings.native_size;
    if native.is_power_of_two()
        && native < base.width
        && base.width == base.height
        && base.width.is_power_of_two()
    {
        let extra = (base.width / native).ilog2();
        let mut mips =
            generate_native(base.resized(native), levels.saturating_sub(extra), settings);
        let mut out: Vec<_> = (0..extra.min(levels + 1))
            .map(|level| mips[0].resized(base.width >> level))
            .collect();
        if levels >= extra {
            out.append(&mut mips);
        }
        return out;
    }
    generate_native(base, levels, settings)
}

fn generate_native(base: Image, levels: u32, settings: Settings) -> Vec<Image> {
    let has_transparent = settings.has_transparent;
    let mut out = vec![base];
    for _ in 1..=levels {
        let last = out.last().expect("the base level");
        let (width, height) = (last.width >> 1, last.height >> 1);
        let mut data = Image {
            width,
            height,
            pixels: vec![[0; 4]; (width * height) as usize],
        };
        for x in 0..width {
            for y in 0..height {
                let c = [
                    last.get(x * 2, y * 2),
                    last.get(x * 2 + 1, y * 2),
                    last.get(x * 2, y * 2 + 1),
                    last.get(x * 2 + 1, y * 2 + 1),
                ];
                data.set(x, y, alpha_blend(c, has_transparent));
            }
        }
        out.push(data);
    }
    if settings.dither_cutouts && has_transparent && out[0].width >= DITHERED_CUTOUT_SIZE {
        dither_cutout_levels(&mut out);
    }
    out
}

/// From this size a cutout texture's mips can be dithered ([`dither_cutout_levels`]).
const DITHERED_CUTOUT_SIZE: u32 = 64;
/// A high-resolution cutout's detail is finer than its small mips: grass blades a few
/// texels wide at 2048 average to about the same alpha everywhere at 64, and no scale of
/// that alpha can keep the coverage, so the alpha test passes the whole quad or none of it.
/// Each level below the base takes the base's mean alpha through an ordered dither instead:
/// its texels are in or out, about as many in as the base has.
fn dither_cutout_levels(levels: &mut [Image]) {
    const BAYER: [[f32; 4]; 4] = [
        [0.0, 8.0, 2.0, 10.0],
        [12.0, 4.0, 14.0, 6.0],
        [3.0, 11.0, 1.0, 9.0],
        [15.0, 7.0, 13.0, 5.0],
    ];
    let (mut w, mut h) = (levels[0].width as usize, levels[0].height as usize);
    let mut alpha: Vec<f32> = levels[0]
        .pixels
        .iter()
        .map(|p| f32::from(p[3]) / 255.0)
        .collect();
    for level in &mut levels[1..] {
        let (nw, nh) = (level.width as usize, level.height as usize);
        let mut next = vec![0.0; nw * nh];
        for y in 0..nh {
            for x in 0..nw {
                let at = |dx: usize, dy: usize| {
                    alpha[(y * 2 + dy).min(h - 1) * w + (x * 2 + dx).min(w - 1)]
                };
                let mean = (at(0, 0) + at(1, 0) + at(0, 1) + at(1, 1)) / 4.0;
                next[y * nw + x] = mean;
                let threshold = (BAYER[y % 4][x % 4] + 0.5) / 16.0;
                level.pixels[y * nw + x][3] = if mean > threshold { 255 } else { 0 };
            }
        }
        alpha = next;
        (w, h) = (nw, nh);
    }
}

#[cfg(test)]
mod alpha_blend_tests {
    use super::*;

    fn mip_of(pixels: [[u8; 4]; 4], has_transparent: bool) -> [u8; 4] {
        let base = Image {
            width: 2,
            height: 2,
            pixels: pixels.to_vec(),
        };
        let settings = Settings {
            has_transparent,
            ..Settings::default()
        };
        generate(base, 1, settings)[1].pixels[0]
    }

    #[test]
    fn a_uniform_opaque_block_round_trips() {
        assert_eq!(
            mip_of([[128, 0, 0, 255]; 4], false),
            [128, 0, 0, 255]
        );
        assert_eq!(
            mip_of([[255, 255, 255, 255]; 4], false),
            [255, 255, 255, 255]
        );
    }

    #[test]
    fn a_transparent_pixels_colour_does_not_bleed() {
        // A fully transparent red pixel is skipped outright: nothing
        // contributes, so the mip is fully transparent black.
        assert_eq!(
            mip_of(
                [[255, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
                true
            ),
            [0, 0, 0, 0]
        );
    }

    #[test]
    fn one_visible_pixel_among_transparent_ones() {
        // Only the red pixel contributes, its pow22 sums divided by four:
        // (1/4)^(1/2.2) * 255 = 135, for red and alpha alike.
        assert_eq!(
            mip_of(
                [[255, 0, 0, 255], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
                true
            ),
            [135, 0, 0, 135]
        );
    }

    #[test]
    fn a_faint_alpha_is_cut_to_zero() {
        // The half-alpha pixel's alpha averages to 68, below the cutout
        // cutoff of 96, so the mip is fully transparent — while its red
        // still averages to 135.
        assert_eq!(
            mip_of(
                [[255, 0, 0, 128], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
                true
            ),
            [135, 0, 0, 0]
        );
    }

    #[test]
    fn two_visible_pixels_average_in_gamma_light() {
        // White and black: each colour channel (1 + 0) / 4 -> 135; alpha
        // (1 + 1) / 4 -> (1/2)^(1/2.2) * 255 = 186.
        assert_eq!(
            mip_of(
                [
                    [255, 255, 255, 255],
                    [0, 0, 0, 255],
                    [0, 0, 0, 0],
                    [0, 0, 0, 0]
                ],
                true
            ),
            [135, 135, 135, 186]
        );
    }
}

#[cfg(test)]
mod dither_tests {
    use super::*;
    #[test]
    fn the_dither_only_runs_when_asked_for_and_from_its_size() {
        let cutout = |side: u32| Image {
            width: side,
            height: side,
            pixels: (0..side * side)
                .map(|i| {
                    [
                        90,
                        140,
                        40,
                        if (i % side).is_multiple_of(4) { 255 } else { 0 },
                    ]
                })
                .collect(),
        };
        let settings = |dither_cutouts| Settings {
            has_transparent: true,
            dither_cutouts,
            ..Settings::default()
        };
        assert_ne!(
            generate(cutout(128), 4, settings(false)),
            generate(cutout(128), 4, settings(true))
        );
        assert_eq!(
            generate(cutout(32), 4, settings(false)),
            generate(cutout(32), 4, settings(true))
        );
    }
    #[test]
    fn a_high_resolution_cutouts_fine_detail_keeps_its_coverage_in_small_mips() {
        // Blades one texel wide, one column in four: too fine for the mips to hold.
        let side = 128u32;
        let pixels = (0..side * side)
            .map(|i| {
                [
                    90,
                    140,
                    40,
                    if (i % side).is_multiple_of(4) { 255 } else { 0 },
                ]
            })
            .collect();
        let base = Image {
            width: side,
            height: side,
            pixels,
        };
        let settings = Settings {
            has_transparent: true,
            dither_cutouts: true,
            ..Settings::default()
        };
        let mips = generate(base, 4, settings);
        for level in &mips[3..] {
            let passing = level.pixels.iter().filter(|p| p[3] >= 128).count() as f32;
            let coverage = passing / level.pixels.len() as f32;
            assert!((0.15..0.35).contains(&coverage), "{coverage}");
        }
    }
}
