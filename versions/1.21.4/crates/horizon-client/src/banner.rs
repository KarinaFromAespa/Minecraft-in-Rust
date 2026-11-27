//! Banner patterns: the `banner_pattern` registry and its tags, the
//! `banner_patterns` layers carried by banner items, shields and placed
//! banners, and the loom rules of `LoomMenu` (selectable patterns, the
//! six-layer limit and the result stack).
use crate::{block_entity_render::DYES, items::ItemStack};
use serde_json::Value;
use std::{collections::HashMap, fs, path::Path, sync::Arc};

pub use horizon_items::banner_layer::{BannerLayer, layers_from_nbt, layers_to_nbt};

/// Layers a loom may stack on a banner (`LoomMenu`'s `hasMaxPatterns`).
pub const MAX_LAYERS: usize = 6;

/// One `banner_pattern` registry entry.
#[derive(Clone, Debug)]
pub struct Pattern {
    pub id: Arc<str>,
    /// `asset_id` path, the texture under `entity/banner` and `entity/shield`.
    pub asset: String,
    pub translation_key: String,
}

/// The pattern registry, its tags and the item components the loom reads.
#[derive(Default)]
pub struct Banners {
    patterns: Vec<Pattern>,
    /// `#minecraft:no_item_required`, in tag order.
    no_item_required: Vec<Arc<str>>,
    /// Item -> `dye` component.
    dyes: HashMap<Arc<str>, Arc<str>>,
    /// Item -> `provides_banner_patterns`, in tag order.
    provided: HashMap<Arc<str>, Vec<Arc<str>>>,
}

fn read_tag(root: &Path, name: &str, depth: u32) -> Vec<Arc<str>> {
    if depth > 8 {
        return Vec::new();
    }
    let path = root.join(format!(
        "data/minecraft/tags/banner_pattern/{}.json",
        name.trim_start_matches("minecraft:")
    ));
    let Some(v) = fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for value in v["values"].as_array().into_iter().flatten() {
        let Some(s) = value.as_str().or_else(|| value["id"].as_str()) else {
            continue;
        };
        match s.strip_prefix('#') {
            Some(tag) => out.extend(read_tag(root, tag, depth + 1)),
            None => out.push(Arc::from(s)),
        }
    }
    out
}

impl Banners {
    pub fn load(root: &Path) -> Self {
        Self::load_with_defaults(root, None)
    }
    pub(crate) fn load_with_defaults(
        root: &Path,
        defaults: Option<&horizon_world::item_defaults::Defaults>,
    ) -> Self {
        let mut banners = Self::default();
        let mut files: Vec<_> = fs::read_dir(root.join("data/minecraft/banner_pattern"))
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "json"))
            .collect();
        files.sort();
        for path in files {
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let Some(v) = fs::read(&path)
                .ok()
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            else {
                continue;
            };
            let asset = v["asset_id"]
                .as_str()
                .unwrap_or(stem)
                .trim_start_matches("minecraft:")
                .to_owned();
            banners.patterns.push(Pattern {
                id: Arc::from(format!("minecraft:{stem}").as_str()),
                asset,
                translation_key: v["translation_key"].as_str().unwrap_or_default().to_owned(),
            });
        }
        banners.no_item_required = read_tag(root, "no_item_required", 0);
        let owned_defaults;
        let defaults = match defaults {
            Some(defaults) => Some(defaults),
            None => {
                owned_defaults = horizon_world::item_defaults::read(root).ok();
                owned_defaults.as_ref()
            }
        };
        if let Some(raw) = defaults {
            for (item, c) in raw {
                let id: Arc<str> = Arc::from(item.as_str());
                if let Some(dye) = c["minecraft:dye"].as_str() {
                    banners.dyes.insert(id.clone(), Arc::from(dye));
                }
                match &c["minecraft:provides_banner_patterns"] {
                    Value::String(tag) if tag.starts_with('#') => {
                        banners.provided.insert(id, read_tag(root, &tag[1..], 0));
                    }
                    Value::String(single) => {
                        banners
                            .provided
                            .insert(id, vec![Arc::from(single.as_str())]);
                    }
                    _ => {}
                }
            }
        }
        // 1.21.4 has no `provides_banner_patterns` component (added in
        // 1.21.5): a `BannerPatternItem` carries its patterns as the
        // `pattern_item/<name>` banner-pattern tag, and the item is
        // `<name>_banner_pattern`. Read the tag directory so the loom
        // knows its pattern items from the version's own data; a tag
        // the component loop already covered keeps its entry.
        if let Ok(dir) = fs::read_dir(root.join("data/minecraft/tags/banner_pattern/pattern_item"))
        {
            let mut files: Vec<_> = dir.flatten().map(|e| e.path()).collect();
            files.sort();
            for path in files {
                let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                if path.extension().is_some_and(|e| e != "json") {
                    continue;
                }
                let id = Arc::from(format!("minecraft:{stem}_banner_pattern").as_str());
                if !banners.provided.contains_key(&id) {
                    banners
                        .provided
                        .insert(id, read_tag(root, &format!("pattern_item/{stem}"), 0));
                }
            }
        }
        banners
    }
    pub fn pattern(&self, id: &str) -> Option<&Pattern> {
        self.patterns.iter().find(|p| &*p.id == id)
    }
    /// `Sheets.getBannerSprite` texture name of a pattern.
    pub fn banner_texture(&self, id: &str) -> String {
        match self.pattern(id) {
            Some(p) => format!("entity/banner/{}", p.asset),
            None => format!("entity/banner/{}", id.trim_start_matches("minecraft:")),
        }
    }
    /// `Sheets.getShieldSprite` texture name of a pattern.
    pub fn shield_texture(&self, id: &str) -> String {
        match self.pattern(id) {
            Some(p) => format!("entity/shield/{}", p.asset),
            None => format!("entity/shield/{}", id.trim_start_matches("minecraft:")),
        }
    }
    /// `BannerPatternLayers.Layer.description` translation key.
    pub fn layer_key(&self, layer: &BannerLayer) -> String {
        let base = self.pattern(&layer.pattern).map_or_else(
            || {
                format!(
                    "block.minecraft.banner.{}",
                    layer.pattern.trim_start_matches("minecraft:")
                )
            },
            |p| p.translation_key.clone(),
        );
        format!("{base}.{}", layer.color)
    }
    /// `LoomMenu$4.mayPlace` in 1.21.4 (and 1.21.5) is `instanceof
    /// DyeItem`, whose colour is the item id's `<colour>_dye` suffix.
    /// The 26.3 form checked the `minecraft:loom_dyes` tag and the
    /// `minecraft:dye` component; neither exists in 1.21.4 data (the
    /// tag is absent and the component arrived in 1.21.5), so fall
    /// back to the id when the component path finds nothing.
    pub fn loom_dye(&self, stack: &ItemStack) -> Option<Arc<str>> {
        if let Some(color) = self.dyes.get(&stack.item).cloned() {
            return Some(color);
        }
        let name = stack
            .item
            .strip_prefix("minecraft:")?
            .strip_suffix("_dye")?;
        DYES.iter()
            .find(|(dye, _)| *dye == name)
            .map(|(dye, _)| Arc::from(*dye))
    }
    /// `LoomMenu$5.mayPlace` in 1.21.4 is `instanceof BannerPatternItem`;
    /// 1.21.5 changed it to `has(PROVIDES_BANNER_PATTERNS)`, and the
    /// `minecraft:loom_patterns` tag the 26.3 form checks does not
    /// exist in 1.21.4 data. `provided` holds exactly the pattern
    /// items (from the `pattern_item/*` tags), so membership is the
    /// instanceof check.
    pub fn is_pattern_item(&self, stack: &ItemStack) -> bool {
        self.provided.contains_key(&stack.item)
    }
    /// `LoomMenu.getSelectablePatterns`.
    pub fn selectable(&self, pattern_item: Option<&ItemStack>) -> Vec<Arc<str>> {
        match pattern_item {
            None => self.no_item_required.clone(),
            Some(stack) => self.provided.get(&stack.item).cloned().unwrap_or_default(),
        }
    }
}

/// `BannerItem`: a banner's dye colour from its item id.
pub fn banner_color(item: &str) -> Option<&'static str> {
    let name = item.strip_prefix("minecraft:")?.strip_suffix("_banner")?;
    DYES.iter().map(|d| d.0).find(|d| *d == name)
}
/// `DyeColor.getTextureDiffuseColor` (RGB) by dye name, as the resource
/// packs draw it (`pack_palette`): banner layers, banner and shield items and the
/// loom follow a pack's dye colours as the banner base does.
pub fn dye_rgb(name: &str) -> u32 {
    DYES.iter()
        .position(|d| d.0 == name)
        .map_or(0xFFFFFF, crate::pack_palette::diffuse)
}
/// The layers of a banner or shield stack.
pub fn layers(stack: &ItemStack) -> &[BannerLayer] {
    stack.data().map_or(&[], |d| &d.banner_patterns)
}
/// The ominous banner's component patch hides its pattern tooltip.
pub fn tooltip_layers(stack: &ItemStack) -> &[BannerLayer] {
    if stack.data().is_some_and(|d| d.ominous_banner) {
        &[]
    } else {
        layers(stack)
    }
}
/// `LoomMenu.setupResultSlot`: one banner with the pattern added in the dye colour.
pub fn loom_result(banner: &ItemStack, dye: &str, pattern: &str) -> ItemStack {
    let mut result = ItemStack {
        count: 1,
        ..banner.clone()
    };
    result
        .data_mut()
        .banner_patterns
        .push(BannerLayer::new(pattern, dye));
    result
}
/// Whether the banner already carries the loom's limit.
pub fn has_max_layers(banner: &ItemStack) -> bool {
    layers(banner).len() >= MAX_LAYERS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn banners() -> Banners {
        let mut b = Banners {
            no_item_required: vec![Arc::from("minecraft:stripe_top")],
            ..Default::default()
        };
        b.provided.insert(
            Arc::from("minecraft:flower_banner_pattern"),
            vec![Arc::from("minecraft:flower")],
        );
        b
    }
    #[test]
    fn selectable_follow_the_pattern_item() {
        let b = banners();
        assert_eq!(b.selectable(None).len(), 1);
        let item = ItemStack::new("minecraft:flower_banner_pattern", 1);
        assert_eq!(&*b.selectable(Some(&item))[0], "minecraft:flower");
    }
    #[test]
    fn pattern_item_is_the_provided_map() {
        let b = banners();
        assert!(b.is_pattern_item(&ItemStack::new("minecraft:flower_banner_pattern", 1)));
        assert!(!b.is_pattern_item(&ItemStack::new("minecraft:stone", 1)));
    }
    #[test]
    fn loom_dye_comes_from_the_item_id() {
        let b = banners();
        assert_eq!(
            b.loom_dye(&ItemStack::new("minecraft:red_dye", 1))
                .as_deref(),
            Some("red")
        );
        assert_eq!(
            b.loom_dye(&ItemStack::new("minecraft:light_gray_dye", 1))
                .as_deref(),
            Some("light_gray")
        );
        assert!(
            b.loom_dye(&ItemStack::new("minecraft:white_wool", 1))
                .is_none()
        );
        assert!(
            b.loom_dye(&ItemStack::new("minecraft:foo_dye", 1))
                .is_none()
        );
    }
    #[test]
    fn pattern_items_load_from_pattern_item_tags() {
        let dir = std::env::temp_dir().join(format!("horizon-loom-test-{}", std::process::id()));
        let tags = dir.join("data/minecraft/tags/banner_pattern/pattern_item");
        fs::create_dir_all(&tags).unwrap();
        fs::write(
            tags.join("creeper.json"),
            r#"{"values": ["minecraft:creeper"]}"#,
        )
        .unwrap();
        let b = Banners::load(&dir);
        let patterns = b.provided.get("minecraft:creeper_banner_pattern");
        assert_eq!(patterns.map(|v| v.len()), Some(1));
        assert_eq!(&*patterns.unwrap()[0], "minecraft:creeper");
        assert!(b.is_pattern_item(&ItemStack::new("minecraft:creeper_banner_pattern", 1)));
        let _ = fs::remove_dir_all(&dir);
    }
    #[test]
    fn banner_colour_comes_from_the_item() {
        assert_eq!(
            banner_color("minecraft:light_gray_banner"),
            Some("light_gray")
        );
        assert_eq!(banner_color("minecraft:white_banner"), Some("white"));
        assert_eq!(banner_color("minecraft:white_wool"), None);
    }
    #[test]
    fn loom_adds_a_layer_up_to_the_limit() {
        let mut banner = ItemStack::new("minecraft:white_banner", 3);
        for i in 0..MAX_LAYERS {
            assert!(!has_max_layers(&banner));
            banner = loom_result(&banner, "red", "minecraft:cross");
            assert_eq!(banner.count, 1);
            assert_eq!(layers(&banner).len(), i + 1);
        }
        assert!(has_max_layers(&banner));
    }
    #[test]
    fn ominous_banner_hides_tooltip_layers_but_keeps_its_patterns() {
        let mut banner = crate::mobs::raider::ominous_banner();
        (banner.damage, banner.extra) = crate::item_data::decode_components(
            crate::item_data::encode_components(&banner).as_ref(),
        );
        assert_eq!(
            layers(&banner).len(),
            crate::item_data::OMINOUS_LAYERS.len()
        );
        assert!(tooltip_layers(&banner).is_empty());
        banner.data_mut().custom_name =
            Some(horizon_world::nbt::Tag::String("Captain's Standard".into()));
        assert!(tooltip_layers(&banner).is_empty());
        banner.data_mut().ominous_banner = false;
        assert_eq!(tooltip_layers(&banner), layers(&banner));
        assert!(!tooltip_layers(&banner).is_empty());
    }

    #[test]
    fn layers_round_trip_through_nbt() {
        let l = vec![
            BannerLayer::new("minecraft:flower", "red"),
            BannerLayer::new("minecraft:border", "black"),
        ];
        assert_eq!(layers_from_nbt(Some(&layers_to_nbt(&l))), l);
    }
}
