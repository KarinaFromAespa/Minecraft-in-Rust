//! `SpecialModelRenderer`s: items drawn with entity models (chests, shulker
//! boxes, banners, heads, shields, tridents, decorated pots, conduits and
//! copper golem statues). On the item path each renderer applies its 1.21.11
//! inline transform in `submit` and `getExtents`: 26.x carried the transform
//! in the item model's `transformation` field, 1.21.11's item models have
//! none, so the renderer does it (`item_transform`). The worn-skull path
//! (`submit_worn`) supplies its own pose and takes none.
use super::{Foil, ItemQuad, ItemRenderer, ItemSink, SpecialHost, argb};
use crate::{block_entity_render::ModelPart, hud::Rect, mobs::EntityModelDocument};
use glam::{Mat4, Quat, Vec3};
use horizon_assets::item_model::{SpecialKind, SpecialModel};
use serde_json::Value;
use std::{collections::HashMap, f32::consts::PI, path::Path};

/// Entity model layers the special renderers bake (`ModelLayers`).
const LAYERS: [(&str, &str); 21] = [
    ("chest", "minecraft:chest#main"),
    ("double_chest_left", "minecraft:double_chest_left#main"),
    ("double_chest_right", "minecraft:double_chest_right#main"),
    ("shulker_box", "minecraft:shulker_box#main"),
    ("standing_banner", "minecraft:standing_banner#main"),
    ("standing_banner_flag", "minecraft:standing_banner#flag"),
    ("wall_banner", "minecraft:wall_banner#main"),
    ("wall_banner_flag", "minecraft:wall_banner#flag"),
    ("skeleton", "minecraft:skeleton_skull#main"),
    ("wither_skeleton", "minecraft:wither_skeleton_skull#main"),
    ("zombie", "minecraft:zombie_head#main"),
    ("creeper", "minecraft:creeper_head#main"),
    ("dragon", "minecraft:dragon_skull#main"),
    ("piglin", "minecraft:piglin_head#main"),
    ("player", "minecraft:player_head#main"),
    ("book", "minecraft:book#main"),
    ("conduit_shell", "minecraft:conduit#shell"),
    ("decorated_pot_base", "minecraft:decorated_pot_base#main"),
    ("decorated_pot_sides", "minecraft:decorated_pot_sides#main"),
    ("shield", "minecraft:shield#main"),
    ("trident", "minecraft:trident#main"),
];
const COPPER_GOLEM_LAYERS: [(&str, &str); 4] = [
    ("copper_golem", "minecraft:copper_golem#main"),
    (
        "copper_golem_running",
        "minecraft:copper_golem_running#main",
    ),
    (
        "copper_golem_sitting",
        "minecraft:copper_golem_sitting#main",
    ),
    ("copper_golem_star", "minecraft:copper_golem_star#main"),
];

pub(super) fn load_parts(root: &Path) -> HashMap<String, ModelPart> {
    let Ok(all) = std::fs::read(root.join("entity-models.json"))
        .map_err(|_| ())
        .and_then(|b| serde_json::from_slice::<Value>(&b).map_err(|_| ()))
    else {
        return HashMap::new();
    };
    load_parts_from_value(&all)
}

pub(super) fn load_parts_from_document(
    document: &EntityModelDocument,
) -> HashMap<String, ModelPart> {
    load_parts_from_value(document.value())
}

fn load_parts_from_value(all: &Value) -> HashMap<String, ModelPart> {
    LAYERS
        .iter()
        .chain(COPPER_GOLEM_LAYERS.iter())
        .filter_map(|(key, layer)| {
            all.get(*layer)
                .map(|v| ((*key).to_owned(), ModelPart::parse(v)))
        })
        .collect()
}

/// Marks every quad passed on as drawn with a back-face-culling render type (`entitySolid`).
struct CullBack<'a>(&'a mut dyn ItemSink);
impl ItemSink for CullBack<'_> {
    fn quad(&mut self, quad: ItemQuad) {
        self.0.quad(ItemQuad {
            cull_back: true,
            ..quad
        });
    }
}

/// Marks every quad passed on as drawn with a render type that has an `OitPipelineSet`
/// (`RenderTypes.entityTranslucent`).
struct OitEntity<'a>(&'a mut dyn ItemSink);
impl ItemSink for OitEntity<'_> {
    fn quad(&mut self, quad: ItemQuad) {
        self.0.quad(ItemQuad { oit: true, ..quad });
    }
}

/// A pose override for a named part (`setupAnim`).
type Adjust<'a> = &'a dyn Fn(&str, &mut [f32; 9]);
fn none(_: &str, _: &mut [f32; 9]) {}

/// `ModelPart.translateAndRotate`.
fn part_matrix(parent: Mat4, p: &[f32; 9]) -> Mat4 {
    parent
        * Mat4::from_translation(Vec3::new(p[0], p[1], p[2]) / 16.0)
        * Mat4::from_quat(Quat::from_euler(glam::EulerRot::ZYX, p[5], p[4], p[3]))
        * Mat4::from_scale(Vec3::new(p[6], p[7], p[8]))
}
/// `ModelPart.render` of `part` (named `name`) as item quads.
#[allow(clippy::too_many_arguments)]
fn draw(
    part: &ModelPart,
    name: &str,
    parent: Mat4,
    adjust: Adjust,
    sprite: Rect,
    color: u32,
    foil: Foil,
    sink: &mut dyn ItemSink,
) {
    let mut p = part.pose;
    adjust(name, &mut p);
    let m = part_matrix(parent, &p);
    for poly in &part.polygons {
        let pos = poly.map(|v| m.transform_point3(Vec3::new(v[0], v[1], v[2]) / 16.0));
        let normal = (pos[1] - pos[0]).cross(pos[2] - pos[0]).normalize_or_zero();
        let normal = if normal == Vec3::ZERO {
            (pos[2] - pos[1])
                .cross(pos[3] - pos[1])
                .normalize_or(Vec3::Y)
        } else {
            normal
        };
        // A reflection reverses winding, not PoseStack normal directions.
        let normal = if m.determinant() < 0.0 {
            -normal
        } else {
            normal
        };
        sink.quad(ItemQuad {
            pos,
            uv: poly.map(|v| {
                [
                    sprite.u0 + (sprite.u1 - sprite.u0) * v[3],
                    sprite.v0 + (sprite.v1 - sprite.v0) * v[4],
                ]
            }),
            foil_uv: poly.map(|v| [v[3], v[4]]),
            gui_glint_uv: None,
            normal,
            color,
            foil,
            light_emission: 0,
            cull_back: false,
            translucent: false,
            oit: false,
        });
    }
    for (child_name, child) in &part.children {
        draw(child, child_name, m, adjust, sprite, color, foil, sink);
    }
}
/// `ModelPart.getExtentsForGui` from an identity pose.
fn extents_of(part: &ModelPart, name: &str, parent: Mat4, adjust: Adjust, out: &mut Vec<Vec3>) {
    if part.polygons.is_empty() && part.children.is_empty() {
        return;
    }
    let mut p = part.pose;
    adjust(name, &mut p);
    let m = part_matrix(parent, &p);
    for poly in &part.polygons {
        for v in poly {
            let q = m.transform_point3(Vec3::new(v[0], v[1], v[2]) / 16.0);
            if !out.contains(&q) {
                out.push(q);
            }
        }
    }
    for (child_name, child) in &part.children {
        extents_of(child, child_name, m, adjust, out);
    }
}
fn child<'a>(part: &'a ModelPart, name: &str) -> Option<&'a ModelPart> {
    part.child(name)
}

/// A texture identifier relative to `textures/` (`minecraft:entity/x` or a
/// full `minecraft:textures/entity/x.png`) as its atlas sprite name.
fn texture_sprite(id: &str) -> String {
    let path = id.strip_prefix("minecraft:").unwrap_or(id);
    let path = path.strip_prefix("textures/").unwrap_or(path);
    path.strip_suffix(".png").unwrap_or(path).to_owned()
}
fn mapped(directory: &str, id: &str) -> String {
    format!(
        "entity/{directory}/{}",
        id.strip_prefix("minecraft:").unwrap_or(id)
    )
}
/// `DyeColor.getTextureDiffuseColor` as ARGB.
fn dye(name: &str) -> u32 {
    0xFF00_0000 | crate::banner::dye_rgb(name)
}

fn skull_layer(kind: &str) -> &'static str {
    match kind {
        "skeleton" => "skeleton",
        "wither_skeleton" => "wither_skeleton",
        "zombie" => "zombie",
        "creeper" => "creeper",
        "dragon" => "dragon",
        "piglin" => "piglin",
        _ => "player",
    }
}
/// `SkullBlockRenderer.SKIN_BY_TYPE`.
fn skull_texture(kind: &str) -> &'static str {
    match kind {
        "skeleton" => "entity/skeleton/skeleton",
        "wither_skeleton" => "entity/skeleton/wither_skeleton",
        "zombie" => "entity/zombie/zombie",
        "creeper" => "entity/creeper/creeper",
        "dragon" => "entity/enderdragon/dragon",
        "piglin" => "entity/piglin/piglin",
        _ => "entity/player/wide/steve",
    }
}
/// `SkullModelBase.setupAnim` for a head at `animation` (yRot = xRot = 0).
fn skull_pose(kind: &str, animation: f32) -> impl Fn(&str, &mut [f32; 9]) + '_ {
    move |name: &str, p: &mut [f32; 9]| match (kind, name) {
        (_, "head") => {
            p[3] = 0.0;
            p[4] = 0.0;
        }
        ("dragon", "jaw") => p[3] = (((animation * PI * 0.2) as f64).sin() as f32 + 1.0) * 0.2,
        ("piglin", "left_ear") => {
            p[5] = -((((animation * PI * 0.2 * 1.2) as f64).cos() + 2.5) as f32) * 0.2;
        }
        ("piglin", "right_ear") => {
            p[5] = ((((animation * PI * 0.2) as f64).cos() + 2.5) as f32) * 0.2;
        }
        _ => {}
    }
}
fn shulker_pose(progress: f32) -> impl Fn(&str, &mut [f32; 9]) {
    move |name: &str, p: &mut [f32; 9]| {
        if name == "lid" {
            p[0] = 0.0;
            p[1] = 24.0 - progress * 0.5 * 16.0;
            p[2] = 0.0;
            p[4] = 270.0 * progress * (PI / 180.0);
        }
    }
}
fn chest_pose(open: f32) -> impl Fn(&str, &mut [f32; 9]) {
    move |name: &str, p: &mut [f32; 9]| {
        if name == "lid" || name == "lock" {
            p[3] = -(open * (PI / 2.0));
        }
    }
}
/// `BannerFlagModel.setupAnim(0)`.
fn flag_pose(name: &str, p: &mut [f32; 9]) {
    if name == "flag" {
        p[3] = (-0.0125 + 0.01 * (2.0 * PI * 0.0f32).cos()) * PI;
    }
}
/// `BookModel.setupAnim`.
fn book_pose(openness: f32, flip1: f32, flip2: f32) -> impl Fn(&str, &mut [f32; 9]) {
    move |name: &str, p: &mut [f32; 9]| match name {
        "left_lid" => p[4] = PI + openness,
        "right_lid" => p[4] = -openness,
        "left_pages" => {
            p[4] = openness;
            p[0] = openness.sin();
        }
        "right_pages" => {
            p[4] = -openness;
            p[0] = openness.sin();
        }
        "flip_page1" => {
            p[4] = openness - openness * 2.0 * flip1;
            p[0] = openness.sin();
        }
        "flip_page2" => {
            p[4] = openness - openness * 2.0 * flip2;
            p[0] = openness.sin();
        }
        _ => {}
    }
}
/// `CopperGolemStatueModel.setupAnim` for the item's `Direction.SOUTH`
/// state: root y 0, yRot the opposite facing (north, 180 degrees),
/// zRot 180 degrees. 26.x's `setupAnim(Unit)` set only y and zRot.
fn statue_pose(name: &str, p: &mut [f32; 9]) {
    if name == "root" {
        p[1] = 0.0;
        p[4] = PI;
        p[5] = PI;
    }
}
fn statue_layer(pose: &str) -> &'static str {
    match pose {
        "running" => "copper_golem_running",
        "sitting" => "copper_golem_sitting",
        "star" => "copper_golem_star",
        _ => "copper_golem",
    }
}

/// `Direction.getRotation`, as `block_entity_render::direction_rotation`.
fn orientation_rotation(orientation: &str) -> Quat {
    let xyz = |x: f32, y: f32, z: f32| Quat::from_euler(glam::EulerRot::XYZ, x, y, z);
    match orientation {
        "down" => Quat::from_rotation_x(PI),
        "north" => xyz(PI / 2.0, 0.0, PI),
        "south" => Quat::from_rotation_x(PI / 2.0),
        "west" => xyz(PI / 2.0, 0.0, PI / 2.0),
        "east" => xyz(PI / 2.0, 0.0, -PI / 2.0),
        _ => Quat::IDENTITY,
    }
}

/// The transform each special renderer applies inline in 1.21.11's `submit`
/// and `getExtents` (26.x carried it in the item model's `transformation`,
/// or a `DEFAULT_TRANSFORMATION`; 1.21.11's item models have none). Chest
/// and decorated pot have none in either version.
fn item_transform(kind: &SpecialKind) -> Mat4 {
    match kind {
        // `BannerRenderer.submitBanner` at angle 0: banner items are always
        // standing (`submitSpecial` takes no attachment in 1.21.11).
        SpecialKind::Banner { .. } => {
            Mat4::from_translation(Vec3::new(0.5, 0.0, 0.5))
                * Mat4::from_scale(Vec3::new(2.0 / 3.0, -2.0 / 3.0, -2.0 / 3.0))
        }
        // `ShieldSpecialRenderer.submit` / `TridentSpecialRenderer.submit`.
        SpecialKind::Shield | SpecialKind::Trident => Mat4::from_scale(Vec3::new(1.0, -1.0, -1.0)),
        // `SkullBlockRenderer.submitSkull(null, 180, 0)`: translate, then
        // the 180-degree yaw the state carries, then the flip — the
        // pose-level form of vanilla's translate + scale + state yRot,
        // the same algebra the block-entity skull path uses.
        SpecialKind::Head { .. } | SpecialKind::PlayerHead => {
            Mat4::from_translation(Vec3::new(0.5, 0.0, 0.5))
                * Mat4::from_rotation_y(PI)
                * Mat4::from_scale(Vec3::new(-1.0, -1.0, 1.0))
        }
        // `ConduitSpecialRenderer.submit`.
        SpecialKind::Conduit => Mat4::from_translation(Vec3::splat(0.5)),
        // `ShulkerBoxRenderer.prepareModel`.
        SpecialKind::ShulkerBox { orientation, .. } => {
            Mat4::from_translation(Vec3::splat(0.5))
                * Mat4::from_scale(Vec3::splat(0.9995))
                * Mat4::from_quat(orientation_rotation(orientation))
                * Mat4::from_scale(Vec3::new(1.0, -1.0, -1.0))
                * Mat4::from_translation(Vec3::new(0.0, -1.0, 0.0))
        }
        // `CopperGolemStatueSpecialRenderer.positionModel`.
        SpecialKind::CopperGolemStatue { .. } => {
            Mat4::from_translation(Vec3::new(0.5, 1.5, 0.5))
                * Mat4::from_scale(Vec3::new(-1.0, -1.0, 1.0))
        }
        SpecialKind::Chest { .. } | SpecialKind::DecoratedPot => Mat4::IDENTITY,
    }
}

/// `SpecialModelRenderer.getExtents`.
pub(super) fn extents(renderer: &ItemRenderer, model: &SpecialModel) -> Vec<Vec3> {
    let mut out = Vec::new();
    let base = item_transform(&model.kind);
    let mut add = |key: &str, adjust: Adjust| {
        if let Some(part) = renderer.part(key) {
            extents_of(part, "root", base, adjust, &mut out);
        }
    };
    match &model.kind {
        SpecialKind::Banner { .. } => {
            // `BannerRenderer.getExtents`: the standing models only.
            add("standing_banner", &none);
            add("standing_banner_flag", &flag_pose);
        }
        SpecialKind::Conduit => add("conduit_shell", &none),
        SpecialKind::Chest { openness, .. } => add("chest", &chest_pose(*openness)),
        SpecialKind::CopperGolemStatue { pose, .. } => add(statue_layer(pose), &statue_pose),
        SpecialKind::DecoratedPot => {
            if let Some(pot_base) = renderer.part("decorated_pot_base") {
                // `DecoratedPotRenderer.getExtents`: neck, top and bottom
                // only, not the sides.
                for name in ["neck", "top", "bottom"] {
                    if let Some(part) = child(pot_base, name) {
                        extents_of(part, name, base, &none, &mut out);
                    }
                }
            }
        }
        SpecialKind::Head {
            kind, animation, ..
        } => add(skull_layer(kind), &skull_pose(kind, *animation)),
        SpecialKind::PlayerHead => add("player", &none),
        SpecialKind::ShulkerBox { openness, .. } => add("shulker_box", &shulker_pose(*openness)),
        SpecialKind::Shield => add("shield", &none),
        SpecialKind::Trident => add("trident", &none),
    }
    out
}

/// Banner pattern layers of a `minecraft:banner_patterns` value: the
/// pattern identifier (or inline asset) and the dye.
fn pattern_layers(v: &Value) -> Vec<(String, String)> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|layer| {
            let pattern = match &layer["pattern"] {
                Value::String(s) => s.clone(),
                Value::Object(o) => format!("#asset:{}", o.get("asset_id")?.as_str()?),
                _ => return None,
            };
            let color = layer["color"].as_str()?.to_owned();
            Some((pattern, color))
        })
        .collect()
}
fn pattern_texture(host: &dyn SpecialHost, banner: bool, pattern: &str) -> Option<String> {
    if let Some(asset) = pattern.strip_prefix("#asset:") {
        let path = asset.strip_prefix("minecraft:").unwrap_or(asset);
        return Some(format!(
            "entity/{}/{path}",
            if banner { "banner" } else { "shield" }
        ));
    }
    if banner {
        host.banner_texture(pattern)
    } else {
        host.shield_texture(pattern)
    }
}
/// `DecoratedPotPatterns.getPatternFromItem` for one side's sherd item.
fn pot_side(item: Option<&str>) -> String {
    let item = item.map(|i| i.strip_prefix("minecraft:").unwrap_or(i));
    match item.and_then(|i| i.strip_suffix("_pottery_sherd")) {
        Some(name) => format!("entity/decorated_pot/{name}_pottery_pattern"),
        None => "entity/decorated_pot/decorated_pot_side".to_owned(),
    }
}
/// `PotDecorations` as [back, left, right, front] item ids, from the list
/// codec or a keyed object.
fn pot_sides(v: &Value) -> [Option<String>; 4] {
    let id = |s: &Value| match s {
        Value::String(s) => Some(s.clone()),
        Value::Object(o) => o.get("id").and_then(Value::as_str).map(str::to_owned),
        _ => None,
    };
    match v {
        Value::Array(a) => std::array::from_fn(|i| a.get(i).and_then(id)),
        Value::Object(o) => ["back", "left", "right", "front"].map(|k| o.get(k).and_then(id)),
        _ => Default::default(),
    }
}

/// `GuiBannerResultRenderer.renderToTexture` under `pose`: the standing
/// banner flag (`BannerFlagModel.setupAnim(0)`) in the banner base texture,
/// then `BannerRenderer.submitPatterns` (the base colour and up to 16 layers
/// of `(texture, dye)`).
pub(super) fn submit_banner_result(
    renderer: &ItemRenderer,
    pose: Mat4,
    base: &str,
    layers: &[(String, String)],
    sprites: &HashMap<String, Rect>,
    sink: &mut dyn ItemSink,
) {
    let Some(flag) = renderer.part("standing_banner_flag") else {
        return;
    };
    let mut part = |texture: &str, color: u32| {
        if let Some(r) = sprites.get(texture).copied() {
            draw(flag, "root", pose, &flag_pose, r, color, Foil::None, sink);
        }
    };
    part("entity/banner_base", argb(-1));
    part("entity/banner/base", dye(base));
    for (texture, color) in layers.iter().take(16) {
        part(texture, dye(color));
    }
}

/// `GuiBookModelRenderer.renderToTexture`'s book under `pose`: `BookModel`
/// set up for `BookModel.State.forAnimation(0, page1, page2, open)`.
pub(super) fn submit_book(
    renderer: &ItemRenderer,
    pose: Mat4,
    open: f32,
    page1: f32,
    page2: f32,
    sprites: &HashMap<String, Rect>,
    sink: &mut dyn ItemSink,
) {
    let (Some(book), Some(r)) = (
        renderer.part("book"),
        sprites.get("entity/enchanting_table_book").copied(),
    ) else {
        return;
    };
    // forAnimation at time 0: (sin(0) * 0.1 + 1.25) * open.
    let pose_for = book_pose(1.25 * open, page1, page2);
    draw(book, "root", pose, &pose_for, r, argb(-1), Foil::None, sink);
}

/// `SpecialModelRenderer.submit`: the item path, with the renderer's 1.21.11
/// inline transform (`item_transform`).
#[allow(clippy::too_many_arguments)]
pub(super) fn submit(
    renderer: &ItemRenderer,
    model: &SpecialModel,
    argument: &Value,
    has_foil: bool,
    pose: Mat4,
    sprites: &HashMap<String, Rect>,
    host: &dyn SpecialHost,
    sink: &mut dyn ItemSink,
) {
    submit_inner(
        renderer, model, argument, has_foil, pose, sprites, host, sink, true,
    );
}

/// A worn skull (`CustomHeadLayer`): the caller supplies the head pose and
/// the model draws untransformed — vanilla's worn path composes the skull
/// transform into the head pose itself (the flips and the 180-degree yaw
/// cancel to a uniform 1.1875 scale), so applying the item transform here
/// would double it.
#[allow(clippy::too_many_arguments)]
pub(super) fn submit_worn(
    renderer: &ItemRenderer,
    model: &SpecialModel,
    argument: &Value,
    has_foil: bool,
    pose: Mat4,
    sprites: &HashMap<String, Rect>,
    host: &dyn SpecialHost,
    sink: &mut dyn ItemSink,
) {
    submit_inner(
        renderer, model, argument, has_foil, pose, sprites, host, sink, false,
    );
}

#[allow(clippy::too_many_arguments)]
fn submit_inner(
    renderer: &ItemRenderer,
    model: &SpecialModel,
    argument: &Value,
    has_foil: bool,
    pose: Mat4,
    sprites: &HashMap<String, Rect>,
    host: &dyn SpecialHost,
    sink: &mut dyn ItemSink,
    as_item: bool,
) {
    let pose = if as_item {
        pose * item_transform(&model.kind)
    } else {
        pose
    };
    let white = argb(-1);
    let mut part = |key: &str, adjust: Adjust, texture: &str, color: u32, foil: Foil| {
        if let (Some(p), Some(r)) = (renderer.part(key), sprites.get(texture).copied()) {
            draw(p, "root", pose, adjust, r, color, foil, sink);
        }
    };
    match &model.kind {
        SpecialKind::Conduit => {
            part(
                "conduit_shell",
                &none,
                "entity/conduit/base",
                white,
                Foil::None,
            );
        }
        SpecialKind::Chest { texture, openness } => part(
            "chest",
            &chest_pose(*openness),
            &mapped("chest", texture),
            white,
            Foil::None,
        ),
        SpecialKind::ShulkerBox {
            texture, openness, ..
        } => part(
            "shulker_box",
            &shulker_pose(*openness),
            &mapped("shulker", texture),
            white,
            Foil::None,
        ),
        SpecialKind::CopperGolemStatue { texture, pose: p } => part(
            statue_layer(p),
            &statue_pose,
            &texture_sprite(texture),
            white,
            Foil::None,
        ),
        SpecialKind::Head {
            kind,
            texture,
            animation,
        } => {
            let skin = texture
                .as_deref()
                .map(|t| format!("entity/{}", t.strip_prefix("minecraft:").unwrap_or(t)))
                .unwrap_or_else(|| skull_texture(kind).to_owned());
            part(
                skull_layer(kind),
                &skull_pose(kind, *animation),
                &skin,
                white,
                Foil::None,
            );
        }
        SpecialKind::PlayerHead => {
            let skin = (!argument.is_null())
                .then(|| host.player_skin(argument))
                .flatten()
                .unwrap_or_else(|| "entity/player/slim/steve".to_owned());
            // `PlayerHeadSpecialRenderer`: the skin's `entityTranslucent` render type.
            if let (Some(p), Some(r)) = (renderer.part("player"), sprites.get(&skin).copied()) {
                draw(
                    p,
                    "root",
                    pose,
                    &none,
                    r,
                    white,
                    Foil::None,
                    &mut OitEntity(sink),
                );
            }
        }
        SpecialKind::Trident => {
            let foil = if has_foil { Foil::Standard } else { Foil::None };
            part("trident", &none, "entity/trident", white, foil);
        }
        SpecialKind::Banner { color } => {
            // `BannerRenderer.submitSpecial`: banner items are always
            // standing in 1.21.11 (no attachment in the codec).
            let (main, flag) = ("standing_banner", "standing_banner_flag");
            part(main, &none, "entity/banner_base", white, Foil::None);
            part(flag, &flag_pose, "entity/banner_base", white, Foil::None);
            // `submitPatterns`: the base layer in the banner's colour, then up to 16 layers.
            part(
                flag,
                &flag_pose,
                "entity/banner/base",
                dye(color),
                Foil::None,
            );
            for (pattern, color) in pattern_layers(argument).into_iter().take(16) {
                if let Some(t) = pattern_texture(host, true, &pattern) {
                    part(flag, &flag_pose, &t, dye(&color), Foil::None);
                }
            }
        }
        SpecialKind::Shield => {
            let patterns = pattern_layers(&argument["minecraft:banner_patterns"]);
            let base_color = argument["minecraft:base_color"].as_str();
            let has_patterns = !patterns.is_empty() || base_color.is_some();
            // `ModelBakery.SHIELD_BASE` / `NO_PATTERN_SHIELD`: 1.21.11 keeps
            // them at `textures/entity/shield_base.png` and
            // `textures/entity/shield_base_nopattern.png`; 26.x moved them
            // under `entity/shield/`.
            let base = if has_patterns {
                "entity/shield_base"
            } else {
                "entity/shield_base_nopattern"
            };
            // Without patterns: `entitySolidGlint`. With them, vanilla draws
            // `patternedShieldGlint` last, the glint alone over the finished
            // plate. A foil quad here is its texture with the glint over it
            // (the GUI adds the glint masked by the layer's alpha, the world
            // shader composites it), so every layer carries the foil: each
            // pixel ends with the glint once, over the layers below it.
            // Redrawing the base with the foil would cover the patterns.
            let foil = if has_foil { Foil::Standard } else { Foil::None };
            part("shield", &none, base, white, foil);
            if has_patterns {
                part(
                    "shield",
                    &none,
                    "entity/shield/base",
                    dye(base_color.unwrap_or("white")),
                    foil,
                );
                for (pattern, color) in patterns.into_iter().take(16) {
                    if let Some(t) = pattern_texture(host, false, &pattern) {
                        part("shield", &none, &t, dye(&color), foil);
                    }
                }
            }
        }
        SpecialKind::DecoratedPot => {
            let sides = pot_sides(argument);
            // `DecoratedPotRenderer` draws every part with `entitySolid`.
            let sink = &mut CullBack(sink);
            if let Some(base) = renderer.part("decorated_pot_base")
                && let Some(r) = sprites
                    .get("entity/decorated_pot/decorated_pot_base")
                    .copied()
            {
                for name in ["neck", "top", "bottom"] {
                    if let Some(p) = child(base, name) {
                        draw(p, name, pose, &none, r, white, Foil::None, sink);
                    }
                }
            }
            if let Some(side_parts) = renderer.part("decorated_pot_sides") {
                for (i, name) in ["back", "left", "right", "front"].into_iter().enumerate() {
                    let texture = pot_side(sides[i].as_deref());
                    if let (Some(p), Some(r)) = (child(side_parts, name), sprites.get(&texture)) {
                        draw(p, name, pose, &none, *r, white, Foil::None, sink);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pot_sides_read_both_shapes() {
        let list = serde_json::json!(["minecraft:brick", "minecraft:angler_pottery_sherd"]);
        let sides = pot_sides(&list);
        assert_eq!(
            pot_side(sides[0].as_deref()),
            "entity/decorated_pot/decorated_pot_side"
        );
        assert_eq!(
            pot_side(sides[1].as_deref()),
            "entity/decorated_pot/angler_pottery_pattern"
        );
        assert_eq!(
            pot_side(sides[3].as_deref()),
            "entity/decorated_pot/decorated_pot_side"
        );
    }
    #[test]
    fn texture_ids_map_to_sprites() {
        assert_eq!(
            texture_sprite("minecraft:textures/entity/copper_golem/copper_golem.png"),
            "entity/copper_golem/copper_golem"
        );
        assert_eq!(
            mapped("chest", "minecraft:christmas"),
            "entity/chest/christmas"
        );
    }

    #[test]
    fn model_normals_follow_the_pose_not_the_mirrored_winding() {
        let part = ModelPart::parse(&serde_json::json!({
            "pose": [0, 0, 0, 0, 0, 0, 1, 1, 1],
            "polygons": [[[0, 0, 0, 0, 0], [1, 0, -1, 1, 0],
                          [0, 1, -1, 1, 1], [-1, 1, 0, 0, 1]]],
            "children": {}
        }));
        let sprite = Rect {
            u0: 0.0,
            v0: 0.0,
            u1: 1.0,
            v1: 1.0,
            w: 16,
            h: 16,
        };
        let local = Vec3::ONE.normalize();
        for scale in [
            Vec3::ONE,
            Vec3::new(16.0, -16.0, 16.0),
            Vec3::new(2.0, 3.0, -4.0),
            Vec3::new(-2.0, -3.0, 4.0),
        ] {
            let pose = Mat4::from_rotation_y(0.7) * Mat4::from_scale(scale);
            let mut quads = Vec::new();
            draw(
                &part,
                "root",
                pose,
                &none,
                sprite,
                argb(-1),
                Foil::None,
                &mut quads,
            );
            assert_eq!(quads.len(), 1);
            let expected = super::super::transform_normal(pose, local);
            assert!(
                (quads[0].normal - expected).length() < 1e-6,
                "scale {scale:?}: {:?} != {expected:?}",
                quads[0].normal
            );
        }
    }
    /// `ShieldSpecialRenderer.submit` of an enchanted shield with a banner:
    /// the glint goes over the patterns, and nothing covers them after.
    #[test]
    fn an_enchanted_banner_shield_keeps_its_patterns_under_the_glint() {
        let root =
            std::env::temp_dir().join(format!("horizon-shield-glint-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("items.json"), "{}").unwrap();
        std::fs::write(
            root.join("entity-models.json"),
            serde_json::json!({"minecraft:shield#main": {
                "pose": [0, 0, 0, 0, 0, 0, 1, 1, 1],
                "polygons": [[[0, 0, 0, 0, 0], [1, 0, 0, 1, 0],
                              [1, 1, 0, 1, 1], [0, 1, 0, 0, 1]]],
                "children": {}
            }})
            .to_string(),
        )
        .unwrap();
        let library = crate::model_library::ParsedModels::default().into_item_library();
        let renderer = ItemRenderer::load_with_library(&root, library).unwrap();
        let _ = std::fs::remove_dir_all(&root);
        let textures = [
            "entity/shield_base",
            "entity/shield_base_nopattern",
            "entity/shield/base",
            "entity/shield/stripe_bottom",
        ];
        // A sprite per texture, told apart by its u0.
        let sprites: HashMap<String, Rect> = textures
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let u0 = i as f32 / 8.0;
                let rect = Rect {
                    u0,
                    v0: 0.0,
                    u1: u0 + 0.1,
                    v1: 0.1,
                    w: 16,
                    h: 16,
                };
                ((*t).to_owned(), rect)
            })
            .collect();
        let drawn = |argument: Value, has_foil: bool| {
            let mut quads: Vec<ItemQuad> = Vec::new();
            submit(
                &renderer,
                &SpecialModel {
                    kind: SpecialKind::Shield,
                },
                &argument,
                has_foil,
                Mat4::IDENTITY,
                &sprites,
                &super::super::NoHost,
                &mut quads,
            );
            quads
                .iter()
                .map(|q| {
                    let texture = textures
                        .iter()
                        .find(|t| sprites[**t].u0 == q.uv[0][0].min(q.uv[1][0]))
                        .unwrap();
                    (*texture, q.foil)
                })
                .collect::<Vec<_>>()
        };
        let banner = serde_json::json!({
            "minecraft:base_color": "red",
            "minecraft:banner_patterns": [{
                "pattern": {"asset_id": "minecraft:stripe_bottom",
                            "translation_key": "block.minecraft.banner.stripe_bottom"},
                "color": "blue"
            }]
        });
        assert_eq!(
            drawn(banner.clone(), true),
            [
                ("entity/shield_base", Foil::Standard),
                ("entity/shield/base", Foil::Standard),
                ("entity/shield/stripe_bottom", Foil::Standard),
            ]
        );
        assert_eq!(
            drawn(banner, false),
            [
                ("entity/shield_base", Foil::None),
                ("entity/shield/base", Foil::None),
                ("entity/shield/stripe_bottom", Foil::None),
            ]
        );
        assert_eq!(
            drawn(Value::Null, true),
            [("entity/shield_base_nopattern", Foil::Standard)]
        );
    }
    /// Each renderer's 1.21.11 inline transform, checked by moving a point
    /// through it: the expected coordinates are worked out by hand from the
    /// Java (`BannerRenderer.submitBanner`, `ShieldSpecialRenderer.submit`,
    /// `SkullBlockRenderer.submitSkull(null, 180, 0)`,
    /// `ShulkerBoxRenderer.prepareModel`, `CopperGolemStatueSpecialRenderer
    /// .positionModel`), not from the matrix the code builds.
    #[test]
    fn item_transforms_match_the_1_21_11_inline_forms() {
        let moved = |kind: &SpecialKind, p: Vec3| item_transform(kind).transform_point3(p);
        let near = |a: Vec3, b: Vec3| (a - b).length() < 1e-5;
        let banner = SpecialKind::Banner {
            color: "white".to_owned(),
        };
        assert!(near(
            moved(&banner, Vec3::new(3.0, 3.0, 3.0)),
            Vec3::new(2.5, -2.0, -1.5)
        ));
        assert!(near(moved(&banner, Vec3::ZERO), Vec3::new(0.5, 0.0, 0.5)));
        for kind in [SpecialKind::Shield, SpecialKind::Trident] {
            assert!(near(
                moved(&kind, Vec3::new(1.0, 2.0, 3.0)),
                Vec3::new(1.0, -2.0, -3.0)
            ));
        }
        let head = SpecialKind::Head {
            kind: "skeleton".to_owned(),
            texture: None,
            animation: 0.0,
        };
        for kind in [&head, &SpecialKind::PlayerHead] {
            assert!(near(
                moved(kind, Vec3::new(1.0, 2.0, 3.0)),
                Vec3::new(1.5, -2.0, -2.5)
            ));
        }
        assert!(near(
            moved(&SpecialKind::Conduit, Vec3::new(1.0, 2.0, 3.0)),
            Vec3::new(1.5, 2.5, 3.5)
        ));
        let shulker = |orientation: &str| SpecialKind::ShulkerBox {
            texture: "minecraft:shulker".to_owned(),
            openness: 0.0,
            orientation: orientation.to_owned(),
        };
        // `prepareModel` at the origin, one orientation each: the up case
        // lands exactly on the 26.x item-model translation (0.5, 1.4995,
        // 0.5), the others on the matching face.
        assert!(near(
            moved(&shulker("up"), Vec3::ZERO),
            Vec3::new(0.5, 1.4995, 0.5)
        ));
        assert!(near(
            moved(&shulker("down"), Vec3::ZERO),
            Vec3::new(0.5, -0.4995, 0.5)
        ));
        assert!(near(
            moved(&shulker("north"), Vec3::ZERO),
            Vec3::new(0.5, 0.5, -0.4995)
        ));
        assert!(near(
            moved(&shulker("south"), Vec3::ZERO),
            Vec3::new(0.5, 0.5, 1.4995)
        ));
        assert!(near(
            moved(&shulker("west"), Vec3::ZERO),
            Vec3::new(-0.4995, 0.5, 0.5)
        ));
        assert!(near(
            moved(&shulker("east"), Vec3::ZERO),
            Vec3::new(1.4995, 0.5, 0.5)
        ));
        let statue = SpecialKind::CopperGolemStatue {
            texture: "minecraft:textures/entity/copper_golem/copper_golem.png".to_owned(),
            pose: "standing".to_owned(),
        };
        assert!(near(
            moved(&statue, Vec3::new(1.0, 2.0, 3.0)),
            Vec3::new(-0.5, -0.5, 3.5)
        ));
        let chest = SpecialKind::Chest {
            texture: "minecraft:normal".to_owned(),
            openness: 0.0,
        };
        for kind in [&chest, &SpecialKind::DecoratedPot] {
            assert!(near(
                moved(kind, Vec3::new(1.0, 2.0, 3.0)),
                Vec3::new(1.0, 2.0, 3.0)
            ));
        }
    }

    /// The item path applies the transform and the worn path does not: a
    /// shield item's quad is flipped in y and z, a worn player head's is
    /// not, and the shield's extents carry the flip too.
    #[test]
    fn submit_applies_the_item_transform_and_submit_worn_does_not() {
        let root =
            std::env::temp_dir().join(format!("horizon-special-pose-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("items.json"), "{}").unwrap();
        std::fs::write(
            root.join("entity-models.json"),
            serde_json::json!({
                "minecraft:shield#main": {
                    "pose": [0, 0, 0, 0, 0, 0, 1, 1, 1],
                    "polygons": [[[0, 0, 0, 0, 0], [1, 0, 0, 1, 0],
                                  [1, 1, 0, 1, 1], [0, 1, 0, 0, 1]]],
                    "children": {}
                },
                "minecraft:player_head#main": {
                    "pose": [0, 0, 0, 0, 0, 0, 1, 1, 1],
                    "polygons": [[[0, 0, 0, 0, 0], [1, 0, 0, 1, 0],
                                  [1, 1, 0, 1, 1], [0, 1, 0, 0, 1]]],
                    "children": {}
                },
            })
            .to_string(),
        )
        .unwrap();
        let library = crate::model_library::ParsedModels::default().into_item_library();
        let renderer = ItemRenderer::load_with_library(&root, library).unwrap();
        let _ = std::fs::remove_dir_all(&root);
        let sprite = Rect {
            u0: 0.0,
            v0: 0.0,
            u1: 1.0,
            v1: 1.0,
            w: 16,
            h: 16,
        };
        let sprites: HashMap<String, Rect> =
            ["entity/shield_base_nopattern", "entity/player/slim/steve"]
                .into_iter()
                .map(|name| (name.to_owned(), sprite))
                .collect();
        let mut quads: Vec<ItemQuad> = Vec::new();
        submit(
            &renderer,
            &SpecialModel {
                kind: SpecialKind::Shield,
            },
            &Value::Null,
            false,
            Mat4::IDENTITY,
            &sprites,
            &super::super::NoHost,
            &mut quads,
        );
        assert_eq!(quads.len(), 1);
        assert_eq!(quads[0].pos[2], Vec3::new(1.0 / 16.0, -1.0 / 16.0, 0.0));
        let points = extents(
            &renderer,
            &SpecialModel {
                kind: SpecialKind::Shield,
            },
        );
        assert!(points.contains(&Vec3::new(1.0 / 16.0, -1.0 / 16.0, 0.0)));
        assert!(!points.contains(&Vec3::new(1.0 / 16.0, 1.0 / 16.0, 0.0)));
        let mut worn: Vec<ItemQuad> = Vec::new();
        submit_worn(
            &renderer,
            &SpecialModel {
                kind: SpecialKind::PlayerHead,
            },
            &Value::Null,
            false,
            Mat4::IDENTITY,
            &sprites,
            &super::super::NoHost,
            &mut worn,
        );
        assert_eq!(worn.len(), 1);
        assert_eq!(worn[0].pos[2], Vec3::new(1.0 / 16.0, 1.0 / 16.0, 0.0));
    }

    /// `DecoratedPotRenderer` draws every part with `RenderTypes.entitySolid`,
    /// whose pipeline culls back faces: the pot's flat `top` and `bottom`
    /// planes would otherwise show both sides and z-fight in the hand too.
    #[test]
    fn the_decorated_pot_item_culls_its_back_faces() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.local/horizon-client")
            .join(horizon_core::VANILLA_VERSION);
        if !root.join("entity-models.json").exists() {
            return;
        }
        let renderer = ItemRenderer::load(&root).unwrap();
        let sprite = Rect {
            u0: 0.25,
            v0: 0.125,
            u1: 0.375,
            v1: 0.25,
            w: 16,
            h: 16,
        };
        let sprites: HashMap<String, Rect> = [
            "entity/decorated_pot/decorated_pot_base",
            "entity/decorated_pot/decorated_pot_side",
        ]
        .into_iter()
        .map(|name| (name.to_owned(), sprite))
        .collect();
        let model = SpecialModel {
            kind: SpecialKind::DecoratedPot,
        };
        let mut quads: Vec<ItemQuad> = Vec::new();
        submit(
            &renderer,
            &model,
            &Value::Null,
            false,
            Mat4::IDENTITY,
            &sprites,
            &super::super::NoHost,
            &mut quads,
        );
        assert!(!quads.is_empty(), "the pot item draws its parts");
        assert!(
            quads.iter().all(|q| q.cull_back),
            "entitySolid culls the pot's back faces"
        );
    }
}
