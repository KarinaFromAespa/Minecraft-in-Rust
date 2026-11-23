//! Timelines (`net.minecraft.world.timeline`): tracks of keyframes that change environment
//! attributes over the level's day time (`Timeline`, `AttributeTrack`, `AttributeTrackSampler`), and
//! the keyframe machinery under them (`KeyframeTrack`, `KeyframeTrackSampler`, `EasingType`,
//! `Ease`).
use crate::environment_attributes::{Attribute, Entry, Value, attribute};
use horizon_core::{geometry::trig, jmath};
use serde_json::Value as Json;

/// `Mth.square` / `Mth.cube`.
fn square(x: f32) -> f32 {
    x * x
}
fn cube(x: f32) -> f32 {
    x * x * x
}

/// `Ease.outBounce` (its `d1 = 2.75` is declared but unused).
fn out_bounce(x: f32) -> f32 {
    if x < 0.363_636_37 {
        7.5625 * square(x)
    } else if x < 0.727_272_75 {
        7.5625 * square(x - 0.545_454_56) + 0.75
    } else if f64::from(x) < 0.909_090_909_090_909_1 {
        7.5625 * square(x - 0.818_181_8) + 0.9375
    } else {
        7.5625 * square(x - 0.954_545_44) + 0.984_375
    }
}

const TWO_THIRDS_PI: f64 = (std::f64::consts::PI * 2.0 / 3.0) as f32 as f64;
const HALF_PI: f32 = std::f64::consts::FRAC_PI_2 as f32;
const PI: f32 = std::f64::consts::PI as f32;

/// `EasingType`'s named easings and `Ease`'s functions, with Java's float and double steps.
/// `Ease.inCirc`: `(float)(-Math.sqrt(1.0F - x * x)) + 1.0F`. Outside [-1, 1] the root is HotSpot's NaN (x86's
/// negative default NaN), which `-` makes positive and `+ 1.0F` keeps. LLVM may fold `-r + 1.0` into `1.0 - r`,
/// whose NaN keeps the root's sign, so the NaN is returned before the addition.
fn in_circ(x: f32) -> f32 {
    let radicand = f64::from(1.0 - x * x);
    if radicand < 0.0 {
        const IN_CIRC_NAN: u32 = if cfg!(target_arch = "aarch64") {
            0xffc0_0000
        } else {
            0x7fc0_0000
        };
        return f32::from_bits(IN_CIRC_NAN);
    }
    ((-radicand.sqrt()) as f32) + 1.0
}

fn out_circ(x: f32) -> f32 {
    let d = x - 1.0;
    let radicand = f64::from(1.0 - d * d);
    if radicand < 0.0 {
        const OUT_CIRC_NAN: u32 = if cfg!(target_arch = "aarch64") {
            0x7fc0_0000
        } else {
            0xffc0_0000
        };
        return f32::from_bits(OUT_CIRC_NAN);
    }
    radicand.sqrt() as f32
}

const EASINGS: [(&str, fn(f32) -> f32); 32] = [
    ("constant", |_| 0.0),
    ("linear", |x| x),
    ("in_back", |x| square(x) * (2.701_58 * x - 1.701_58)),
    ("in_bounce", |x| 1.0 - out_bounce(1.0 - x)),
    ("in_circ", in_circ),
    ("in_cubic", cube),
    ("in_elastic", |x| {
        if x == 0.0 {
            return 0.0;
        }
        if x == 1.0 {
            return 1.0;
        }
        let x = f64::from(x);
        (-jmath::pow(2.0, 10.0 * x - 10.0) * jmath::sin((x * 10.0 - 10.75) * TWO_THIRDS_PI)) as f32
    }),
    ("in_expo", |x| {
        if x == 0.0 {
            0.0
        } else {
            jmath::pow(2.0, 10.0 * f64::from(x) - 10.0) as f32
        }
    }),
    ("in_quad", |x| x * x),
    ("in_quart", |x| square(square(x))),
    ("in_quint", |x| square(square(x)) * x),
    ("in_sine", |x| 1.0 - trig::cos(f64::from(x * HALF_PI))),
    ("in_out_back", |x| {
        if x < 0.5 {
            4.0 * x * x * (7.189_819 * x - 2.594_909_4) / 2.0
        } else {
            let dt = 2.0 * x - 2.0;
            (dt * dt * (3.594_909_4 * dt + 2.594_909_4) + 2.0) / 2.0
        }
    }),
    ("in_out_bounce", |x| {
        if x < 0.5 {
            (1.0 - out_bounce(1.0 - 2.0 * x)) / 2.0
        } else {
            (1.0 + out_bounce(2.0 * x - 1.0)) / 2.0
        }
    }),
    ("in_out_circ", |x| {
        let x = f64::from(x);
        if x < 0.5 {
            ((1.0 - (1.0 - jmath::pow(2.0 * x, 2.0)).sqrt()) / 2.0) as f32
        } else {
            (((1.0 - jmath::pow(-2.0 * x + 2.0, 2.0)).sqrt() + 1.0) / 2.0) as f32
        }
    }),
    ("in_out_cubic", |x| {
        if x < 0.5 {
            4.0 * cube(x)
        } else {
            (1.0 - jmath::pow(-2.0 * f64::from(x) + 2.0, 3.0) / 2.0) as f32
        }
    }),
    ("in_out_elastic", |x| {
        if x == 0.0 {
            return 0.0;
        }
        if x == 1.0 {
            return 1.0;
        }
        let xd = f64::from(x);
        // The bytecode multiplies by the float `c5 = (float) PI * 4 / 9`, widened.
        let sin = jmath::sin((20.0 * xd - 11.125) * f64::from(PI * 4.0 / 9.0));
        if x < 0.5 {
            (-(jmath::pow(2.0, 20.0 * xd - 10.0) * sin) / 2.0) as f32
        } else {
            (jmath::pow(2.0, -20.0 * xd + 10.0) * sin / 2.0 + 1.0) as f32
        }
    }),
    ("in_out_expo", |x| {
        let xd = f64::from(x);
        if x < 0.5 {
            if x == 0.0 {
                0.0
            } else {
                (jmath::pow(2.0, 20.0 * xd - 10.0) / 2.0) as f32
            }
        } else if x == 1.0 {
            1.0
        } else {
            ((2.0 - jmath::pow(2.0, -20.0 * xd + 10.0)) / 2.0) as f32
        }
    }),
    ("in_out_quad", |x| {
        if x < 0.5 {
            2.0 * square(x)
        } else {
            (1.0 - jmath::pow(-2.0 * f64::from(x) + 2.0, 2.0) / 2.0) as f32
        }
    }),
    ("in_out_quart", |x| {
        if x < 0.5 {
            8.0 * square(square(x))
        } else {
            (1.0 - jmath::pow(-2.0 * f64::from(x) + 2.0, 4.0) / 2.0) as f32
        }
    }),
    ("in_out_quint", |x| {
        if f64::from(x) < 0.5 {
            16.0 * x * x * x * x * x
        } else {
            (1.0 - jmath::pow(-2.0 * f64::from(x) + 2.0, 5.0) / 2.0) as f32
        }
    }),
    ("in_out_sine", |x| {
        -(trig::cos(f64::from(PI * x)) - 1.0) / 2.0
    }),
    ("out_back", |x| {
        1.0 + 2.701_58 * cube(x - 1.0) + 1.701_58 * square(x - 1.0)
    }),
    ("out_bounce", out_bounce),
    ("out_circ", out_circ),
    ("out_cubic", |x| 1.0 - cube(1.0 - x)),
    ("out_elastic", |x| {
        if x == 0.0 {
            return 0.0;
        }
        if x == 1.0 {
            return 1.0;
        }
        let x = f64::from(x);
        (jmath::pow(2.0, -10.0 * x) * jmath::sin((x * 10.0 - 0.75) * TWO_THIRDS_PI) + 1.0) as f32
    }),
    ("out_expo", |x| {
        if x == 1.0 {
            1.0
        } else {
            1.0 - jmath::pow(2.0, -10.0 * f64::from(x)) as f32
        }
    }),
    ("out_quad", |x| 1.0 - square(1.0 - x)),
    ("out_quart", |x| 1.0 - square(square(1.0 - x))),
    ("out_quint", |x| {
        1.0 - jmath::pow(1.0 - f64::from(x), 5.0) as f32
    }),
    ("out_sine", |x| trig::sin(f64::from(x * HALF_PI))),
];

/// `EasingType.CubicBezier.CubicCurve`.
#[derive(Clone, Copy, Debug)]
pub struct Curve {
    a: f32,
    b: f32,
    c: f32,
}

impl Curve {
    /// `EasingType.CubicBezier.curveFromControls`.
    fn from_controls(v1: f32, v2: f32) -> Self {
        Self {
            a: 3.0 * v1 - 3.0 * v2 + 1.0,
            b: -6.0 * v1 + 3.0 * v2,
            c: 3.0 * v1,
        }
    }
    /// `EasingType.CubicBezier.CubicCurve.sample`.
    fn sample(self, t: f32) -> f32 {
        ((self.a * t + self.b) * t + self.c) * t
    }
    /// `EasingType.CubicBezier.CubicCurve.sampleGradient`.
    fn gradient(self, t: f32) -> f32 {
        (3.0 * self.a * t + 2.0 * self.b) * t + self.c
    }
}

/// `EasingType`: a named easing or a cubic Bézier.
#[derive(Clone, Copy, Debug)]
pub enum Easing {
    Named(fn(f32) -> f32),
    CubicBezier { x: Curve, y: Curve },
}

impl Easing {
    pub const LINEAR: Self = Self::Named(|x| x);
    const CONSTANT: Self = Self::Named(|_| 0.0);

    /// `EasingType.CODEC`: a name, or `{"cubic_bezier": [x1, y1, x2, y2]}` with x1 and x2 in
    /// 0-1 (`EasingType.CubicBezierControls.validate`).
    pub fn parse(json: &Json) -> Result<Self, String> {
        if let Some(name) = json.as_str() {
            return EASINGS
                .iter()
                .find(|e| e.0 == name)
                .map(|e| Self::Named(e.1))
                .ok_or_else(|| format!("unknown easing {name}"));
        }
        let controls = json
            .get("cubic_bezier")
            .and_then(Json::as_array)
            .filter(|c| c.len() == 4);
        let controls = controls.ok_or_else(|| format!("not an easing: {json}"))?;
        let mut c = [0.0f32; 4];
        for (slot, value) in c.iter_mut().zip(controls) {
            *slot = value.as_f64().ok_or("cubic_bezier needs numbers")? as f32;
        }
        let [x1, y1, x2, y2] = c;
        if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
            return Err("cubic_bezier x1 and x2 must be in [0; 1]".into());
        }
        Ok(Self::CubicBezier {
            x: Curve::from_controls(x1, x2),
            y: Curve::from_controls(y1, y2),
        })
    }

    /// `EasingType.apply`.
    pub fn apply(&self, x: f32) -> f32 {
        match *self {
            Self::Named(f) => f(x),
            Self::CubicBezier { x: xc, y: yc } => yc.sample(solve_t(xc, x)),
        }
    }
}

/// `EasingType.CubicBezier.apply`: 4 Newton-Raphson steps on the x curve, no step
/// clamp and no bisection — the gradient is checked before the error is drawn.
fn solve_t(curve: Curve, x: f32) -> f32 {
    let mut t = x;
    for _ in 0..4 {
        let gradient = curve.gradient(t);
        if gradient < 1.0e-5 {
            break;
        }
        t -= (curve.sample(t) - x) / gradient;
    }
    t
}

/// `AttributeTrack`: a modifier and the keyframes of its argument (`KeyframeTrack`).
pub struct Track {
    pub attribute: &'static Attribute,
    keyframes: Vec<(i32, Entry)>,
    easing: Easing,
}

impl Track {
    /// `AttributeTrack.createCodec`: `{"modifier"?, "keyframes": [{"ticks", "value"}],
    /// "ease"?}`, the modifier `override` by default, with `KeyframeTrack.validateKeyframes`.
    pub fn parse(attribute: &'static Attribute, json: &Json) -> Result<Self, String> {
        let op = json
            .get("modifier")
            .map_or(Some("override"), Json::as_str)
            .ok_or("modifier is not a string")?;
        let list = json
            .get("keyframes")
            .and_then(Json::as_array)
            .ok_or("no keyframes")?;
        let keyframes = list
            .iter()
            .map(|k| {
                let ticks = k
                    .get("ticks")
                    .and_then(Json::as_i64)
                    .ok_or("keyframe without ticks")?;
                let ticks = i32::try_from(ticks)
                    .ok()
                    .filter(|t| *t >= 0)
                    .ok_or("ticks must be non-negative")?;
                let value = k.get("value").ok_or("keyframe without value")?;
                Ok((ticks, Entry::with_modifier(attribute, op, value)?))
            })
            .collect::<Result<Vec<_>, String>>()?;
        if keyframes.is_empty() {
            return Err("Keyframes must not be empty".into());
        }
        if keyframes.windows(2).any(|w| w[1].0 < w[0].0) {
            return Err("Keyframes must be ordered by ticks field".into());
        }
        if keyframes.len() > 1 {
            // As vanilla counts it, from the last keyframe's ticks.
            let (mut repeat, mut last) = (0, keyframes[keyframes.len() - 1].0);
            for &(ticks, _) in &keyframes {
                if ticks == last {
                    repeat += 1;
                    if repeat > 2 {
                        return Err(format!("More than 2 keyframes on same tick: {ticks}"));
                    }
                } else {
                    repeat = 0;
                }
                last = ticks;
            }
        }
        let easing = json.get("ease").map_or(Ok(Easing::LINEAR), Easing::parse)?;
        Ok(Self {
            attribute,
            keyframes,
            easing,
        })
    }

    /// `KeyframeTrack.validatePeriod`: every keyframe within `[0; period]`.
    pub fn validate_period(&self, period: i32) -> Result<(), String> {
        match self.keyframes.iter().find(|k| k.0 < 0 || k.0 > period) {
            Some((ticks, _)) => Err(format!(
                "Keyframe at tick {ticks} must be in range [0; {period}]"
            )),
            None => Ok(()),
        }
    }

    /// `KeyframeTrackSampler.bakeSegments`: with a period, the last keyframe also leads into
    /// the first across the loop.
    pub fn sampler(&self, period: Option<i32>) -> TrackSampler {
        let k = &self.keyframes;
        let segment =
            |from: &(i32, Entry), from_ticks: i32, to: &(i32, Entry), to_ticks: i32| Segment {
                easing: self.easing,
                from: from.1.clone(),
                from_ticks,
                to: to.1.clone(),
                to_ticks,
            };
        let mut segments = Vec::new();
        if k.len() == 1 {
            segments.push(Segment {
                easing: Easing::CONSTANT,
                from: k[0].1.clone(),
                from_ticks: 0,
                to: k[0].1.clone(),
                to_ticks: 0,
            });
        } else {
            let (first, last) = (&k[0], &k[k.len() - 1]);
            if let Some(p) = period {
                segments.push(segment(last, last.0 - p, first, first.0));
            }
            self.add_segments_from_keyframes(&mut segments);
            if let Some(p) = period {
                segments.push(segment(last, last.0, first, first.0 + p));
            }
        }
        TrackSampler {
            attribute: self.attribute,
            period,
            segments,
        }
    }
}

impl Track {
    /// `KeyframeTrackSampler.addSegmentsFromKeyframes`: one segment per pair of keyframes.
    fn add_segments_from_keyframes(&self, out: &mut Vec<Segment>) {
        for w in self.keyframes.windows(2) {
            out.push(Segment {
                easing: self.easing,
                from: w[0].1.clone(),
                from_ticks: w[0].0,
                to: w[1].1.clone(),
                to_ticks: w[1].0,
            });
        }
    }
}

struct Segment {
    easing: Easing,
    from: Entry,
    from_ticks: i32,
    to: Entry,
    to_ticks: i32,
}

/// `AttributeTrackSampler` over a `KeyframeTrackSampler`: the track's modifier, with the
/// argument sampled at the level's day time.
pub struct TrackSampler {
    pub attribute: &'static Attribute,
    period: Option<i32>,
    segments: Vec<Segment>,
}

impl TrackSampler {
    /// `KeyframeTrackSampler.sample`: the argument at `ticks`, looped by the period
    /// (`KeyframeTrackSampler.loopTicks`: Java's floor modulo).
    pub fn sample(&self, ticks: i64) -> Entry {
        let t = self
            .period
            .map_or(ticks, |p| ticks.rem_euclid(i64::from(p)));
        let segment = self
            .segments
            .iter()
            .find(|s| t < i64::from(s.to_ticks))
            .unwrap_or_else(|| &self.segments[self.segments.len() - 1]);
        if t <= i64::from(segment.from_ticks) {
            return segment.from.clone();
        }
        if t >= i64::from(segment.to_ticks) {
            return segment.to.clone();
        }
        let alpha = (t - i64::from(segment.from_ticks)) as f32
            / (segment.to_ticks - segment.from_ticks) as f32;
        Entry::keyframe_lerp(
            self.attribute,
            segment.easing.apply(alpha),
            &segment.from,
            &segment.to,
        )
    }

    /// `AttributeTrackSampler.applyTimeBased`.
    pub fn apply(&self, base: &Value, ticks: i64) -> Value {
        self.sample(ticks).apply(base)
    }
}

/// `Timeline`: tracks sampled at the level's day time, optionally looping every
/// `period_ticks`. (1.21.11's `DIRECT_CODEC` has no `clock` or `time_markers` field —
/// `day.json` still carries them, but the codec ignores them.)
pub struct Timeline {
    pub period: Option<i32>,
    pub tracks: Vec<Track>,
}

impl Timeline {
    /// `Timeline.DIRECT_CODEC` with `Timeline.validateInternal`: every track's keyframes
    /// within the period. Tracks of attributes that aren't registered fail.
    pub fn parse(json: &Json) -> Result<Self, String> {
        let period = match json.get("period_ticks") {
            None => None,
            Some(p) => Some(
                p.as_i64()
                    .and_then(|p| i32::try_from(p).ok())
                    .filter(|p| *p > 0)
                    .ok_or("period_ticks must be positive")?,
            ),
        };
        let mut tracks = Vec::new();
        for (id, track) in json
            .get("tracks")
            .and_then(Json::as_object)
            .into_iter()
            .flatten()
        {
            let attribute = attribute(id).ok_or_else(|| format!("unknown attribute {id}"))?;
            tracks.push(Track::parse(attribute, track).map_err(|e| format!("{id}: {e}"))?);
        }
        if let Some(period) = period {
            for track in &tracks {
                track.validate_period(period)?;
            }
        }
        Ok(Self { period, tracks })
    }

    /// `Timeline.getCurrentTicks`: the level's day time, within the period when there is one.
    pub fn current_ticks(&self, total: i64) -> i64 {
        self.period.map_or(total, |p| total % i64::from(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_domain_circ_easings_preserve_vanilla_nan_bits() {
        let expected_in = if cfg!(target_arch = "aarch64") {
            0xffc0_0000
        } else {
            0x7fc0_0000
        };
        let expected_out = if cfg!(target_arch = "aarch64") {
            0x7fc0_0000
        } else {
            0xffc0_0000
        };
        let in_circ = Easing::parse(&serde_json::json!("in_circ")).unwrap();
        let out_circ = Easing::parse(&serde_json::json!("out_circ")).unwrap();
        assert_eq!(in_circ.apply(1.2).to_bits(), expected_in);
        assert_eq!(out_circ.apply(-0.2).to_bits(), expected_out);
    }

    #[test]
    fn cubic_bezier_matches_the_1_21_11_oracle() {
        // Bits from the 1.21.11 client.jar's `EasingType.CubicBezier.apply`
        // (scratch-vf/m74/BezierOracle.java), the harness driver's five BEZIERS.
        let xs: [f32; 13] = [
            0.0, -0.0, 1.0, 0.5, 0.25, 0.75, 1e-6, 0.999999, -0.2, 1.2, 0.36363637, 0.72727275,
            0.9090909,
        ];
        let cases: [([f32; 4], [u32; 13]); 5] = [
            (
                [0.362, 0.241, 0.638, 0.759],
                [
                    0x00000000, 0x80000000, 0x3f800000, 0x3f000000, 0x3e5ccfa2, 0x3f48cc18,
                    0x3532b5c5, 0x3f7ffff6, 0xbdc9a876, 0x3f8c9a88, 0x3eae0d52, 0x3f42b0ef,
                    0x3f6ebe37,
                ],
            ),
            (
                [0.25, 0.1, 0.25, 1.0],
                [
                    0x00000000, 0x80000000, 0x3f800000, 0x3f4d6a4f, 0x3ed1284e, 0x3f75e0a4,
                    0x34d6c03e, 0x3f800000, 0x3d707b83, 0x3f7b1eec, 0x3f209817, 0x3f73bcfa,
                    0x3f7ece05,
                ],
            ),
            (
                [0.0, 0.0, 1.0, 1.0],
                [
                    0x00000000, 0x80000000, 0x3f800000, 0x3f000000, 0x3e7fffff, 0x3f3ffffe,
                    0x2c531b29, 0x3f7fffff, 0x3e0b4396, 0x3f5d2f19, 0x3eba2e8c, 0x3f3a2e8c,
                    0x3f68ba2d,
                ],
            ),
            (
                [0.9, -0.5, 0.1, 1.5],
                [
                    0x80000000, 0x00000000, 0x3f800001, 0x3f000000, 0xbda5a550, 0x3f8a5a55,
                    0xb5152172, 0x3f800006, 0x3e073c78, 0x3f5e30e7, 0xbd228aff, 0x3f8a227e,
                    0x3f85b4cf,
                ],
            ),
            (
                [1.0, 0.0, 0.0, 1.0],
                [
                    0x00000000, 0x80000000, 0x3f800000, 0x3f000000, 0x3cf37e7f, 0x3f78640d,
                    0x2abba671, 0x3f800000, 0x3c33f348, 0x3f7d3033, 0x3d9c3db7, 0x3f768954,
                    0x3f7f36ac,
                ],
            ),
        ];
        for (controls, expected) in cases {
            let easing = Easing::parse(&serde_json::json!({ "cubic_bezier": controls })).unwrap();
            for (x, want) in xs.iter().zip(expected) {
                assert_eq!(
                    easing.apply(*x).to_bits(),
                    want,
                    "bezier {controls:?} x={x}"
                );
            }
        }
    }

    #[test]
    fn timeline_parse_takes_the_1_21_11_codec_shape() {
        // moon.json has period_ticks + tracks and no clock; early_game.json has
        // tracks only; day.json still carries a vestigial clock and time_markers
        // the 1.21.11 codec ignores.
        let moon = serde_json::json!({
            "period_ticks": 192000,
            "tracks": {
                "minecraft:gameplay/surface_slime_spawn_chance": {
                    "ease": "constant",
                    "keyframes": [
                        { "ticks": 0, "value": 0.5 },
                        { "ticks": 24000, "value": 0.375 },
                    ],
                },
            },
        });
        let t = Timeline::parse(&moon).unwrap();
        assert_eq!(t.period, Some(192000));
        assert_eq!(t.tracks.len(), 1);
        let early = serde_json::json!({
            "tracks": {
                "minecraft:gameplay/can_pillager_patrol_spawn": {
                    "modifier": "and",
                    "keyframes": [
                        { "ticks": 0, "value": false },
                        { "ticks": 120000, "value": true },
                    ],
                },
            },
        });
        let t = Timeline::parse(&early).unwrap();
        assert_eq!(t.period, None);
        assert_eq!(t.tracks.len(), 1);
        // A vestigial clock and an out-of-period marker are ignored, as the
        // 1.21.11 codec ignores unknown fields.
        let day = serde_json::json!({
            "clock": "minecraft:overworld",
            "period_ticks": 24000,
            "time_markers": { "minecraft:noon": { "ticks": 999999, "show_in_commands": true } },
            "tracks": {},
        });
        let t = Timeline::parse(&day).unwrap();
        assert_eq!(t.period, Some(24000));
        assert!(t.tracks.is_empty());
    }
}
