//! A dimension's environment attributes (`EnvironmentAttributeSystem.addDefaultLayers`): the
//! dimension type's attributes, the biome layer, the dimension's timelines and, where it can
//! have weather, the rain and thunder layers, sampled into the sky state the renderer draws.
use crate::Result;
use horizon_world::environment_attributes::{
    Attribute, AttributeMap, Entry, Kind, Value, attribute,
};
use horizon_world::environment_system::{BiomeBlend, Builder, Conditions, System};
use horizon_world::environment_timeline::Timeline;
use serde_json::Value as Json;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

pub struct Environment {
    system: System,
    /// What the packs' expressions read; the frame sets it before sampling.
    frame: Arc<RwLock<Frame>>,
    /// Biome attribute maps by biome id, where resource packs changed them; the biome layer
    /// reads these instead of the biome's own.
    biomes: HashMap<String, AttributeMap>,
    /// Packs' biome `post_process` maps by biome id: a biome layer after the weather.
    post: HashMap<String, AttributeMap>,
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Sky {
    pub sky_color: [f32; 3],
    pub fog_color: [f32; 3],
    pub sky_light_factor: f32,
    pub sky_light_color: [f32; 3],
    pub block_light_tint: [f32; 3],
    pub ambient_light_color: [f32; 3],
    pub sun_angle: f32,
    pub moon_angle: f32,
    pub star_angle: f32,
    pub star_brightness: f32,
    pub sunrise_color: [f32; 4],
    pub cloud_color: [f32; 4],
    /// `gameplay/sky_light_level`, the server-side sky darkening.
    pub sky_light_level: f32,
    /// Environmental fog (`visual/fog_start_distance` / `fog_end_distance`).
    pub fog_start: f32,
    pub fog_end: f32,
    /// `visual/cloud_fog_end_distance`.
    pub cloud_fog_end: f32,
    /// `visual/cloud_height`.
    pub cloud_height: f32,
}

/// Where and when the sky is sampled.
#[derive(Clone, Copy, Default)]
pub struct Sample<'a> {
    /// The day clock's total ticks.
    pub day_time: i64,
    /// `Level.getRainLevel` and `getThunderLevel`, 0 where the dimension has no weather.
    pub rain: f32,
    pub thunder: f32,
    /// The biomes around the camera (`EnvironmentAttributeProbe`'s Gaussian mix).
    pub blend: Option<&'a BiomeBlend<'a>>,
    /// The biome at the camera, for attributes that don't blend.
    pub biome: Option<&'a AttributeMap>,
    /// The same for the packs' biome post-process group.
    pub post_blend: Option<&'a BiomeBlend<'a>>,
    pub post_biome: Option<&'a AttributeMap>,
}

/// Resource packs' Polytone biome and dimension modifiers (#11486): layers they add to, change
/// in or remove from the attribute stack.
#[derive(Default)]
pub struct PackModifiers {
    biomes: Vec<horizon_polytone::BiomeModifier>,
    dimensions: Vec<horizon_polytone::DimensionModifier>,
    /// The packs' config values, for `config(id)` in expressions.
    configs: horizon_polytone::pack::Configs,
    /// The pack overlay root they were read from, where their colormaps are.
    root: Option<std::path::PathBuf>,
    colormaps: ColormapCache,
}

impl PackModifiers {
    /// Every `assets/<namespace>/polytone/{biome,dimension}_modifiers/**.json` under a pack
    /// overlay root. A file that fails to parse is skipped with a warning.
    pub fn read(pack_root: &Path) -> Self {
        let mut out = Self {
            root: Some(pack_root.to_path_buf()),
            ..Self::default()
        };
        let Ok(namespaces) = std::fs::read_dir(pack_root.join("assets")) else {
            return out;
        };
        for namespace in namespaces.flatten() {
            for kind in ["biome_modifiers", "dimension_modifiers"] {
                let mut stack = vec![namespace.path().join("polytone").join(kind)];
                while let Some(dir) = stack.pop() {
                    for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            stack.push(path);
                            continue;
                        }
                        let Ok(relative) = path.strip_prefix(pack_root) else {
                            continue;
                        };
                        let relative = relative.to_string_lossy();
                        let implicit = horizon_polytone::implicit_id(&relative, kind);
                        let json = match horizon_world::json_file::read::<Json>(&path) {
                            Ok(json) => json,
                            Err(e) => {
                                log::warn!("{relative}: {e}");
                                continue;
                            }
                        };
                        let parsed = if kind == "biome_modifiers" {
                            horizon_polytone::BiomeModifier::parse(&json, implicit.as_deref())
                                .map(|m| m.map(|m| out.biomes.push(m)))
                        } else {
                            horizon_polytone::DimensionModifier::parse(&json, implicit.as_deref())
                                .map(|m| m.map(|m| out.dimensions.push(m)))
                        };
                        if let Err(e) = parsed {
                            log::warn!("{relative}: {e}");
                        }
                    }
                }
            }
        }
        out.biomes.sort_by_key(|m| m.priority);
        out.dimensions.sort_by_key(|m| m.priority);
        out
    }

    pub fn is_empty(&self) -> bool {
        self.biomes.is_empty() && self.dimensions.is_empty()
    }

    /// The config values expressions read.
    pub fn with_configs(mut self, configs: horizon_polytone::pack::Configs) -> Self {
        self.configs = configs;
        self
    }
}

/// What a pack's expression can ask about the frame (wiki "Scripting Expressions": the global,
/// player, camera and positional contexts), set before the frame samples the sky.
#[derive(Clone, Debug, Default)]
pub struct Frame {
    pub rain: f32,
    pub thunder: f32,
    pub game_time: i64,
    pub day_time: i64,
    /// The render distance in chunks.
    pub render_distance: i32,
    /// `skyType`: 1 for the overworld's sky, 2 for the End's, 0 for none.
    pub sky_type: i32,
    pub dimension: String,
    pub player: [f64; 3],
    /// The player's motion per tick (`p.speedSq`).
    pub player_velocity: [f64; 3],
    pub player_on_ground: bool,
    pub in_water: bool,
    pub camera: [f64; 3],
    /// The sky and block light at the camera.
    pub sky_light: i32,
    pub block_light: i32,
    /// The camera's biome (`o.biome()`), and its climate (`o.temperature`, `o.downfall`).
    pub biome: String,
    pub temperature: f32,
    pub downfall: f32,
}

impl Frame {
    /// `g.rain`: the mean of the rain and thunder levels, so plain rain is 0.5 and a
    /// thunderstorm 1 (observed on the real mod: a pack's `g.rain` colormap row).
    pub fn global_rain(&self) -> f64 {
        (f64::from(self.rain) + f64::from(self.thunder)) / 2.0
    }
}

/// The game side of a pack expression in an attribute value: `g`, `p`, `c`, `o` (the camera's
/// position) and `r`, `v` (the value so far) and the pack's `config`.
struct FrameHost<'a> {
    frame: &'a Frame,
    configs: &'a horizon_polytone::pack::Configs,
    value: f64,
    seed: u64,
}

/// `program`'s value for the frame (a pack's expression outside attribute layers: its `v` is 0).
pub(crate) fn eval_frame(
    program: &horizon_polytone::expr::Program,
    frame: &Frame,
    configs: &horizon_polytone::pack::Configs,
) -> Option<horizon_polytone::expr::Value> {
    let mut host = FrameHost {
        frame,
        configs,
        value: 0.0,
        seed: frame.game_time as u64,
    };
    program.eval(&mut host).ok()
}

impl horizon_polytone::expr::Host for FrameHost<'_> {
    fn root(&mut self, root: horizon_polytone::expr::Root) -> horizon_polytone::expr::Value {
        use horizon_polytone::expr::{Root, Value as V};
        match root {
            Root::Value => V::Num(self.value),
            other => V::Object(other as u32),
        }
    }
    fn method(
        &mut self,
        target: &horizon_polytone::expr::Value,
        name: &str,
        _args: &[horizon_polytone::expr::Value],
    ) -> std::result::Result<horizon_polytone::expr::Value, String> {
        use horizon_polytone::expr::{Root, Value as V};
        let f = self.frame;
        let on = |root: Root| *target == V::Object(root as u32);
        let num = |n: f64| Ok(V::Num(n));
        match name {
            "rain" if on(Root::Global) => num(f.global_rain()),
            "time" if on(Root::Global) => num(f.game_time as f64),
            "dayTime" if on(Root::Global) => num(f.day_time as f64),
            "skyType" if on(Root::Global) => num(f64::from(f.sky_type)),
            "dimensionType" if on(Root::Global) => Ok(V::Str(f.dimension.as_str().into())),
            "renderDistance" if on(Root::Global) => num(f64::from(f.render_distance)),
            "moonPhase" if on(Root::Global) => {
                num(f.day_time.div_euclid(24_000).rem_euclid(8) as f64)
            }
            "seaLevel" if on(Root::Global) => num(63.0),
            "viewDistance" if on(Root::Camera) => num(f64::from(f.render_distance * 16)),
            "x" | "y" | "z" if on(Root::Player) || on(Root::Camera) || on(Root::Object) => {
                let at = if on(Root::Player) { f.player } else { f.camera };
                num(at[usize::from(name.as_bytes()[0] - b'x')])
            }
            "inWater" if on(Root::Player) => Ok(V::Bool(f.in_water)),
            "skyLight" if on(Root::Object) || on(Root::Player) => num(f64::from(f.sky_light)),
            "blockLight" if on(Root::Object) || on(Root::Player) => num(f64::from(f.block_light)),
            "temperature" if on(Root::Object) => num(f64::from(f.temperature)),
            "downfall" if on(Root::Object) => num(f64::from(f.downfall)),
            "biome" if on(Root::Object) || on(Root::Player) || on(Root::Camera) => {
                Ok(V::Str(f.biome.as_str().into()))
            }
            "rand" if on(Root::Random) => {
                // A splitmix64 step: the frame's own sequence.
                self.seed = self.seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
                let mut z = self.seed;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                num(((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64)
            }
            _ => Err(format!(
                "{name} is not available in an attribute expression"
            )),
        }
    }
    fn function(
        &mut self,
        name: &str,
        args: &[horizon_polytone::expr::Value],
    ) -> std::result::Result<horizon_polytone::expr::Value, String> {
        match (name, args) {
            ("config", [horizon_polytone::expr::Value::Str(id)]) => self
                .configs
                .get(id)
                .map(horizon_polytone::pack::ConfigValue::to_value)
                .ok_or_else(|| format!("no config {id}")),
            _ => Err(format!(
                "{name} is not available in an attribute expression"
            )),
        }
    }
}

/// A value as an expression's `v`: floats as they are, colours packed ARGB (`ARGB.as8BitChannel`).
fn as_number(value: &Value) -> f64 {
    let byte = |c: f32| ((c.clamp(0.0, 1.0) * 255.0).floor() as u32) & 255;
    match value {
        Value::Float(f) => f64::from(*f),
        Value::Bool(b) => f64::from(u8::from(*b)),
        Value::Rgb([r, g, b]) => {
            f64::from((0xff00_0000 | byte(*r) << 16 | byte(*g) << 8 | byte(*b)) as i32)
        }
        Value::Argb([r, g, b, a]) => {
            f64::from((byte(*a) << 24 | byte(*r) << 16 | byte(*g) << 8 | byte(*b)) as i32)
        }
        Value::Other(_) => 0.0,
    }
}

/// A Polytone colormap used as an attribute value (wiki "Colormaps"): an image read at the
/// column and row its two expressions pick, each 0 to 1 with 1 the top-left corner.
pub(crate) struct Colormap {
    x: Axis,
    y: Axis,
    /// Only the lower-left triangle: the row value is scaled by the column value, as vanilla's
    /// grass colormap scales downfall by temperature.
    triangular: bool,
    /// Rounds to the nearest pixel (the default) instead of the one below.
    rounds: bool,
    /// `biome_blend`: blocks take the mean of the colours around them, over the Biome Blend
    /// square, as vanilla's biome tints do. Without the field, maps that sample the biome
    /// (`temperature`, `downfall`, `biome_id`) blend: Polytone 6.8.6's water by `biome_id`
    /// fades over a biome edge.
    pub(crate) biome_blend: bool,
    /// Its axes read nothing that changes up a column of one biome (no `o.y`): a section of
    /// one biome can sample it once per column.
    pub(crate) column: bool,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

/// A colormap axis: an expression (or a named sampler, wiki "Default Samplers"), or the
/// `biome_id` sampler with its mapper.
enum Axis {
    Expr(horizon_polytone::expr::Program),
    /// `biome_id`: the value the mapper gives the biome where it samples (0 for the others),
    /// and the expression naming that biome. Unlike the other axes it counts from the left (or
    /// top) edge, in pixel coordinates over the mapper's `texture_size`: the column (or row) is
    /// `floor(value * size)`, so `1` over a `texture_size` of 16 is the second pixel, and the
    /// biomes left out are the first, as the wiki's "start your mapper at 1" advice expects.
    BiomeId(HashMap<String, f64>, horizon_polytone::expr::Program),
}

impl Axis {
    /// Whether the axis reads only what stays the same up a column of one biome.
    fn column(&self) -> bool {
        use horizon_polytone::expr::Root;
        let flat = |root: Root, name: &str| match root {
            Root::Object => matches!(name, "x" | "z" | "biome" | "temperature" | "downfall"),
            Root::Random => name == "noise",
            _ => false,
        };
        match self {
            Self::Expr(program) | Self::BiomeId(_, program) => program.reads_only(&flat),
        }
    }
    fn value(&self, host: &mut dyn horizon_polytone::expr::Host) -> Option<f64> {
        match self {
            Self::Expr(program) => Some(program.eval(host).ok()?.num()),
            Self::BiomeId(mapper, biome) => {
                let biome = match biome.eval(host).ok()? {
                    horizon_polytone::expr::Value::Str(s) => s,
                    _ => return Some(0.0),
                };
                Some(mapper.get(biome.as_ref()).copied().unwrap_or(0.0))
            }
        }
    }
}

/// A `biome_id_mapper`: inline, or an id of `polytone/biome_id_mappers/<path>.json`. Its values
/// are divided by its `texture_size`, when it has one (pixel coordinates).
fn biome_id_mapper(pack_root: &Path, json: &Json) -> HashMap<String, f64> {
    let object = match json {
        Json::String(id) => {
            let (namespace, path) = id.split_once(':').unwrap_or(("minecraft", id));
            let file = pack_root.join(format!(
                "assets/{namespace}/polytone/biome_id_mappers/{path}.json"
            ));
            horizon_world::json_file::read::<Json>(&file).unwrap_or(Json::Null)
        }
        other => other.clone(),
    };
    let Some(object) = object.as_object() else {
        return HashMap::new();
    };
    let size = object
        .get("texture_size")
        .and_then(|v| v.as_f64().or_else(|| v.as_str()?.parse().ok()))
        .filter(|s| *s > 0.0)
        .unwrap_or(1.0);
    object
        .iter()
        .filter(|(k, _)| *k != "texture_size")
        .filter_map(|(k, v)| {
            let id = if k.contains(':') {
                k.clone()
            } else {
                format!("minecraft:{k}")
            };
            Some((id, v.as_f64()? / size))
        })
        .collect()
}

impl Colormap {
    /// One colour wherever it samples (a `biome_colors` flat colour, RGB).
    pub(crate) fn flat(rgb: u32) -> Self {
        let zero = || Axis::Expr(horizon_polytone::expr::Program::compile("0").expect("a number"));
        let [_, r, g, b] = rgb.to_be_bytes();
        Self {
            x: zero(),
            y: zero(),
            triangular: false,
            rounds: true,
            biome_blend: false,
            column: true,
            width: 1,
            height: 1,
            rgba: vec![r, g, b, 255],
        }
    }

    /// `assets/<namespace>/polytone/colormaps/<path>.png` and its optional `.json` (without
    /// one, the axes are the biome's temperature and downfall).
    pub(crate) fn load(pack_root: &Path, id: &str) -> Option<Self> {
        let (namespace, path) = id.split_once(':')?;
        let base = pack_root.join(format!("assets/{namespace}/polytone/colormaps/{path}"));
        let png = base.with_extension("png");
        let (width, height, rgba) = match crate::sprites::read_rgba(&png) {
            Ok(image) => image,
            Err(e) => {
                log::warn!("colormap {id}: {e}");
                return None;
            }
        };
        let json: Json =
            horizon_world::json_file::read(&base.with_extension("json")).unwrap_or(Json::Null);
        let compile = horizon_polytone::expr::Program::compile;
        let mut by_biome = false;
        let mut axis = |key: &str, default: &str| -> Option<Axis> {
            let src = match &json[key] {
                Json::String(s) => s.clone(),
                Json::Number(n) => n.to_string(),
                _ => default.to_owned(),
            };
            by_biome |= matches!(
                src.trim(),
                "biome_id" | "temperature" | "legacy_temperature" | "downfall"
            ) || matches!(src.trim(), "o.temperature" | "o.downfall");
            // The named samplers (wiki "Default Samplers") as the expressions they stand for.
            let src = match src.trim() {
                "biome_id" => {
                    let mapper = biome_id_mapper(pack_root, &json["biome_id_mapper"]);
                    return Some(Axis::BiomeId(mapper, compile("o.biome()").ok()?));
                }
                "temperature" | "legacy_temperature" => "o.temperature".to_owned(),
                "downfall" => "o.downfall".to_owned(),
                "y_level" => "o.y / 128".to_owned(),
                "day_time" => "1 - (g.dayTime % 24000) / 24000".to_owned(),
                _ => src,
            };
            match compile(&src).map(Axis::Expr) {
                Ok(program) => Some(program),
                Err(e) => {
                    log::warn!("colormap {id} {key}: {e}");
                    None
                }
            }
        };
        let (x, y) = (
            axis("x_axis", "o.temperature")?,
            axis("y_axis", "o.downfall")?,
        );
        let column = [&x, &y].iter().all(|axis| axis.column());
        Some(Self {
            x,
            y,
            column,
            biome_blend: json["biome_blend"].as_bool().unwrap_or(by_biome),
            triangular: json["triangular"].as_bool() == Some(true),
            rounds: json["rounds"].as_bool() != Some(false),
            width,
            height,
            rgba,
        })
    }

    /// The colour the axes pick, ARGB; `None` for a transparent pixel (the value stays what
    /// it was, the default `fallback_colormap: original`) or an axis that fails.
    pub(crate) fn sample(&self, host: &mut dyn horizon_polytone::expr::Host) -> Option<u32> {
        let x = self.x.value(host)?.clamp(0.0, 1.0);
        let mut y = self.y.value(host)?.clamp(0.0, 1.0);
        if self.triangular {
            y *= x;
        }
        let pixel = |axis: &Axis, v: f64, size: u32| {
            let last = size.saturating_sub(1);
            if let Axis::BiomeId(..) = axis {
                return ((v * f64::from(size)).floor() as u32).min(last);
            }
            let p = (1.0 - v) * f64::from(last);
            let p = if self.rounds { p.round() } else { p.floor() };
            (p as u32).min(last)
        };
        let (px, py) = (
            pixel(&self.x, x, self.width),
            pixel(&self.y, y, self.height),
        );
        let i = (py as usize * self.width as usize + px as usize) * 4;
        let [r, g, b, a] = self.rgba.get(i..i + 4)?.try_into().ok()?;
        (a != 0).then(|| u32::from(a) << 24 | u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b))
    }
}

/// The colormaps of the Polytone block and fluid modifiers of the packs merged at `pack_root`:
/// (a fluid modifier, its targets, the colormap's id). Inline colormaps aren't read.
pub(crate) fn polytone_colormap_modifiers(
    pack_root: &Path,
) -> Vec<(bool, horizon_polytone::Targets, String)> {
    let mut out = Vec::new();
    let namespaces = std::fs::read_dir(pack_root.join("assets"))
        .into_iter()
        .flatten()
        .flatten();
    for namespace in namespaces {
        for (kind, fluid) in [("block_modifiers", false), ("fluid_modifiers", true)] {
            let mut stack = vec![namespace.path().join("polytone").join(kind)];
            while let Some(dir) = stack.pop() {
                for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                        continue;
                    }
                    let Ok(json) = horizon_world::json_file::read::<Json>(&path) else {
                        continue;
                    };
                    let Some(id) = json.get("colormap").and_then(Json::as_str) else {
                        continue;
                    };
                    let id = if id.contains(':') {
                        id.to_owned()
                    } else {
                        format!("minecraft:{id}")
                    };
                    let relative = path
                        .strip_prefix(pack_root)
                        .unwrap_or(&path)
                        .to_string_lossy();
                    let implicit = horizon_polytone::implicit_id(&relative, kind);
                    match horizon_polytone::Targets::parse(json.get("targets"), implicit.as_deref())
                    {
                        Ok(targets) => out.push((fluid, targets, id)),
                        Err(e) => log::warn!("{relative}: {e}"),
                    }
                }
            }
        }
    }
    out
}

/// What a block colormap's axes read where it samples: the block (`o.x`, `o.biome()`, its
/// biome's climate) and `r.noise`. The world isn't reachable from the mesher, so the rest fails
/// and the block keeps its own tint.
pub(crate) struct BlockHost<'a> {
    at: [i32; 3],
    biome: &'a crate::biome::BiomeInfo,
}

impl<'a> BlockHost<'a> {
    pub(crate) fn new(at: [i32; 3], biome: &'a crate::biome::BiomeInfo) -> Self {
        Self { at, biome }
    }
}

/// The simplex noise `r.noise(x, y)` reads, the same for blocks and particles.
pub(crate) fn polytone_noise() -> &'static horizon_core::noise::SimplexNoise {
    static NOISE: std::sync::OnceLock<horizon_core::noise::SimplexNoise> =
        std::sync::OnceLock::new();
    NOISE.get_or_init(|| {
        horizon_core::noise::SimplexNoise::new(&mut horizon_core::random::LegacyRandom::new(0))
    })
}

impl horizon_polytone::expr::Host for BlockHost<'_> {
    fn root(&mut self, root: horizon_polytone::expr::Root) -> horizon_polytone::expr::Value {
        use horizon_polytone::expr::Root;
        horizon_polytone::expr::Value::Object(match root {
            Root::Object => 1,
            Root::Random => 2,
            _ => 0,
        })
    }

    fn method(
        &mut self,
        target: &horizon_polytone::expr::Value,
        name: &str,
        args: &[horizon_polytone::expr::Value],
    ) -> std::result::Result<horizon_polytone::expr::Value, String> {
        use horizon_polytone::expr::Value as V;
        let num = |n: f64| Ok(V::Num(n));
        match (target, name) {
            (V::Object(1), "x" | "y" | "z") => {
                num(f64::from(self.at[usize::from(name.as_bytes()[0] - b'x')]))
            }
            (V::Object(1), "biome") => Ok(V::Str(self.biome.name.as_str().into())),
            (V::Object(1), "temperature") => num(f64::from(self.biome.temperature)),
            (V::Object(1), "downfall") => num(f64::from(self.biome.downfall)),
            (V::Object(2), "noise") => {
                let mut coords = args.iter().filter(|v| !matches!(v, V::Str(_))).map(V::num);
                let (x, y) = (coords.next().unwrap_or(0.0), coords.next().unwrap_or(0.0));
                num(f64::from(polytone_noise().get2(x, y)))
            }
            _ => Err(format!("{name} is not available to block colormaps")),
        }
    }

    fn function(
        &mut self,
        name: &str,
        _: &[horizon_polytone::expr::Value],
    ) -> std::result::Result<horizon_polytone::expr::Value, String> {
        Err(format!("{name} is not available to block colormaps"))
    }
}

/// Turns Polytone layers into vanilla entries, compiling expressions against the frame.
struct Resolver {
    frame: Arc<RwLock<Frame>>,
    configs: Arc<horizon_polytone::pack::Configs>,
    /// Where the packs' colormaps are, and the ones loaded so far (`None`: missing).
    pack_root: Option<std::path::PathBuf>,
    colormaps: ColormapCache,
}

/// Colormaps by id, loaded once for all three dimensions' stacks.
type ColormapCache = Arc<std::sync::Mutex<HashMap<String, Option<Arc<Colormap>>>>>;

impl Resolver {
    /// A Polytone layer as a vanilla entry: `Some(None)` removes the attribute's entry, `None`
    /// is a value not supported yet (colormaps) or a broken one.
    fn entry(
        &self,
        attribute: &'static Attribute,
        layer: &horizon_polytone::Layer,
    ) -> Option<Option<Entry>> {
        use horizon_polytone::{Argument, Layer, Modifier};
        let (modifier, argument) = match layer {
            Layer::Remove => return Some(None),
            Layer::Apply {
                modifier, argument, ..
            } => (modifier, argument),
        };
        let op = match modifier {
            Modifier::Override => "override",
            Modifier::Add => "add",
            Modifier::Subtract => "subtract",
            Modifier::Multiply => "multiply",
            Modifier::Minimum => "minimum",
            Modifier::Maximum => "maximum",
            Modifier::AlphaBlend => "alpha_blend",
            Modifier::BlendToGray => "blend_to_gray",
            Modifier::And => "and",
            Modifier::Nand => "nand",
            Modifier::Or => "or",
            Modifier::Nor => "nor",
            Modifier::Xor => "xor",
            Modifier::Xnor => "xnor",
        };
        let json = match argument {
            Argument::Number(n) => Json::from(*n),
            Argument::Bool(b) => Json::Bool(*b),
            // Polytone colours may leave out the `#`; vanilla's codec wants it, 6 digits for RGB.
            Argument::Color(c) if attribute.kind == Kind::RgbColor => {
                Json::String(format!("#{:06x}", c & 0x00ff_ffff))
            }
            Argument::Color(c) => Json::String(format!("#{c:08x}")),
            Argument::Object(o) => Json::Object(o.clone()),
            Argument::Expression(src) => return self.expression(attribute, op, src).map(Some),
            Argument::Colormap(id) => return self.colormap(attribute, op, id).map(Some),
        };
        match Entry::with_modifier(attribute, op, &json) {
            Ok(entry) => Some(Some(entry)),
            Err(e) => {
                log::warn!("{}: {e}", attribute.id);
                None
            }
        }
    }

    /// An expression value: compiled once, evaluated against the frame each time the layer
    /// applies, its result read as the modifier's argument (a number, a boolean, or a packed
    /// colour for colour attributes).
    fn expression(
        &self,
        attribute: &'static Attribute,
        op: &'static str,
        src: &str,
    ) -> Option<Entry> {
        let program = match horizon_polytone::expr::Program::compile(src) {
            Ok(program) => program,
            Err(e) => {
                log::warn!("{}: {e}", attribute.id);
                return None;
            }
        };
        let (frame, configs) = (self.frame.clone(), self.configs.clone());
        let f = move |value: &Value| -> Option<Entry> {
            let frame = frame.read().ok()?;
            let mut host = FrameHost {
                frame: &frame,
                configs: &configs,
                value: as_number(value),
                seed: 0,
            };
            let json = match program.eval(&mut host).ok()? {
                horizon_polytone::expr::Value::Bool(b) => Json::Bool(b),
                result => Json::from(result.num()),
            };
            // A number out of the attribute's range is clamped into it, as the sampled value is
            // (`AttributeRange.sanitize`), not dropped: a pack's fog end that goes negative deep
            // underground is the thickest fog, not the default.
            Entry::with_modifier(attribute, op, &json).ok().or_else(|| {
                let n = json.as_f64()? as f32;
                let Value::Float(clamped) = attribute.sanitize(Value::Float(n)) else {
                    return None;
                };
                Entry::with_modifier(attribute, op, &Json::from(clamped)).ok()
            })
        };
        Some(Entry::Dynamic(
            horizon_world::environment_attributes::Dynamic(Arc::new(f)),
        ))
    }

    /// A colormap value: sampled whenever its layer applies, its colour the modifier's argument
    /// (`#rrggbb` on RGB attributes, `#aarrggbb` on ARGB ones).
    fn colormap(&self, attribute: &'static Attribute, op: &'static str, id: &str) -> Option<Entry> {
        let root = self.pack_root.as_deref()?;
        let colormap = self
            .colormaps
            .lock()
            .ok()?
            .entry(id.to_owned())
            .or_insert_with(|| Colormap::load(root, id).map(Arc::new))
            .clone()?;
        let (frame, configs) = (self.frame.clone(), self.configs.clone());
        let rgb = attribute.kind == Kind::RgbColor;
        let f = move |value: &Value| -> Option<Entry> {
            let frame = frame.read().ok()?;
            let mut host = FrameHost {
                frame: &frame,
                configs: &configs,
                value: as_number(value),
                seed: 0,
            };
            let color = colormap.sample(&mut host)?;
            let text = if rgb {
                format!("#{:06x}", color & 0x00ff_ffff)
            } else {
                format!("#{color:08x}")
            };
            Entry::with_modifier(attribute, op, &Json::String(text)).ok()
        };
        Some(Entry::Dynamic(
            horizon_world::environment_attributes::Dynamic(Arc::new(f)),
        ))
    }

    /// Applies a group of Polytone layers to `map`: entries set or replaced, `REMOVE` drops
    /// them.
    fn apply(&self, map: &mut AttributeMap, group: &horizon_polytone::Attributes) {
        for (id, layer) in group {
            let Some(attribute) = attribute(id) else {
                log::warn!("unknown attribute {id}");
                continue;
            };
            match self.entry(attribute, layer) {
                // A computed value that can't decide (a transparent colormap pixel, a failing
                // expression) falls back to the entry it replaces: the value before the pack.
                Some(Some(Entry::Dynamic(f))) => match map.get(attribute).cloned() {
                    Some(original) => {
                        let fallback = move |v: &Value| (f.0)(v).or_else(|| Some(original.clone()));
                        let entry =
                            horizon_world::environment_attributes::Dynamic(Arc::new(fallback));
                        map.set(attribute, Entry::Dynamic(entry));
                    }
                    None => map.set(attribute, Entry::Dynamic(f)),
                },
                Some(Some(entry)) => map.set(attribute, entry),
                Some(None) => map.remove(attribute),
                None => {}
            }
        }
    }
}

/// The timeline ids a dimension type's `timelines` names: an id, a `#tag` or a list,
/// tags resolved through `tags/timeline/` (their entries may be `{"id", "required"}`).
fn timeline_ids(root: &Path, json: &Json, out: &mut Vec<String>, depth: u32) {
    let entry = |e: &Json| {
        e.as_str()
            .or_else(|| e.get("id").and_then(Json::as_str))
            .map(str::to_owned)
    };
    let items: Vec<String> = match json {
        Json::Array(list) => list.iter().filter_map(entry).collect(),
        other => entry(other).into_iter().collect(),
    };
    for item in items {
        if let Some(tag) = item.strip_prefix('#') {
            let (namespace, path) = tag.split_once(':').unwrap_or(("minecraft", tag));
            let file = root.join(format!("data/{namespace}/tags/timeline/{path}.json"));
            if depth < 8
                && let Ok(tag) = horizon_world::json_file::read::<Json>(&file)
            {
                timeline_ids(root, &tag["values"], out, depth + 1);
            }
        } else if !out.contains(&item) {
            out.push(item);
        }
    }
}

/// Legacy (pre-attribute-system) dimension data, synthesized at vanilla 1.21.7
/// values. 1.21.7's dimension types and biomes carry no `attributes` — the
/// environment-attribute system arrived in 1.21.11 (Polytone documents its
/// `attributes_modifiers` as "1.21.11+ only"), and 1.21.7 still colours its
/// sky from the biome's legacy `effects`. Without synthesis every visual
/// attribute falls back to its default: a black sky and fog, a sun frozen at
/// 0°, no stars, and no clouds (the cloud colour's alpha gates them) — the
/// 2026-11-21 laptop playtest's black sky. The synthesis follows the same
/// pattern as the missing movement attributes (83d24e28): derive what the
/// data lacks from what it has, in code, at vanilla values.
///
/// The day-cycle curves are vanilla 1.21.7's, verified against its server
/// jar (`DimensionType.timeOfDay`, `Level.updateSkyBrightness`) and the
/// 1.21.6 client jar (`ClientLevel.getSkyColor`). `time_of_day` is
/// `DimensionType.timeOfDay`: 0 at noon, 0.5 at midnight — vanilla's
/// non-linear blend, not the linear day fraction. The sky and fog
/// darken by `ClientLevel.getSkyColor`'s factor, the stars by
/// `Level.getStarBrightness`, the sky light by `15 - Level.getSkyDarken`
/// (kept as a float; `sky_darken` truncates it to vanilla's int steps).
/// `sun_angle` is linear in time, 0° at noon — the engine's `SUN_ANGLE`
/// convention, probe-verified against the 26.x day timeline — not the
/// wiki's non-linear visual angle. The sunrise/sunset colour is NOT
/// synthesized: its exact vanilla constants couldn't be verified, and a
/// wrong glow would look like a bug; the attribute stays transparent (no
/// dawn/dusk fan) until the 26.x timeline values are available.
mod legacy {
    use super::*;

    /// `DimensionType.timeOfDay` at `ticks` (0-23999): 0 at noon, 0.5 at
    /// midnight. Vanilla blends the linear fraction `d3` with
    /// `d5 = 0.5 - cos(d3*PI)/2` and returns `(2*d3 + d5)/3` (1.21.7
    /// server jar) — the linear form kept dawn dark until tick ~841 and
    /// brought dusk early from tick ~11160, so undead did not burn at
    /// dawn, where vanilla's `skyDarken` is already 0 from tick 0.
    fn time_of_day(ticks: i32) -> f32 {
        let d3 = (ticks as f64 / 24_000.0 - 0.25).rem_euclid(1.0);
        let d5 = 0.5 - (d3 * std::f64::consts::PI).cos() / 2.0;
        ((d3 * 2.0 + d5) / 3.0) as f32
    }

    /// `Mth.cos(timeOfDay * 2π)`, the shared input of the daylight curves.
    fn cos_day(ticks: i32) -> f32 {
        (time_of_day(ticks) * 2.0 * std::f32::consts::PI).cos()
    }

    /// `ClientLevel.getSkyColor`'s daylight factor: 1 at noon, 0 at night.
    fn daylight(ticks: i32) -> f32 {
        (cos_day(ticks) * 2.0 + 0.5).clamp(0.0, 1.0)
    }

    /// `Level.getStarBrightness`: 0 by day, 0.5 at midnight.
    fn star_brightness(ticks: i32) -> f32 {
        let f = (1.0 - (cos_day(ticks) * 2.0 + 0.25)).clamp(0.0, 1.0);
        f * f * 0.5
    }

    /// `15 - Level.getSkyDarken` at clear weather (the timeline models
    /// clear; the weather layer darkens it separately): vanilla's
    /// `Level.updateSkyBrightness` is
    /// `(int)((1 - (0.5 + 2*clamp(cos(timeOfDay*2*PI), -0.25, 0.25))) * 11)`
    /// (1.21.7 server jar), so the light is already full at dawn — the
    /// old `(1 - (cos*2 + 0.2))` form kept it dark until tick ~841.
    /// Computed in f64 like vanilla's `Math.cos` path.
    fn sky_light_level(ticks: i32) -> f32 {
        let c = (time_of_day(ticks) as f64 * 2.0 * std::f64::consts::PI)
            .cos()
            .clamp(-0.25, 0.25);
        (15.0 - (1.0 - (0.5 + 2.0 * c)) * 11.0) as f32
    }

    /// A biome's legacy `effects` colours as an attribute map, or `None`
    /// when the biome carries neither attributes nor colours.
    pub fn biome_map(biome: &Json) -> Option<AttributeMap> {
        let effects = biome.get("effects")?.as_object()?;
        let mut entries = serde_json::Map::new();
        for (legacy, attribute) in [
            ("sky_color", "visual/sky_color"),
            ("fog_color", "visual/fog_color"),
            ("water_fog_color", "visual/water_fog_color"),
        ] {
            if let Some(value) = effects.get(legacy) {
                entries.insert(attribute.to_owned(), value.clone());
            }
        }
        if entries.is_empty() {
            return None;
        }
        AttributeMap::parse(&Json::Object(entries)).ok()
    }

    /// A dimension's constant attributes, from its legacy fields. The
    /// overworld takes the plains sky and fog as its dimension defaults (the
    /// biome layer overrides them where the player stands) and its own
    /// `cloud_height`; the Nether and the End take their fixed-time values
    /// (Nether midnight: sky light 4; End noon: sky light 15, the default)
    /// and their primary biome's colours. The End's stars are always out at
    /// vanilla's maximum brightness — the End sky is black with stars.
    pub fn dimension_map(dimension: &str, json: &Json) -> AttributeMap {
        let mut entries = serde_json::Map::new();
        let mut put = |id: &str, value: Json| {
            entries.insert(id.to_owned(), value);
        };
        match dimension {
            "overworld" | "overworld_caves" => {
                // Plains, the vanilla overworld default sky and fog.
                put("visual/sky_color", Json::from(7_907_327));
                put("visual/fog_color", Json::from(12_638_463));
                if let Some(height) = json.get("cloud_height") {
                    put("visual/cloud_height", height.clone());
                }
            }
            "the_nether" => {
                // Nether wastes sky and fog; fixed time 18000 (midnight).
                put("visual/sky_color", Json::from(7_254_527));
                put("visual/fog_color", Json::from(3_344_392));
                put("gameplay/sky_light_level", Json::from(4.0));
                put("visual/sky_light_factor", Json::from(4.0 / 15.0));
                put("visual/sun_angle", Json::from(180.0));
                put("visual/star_angle", Json::from(180.0));
            }
            "the_end" => {
                // The End's fog; its sky is black (the default) with stars.
                put("visual/fog_color", Json::from(10_518_688));
                put("visual/star_brightness", Json::from(0.5));
                put("visual/moon_angle", Json::from(180.0));
            }
            _ => {}
        }
        AttributeMap::parse(&Json::Object(entries)).unwrap_or_default()
    }

    /// The overworld's day timeline: the classic day cycle as keyframe
    /// tracks, one keyframe per 1000 ticks (linear interpolation over a
    /// 15° arc is within 0.3% of the cosine curves). The angles are exact
    /// with two keyframes each. Sky and fog colours multiply the biome's
    /// colours by the daylight factor (vanilla darkens both at night); the
    /// sky light factor is the sky light level as a fraction of 15, and
    /// the sky light colour runs from white to the engine's
    /// `NIGHT_SKY_LIGHT_COLOR` (0.48, 0.48, 1.0) as the day fades.
    pub fn day_timeline() -> Option<Timeline> {
        let mut tracks = serde_json::Map::new();
        let keyframes = |values: &[(i32, Json)]| {
            Json::Object(serde_json::Map::from_iter([(
                "keyframes".to_owned(),
                Json::Array(
                    values
                        .iter()
                        .map(|(ticks, value)| serde_json::json!({"ticks": ticks, "value": value}))
                        .collect(),
                ),
            )]))
        };
        let every_1000 =
            |f: &dyn Fn(i32) -> Json| (0..24).map(|h| (h * 1000, f(h * 1000))).collect::<Vec<_>>();
        let float_track = |f: &dyn Fn(i32) -> f32| keyframes(&every_1000(&|t| Json::from(f(t))));
        let angle = |at_zero: f32| {
            keyframes(&[
                (0, Json::from(at_zero)),
                (24_000, Json::from(at_zero + 360.0)),
            ])
        };
        tracks.insert("visual/sun_angle".into(), angle(-90.0));
        tracks.insert("visual/moon_angle".into(), angle(90.0));
        tracks.insert("visual/star_angle".into(), angle(-90.0));
        tracks.insert(
            "visual/star_brightness".into(),
            float_track(&star_brightness),
        );
        tracks.insert(
            "gameplay/sky_light_level".into(),
            float_track(&sky_light_level),
        );
        let mut multiply = |id: &str, f: &dyn Fn(i32) -> Json| {
            let mut track = keyframes(&every_1000(f));
            track
                .as_object_mut()
                .unwrap()
                .insert("modifier".into(), Json::from("multiply"));
            tracks.insert(id.into(), track);
        };
        multiply("visual/sky_color", &|t| {
            let d = daylight(t);
            serde_json::json!([d, d, d])
        });
        multiply("visual/fog_color", &|t| {
            let d = daylight(t);
            serde_json::json!([d, d, d])
        });
        tracks.insert(
            "visual/sky_light_factor".into(),
            float_track(&|t| sky_light_level(t) / 15.0),
        );
        tracks.insert(
            "visual/sky_light_color".into(),
            keyframes(&every_1000(&|t| {
                let night = 1.0 - daylight(t);
                let c = 1.0 - 0.52 * night;
                serde_json::json!([c, c, 1.0])
            })),
        );
        tracks.insert(
            "visual/cloud_color".into(),
            keyframes(&every_1000(&|t| {
                let d = daylight(t);
                serde_json::json!([d, d, d, 1.0])
            })),
        );
        let json = serde_json::json!({
            "clock": "minecraft:overworld",
            "period_ticks": 24000,
            "tracks": tracks,
        });
        match Timeline::parse(&json) {
            Ok(timeline) => Some(timeline),
            Err(e) => {
                log::warn!("synthesized day timeline: {e}");
                None
            }
        }
    }
}

impl Environment {
    pub fn load(root: &Path) -> Result<Self> {
        Self::load_dimension(root, "overworld")
    }

    /// `EnvironmentAttributeSystem.addDefaultLayers` for a dimension type: its attributes, a
    /// biome layer for every attribute some biome sets, its timelines (in tag order), and the
    /// weather where `Level.canHaveWeather` (sky light, no ceiling, not the End).
    pub fn load_dimension(root: &Path, dimension: &str) -> Result<Self> {
        Self::load_dimension_with(root, dimension, &PackModifiers::default())
    }

    /// The same with resource packs' Polytone modifiers (wiki "Environment Attributes"): a
    /// dimension modifier's `base` changes the dimension's own entries, `rain` and `thunder`
    /// the weather's, `post_process` adds a layer after all others; a biome modifier's `base`
    /// changes the biome's entries, `post_process` adds a biome layer after the weather.
    /// Modifiers apply lowest priority first, so the highest wins.
    pub fn load_dimension_with(
        root: &Path,
        dimension: &str,
        packs: &PackModifiers,
    ) -> Result<Self> {
        let json: Json = horizon_world::json_file::read(
            &root.join(format!("data/minecraft/dimension_type/{dimension}.json")),
        )?;
        let id = format!("minecraft:{dimension}");
        let tags = crate::pack_tags::Tags::new(root);
        let mine: Vec<_> = packs
            .dimensions
            .iter()
            .filter(|m| {
                m.targets
                    .matches(&id, |tag| tags.contains("dimension_type", tag, &id))
            })
            .collect();
        let frame = Arc::new(RwLock::new(Frame::default()));
        let resolver = Resolver {
            frame: frame.clone(),
            configs: Arc::new(packs.configs.clone()),
            pack_root: packs.root.clone(),
            colormaps: packs.colormaps.clone(),
        };
        // 1.21.7 dimension types carry no "attributes" (the
        // environment-attributes system is a 26.x addition); a missing or
        // malformed map is synthesized from the legacy fields instead of
        // leaving every attribute at its default (a black sky).
        let mut dimension_map = match AttributeMap::parse(&json["attributes"]) {
            Ok(map) => map,
            Err(e) => {
                log::warn!("dimension {dimension}: {e}");
                AttributeMap::default()
            }
        };
        // Legacy data (no attributes at all) is what the day timeline
        // synthesis keys on below: a dimension with attributes but no
        // timelines — a pack fixture, or a 26.x dimension with a constant
        // sky — must not gain a day cycle it never had.
        let legacy_dimension = dimension_map.is_empty();
        if legacy_dimension {
            dimension_map = legacy::dimension_map(dimension, &json);
        }
        let (mut rain, mut thunder) = horizon_world::environment_system::weather_maps();
        let mut dimension_post = AttributeMap::default();
        for m in &mine {
            resolver.apply(&mut dimension_map, &m.base);
            resolver.apply(&mut rain, &m.rain);
            resolver.apply(&mut thunder, &m.thunder);
            resolver.apply(&mut dimension_post, &m.post_process);
        }
        let mut builder = Builder::default();
        builder.constant_layer(&dimension_map);
        // Without biome data (a bare dimension type), there is no biome layer.
        let mut biomes = HashMap::new();
        let mut post = HashMap::new();
        if let Ok(folder) =
            horizon_world::json_folder::read(&root.join("data/minecraft/worldgen/biome"))
        {
            for file in folder.files.iter().filter(|f| !f.name.contains('/')) {
                let biome: Json = horizon_world::json_file::parse(&file.path, file.bytes()?)?;
                let name = format!("minecraft:{}", file.name);
                // 1.21.7 biomes carry no "attributes" either; their legacy
                // `effects` colours synthesize the map, so the biome layer
                // survives (dropping every biome left the sky black). Only
                // a biome with neither attributes nor colours is dropped.
                let mut map = match AttributeMap::parse(&biome["attributes"]) {
                    Ok(map) => map,
                    Err(e) => match legacy::biome_map(&biome) {
                        Some(map) => map,
                        None => {
                            log::warn!("biome {}: {e}", file.name);
                            continue;
                        }
                    },
                };
                let mut post_map = AttributeMap::default();
                for m in &packs.biomes {
                    let in_tag = |tag: &str| tags.contains("worldgen/biome", tag, &name);
                    if m.targets.matches(&name, in_tag) {
                        resolver.apply(&mut map, &m.base);
                        resolver.apply(&mut post_map, &m.post_process);
                    }
                }
                if !post_map.is_empty() {
                    post.insert(name.clone(), post_map);
                }
                biomes.insert(name, map);
            }
        }
        builder.biome_layer(biomes.values());
        let mut ids = Vec::new();
        timeline_ids(root, &json["timelines"], &mut ids, 0);
        let no_timelines = ids.is_empty();
        for id in ids {
            let (namespace, path) = id.split_once(':').unwrap_or(("minecraft", &id));
            let file = root.join(format!("data/{namespace}/timeline/{path}.json"));
            match horizon_world::json_file::read::<Json>(&file)
                .map_err(|e| e.to_string())
                .and_then(|t| Timeline::parse(&t))
            {
                Ok(timeline) => {
                    builder.timeline_layer(&timeline);
                }
                Err(e) => log::warn!("timeline {id}: {e}"),
            }
        }
        // Legacy data lists no timelines at all: the overworld's day cycle
        // is synthesized (the Nether and the End have fixed times, carried
        // by their dimension maps above). Only for a legacy dimension, and
        // only when it lists none — a listed-but-unreadable timeline
        // already warned above.
        if no_timelines && legacy_dimension && matches!(dimension, "overworld" | "overworld_caves")
        {
            if let Some(timeline) = legacy::day_timeline() {
                builder.timeline_layer(&timeline);
            }
        }
        let weather = json["has_skylight"].as_bool() == Some(true)
            && json["has_ceiling"].as_bool() != Some(true)
            && dimension != "the_end";
        if weather {
            builder.weather_layers_with(&rain, &thunder);
        }
        builder.biome_post_layer(post.values());
        builder.constant_layer(&dimension_post);
        Ok(Self {
            system: builder.build(),
            frame,
            biomes,
            post,
        })
    }

    /// Sets what the packs' expressions read for the next samples.
    pub fn set_frame(&self, frame: Frame) {
        if let Ok(mut f) = self.frame.write() {
            *f = frame;
        }
    }

    /// A biome's attribute map (by id, `minecraft:plains`), as the packs left it.
    pub fn biome(&self, id: &str) -> Option<&AttributeMap> {
        self.biomes.get(id)
    }

    /// A biome's post-process map from the packs, if any sets one.
    pub fn biome_post(&self, id: &str) -> Option<&AttributeMap> {
        self.post.get(id)
    }

    /// Whether packs added post-process biome layers (the frame then mixes them too).
    pub fn has_biome_post(&self) -> bool {
        !self.post.is_empty()
    }

    /// `EnvironmentAttributeSystem.getValue` of one attribute, or its dimension value without
    /// a biome.
    pub fn value(&self, id: &str, at: &Sample) -> Option<Value> {
        let attribute = attribute(id)?;
        let biome = |_full_resolution: bool| at.biome;
        let clock = |_clock: &str| at.day_time;
        let post_biome = |_full_resolution: bool| at.post_biome;
        let conditions = Conditions {
            rain: at.rain,
            thunder: at.thunder,
            blend: at.blend,
            biome: &biome,
            clock: &clock,
            post_blend: at.post_blend,
            post_biome: Some(&post_biome),
        };
        match (at.blend, at.biome) {
            (None, None) => self.system.dimension_value(attribute, &conditions),
            _ => self.system.value(attribute, &conditions),
        }
    }

    /// The sky state the renderer draws.
    pub fn sample(&self, at: &Sample) -> Sky {
        let float = |id: &str| match self.value(id, at) {
            Some(Value::Float(f)) => f,
            _ => 0.0,
        };
        let rgb = |id: &str| match self.value(id, at) {
            Some(Value::Rgb(c)) => c,
            _ => [0.0; 3],
        };
        let argb = |id: &str| match self.value(id, at) {
            Some(Value::Argb(c)) => c,
            _ => [0.0; 4],
        };
        Sky {
            sky_color: rgb("visual/sky_color"),
            fog_color: rgb("visual/fog_color"),
            sky_light_factor: float("visual/sky_light_factor"),
            sky_light_color: rgb("visual/sky_light_color"),
            block_light_tint: rgb("visual/block_light_tint"),
            ambient_light_color: rgb("visual/ambient_light_color"),
            sun_angle: float("visual/sun_angle"),
            moon_angle: float("visual/moon_angle"),
            star_angle: float("visual/star_angle"),
            star_brightness: float("visual/star_brightness"),
            sunrise_color: argb("visual/sunrise_sunset_color"),
            cloud_color: argb("visual/cloud_color"),
            sky_light_level: float("gameplay/sky_light_level") / 15.0,
            fog_start: float("visual/fog_start_distance"),
            fog_end: float("visual/fog_end_distance"),
            cloud_fog_end: float("visual/cloud_fog_end_distance"),
            cloud_height: float("visual/cloud_height"),
        }
    }
}

/// `Level.updateSkyBrightness`'s darkening: from the sky light level, a dimension value.
pub fn sky_darken(environment: &Environment, day_time: i64, rain: f32, thunder: f32) -> i32 {
    let sky = environment.sample(&Sample {
        day_time,
        rain,
        thunder,
        ..Sample::default()
    });
    // Level.updateSkyBrightness: `(int)(15.0F - skyLightLevel)`, the float
    // truncated, not rounded (#11323): the darkening steps up only once the
    // light has fallen a whole level. `sky_light_level` is the attribute as a
    // fraction of 15, so the product is its raw value in `f32`.
    (15.0 - 15.0 * sky.sky_light_level) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overworld() -> Option<Environment> {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.local/horizon-client/26.3");
        root.join("data/minecraft/timeline/day.json")
            .exists()
            .then(|| Environment::load(&root).unwrap())
    }

    #[test]
    fn dark_outside_follows_time_and_weather_levels() {
        let Some(env) = overworld() else {
            return;
        };
        let dark = |time, rain, thunder| sky_darken(&env, time, rain, thunder) >= 4;
        assert!(!dark(6000, 0.0, 0.0), "clear noon");
        assert!(!dark(6000, 1.0, 0.0), "rainy noon");
        assert!(dark(6000, 1.0, 1.0), "thunderstorm at noon");
        assert!(dark(18000, 0.0, 0.0), "clear midnight");
        assert!(!dark(1000, 0.0, 0.0), "clear morning");
    }

    /// A world with one sky-lit dimension and two biomes, one of them in a tag, and a pack with
    /// Polytone modifiers.
    fn with_packs(pack: &[(&str, &str)]) -> Environment {
        let files: Vec<(&str, &[u8])> = pack.iter().map(|(p, t)| (*p, t.as_bytes())).collect();
        with_pack_files(&files)
    }

    fn with_pack_files(pack: &[(&str, &[u8])]) -> Environment {
        static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "horizon-env-packs-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let write = |path: &str, text: &[u8]| {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        };
        write(
            "world/data/minecraft/dimension_type/overworld.json",
            r##"{"has_skylight": true, "has_ceiling": false,
                "attributes": {"minecraft:visual/sky_color": "#808080", "minecraft:visual/fog_color": "#000000"}}"##.as_bytes(),
        );
        write(
            "world/data/minecraft/worldgen/biome/plains.json",
            r##"{"attributes": {"minecraft:visual/sky_color": "#404040"}}"##.as_bytes(),
        );
        write(
            "world/data/minecraft/worldgen/biome/desert.json",
            r#"{"attributes": {}}"#.as_bytes(),
        );
        write(
            "world/data/minecraft/tags/worldgen/biome/dry.json",
            r#"{"values": ["minecraft:desert"]}"#.as_bytes(),
        );
        for (path, text) in pack {
            write(&format!("pack/{path}"), text);
        }
        let modifiers = PackModifiers::read(&dir.join("pack")).with_configs(
            crate::packs::polytone_configs(&crate::packs::Source::Dir(dir.join("pack"))),
        );
        let env =
            Environment::load_dimension_with(&dir.join("world"), "overworld", &modifiers).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        env
    }

    fn sky_at(env: &Environment, biome: &str, rain: f32) -> Sky {
        let map = env.biome(biome).unwrap();
        env.sample(&Sample {
            rain,
            biome: Some(map),
            post_biome: env.biome_post(biome),
            ..Sample::default()
        })
    }

    const GRAY_40: f32 = 64.0 / 255.0;

    #[test]
    fn pack_biome_modifiers_change_their_targets_only() {
        let env = with_packs(&[(
            "assets/mypack/polytone/biome_modifiers/dry_fog.json",
            r##"{"targets": "#minecraft:dry", "attributes_modifiers": {"visual/fog_color": "ff0000"}}"##,
        )]);
        assert_eq!(
            sky_at(&env, "minecraft:desert", 0.0).fog_color,
            [1.0, 0.0, 0.0]
        );
        assert_eq!(sky_at(&env, "minecraft:plains", 0.0).fog_color, [0.0; 3]);
        // Without packs the stack is vanilla's: the biome's own sky.
        let plain = with_packs(&[]);
        assert_eq!(
            sky_at(&plain, "minecraft:plains", 0.0).sky_color,
            [GRAY_40; 3]
        );
    }

    #[test]
    fn dimension_modifiers_remove_weather_layers_and_post_process_last() {
        let raining = |env: &Environment| sky_at(env, "minecraft:plains", 1.0).sky_color;
        // Vanilla's rain grays the sky toward a darker gray.
        assert_ne!(raining(&with_packs(&[])), [GRAY_40; 3]);
        let no_rain = with_packs(&[(
            "assets/minecraft/polytone/dimension_modifiers/overworld.json",
            r#"{"attributes_modifiers": {"rain": {"visual/sky_color": "REMOVE"}}}"#,
        )]);
        assert_eq!(raining(&no_rain), [GRAY_40; 3]);
        let post = with_packs(&[(
            "assets/minecraft/polytone/dimension_modifiers/overworld.json",
            r##"{"attributes_modifiers": {"post_process": {"visual/sky_color": "#0000ff"}}}"##,
        )]);
        assert_eq!(raining(&post), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn biome_post_process_and_priority() {
        let env = with_packs(&[
            (
                "assets/a/polytone/biome_modifiers/wins.json",
                r##"{"targets": "minecraft:plains", "priority": 5,
                    "attributes_modifiers": {"base": {"visual/fog_color": "#00ff00"},
                                             "post_process": {"visual/sky_color": "#ffffff"}}}"##,
            ),
            (
                "assets/b/polytone/biome_modifiers/loses.json",
                r##"{"targets": "minecraft:plains", "priority": -1,
                    "attributes_modifiers": {"visual/fog_color": "#0000ff"}}"##,
            ),
        ]);
        // The higher priority's fog wins.
        assert_eq!(
            sky_at(&env, "minecraft:plains", 0.0).fog_color,
            [0.0, 1.0, 0.0]
        );
        // The post-process sky comes after the weather, so the rain doesn't gray it.
        assert_eq!(
            sky_at(&env, "minecraft:plains", 1.0).sky_color,
            [1.0, 1.0, 1.0]
        );
        assert!(env.biome_post("minecraft:desert").is_none());
    }

    /// A number an expression gives outside the attribute's range is clamped into it, as vanilla
    /// clamps a sampled value: a fog end that a pack takes below zero deep underground is fog
    /// at once, not the default end.
    #[test]
    fn an_expression_value_out_of_range_is_clamped() {
        let env = with_packs(&[(
            "assets/minecraft/polytone/dimension_modifiers/overworld.json",
            r#"{"attributes_modifiers": {"visual/fog_end_distance": "64 - (p.y < 0) * 80"}}"#,
        )]);
        let end_at = |y: f64| {
            env.set_frame(Frame {
                player: [0.5, y, 0.5],
                ..Frame::default()
            });
            sky_at(&env, "minecraft:plains", 0.0).fog_end
        };
        assert_eq!(end_at(70.0), 64.0);
        assert_eq!(end_at(-2.0), 0.0);
    }

    #[test]
    fn expression_values_read_the_frame_and_the_value_so_far() {
        let env = with_packs(&[
            (
                "assets/minecraft/polytone/dimension_modifiers/overworld.json",
                r#"{"attributes_modifiers": {
                    "visual/fog_end_distance": "14 * (c.viewDistance / 16) + min(g.rain * 2, 1) * 100",
                    "visual/fog_start_distance": {"argument": "v + config('mypack:shift')", "modifier": "override"},
                    "visual/cloud_height": "v * 0.5"}}"#,
            ),
            (
                "assets/mypack/polytone/config_entries/shift.json",
                r#"{"default_value": 7}"#,
            ),
        ]);
        let at = |rain: f32, thunder: f32, chunks: i32| {
            env.set_frame(Frame {
                rain,
                thunder,
                render_distance: chunks,
                ..Frame::default()
            });
            sky_at(&env, "minecraft:plains", 0.0)
        };
        // `c.viewDistance` is in blocks: 16 chunks are 256.
        assert_eq!(at(0.0, 0.0, 16).fog_end, 224.0);
        // `g.rain` is the mean of rain and thunder: half rain is 0.25 (448 + 50), plain rain
        // 0.5 and a thunderstorm 1, both past the `min` (224 + 100).
        assert_eq!(at(0.5, 0.0, 32).fog_end, 498.0);
        assert_eq!(at(1.0, 0.0, 16).fog_end, 324.0);
        assert_eq!(at(1.0, 1.0, 16).fog_end, 324.0);
        // `v` is the value the layers before gave: the default start 0, and 192.33 for clouds.
        assert_eq!(at(0.0, 0.0, 16).fog_start, 7.0);
        assert_eq!(at(0.0, 0.0, 16).cloud_height, 192.33 * 0.5);
    }

    /// A PNG of RGBA pixels in one row.
    fn png_row(pixels: &[[u8; 4]]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, pixels.len() as u32, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&pixels.concat()).unwrap();
        writer.finish().unwrap();
        out
    }

    #[test]
    fn colormap_values_read_the_pixel_their_axes_pick() {
        // Left to right: red, green, transparent, blue; x 1 is the left edge.
        let png = png_row(&[
            [255, 0, 0, 255],
            [0, 255, 0, 255],
            [9, 9, 9, 0],
            [0, 0, 255, 255],
        ]);
        let env = with_pack_files(&[
            (
                "assets/minecraft/polytone/dimension_modifiers/overworld.json",
                br#"{"attributes_modifiers": {"visual/sky_color": "mypack:sky"}}"#,
            ),
            (
                "assets/mypack/polytone/colormaps/sky.json",
                br#"{"x_axis": "1 - g.rain", "y_axis": 0}"#,
            ),
            ("assets/mypack/polytone/colormaps/sky.png", &png),
        ]);
        let sky = |rain: f32, thunder: f32| {
            env.set_frame(Frame {
                rain,
                thunder,
                ..Frame::default()
            });
            env.sample(&Sample::default()).sky_color
        };
        // `g.rain` 0, 0.3 (rain 0.6), 1 (a thunderstorm).
        assert_eq!(sky(0.0, 0.0), [1.0, 0.0, 0.0]);
        assert_eq!(sky(0.6, 0.0), [0.0, 1.0, 0.0]);
        assert_eq!(sky(1.0, 1.0), [0.0, 0.0, 1.0]);
        // A transparent pixel keeps the colour the layers before gave (the dimension's #808080):
        // `g.rain` 0.7.
        assert_eq!(sky(1.0, 0.4), [128.0 / 255.0; 3]);
    }

    #[test]
    fn biome_id_colormaps_read_the_column_their_mapper_gives_the_biome() {
        // Left to right: red, green, transparent, blue.
        let png = png_row(&[
            [255, 0, 0, 255],
            [0, 255, 0, 255],
            [9, 9, 9, 0],
            [0, 0, 255, 255],
        ]);
        let env = with_pack_files(&[
            (
                "assets/minecraft/polytone/dimension_modifiers/overworld.json",
                br#"{"attributes_modifiers": {"visual/sky_color": "mypack:sky"}}"#,
            ),
            (
                "assets/mypack/polytone/colormaps/sky.json",
                br#"{"x_axis": "biome_id", "y_axis": 0, "biome_id_mapper": "mypack:biomes"}"#,
            ),
            ("assets/mypack/polytone/colormaps/sky.png", &png),
            (
                // Pixel columns from the left, over a texture_size of the image's width.
                "assets/mypack/polytone/biome_id_mappers/biomes.json",
                br#"{"minecraft:desert": 3, "swamp": 1, "minecraft:plains": 2, "texture_size": 4}"#,
            ),
        ]);
        let sky = |biome: &str| {
            env.set_frame(Frame {
                biome: biome.to_owned(),
                ..Frame::default()
            });
            env.sample(&Sample::default()).sky_color
        };
        assert_eq!(sky("minecraft:desert"), [0.0, 0.0, 1.0]);
        assert_eq!(sky("minecraft:swamp"), [0.0, 1.0, 0.0]);
        // A biome the mapper doesn't name is 0: the first column.
        assert_eq!(sky("minecraft:forest"), [1.0, 0.0, 0.0]);
        // Plains land on the transparent pixel: the colour before stays.
        assert_eq!(sky("minecraft:plains"), [128.0 / 255.0; 3]);
    }

    /// A legacy (1.21.7-shaped) fixture: dimension types and biomes with no
    /// `attributes`, biomes coloured by their legacy `effects`.
    fn legacy_root() -> std::path::PathBuf {
        static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "horizon-env-legacy-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let write = |path: &str, text: &str| {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        };
        write(
            "data/minecraft/dimension_type/overworld.json",
            r#"{"has_skylight": true, "has_ceiling": false, "cloud_height": 192}"#,
        );
        write(
            "data/minecraft/dimension_type/the_nether.json",
            r#"{"has_skylight": false, "has_ceiling": true, "fixed_time": 18000}"#,
        );
        write(
            "data/minecraft/dimension_type/the_end.json",
            r#"{"has_skylight": false, "has_ceiling": false, "fixed_time": 6000}"#,
        );
        write(
            "data/minecraft/worldgen/biome/plains.json",
            r#"{"effects": {"sky_color": 7907327, "fog_color": 12638463, "water_color": 4159204, "water_fog_color": 329011}}"#,
        );
        write(
            "data/minecraft/worldgen/biome/nether_wastes.json",
            r#"{"effects": {"sky_color": 7254527, "fog_color": 3344392, "water_color": 4159204, "water_fog_color": 329011}}"#,
        );
        write(
            "data/minecraft/worldgen/biome/the_end.json",
            r#"{"effects": {"sky_color": 0, "fog_color": 10518688, "water_color": 4159204, "water_fog_color": 329011}}"#,
        );
        dir
    }

    fn legacy_sample(env: &Environment, biome: &str, day_time: i64) -> Sky {
        let map = env.biome(biome).unwrap();
        env.sample(&Sample {
            day_time,
            biome: Some(map),
            ..Sample::default()
        })
    }

    /// The 2026-11-21 laptop playtest's black sky: a legacy overworld's
    /// plains sky is its vanilla colour by day and black at night, with the
    /// sun, stars and sky light following the day cycle.
    #[test]
    fn legacy_overworld_sky_follows_the_day() {
        let root = legacy_root();
        let env = Environment::load_dimension(&root, "overworld").unwrap();
        // Morning (the playtest world's Time=2002): full plains sky.
        let morning = legacy_sample(&env, "minecraft:plains", 2002);
        assert_eq!(
            morning.sky_color,
            [120.0 / 255.0, 167.0 / 255.0, 1.0],
            "plains 0x78A7FF, not black"
        );
        assert_eq!(
            morning.fog_color,
            [192.0 / 255.0, 216.0 / 255.0, 1.0],
            "plains fog 0xC0D8FF"
        );
        assert!((morning.sun_angle - -59.97).abs() < 0.01, "morning sun");
        assert!((morning.moon_angle - 120.03).abs() < 0.01, "moon opposite");
        assert_eq!(morning.star_brightness, 0.0, "no stars by day");
        assert_eq!(morning.sky_light_level, 1.0, "full sky light by day");
        assert_eq!(morning.cloud_color, [1.0, 1.0, 1.0, 1.0], "white clouds");
        assert_eq!(morning.cloud_height, 192.0, "the dimension's own height");
        // Noon: the sun overhead, still full light.
        let noon = legacy_sample(&env, "minecraft:plains", 6000);
        assert_eq!(noon.sun_angle, 0.0, "0 at noon");
        assert_eq!(noon.sky_light_level, 1.0);
        // Midnight: a black sky with stars at vanilla's maximum, sky light 4.
        let night = legacy_sample(&env, "minecraft:plains", 18000);
        assert_eq!(night.sky_color, [0.0; 3], "black at night");
        assert_eq!(night.fog_color, [0.0; 3], "fog darkens too");
        assert_eq!(night.star_brightness, 0.5, "vanilla's maximum");
        assert_eq!(night.sky_light_level, 4.0 / 15.0, "sky light 4 at night");
        assert_eq!(night.sun_angle, 180.0, "sun below at midnight");
        assert_eq!(night.sky_light_factor, 4.0 / 15.0);
        std::fs::remove_dir_all(&root).unwrap();
    }

    /// The gameplay path sees the synthesized day timeline too: `sky_darken`
    /// samples without a biome (the dimension value), as the mob logic does,
    /// so a timeline visible only to the renderer's biome samples would
    /// leave every day dark for gameplay — and undead would never burn.
    #[test]
    fn legacy_day_timeline_feeds_sky_darken() {
        let root = legacy_root();
        let env = Environment::load_dimension(&root, "overworld").unwrap();
        assert_eq!(sky_darken(&env, 6000, 0.0, 0.0), 0, "clear noon");
        assert!(sky_darken(&env, 1000, 0.0, 0.0) < 4, "clear morning");
        assert!(sky_darken(&env, 18000, 0.0, 0.0) >= 4, "clear midnight");
        assert!(
            sky_darken(&env, 6000, 1.0, 1.0) >= 4,
            "thunderstorm at noon"
        );
        // Vanilla's `isBrightOutside` (skyDarken < 4) is already true at
        // dawn: a new world starts at tick 0 with undead burning from the
        // first tick. The old linear day curve kept dawn dark until tick
        // ~841 and brought dusk early from tick ~11160 (follow-up burn
        // report, Nov 22 2026). Vanilla's night boundary is
        // 12541..23459; the 1000-tick keyframes shift it by at most ~7
        // ticks, so the assertions sit well inside each side.
        assert!(sky_darken(&env, 0, 0.0, 0.0) < 4, "clear dawn");
        assert!(sky_darken(&env, 500, 0.0, 0.0) < 4, "dawn + 500");
        assert!(sky_darken(&env, 12000, 0.0, 0.0) < 4, "clear sunset");
        assert!(sky_darken(&env, 12500, 0.0, 0.0) < 4, "just before dusk");
        assert!(sky_darken(&env, 13000, 0.0, 0.0) >= 4, "after dusk");
        assert!(sky_darken(&env, 23000, 0.0, 0.0) >= 4, "before dawn");
        assert!(sky_darken(&env, 23500, 0.0, 0.0) < 4, "night ends");
        for t in (0..24000).step_by(250) {
            let dark = sky_darken(&env, t, 0.0, 0.0) >= 4;
            if (13000..23000).contains(&t) {
                assert!(dark, "tick {t} is night");
            } else if t < 12500 || t >= 23500 {
                assert!(!dark, "tick {t} is day");
            }
            // 12500..13000 and 23000..23500 straddle the boundary; the
            // named assertions above pin both sides of it.
        }
        std::fs::remove_dir_all(&root).unwrap();
    }

    /// The fixed-time dimensions: the Nether dark with its red fog, the End
    /// black with stars.
    #[test]
    fn legacy_nether_and_end_get_sane_values() {
        let root = legacy_root();
        let nether = Environment::load_dimension(&root, "the_nether").unwrap();
        let sky = legacy_sample(&nether, "minecraft:nether_wastes", 6000);
        assert_eq!(
            sky.fog_color,
            [51.0 / 255.0, 8.0 / 255.0, 8.0 / 255.0],
            "nether wastes fog 0x330808"
        );
        assert_eq!(sky.sky_light_level, 4.0 / 15.0, "midnight sky light");
        assert_eq!(sky.star_brightness, 0.0, "no stars in the Nether");
        let end = Environment::load_dimension(&root, "the_end").unwrap();
        let sky = legacy_sample(&end, "minecraft:the_end", 6000);
        assert_eq!(sky.sky_color, [0.0; 3], "the End sky is black");
        assert_eq!(sky.star_brightness, 0.5, "but its stars are out");
        assert_eq!(sky.sky_light_level, 1.0, "the End stays lit");
        assert_eq!(
            sky.fog_color,
            [160.0 / 255.0, 128.0 / 255.0, 160.0 / 255.0],
            "the End fog 0xA080A0"
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    /// The real prepared 1.21.7 data: the plains sky at the playtest world's
    /// time is its vanilla colour. Skips without prepared data.
    #[test]
    fn prepared_1_21_7_overworld_sky_is_not_black() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.local/horizon-client/1.21.7");
        if !root
            .join("data/minecraft/dimension_type/overworld.json")
            .exists()
        {
            return;
        }
        let env = Environment::load(&root).unwrap();
        let sky = legacy_sample(&env, "minecraft:plains", 2002);
        assert_eq!(
            sky.sky_color,
            [120.0 / 255.0, 167.0 / 255.0, 1.0],
            "plains 0x78A7FF"
        );
        assert_ne!(sky.fog_color, [0.0; 3], "fog is not black either");
    }
}
