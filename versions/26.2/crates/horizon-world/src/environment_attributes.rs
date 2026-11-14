//! Environment attributes (`net.minecraft.world.attribute`): the registry of attributes with
//! their types, defaults and ranges (`EnvironmentAttributes`, `AttributeTypes`,
//! `AttributeRange`), and one map entry, a value or a `{modifier, argument}` object, parsed as
//! `EnvironmentAttributeMap.Entry`'s codec does and applied as `AttributeModifier.apply`.
//! Colors are float vectors as in 26.3 (`Vector3fc`, `Vector4fc` with alpha last), and all math
//! is `f32` in Java's order of operations.
use serde_json::Value as Json;

/// `AttributeTypes`. The types without a ported codec are `Other`: their values are kept as
/// JSON and can only be overridden.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Boolean,
    Float,
    AngleDegrees,
    RgbColor,
    ArgbColor,
    Other,
}

/// An attribute's value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Float(f32),
    Rgb([f32; 3]),
    Argb([f32; 4]),
    Other(Json),
}

enum Default {
    Bool(bool),
    Float(f32),
    Rgb(i32),
    Argb(i32),
    Unported,
}

/// `AttributeRange`: `ofFloat(min, max)` or `any()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Range {
    Any,
    Float(f32, f32),
}

const UNIT: Range = Range::Float(0.0, 1.0);
const NON_NEGATIVE: Range = Range::Float(0.0, f32::INFINITY);

/// One registered attribute (`EnvironmentAttribute`).
pub struct Attribute {
    /// The id without its `minecraft:` namespace.
    pub id: &'static str,
    pub kind: Kind,
    pub range: Range,
    default: Default,
}

const fn a(id: &'static str, kind: Kind, default: Default, range: Range) -> Attribute {
    Attribute {
        id,
        kind,
        range,
        default,
    }
}

/// `EnvironmentAttributes`, in registration order.
pub static ATTRIBUTES: [Attribute; 48] = {
    use Default as D;
    use Kind::*;
    [
        a("visual/fog_color", RgbColor, D::Rgb(0), Range::Any),
        a(
            "visual/fog_start_distance",
            Float,
            D::Float(0.0),
            Range::Any,
        ),
        a(
            "visual/fog_end_distance",
            Float,
            D::Float(1024.0),
            NON_NEGATIVE,
        ),
        a(
            "visual/sky_fog_end_distance",
            Float,
            D::Float(512.0),
            NON_NEGATIVE,
        ),
        a(
            "visual/cloud_fog_end_distance",
            Float,
            D::Float(2048.0),
            NON_NEGATIVE,
        ),
        a(
            "visual/water_fog_color",
            RgbColor,
            D::Rgb(-16_448_205),
            Range::Any,
        ),
        a(
            "visual/water_fog_start_distance",
            Float,
            D::Float(-8.0),
            Range::Any,
        ),
        a(
            "visual/water_fog_end_distance",
            Float,
            D::Float(96.0),
            NON_NEGATIVE,
        ),
        a("visual/sky_color", RgbColor, D::Rgb(0), Range::Any),
        a(
            "visual/sunrise_sunset_color",
            ArgbColor,
            D::Argb(0),
            Range::Any,
        ),
        a("visual/cloud_color", ArgbColor, D::Argb(0), Range::Any),
        a("visual/cloud_height", Float, D::Float(192.33), Range::Any),
        a("visual/sun_angle", AngleDegrees, D::Float(0.0), Range::Any),
        a("visual/moon_angle", AngleDegrees, D::Float(0.0), Range::Any),
        a("visual/star_angle", AngleDegrees, D::Float(0.0), Range::Any),
        a("visual/moon_phase", Other, D::Unported, Range::Any),
        a("visual/star_brightness", Float, D::Float(0.0), UNIT),
        a(
            "visual/block_light_tint",
            RgbColor,
            D::Rgb(-10_100),
            Range::Any,
        ),
        a("visual/sky_light_color", RgbColor, D::Rgb(-1), Range::Any),
        a("visual/sky_light_factor", Float, D::Float(1.0), UNIT),
        a(
            "visual/night_vision_color",
            RgbColor,
            D::Rgb(-6_710_887),
            Range::Any,
        ),
        a(
            "visual/ambient_light_color",
            RgbColor,
            D::Rgb(-16_777_216),
            Range::Any,
        ),
        a(
            "visual/default_dripstone_particle",
            Other,
            D::Unported,
            Range::Any,
        ),
        a("visual/ambient_particles", Other, D::Unported, Range::Any),
        a("audio/background_music", Other, D::Unported, Range::Any),
        a("audio/music_volume", Float, D::Float(1.0), UNIT),
        a("audio/ambient_sounds", Other, D::Unported, Range::Any),
        a(
            "audio/firefly_bush_sounds",
            Boolean,
            D::Bool(false),
            Range::Any,
        ),
        a(
            "gameplay/sky_light_level",
            Float,
            D::Float(15.0),
            Range::Float(0.0, 15.0),
        ),
        a(
            "gameplay/can_start_raid",
            Boolean,
            D::Bool(true),
            Range::Any,
        ),
        a(
            "gameplay/water_evaporates",
            Boolean,
            D::Bool(false),
            Range::Any,
        ),
        a("gameplay/bed_rule", Other, D::Unported, Range::Any),
        a(
            "gameplay/respawn_anchor_works",
            Boolean,
            D::Bool(false),
            Range::Any,
        ),
        a(
            "gameplay/nether_portal_spawns_piglin",
            Boolean,
            D::Bool(false),
            Range::Any,
        ),
        a("gameplay/fast_lava", Boolean, D::Bool(false), Range::Any),
        a(
            "gameplay/increased_fire_burnout",
            Boolean,
            D::Bool(false),
            Range::Any,
        ),
        a("gameplay/eyeblossom_open", Other, D::Unported, Range::Any),
        a(
            "gameplay/turtle_egg_hatch_chance",
            Float,
            D::Float(0.002),
            UNIT,
        ),
        a(
            "gameplay/piglins_zombify",
            Boolean,
            D::Bool(true),
            Range::Any,
        ),
        a(
            "gameplay/snow_golem_melts",
            Boolean,
            D::Bool(false),
            Range::Any,
        ),
        a(
            "gameplay/creaking_active",
            Boolean,
            D::Bool(false),
            Range::Any,
        ),
        a(
            "gameplay/surface_slime_spawn_chance",
            Float,
            D::Float(0.0),
            UNIT,
        ),
        a(
            "gameplay/cat_waking_up_gift_chance",
            Float,
            D::Float(0.0),
            UNIT,
        ),
        a(
            "gameplay/bees_stay_in_hive",
            Boolean,
            D::Bool(false),
            Range::Any,
        ),
        a(
            "gameplay/monsters_burn",
            Boolean,
            D::Bool(false),
            Range::Any,
        ),
        a(
            "gameplay/can_pillager_patrol_spawn",
            Boolean,
            D::Bool(true),
            Range::Any,
        ),
        a("gameplay/villager_activity", Other, D::Unported, Range::Any),
        a(
            "gameplay/baby_villager_activity",
            Other,
            D::Unported,
            Range::Any,
        ),
    ]
};

/// The attribute an id names, with or without its `minecraft:` namespace (`Identifier.parse`).
pub fn attribute(id: &str) -> Option<&'static Attribute> {
    let path = match id.split_once(':') {
        Some(("minecraft", path)) => path,
        Some(_) => return None,
        None => id,
    };
    ATTRIBUTES.iter().find(|a| a.id == path)
}

/// `ARGB.redFloat`/`greenFloat`/`blueFloat`/`alphaFloat` of each byte:
/// `from8BitChannel`, the byte over 255.
fn channels(color: i32) -> [f32; 4] {
    let c = color as u32;
    let byte = |shift: u32| (c >> shift & 255) as f32 / 255.0;
    [byte(16), byte(8), byte(0), byte(24)]
}

/// 26.2's colors are packed ints (`ARGB`); the float channels here are
/// always exact k/255, so packing recovers the int exactly.
fn pack_argb(argb: [f32; 4]) -> i32 {
    let byte = |f: f32| (f * 255.0).round() as i32 & 0xFF;
    (byte(argb[3]) << 24) | (byte(argb[0]) << 16) | (byte(argb[1]) << 8) | byte(argb[2])
}
fn pack_rgb(rgb: [f32; 3]) -> i32 {
    pack_argb([rgb[0], rgb[1], rgb[2], 1.0])
}
fn unpack_rgb(color: i32) -> [f32; 3] {
    let [r, g, b, _] = channels(color);
    [r, g, b]
}

/// `ARGB.as8BitChannel` + `color`'s byte mask, back to a float channel:
/// `floor(v * 255) & 0xFF`, over 255. Out-of-range values wrap, as
/// vanilla's do.
fn quantize_channel(v: f32) -> f32 {
    ((v * 255.0).floor() as i32 & 0xFF) as f32 / 255.0
}

fn argb_channel(color: i32, shift: u32) -> i32 {
    (color >> shift) & 0xFF
}

/// `ARGB.color(a, r, g, b)`: each channel masked to a byte.
fn argb_color(a: i32, r: i32, g: i32, b: i32) -> i32 {
    (a & 0xFF) << 24 | (r & 0xFF) << 16 | (g & 0xFF) << 8 | (b & 0xFF)
}

/// `Mth.lerpInt`: `p0 + floor(alpha * (p1 - p0))`.
fn lerp_int(alpha: f32, p0: i32, p1: i32) -> i32 {
    p0 + (alpha * (p1 - p0) as f32).floor() as i32
}

/// `ARGB.multiply` on packed ints: -1 (white) is the identity; else
/// `(a * b) / 255` per channel, integer division, alpha included.
fn argb_multiply(lhs: i32, rhs: i32) -> i32 {
    if lhs == -1 {
        rhs
    } else if rhs == -1 {
        lhs
    } else {
        argb_color(
            argb_channel(lhs, 24) * argb_channel(rhs, 24) / 255,
            argb_channel(lhs, 16) * argb_channel(rhs, 16) / 255,
            argb_channel(lhs, 8) * argb_channel(rhs, 8) / 255,
            argb_channel(lhs, 0) * argb_channel(rhs, 0) / 255,
        )
    }
}

/// `ARGB.addRgb` / `subtractRgb`: the subject's alpha kept, RGB clamped.
fn argb_add(lhs: i32, rhs: i32) -> i32 {
    argb_color(
        argb_channel(lhs, 24),
        (argb_channel(lhs, 16) + argb_channel(rhs, 16)).min(255),
        (argb_channel(lhs, 8) + argb_channel(rhs, 8)).min(255),
        (argb_channel(lhs, 0) + argb_channel(rhs, 0)).min(255),
    )
}
fn argb_subtract(lhs: i32, rhs: i32) -> i32 {
    argb_color(
        argb_channel(lhs, 24),
        (argb_channel(lhs, 16) - argb_channel(rhs, 16)).max(0),
        (argb_channel(lhs, 8) - argb_channel(rhs, 8)).max(0),
        (argb_channel(lhs, 0) - argb_channel(rhs, 0)).max(0),
    )
}

/// `ARGB.alphaBlend(destination, source)` on packed ints.
fn argb_alpha_blend(dst: i32, src: i32) -> i32 {
    let da = argb_channel(dst, 24);
    let sa = argb_channel(src, 24);
    if sa == 255 {
        return src;
    }
    if sa == 0 {
        return dst;
    }
    let alpha = sa + da * (255 - sa) / 255;
    let channel = |shift: u32| {
        (argb_channel(src, shift) * sa + argb_channel(dst, shift) * (alpha - sa)) / alpha
    };
    argb_color(alpha, channel(16), channel(8), channel(0))
}

/// `ARGB.srgbLerp`: `Mth.lerpInt` per channel, alpha included.
fn argb_srgb_lerp(alpha: f32, p0: i32, p1: i32) -> i32 {
    argb_color(
        lerp_int(alpha, argb_channel(p0, 24), argb_channel(p1, 24)),
        lerp_int(alpha, argb_channel(p0, 16), argb_channel(p1, 16)),
        lerp_int(alpha, argb_channel(p0, 8), argb_channel(p1, 8)),
        lerp_int(alpha, argb_channel(p0, 0), argb_channel(p1, 0)),
    )
}

/// `ARGB.greyscale`: `(int)(r * 0.3 + g * 0.59 + b * 0.11)`, alpha kept.
fn argb_greyscale(color: i32) -> i32 {
    let grey = (argb_channel(color, 16) as f32 * 0.3
        + argb_channel(color, 8) as f32 * 0.59
        + argb_channel(color, 0) as f32 * 0.11) as i32;
    argb_color(argb_channel(color, 24), grey, grey, grey)
}

/// `ARGB.scaleRGB(color, scale)`: `(int)(channel * scale)` clamped to a byte.
fn argb_scale_rgb(color: i32, scale: f32) -> i32 {
    let scaled = |shift: u32| (argb_channel(color, shift) as f32 * scale) as i32;
    argb_color(
        argb_channel(color, 24),
        scaled(16).clamp(0, 255),
        scaled(8).clamp(0, 255),
        scaled(0).clamp(0, 255),
    )
}

impl Attribute {
    /// `EnvironmentAttribute.defaultValue`; `None` for the types not ported yet.
    pub fn default_value(&self) -> Option<Value> {
        Some(match self.default {
            Default::Bool(b) => Value::Bool(b),
            Default::Float(f) => Value::Float(f),
            Default::Rgb(c) => {
                let [r, g, b, _] = channels(c);
                Value::Rgb([r, g, b])
            }
            Default::Argb(c) => Value::Argb(channels(c)),
            Default::Unported => return None,
        })
    }

    /// `AttributeRange.sanitize`: a float outside the range goes through `Mth.clamp`
    /// (`v < min ? min : Math.min(v, max)`, so NaN stays NaN).
    pub fn sanitize(&self, value: Value) -> Value {
        match (self.range, value) {
            (Range::Float(min, max), Value::Float(v)) if !(v >= min && v <= max) => {
                Value::Float(if v < min { min } else { java_min(v, max) })
            }
            (_, value) => value,
        }
    }

    /// `EnvironmentAttribute.valueCodec`: the type's codec, then `AttributeRange.validate`.
    fn value(&self, json: &Json) -> Result<Value, String> {
        let value = match self.kind {
            Kind::Boolean => Value::Bool(boolean(json)?),
            Kind::Float | Kind::AngleDegrees => Value::Float(float(json)?),
            Kind::RgbColor => Value::Rgb(rgb(json)?),
            Kind::ArgbColor => Value::Argb(argb(json)?),
            // These types' modifiers (`append`, `overlay`) aren't ported yet.
            Kind::Other if json.get("modifier").is_some() => {
                return Err("modifiers on this attribute are not ported".into());
            }
            Kind::Other => Value::Other(json.clone()),
        };
        if let (Range::Float(min, max), Value::Float(v)) = (self.range, &value)
            && !(*v >= min && *v <= max)
        {
            return Err(format!("{v} is not in range [{min}; {max}]"));
        }
        Ok(value)
    }
}

/// `Codec.BOOL` through `JsonOps.getBooleanValue`: only a JSON boolean.
fn boolean(json: &Json) -> Result<bool, String> {
    json.as_bool()
        .ok_or_else(|| format!("not a boolean: {json}"))
}

/// `Codec.FLOAT` through `JsonOps.getNumberValue`: any JSON number, as a float. (Gson rounds the
/// decimal text to a float once; this rounds through a double, which differs only for decimals
/// a hair from a float midpoint.)
fn float(json: &Json) -> Result<f32, String> {
    json.as_f64()
        .map(|f| f as f32)
        .ok_or_else(|| format!("not a number: {json}"))
}

/// `Codec.INT` of a JSON number (`Number.intValue`: truncated).
fn int(json: &Json) -> Option<i32> {
    let n = json.as_number()?;
    n.as_i64()
        .map(|i| i as i32)
        .or_else(|| n.as_f64().map(|f| f as i64 as i32))
}

/// `ExtraCodecs.hexColor`: `#` and exactly `digits` hex digits.
fn hex(json: &Json, digits: usize) -> Option<i32> {
    let hex = json.as_str()?.strip_prefix('#')?;
    if hex.len() != digits || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    u32::from_str_radix(hex, 16).ok().map(|v| v as i32)
}

/// `Codec.FLOAT.listOf()` of exactly `N` numbers (`VECTOR3F`, `VECTOR4F`).
fn vector<const N: usize>(json: &Json) -> Option<[f32; N]> {
    let list = json.as_array().filter(|l| l.len() == N)?;
    let mut out = [0.0; N];
    for (slot, item) in out.iter_mut().zip(list) {
        *slot = float(item).ok()?;
    }
    Some(out)
}

/// `ExtraCodecs.STRING_RGB_COLOR`: `#rrggbb`, else an int, else `[r, g, b]`
/// through `ARGB.colorFromFloat` (each channel floored to a byte).
fn rgb(json: &Json) -> Result<[f32; 3], String> {
    let from_int = |c: i32| {
        let [r, g, b, _] = channels(c);
        [r, g, b]
    };
    hex(json, 6)
        .map(from_int)
        .or_else(|| vector(json).map(|v: [f32; 3]| v.map(quantize_channel)))
        .or_else(|| int(json).map(from_int))
        .ok_or_else(|| format!("not an RGB color: {json}"))
}

/// `ExtraCodecs.STRING_ARGB_COLOR`: `#aarrggbb`, else an int, else
/// `[r, g, b, a]` through `ARGB.colorFromFloat`.
fn argb(json: &Json) -> Result<[f32; 4], String> {
    hex(json, 8)
        .map(channels)
        .or_else(|| vector(json).map(|v: [f32; 4]| v.map(quantize_channel)))
        .or_else(|| int(json).map(channels))
        .ok_or_else(|| format!("not an ARGB color: {json}"))
}

/// `ArgbModifier`'s argument codec: `STRING_ARGB_COLOR`, else
/// `RGB_COLOR_CODEC` — an int is already ARGB, so the fallback is only a
/// 3-vector, made opaque (`colorFromFloat(1, ...)`). A 6-digit hex fails
/// both and is an error.
fn argb_or_rgb(json: &Json) -> Result<[f32; 4], String> {
    argb(json).or_else(|_| {
        vector(json)
            .map(|v: [f32; 3]| {
                let q = v.map(quantize_channel);
                [q[0], q[1], q[2], 1.0]
            })
            .ok_or_else(|| format!("not an ARGB color: {json}"))
    })
}

/// `AttributeModifier.OperationId`, with the argument each operation reads.
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    /// `override`: the argument replaces the value.
    Override(Value),
    /// `FloatModifier.ALPHA_BLEND`: `Mth.lerp(alpha, subject, value)`.
    AlphaBlendFloat {
        value: f32,
        alpha: f32,
    },
    Add(Value),
    Subtract(Value),
    Multiply(Value),
    Minimum(f32),
    Maximum(f32),
    /// `ColorModifier.ALPHA_BLEND`, by an ARGB color.
    AlphaBlendColor([f32; 4]),
    /// `ColorModifier.BLEND_TO_GRAY`.
    BlendToGray {
        brightness: f32,
        factor: f32,
    },
    Boolean(BoolOp, bool),
    /// A modifier of a type not ported yet (kept, and applying it leaves
    /// the value as it was). 26.3's `ListModifier.append` and
    /// `MobSpawnSettingsModifier` overlay do not exist in 26.2.
    Unported(Json),
    /// An entry computed when it applies, from the value so far (not vanilla: a resource pack's
    /// expression). `None` leaves the value as it was.
    Dynamic(Dynamic),
}

/// The function behind [`Entry::Dynamic`]; two are equal when they are the same function.
#[derive(Clone)]
pub struct Dynamic(pub std::sync::Arc<dyn Fn(&Value) -> Option<Entry> + Send + Sync>);

impl std::fmt::Debug for Dynamic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Dynamic")
    }
}

impl PartialEq for Dynamic {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
}

/// `BooleanModifier`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoolOp {
    And,
    Nand,
    Or,
    Nor,
    Xor,
    Xnor,
}

/// `Codec.floatRange(0, 1)`.
fn unit(json: &Json, name: &str) -> Result<f32, String> {
    let v = float(json)?;
    if (0.0..=1.0).contains(&v) {
        Ok(v)
    } else {
        Err(format!("{name} {v} is outside [0, 1]"))
    }
}

impl Entry {
    /// `EnvironmentAttributeMap.Entry.createCodec`: the attribute's value (an override), else
    /// `{"modifier": op, "argument": ...}` with an operation the attribute's type allows.
    pub fn parse(attribute: &Attribute, json: &Json) -> Result<Self, String> {
        let value_error = match attribute.value(json) {
            Ok(value) => return Ok(Self::Override(value)),
            Err(e) => e,
        };
        let Some(object) = json.as_object() else {
            return Err(value_error);
        };
        if attribute.kind == Kind::Other && object.contains_key("modifier") {
            return Ok(Self::Unported(json.clone()));
        }
        let op = object
            .get("modifier")
            .and_then(Json::as_str)
            .ok_or_else(|| format!("{value_error}; and no modifier"))?;
        let argument = object.get("argument").ok_or("no argument")?;
        Self::with_modifier(attribute, op, argument)
    }

    /// `EnvironmentAttributeMap.Entry.createFullCodec`: the entry `op` makes of `argument` on
    /// this attribute, with the modifier's argument codec (also a timeline keyframe's value,
    /// under its track's modifier).
    pub fn with_modifier(attribute: &Attribute, op: &str, argument: &Json) -> Result<Self, String> {
        let kind = attribute.kind;
        let color = matches!(kind, Kind::RgbColor | Kind::ArgbColor);
        let simple = |argument: &Json| -> Result<Value, String> {
            Ok(match kind {
                Kind::Float | Kind::AngleDegrees => Value::Float(float(argument)?),
                // Add/subtract are `RgbModifier`s (`STRING_RGB_COLOR`)
                // on both color kinds; the argument's alpha is ignored by
                // `addRgb`/`subtractRgb` anyway. Multiply reads ARGB on
                // ARGB attributes (handled below).
                Kind::RgbColor | Kind::ArgbColor => Value::Rgb(rgb(argument)?),
                _ => return Err(format!("{op} is not allowed on this attribute")),
            })
        };
        let bool_op = |op: BoolOp| -> Result<Self, String> {
            match kind {
                Kind::Boolean => Ok(Self::Boolean(op, boolean(argument)?)),
                _ => Err("boolean modifiers need a boolean attribute".into()),
            }
        };
        Ok(match op {
            "override" => Self::Override(attribute.value(argument)?),
            "add" => Self::Add(simple(argument)?),
            "subtract" => Self::Subtract(simple(argument)?),
            "multiply" if kind == Kind::ArgbColor => {
                Self::Multiply(Value::Argb(argb_or_rgb(argument)?))
            }
            "multiply" => Self::Multiply(simple(argument)?),
            "minimum" | "maximum" if matches!(kind, Kind::Float | Kind::AngleDegrees) => {
                let v = float(argument)?;
                if op == "minimum" {
                    Self::Minimum(v)
                } else {
                    Self::Maximum(v)
                }
            }
            "alpha_blend" if color => Self::AlphaBlendColor(argb(argument)?),
            "alpha_blend" if matches!(kind, Kind::Float | Kind::AngleDegrees) => match argument {
                // `FloatWithAlpha.CODEC`: a float, or `{value, alpha = 1}`.
                Json::Object(o) => Self::AlphaBlendFloat {
                    value: float(o.get("value").ok_or("no value")?)?,
                    alpha: o.get("alpha").map_or(Ok(1.0), |a| unit(a, "alpha"))?,
                },
                _ => Self::AlphaBlendFloat {
                    value: float(argument)?,
                    alpha: 1.0,
                },
            },
            "blend_to_gray" if color => Self::BlendToGray {
                brightness: unit(
                    argument.get("brightness").ok_or("no brightness")?,
                    "brightness",
                )?,
                factor: unit(argument.get("factor").ok_or("no factor")?, "factor")?,
            },
            "and" => bool_op(BoolOp::And)?,
            "nand" => bool_op(BoolOp::Nand)?,
            "or" => bool_op(BoolOp::Or)?,
            "nor" => bool_op(BoolOp::Nor)?,
            "xor" => bool_op(BoolOp::Xor)?,
            "xnor" => bool_op(BoolOp::Xnor)?,
            _ => return Err(format!("modifier {op} is not allowed on this attribute")),
        })
    }

    /// `AttributeTrack.bakeSampler`'s argument lerp: two keyframes of one track (same
    /// modifier) blended at `alpha`. Overrides use the type's keyframe lerp; float arguments
    /// `Mth.lerp`; color arguments `ARGB.srgbLerp` (packed ints, `Mth.lerpInt` per
    /// channel); `FloatWithAlpha` and `ColorModifier.BlendToGray.lerp` each field;
    /// boolean arguments step at 1 (as `AttributeType.ofNotInterpolated` keyframes do).
    pub fn keyframe_lerp(attribute: &Attribute, alpha: f32, from: &Self, to: &Self) -> Self {
        let float = |a: f32, b: f32| a + alpha * (b - a);
        let value = |a: &Value, b: &Value| match (a, b) {
            (Value::Float(a), Value::Float(b)) => Value::Float(float(*a, *b)),
            (Value::Rgb(a), Value::Rgb(b)) => Value::Rgb(unpack_rgb(argb_srgb_lerp(
                alpha,
                pack_rgb(*a),
                pack_rgb(*b),
            ))),
            (Value::Argb(a), Value::Argb(b)) => Value::Argb(channels(argb_srgb_lerp(
                alpha,
                pack_argb(*a),
                pack_argb(*b),
            ))),
            _ => {
                if alpha >= 1.0 {
                    b.clone()
                } else {
                    a.clone()
                }
            }
        };
        match (from, to) {
            (Self::Override(a), Self::Override(b)) => {
                Self::Override(attribute.lerp(Lerp::Keyframe, alpha, a, b))
            }
            (
                Self::AlphaBlendFloat {
                    value: v1,
                    alpha: a1,
                },
                Self::AlphaBlendFloat {
                    value: v2,
                    alpha: a2,
                },
            ) => Self::AlphaBlendFloat {
                value: float(*v1, *v2),
                alpha: float(*a1, *a2),
            },
            (Self::Add(a), Self::Add(b)) => Self::Add(value(a, b)),
            (Self::Subtract(a), Self::Subtract(b)) => Self::Subtract(value(a, b)),
            (Self::Multiply(a), Self::Multiply(b)) => Self::Multiply(value(a, b)),
            (Self::Minimum(a), Self::Minimum(b)) => Self::Minimum(float(*a, *b)),
            (Self::Maximum(a), Self::Maximum(b)) => Self::Maximum(float(*a, *b)),
            (Self::AlphaBlendColor(a), Self::AlphaBlendColor(b)) => Self::AlphaBlendColor(
                channels(argb_srgb_lerp(alpha, pack_argb(*a), pack_argb(*b))),
            ),
            (
                Self::BlendToGray {
                    brightness: b1,
                    factor: f1,
                },
                Self::BlendToGray {
                    brightness: b2,
                    factor: f2,
                },
            ) => Self::BlendToGray {
                brightness: float(*b1, *b2),
                factor: float(*f1, *f2),
            },
            _ => {
                if alpha >= 1.0 {
                    to.clone()
                } else {
                    from.clone()
                }
            }
        }
    }

    /// `AttributeModifier.apply(subject, argument)`. 26.2's color modifiers
    /// work on packed ints (`ARGB`); the float channels pack back to the
    /// int exactly, so each arm packs, applies the int formula, unpacks.
    pub fn apply(&self, subject: &Value) -> Value {
        use Value::{Argb, Float, Rgb};
        match (self, subject) {
            (Self::Override(v), _) => v.clone(),
            (Self::Dynamic(f), subject) => match (f.0)(subject) {
                Some(entry) => entry.apply(subject),
                None => subject.clone(),
            },
            (Self::AlphaBlendFloat { value, alpha }, Float(s)) => Float(s + alpha * (value - s)),
            (Self::Add(Float(b)), Float(a)) => Float(a + b),
            (Self::Subtract(Float(b)), Float(a)) => Float(a - b),
            (Self::Multiply(Float(b)), Float(a)) => Float(a * b),
            (Self::Minimum(b), Float(a)) => Float(java_min(*a, *b)),
            (Self::Maximum(b), Float(a)) => Float(java_max(*a, *b)),
            (Self::Add(Rgb(b)), Rgb(a)) => Rgb(unpack_rgb(argb_add(pack_rgb(*a), pack_rgb(*b)))),
            (Self::Add(Rgb(b)), Argb(a)) => Argb(channels(argb_add(pack_argb(*a), pack_rgb(*b)))),
            (Self::Subtract(Rgb(b)), Rgb(a)) => {
                Rgb(unpack_rgb(argb_subtract(pack_rgb(*a), pack_rgb(*b))))
            }
            (Self::Subtract(Rgb(b)), Argb(a)) => {
                Argb(channels(argb_subtract(pack_argb(*a), pack_rgb(*b))))
            }
            (Self::Multiply(Rgb(b)), Rgb(a)) => {
                Rgb(unpack_rgb(argb_multiply(pack_rgb(*a), pack_rgb(*b))))
            }
            (Self::Multiply(Argb(b)), Argb(a)) => {
                Argb(channels(argb_multiply(pack_argb(*a), pack_argb(*b))))
            }
            (Self::AlphaBlendColor(src), Rgb(dst)) => Rgb(unpack_rgb(argb_alpha_blend(
                pack_rgb(*dst),
                pack_argb(*src),
            ))),
            (Self::AlphaBlendColor(src), Argb(dst)) => {
                Argb(channels(argb_alpha_blend(pack_argb(*dst), pack_argb(*src))))
            }
            // `srgbLerp(factor, subject, scaleRGB(greyscale(subject), brightness))`.
            (Self::BlendToGray { brightness, factor }, Rgb(s)) => {
                let subject = pack_rgb(*s);
                let grey = argb_scale_rgb(argb_greyscale(subject), *brightness);
                Rgb(unpack_rgb(argb_srgb_lerp(*factor, subject, grey)))
            }
            (Self::BlendToGray { brightness, factor }, Argb(s)) => {
                let subject = pack_argb(*s);
                let grey = argb_scale_rgb(argb_greyscale(subject), *brightness);
                Argb(channels(argb_srgb_lerp(*factor, subject, grey)))
            }
            (Self::Boolean(op, arg), Value::Bool(s)) => Value::Bool(match op {
                BoolOp::And => *arg && *s,
                BoolOp::Nand => !*arg || !*s,
                BoolOp::Or => *arg || *s,
                BoolOp::Nor => !*arg && !*s,
                BoolOp::Xor => arg ^ s,
                BoolOp::Xnor => arg == s,
            }),
            // A parsed entry always matches its attribute's type.
            (_, subject) => subject.clone(),
        }
    }
}

/// Java's `Math.min(float, float)`: NaN wins, and -0.0 is below 0.0.
fn java_min(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        f32::NAN
    } else if a == b && a == 0.0 {
        if a.is_sign_negative() { a } else { b }
    } else if a < b {
        a
    } else {
        b
    }
}

/// Java's `Math.max(float, float)`.
fn java_max(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        f32::NAN
    } else if a == b && a == 0.0 {
        if a.is_sign_negative() { b } else { a }
    } else if a > b {
        a
    } else {
        b
    }
}

/// `EnvironmentAttributeMap`: at most one entry per attribute.
#[derive(Clone, Default)]
pub struct AttributeMap(Vec<(&'static Attribute, Entry)>);

impl std::fmt::Debug for AttributeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|(a, _)| a.id))
            .finish()
    }
}

impl AttributeMap {
    /// `EnvironmentAttributeMap.CODEC`: a JSON object from attribute id to entry.
    pub fn parse(json: &Json) -> Result<Self, String> {
        let map = json.as_object().ok_or("attributes are not an object")?;
        map.iter()
            .map(|(id, entry)| {
                let attribute = attribute(id).ok_or_else(|| format!("unknown attribute {id}"))?;
                let entry = Entry::parse(attribute, entry).map_err(|e| format!("{id}: {e}"))?;
                Ok((attribute, entry))
            })
            .collect::<Result<_, _>>()
            .map(Self)
    }

    /// A map built in code (`EnvironmentAttributeMap.Builder`); a later entry for the same
    /// attribute replaces the earlier one.
    pub fn from_entries(entries: impl IntoIterator<Item = (&'static Attribute, Entry)>) -> Self {
        let mut map = Self::default();
        for (attribute, entry) in entries {
            map.0.retain(|(a, _)| a.id != attribute.id);
            map.0.push((attribute, entry));
        }
        map
    }

    /// Sets the attribute's entry, replacing the one it had.
    pub fn set(&mut self, attribute: &'static Attribute, entry: Entry) {
        self.0.retain(|(a, _)| a.id != attribute.id);
        self.0.push((attribute, entry));
    }

    /// Drops the attribute's entry.
    pub fn remove(&mut self, attribute: &Attribute) {
        self.0.retain(|(a, _)| a.id != attribute.id);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, attribute: &Attribute) -> Option<&Entry> {
        self.0
            .iter()
            .find(|(a, _)| a.id == attribute.id)
            .map(|(_, e)| e)
    }

    /// The attributes it has an entry for.
    pub fn attributes(&self) -> impl Iterator<Item = &'static Attribute> + '_ {
        self.0.iter().map(|(a, _)| *a)
    }

    /// `EnvironmentAttributeMap.applyModifier`: the entry applied to `base`, or `base`.
    pub fn apply(&self, attribute: &Attribute, base: &Value) -> Value {
        self.get(attribute)
            .map_or_else(|| base.clone(), |e| e.apply(base))
    }
}

/// What an `AttributeType` lerps for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lerp {
    /// Between a timeline's keyframes (`keyframeLerp`).
    Keyframe,
    /// Into rain or thunder (`stateChangeLerp`).
    StateChange,
    /// Across biomes (`spatialLerp`).
    Spatial,
}

impl Attribute {
    /// `EnvironmentAttribute.isSpatiallyInterpolated`: mixed across nearby biomes.
    pub fn spatially_interpolated(&self) -> bool {
        self.id.starts_with("visual/")
            && !matches!(
                self.id,
                "visual/moon_phase"
                    | "visual/default_dripstone_particle"
                    | "visual/ambient_particles"
            )
    }

    /// `AttributeType`'s lerps. Floats and angles use `Mth.lerp`; colors
    /// `ARGB.srgbLerp` (packed ints, `Mth.lerpInt` per channel); the types
    /// `ofNotInterpolated` step to `to` once alpha reaches 1 (keyframes), 0
    /// (state changes) or 0.5 (biomes).
    pub fn lerp(&self, purpose: Lerp, alpha: f32, from: &Value, to: &Value) -> Value {
        match (from, to) {
            (Value::Float(a), Value::Float(b)) => Value::Float(a + alpha * (b - a)),
            (Value::Rgb(a), Value::Rgb(b)) => Value::Rgb(unpack_rgb(argb_srgb_lerp(
                alpha,
                pack_rgb(*a),
                pack_rgb(*b),
            ))),
            (Value::Argb(a), Value::Argb(b)) => Value::Argb(channels(argb_srgb_lerp(
                alpha,
                pack_argb(*a),
                pack_argb(*b),
            ))),
            _ => {
                let threshold = match purpose {
                    Lerp::Keyframe => 1.0,
                    Lerp::StateChange => 0.0,
                    Lerp::Spatial => 0.5,
                };
                if alpha >= threshold {
                    to.clone()
                } else {
                    from.clone()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fog() -> &'static Attribute {
        attribute("visual/fog_color").unwrap()
    }

    fn rgb_value(hex: &str) -> Value {
        fog().value(&serde_json::json!(hex)).unwrap()
    }

    #[test]
    fn color_multiply_truncates_like_26_2_argb() {
        // (200 * 200) / 255 = 156 by integer division; float math would round to 157.
        let entry = Entry::with_modifier(fog(), "multiply", &serde_json::json!("#c8c8c8")).unwrap();
        assert_eq!(
            entry.apply(&rgb_value("#c8c8c8")),
            Value::Rgb([156.0 / 255.0, 156.0 / 255.0, 156.0 / 255.0])
        );
    }

    #[test]
    fn color_alpha_blend_uses_the_int_formula() {
        // dst opaque red, src half-alpha green: out alpha 255, R 127, G 128.
        let entry =
            Entry::with_modifier(fog(), "alpha_blend", &serde_json::json!("#8000ff00")).unwrap();
        assert_eq!(
            entry.apply(&rgb_value("#ff0000")),
            Value::Rgb([127.0 / 255.0, 128.0 / 255.0, 0.0])
        );
    }

    #[test]
    fn color_lerp_uses_lerp_int() {
        // lerpInt(0.5, 0, 255) = floor(127.5) = 127, not 128.
        assert_eq!(
            fog().lerp(
                Lerp::Keyframe,
                0.5,
                &rgb_value("#000000"),
                &rgb_value("#ffffff")
            ),
            Value::Rgb([127.0 / 255.0, 127.0 / 255.0, 127.0 / 255.0])
        );
    }

    #[test]
    fn vector_colors_quantize_through_the_packed_int() {
        // colorFromFloat floors: 0.5 -> 127; out-of-range wraps: 1.5 -> 126, -0.5 -> 128.
        assert_eq!(
            fog().value(&serde_json::json!([0.5, 0.5, 0.5])).unwrap(),
            Value::Rgb([127.0 / 255.0, 127.0 / 255.0, 127.0 / 255.0])
        );
        assert_eq!(
            fog().value(&serde_json::json!([1.5, -0.5, 0.25])).unwrap(),
            Value::Rgb([126.0 / 255.0, 128.0 / 255.0, 63.0 / 255.0])
        );
    }

    #[test]
    fn the_26_3_only_attributes_are_gone() {
        assert!(attribute("gameplay/natural_mob_spawns").is_none());
        assert!(attribute("gameplay/straw_bed_rule").is_none());
        assert!(attribute("gameplay/creature_world_gen_spawn_probability").is_none());
        assert_eq!(ATTRIBUTES.len(), 48);
    }
}
