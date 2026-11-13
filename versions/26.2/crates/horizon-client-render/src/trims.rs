//! Armour trims on worn armour: `EquipmentLayerRenderer`'s trim layer. A
//! trim's texture is `textures/trims/entity/<layer>/<pattern>.png` recoloured
//! by palette swap: every pixel whose colour is in the base palette
//! (`textures/trims/color_palettes/trim_palette.png`, the `armor_trims` atlas's
//! `palette_key`) takes the colour at the same index of the material's palette
//! (`textures/trims/color_palettes/<asset_name>.png`), where `asset_name` and
//! the per-armour-asset darker override come from the material's
//! `trim_material` data (`override_armor_assets`: iron armour with an iron
//! trim uses `iron_darker`).
use horizon_assets::palette::apply_palette;
use serde_json::Value;
use std::{collections::HashMap, fs, path::Path};

/// Runtime trim slots and the completed ordered requests for their current atlas.
/// A frame with more requests than slots keeps the same final residents as the
/// sequential LRU pass, without decoding and evicting them again every frame.
#[derive(Default)]
pub struct TrimSlots {
    slots: Vec<(String, usize)>,
    atlas_revision: u64,
    prepared: Option<Vec<TrimTexture>>,
}

impl TrimSlots {
    /// The renderer was rebuilt: runtime pixels must be uploaded again.
    pub fn forget(&mut self) {
        self.atlas_revision = 0;
        self.prepared = None;
    }

    pub fn prepare(
        &mut self,
        wanted: Vec<TrimTexture>,
        root: &Path,
        atlas: &mut crate::hud::HudAtlas,
        patches: &mut Vec<crate::font::Patch>,
    ) {
        if self.atlas_revision != atlas.revision {
            for (name, _) in self.slots.drain(..) {
                atlas.sprites.remove(&name);
            }
            self.atlas_revision = atlas.revision;
            self.prepared = None;
        }
        let before = self.slots.len();
        self.slots
            .retain(|(name, _)| atlas.sprites.contains_key(name));
        if before != self.slots.len() {
            self.prepared = None;
        }
        if self.prepared.as_ref() == Some(&wanted) {
            return;
        }
        let mut complete = true;
        for t in &wanted {
            let name = t.sprite();
            if let Some(i) = self.slots.iter().position(|e| e.0 == name) {
                let e = self.slots.remove(i);
                self.slots.push(e);
                continue;
            }
            let Some((w, h, rgba)) = t.generate(root) else {
                complete = false;
                continue;
            };
            let slot =
                match (0..crate::hud::TRIM_SLOTS).find(|i| !self.slots.iter().any(|e| e.1 == *i)) {
                    Some(free) => free,
                    None => {
                        let (old, slot) = self.slots.remove(0);
                        atlas.sprites.remove(&old);
                        slot
                    }
                };
            let Some(area) = atlas.sprites.get(&format!("horizon/trim{slot}")).copied() else {
                complete = false;
                continue;
            };
            if w > area.w || h > area.h {
                complete = false;
                continue;
            }
            let (aw, ah) = (atlas.width as f32, atlas.height as f32);
            let (x, y) = ((area.u0 * aw).round(), (area.v0 * ah).round());
            patches.push(crate::font::Patch {
                x: x as u32,
                y: y as u32,
                width: w,
                height: h,
                rgba,
            });
            atlas.sprites.insert(
                name.clone(),
                crate::hud::Rect {
                    u0: area.u0,
                    v0: area.v0,
                    u1: (x + w as f32) / aw,
                    v1: (y + h as f32) / ah,
                    w,
                    h,
                },
            );
            self.slots.push((name, slot));
        }
        // Failed or not-yet-available textures retain their per-frame retries.
        self.prepared = complete.then_some(wanted);
    }
}

/// An RGBA image: width, height, pixels.
pub type Image = (u32, u32, Vec<u8>);

/// `minecraft:` added to an identifier without a namespace.
fn qualified(id: &str) -> String {
    if id.contains(':') {
        id.to_owned()
    } else {
        format!("minecraft:{id}")
    }
}

/// `assets/<namespace>/<folder>/<path><suffix>` of an identifier.
fn asset_path(root: &Path, folder: &str, id: &str, suffix: &str) -> std::path::PathBuf {
    let id = qualified(id);
    let (namespace, path) = id.split_once(':').unwrap_or(("minecraft", &id));
    root.join("assets")
        .join(namespace)
        .join(folder)
        .join(format!("{path}{suffix}"))
}

/// The `trim_pattern` (asset id) and `trim_material` (asset name, plus its
/// `override_armor_assets`: armour asset id -> darker asset name) registries.
#[derive(Clone, Debug, Default)]
pub struct TrimRegistry {
    patterns: HashMap<String, String>,
    materials: HashMap<String, String>,
    overrides: HashMap<(String, String), String>,
}
impl TrimRegistry {
    /// Every `data/<namespace>/trim_pattern` and `trim_material` entry.
    pub fn load(root: &Path) -> Self {
        let mut out = Self::default();
        let Ok(namespaces) = fs::read_dir(root.join("data")) else {
            return out;
        };
        for ns in namespaces.flatten() {
            let namespace = ns.file_name().to_string_lossy().into_owned();
            for folder in ["trim_pattern", "trim_material"] {
                let Ok(files) = fs::read_dir(ns.path().join(folder)) else {
                    continue;
                };
                for f in files.flatten() {
                    let path = f.path();
                    if path.extension().is_none_or(|e| e != "json") {
                        continue;
                    }
                    let Some(value) = fs::read(&path)
                        .ok()
                        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                    else {
                        continue;
                    };
                    let name = path.file_stem().unwrap_or_default().to_string_lossy();
                    let id = format!("{namespace}:{name}");
                    if folder == "trim_pattern" {
                        if let Some(v) = value.get("asset_id").and_then(Value::as_str) {
                            out.patterns.insert(id, qualified(v));
                        }
                    } else {
                        if let Some(v) = value.get("asset_name").and_then(Value::as_str) {
                            out.materials.insert(id.clone(), v.to_owned());
                        }
                        if let Some(over) = value
                            .get("override_armor_assets")
                            .and_then(Value::as_object)
                        {
                            for (asset, darker) in over {
                                if let Some(darker) = darker.as_str() {
                                    out.overrides.insert(
                                        (id.clone(), qualified(asset)),
                                        darker.to_owned(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        out
    }
    #[cfg(test)]
    fn of(
        patterns: &[(&str, &str)],
        materials: &[(&str, &str)],
        overrides: &[(&str, &str, &str)],
    ) -> Self {
        let map = |l: &[(&str, &str)]| {
            l.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        };
        Self {
            patterns: map(patterns),
            materials: map(materials),
            overrides: overrides
                .iter()
                .map(|(m, a, d)| ((m.to_string(), a.to_string()), d.to_string()))
                .collect(),
        }
    }
}

/// The texture and palette of one trim layer.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TrimTexture {
    /// `<namespace>:trims/entity/<layer>/<path>`, under `textures/`.
    pub texture: String,
    /// The target palette's asset name under
    /// `textures/trims/color_palettes/`; none draws the texture's own
    /// colours.
    pub palette: Option<String>,
}
impl TrimTexture {
    /// `TrimTextureKey.getOrPrepareTexture`: the trim (`material`,
    /// `pattern` registry ids) on the `layer` (`humanoid`,
    /// `humanoid_leggings`...) of an equipment `asset`; the material's
    /// `override_armor_assets` picks the darker palette for the asset it
    /// names (iron armour with an iron trim uses `iron_darker`).
    pub fn resolve(
        registry: &TrimRegistry,
        asset: &str,
        material: &str,
        pattern: &str,
        layer: &str,
    ) -> Option<Self> {
        let (material, pattern) = (qualified(material), qualified(pattern));
        let texture = registry.patterns.get(&pattern)?.clone();
        let mut palette = registry.materials.get(&material)?.clone();
        if let Some(darker) = registry
            .overrides
            .get(&(material, qualified(asset)))
        {
            palette.clone_from(darker);
        }
        let (namespace, path) = texture.split_once(':')?;
        Some(Self {
            texture: format!("{namespace}:trims/entity/{layer}/{path}"),
            palette: Some(palette),
        })
    }
    /// The atlas sprite name the generated texture is stored under.
    pub fn sprite(&self) -> String {
        format!(
            "horizon/trim/{}/{}",
            self.texture,
            self.palette.as_deref().unwrap_or("-")
        )
    }
    /// The trim texture with its base palette's colours swapped for the
    /// target palette's (`PalettedPermutations`' mapping, keyed on the
    /// `armor_trims` atlas's `trim_palette`); unchanged when a palette is
    /// missing or the sizes differ. None when the texture itself is
    /// missing.
    pub fn generate(&self, root: &Path) -> Option<Image> {
        let path = asset_path(root, "textures", &self.texture, ".png");
        let (w, h, pixels) = crate::hud::decode(&path).ok()?;
        let Some(palette) = &self.palette else {
            return Some((w, h, pixels));
        };
        let load = |id: &str| {
            crate::hud::decode(&asset_path(root, "textures/trims/color_palettes", id, ".png"))
                .ok()
                .map(|(_, _, p)| argb(&p))
        };
        let mut mapped = argb(&pixels);
        if let (Some(key), Some(target)) = (load("trim_palette"), load(palette)) {
            // Different sizes: no mapping, the texture as it is.
            let _ = apply_palette(&mut mapped, &key, &target);
        }
        let out = mapped
            .iter()
            .flat_map(|p| {
                let [a, r, g, b] = p.to_be_bytes();
                [r, g, b, a]
            })
            .collect();
        Some((w, h, out))
    }
}

fn argb(rgba: &[u8]) -> Vec<u32> {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .map(|p| u32::from_be_bytes([p[3], p[0], p[1], p[2]]))
        .collect()
}

/// The largest trim texture of the resources (the size of the atlas's trim
/// slots), at least vanilla's 64x32.
pub fn slot_size(root: &Path) -> (u32, u32) {
    let mut size = (64, 32);
    let Ok(namespaces) = fs::read_dir(root.join("assets")) else {
        return size;
    };
    for ns in namespaces.flatten() {
        let Ok(layers) = fs::read_dir(ns.path().join("textures/trims/entity")) else {
            continue;
        };
        for layer in layers.flatten() {
            for f in fs::read_dir(layer.path()).into_iter().flatten().flatten() {
                let path = f.path();
                if path.extension().is_some_and(|e| e == "png")
                    && let Some((w, h)) = png_size(&path)
                {
                    size = (size.0.max(w.min(512)), size.1.max(h.min(512)));
                }
            }
        }
    }
    size
}

/// The width and height in a PNG's IHDR chunk.
fn png_size(path: &Path) -> Option<(u32, u32)> {
    let head: [u8; 24] = horizon_core::prepared_files::read_head(path, 24)
        .ok()?
        .try_into()
        .ok()?;
    let be = |i: usize| u32::from_be_bytes(head[i..i + 4].try_into().unwrap());
    (&head[12..16] == b"IHDR").then(|| (be(16), be(20)))
}

#[cfg(test)]
mod slots_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> TrimRegistry {
        TrimRegistry::of(
            &[("minecraft:coast", "minecraft:coast")],
            &[("minecraft:iron", "iron"), ("minecraft:gold", "gold")],
            &[
                ("minecraft:iron", "minecraft:iron", "iron_darker"),
                ("minecraft:gold", "minecraft:gold", "gold_darker"),
            ],
        )
    }

    #[test]
    fn material_palette_and_layer_folder() {
        let t = TrimTexture::resolve(
            &registry(),
            "minecraft:diamond",
            "minecraft:gold",
            "minecraft:coast",
            "humanoid_leggings",
        )
        .unwrap();
        assert_eq!(t.texture, "minecraft:trims/entity/humanoid_leggings/coast");
        assert_eq!(t.palette.as_deref(), Some("gold"));
        // Unknown registry entries draw nothing.
        assert!(
            TrimTexture::resolve(
                &registry(),
                "minecraft:diamond",
                "minecraft:wood",
                "minecraft:coast",
                "humanoid"
            )
            .is_none()
        );
    }

    #[test]
    fn override_armor_assets_pick_the_darker_palette() {
        let resolve = |asset: &str, m: &str| {
            TrimTexture::resolve(&registry(), asset, m, "coast", "humanoid").unwrap()
        };
        // Gold trim on gold armour uses the darker palette; on other armour
        // it uses the material's own.
        assert_eq!(
            resolve("minecraft:gold", "gold").palette.as_deref(),
            Some("gold_darker")
        );
        assert_eq!(
            resolve("minecraft:iron", "gold").palette.as_deref(),
            Some("gold")
        );
        assert_eq!(
            resolve("minecraft:iron", "iron").palette.as_deref(),
            Some("iron_darker")
        );
    }

    #[test]
    fn vanilla_trims_are_recoloured() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/horizon-client/26.2");
        if !root.join("assets/minecraft/textures/trims").is_dir() {
            return;
        }
        let registry = TrimRegistry::load(&root);
        let t = TrimTexture::resolve(
            &registry,
            "minecraft:diamond",
            "minecraft:gold",
            "minecraft:coast",
            "humanoid",
        )
        .unwrap();
        let (w, h, gold) = t.generate(&root).unwrap();
        assert_eq!((w, h), (64, 32));
        let plain = TrimTexture {
            palette: None,
            ..t.clone()
        };
        let (_, _, grey) = plain.generate(&root).unwrap();
        // Every opaque pixel of the grey pattern takes a gold palette colour.
        let gold_palette = argb(
            &crate::hud::decode(
                &root.join("assets/minecraft/textures/trims/color_palettes/gold.png"),
            )
            .unwrap()
            .2,
        );
        let mut opaque = 0;
        for (g, c) in grey.chunks(4).zip(gold.chunks(4)) {
            if g[3] != 0 {
                opaque += 1;
                let c = u32::from_be_bytes([0xFF, c[0], c[1], c[2]]);
                assert!(gold_palette.contains(&c), "{c:08x}");
            }
        }
        assert!(opaque > 0);
        assert_eq!(slot_size(&root), (64, 32));
    }
}
