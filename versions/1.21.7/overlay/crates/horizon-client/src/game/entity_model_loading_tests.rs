//! Model-batch equivalence tests and an opt-in benchmark of the actual loaders.
use super::{armor_stands::ArmorStandAssets, boats::BoatModels, minecarts::CartModels};
use horizon_client::mobs::{EntityModelDocument, Part, load_model};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, hint::black_box, path::Path, time::Instant};

const WOODS: [&str; 11] = [
    "oak",
    "spruce",
    "birch",
    "jungle",
    "acacia",
    "dark_oak",
    "mangrove",
    "cherry",
    "bamboo",
    "pale_oak",
    "test_wood",
];
const CARTS: [&str; 7] = [
    "minecart",
    "chest_minecart",
    "hopper_minecart",
    "furnace_minecart",
    "tnt_minecart",
    "command_block_minecart",
    "spawner_minecart",
];

fn fixture() -> Value {
    let mut all = serde_json::Map::new();
    let part = json!({
        "pose": [1,2,3,0.25,-0.5,0.75,1,2,0.5],
        "polygons": [[[0,0,0,0,0],[1,0,0,1,0],[1,1,0,1,1],[0,1,0,0,1]]],
        "normals": [[0,0,1]],
        "children": {"root": {"children": {"bone": {"pose": [4,5,6]}}}, "other": {}}
    });
    for wood in WOODS {
        for prefix in ["boat", "chest_boat"] {
            all.insert(format!("minecraft:{prefix}/{wood}#main"), part.clone());
        }
    }
    for cart in CARTS {
        all.insert(format!("minecraft:{cart}#main"), part.clone());
    }
    // Vanilla defines no per-slot armour layers, only the inner/outer
    // models (`HumanoidArmorLayer`).
    for model in ["player", "player_slim", "armor_stand", "armor_stand_small"] {
        for layer in ["inner_armor", "outer_armor"] {
            all.insert(format!("minecraft:{model}#{layer}"), part.clone());
        }
    }
    for model in ["armor_stand", "armor_stand_small"] {
        all.insert(format!("minecraft:{model}#main"), part.clone());
    }
    // Discovery must exclude other models and non-main boat layers.
    all.insert("minecraft:boat/oak#water_patch".into(), part.clone());
    all.insert("minecraft:unrelated#main".into(), part);
    Value::Object(all)
}

struct FixtureRoot(std::path::PathBuf);
impl FixtureRoot {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let next = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "horizon-entity-models-{}-{next}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for FixtureRoot {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

// Preserve the previous production algorithm: one discovery parse, then a full
// file read/parse through load_model for each of the 42 selected layers.
fn baseline(root: &Path) -> BTreeMap<String, Part> {
    let mut models = BTreeMap::new();
    if let Ok(bytes) = fs::read(root.join("entity-models.json"))
        && let Ok(Value::Object(all)) = serde_json::from_slice::<Value>(&bytes)
    {
        for key in all.keys() {
            for (prefix, label) in [
                ("minecraft:boat/", "boat"),
                ("minecraft:chest_boat/", "chest_boat"),
            ] {
                if let Some(wood) = key
                    .strip_prefix(prefix)
                    .and_then(|k| k.strip_suffix("#main"))
                    && let Some(part) = load_model(root, key)
                {
                    models.insert(format!("{label}/{wood}"), part);
                }
            }
        }
    }
    for cart in CARTS {
        if let Some(part) = load_model(root, &format!("minecraft:{cart}#main")) {
            models.insert(format!("cart/{cart}"), part);
        }
    }
    // The production loader maps each armour slot to vanilla's inner
    // (leggings) or outer (the rest) model; the baseline does the same
    // mapping directly. Production keys the slim-skinned player's slots
    // `slim_<slot>`, so the baseline uses the same keys.
    for (model, prefix) in [
        ("player", ""),
        ("player_slim", "slim_"),
        ("armor_stand", ""),
        ("armor_stand_small", ""),
    ] {
        for (slot, layer) in [
            ("helmet", "outer_armor"),
            ("chestplate", "outer_armor"),
            ("leggings", "inner_armor"),
            ("boots", "outer_armor"),
        ] {
            if let Some(part) = load_model(root, &format!("minecraft:{model}#{layer}")) {
                let label = if model.starts_with("armor_stand") {
                    format!("{model}/{slot}")
                } else {
                    format!("player/{prefix}{slot}")
                };
                models.insert(label, part);
            }
        }
    }
    // The fixture contains only entity-models.json; armor equipment is absent.
    let _ = fs::read_dir(root.join("assets/minecraft/equipment"));
    for model in ["armor_stand", "armor_stand_small"] {
        if let Some(part) = load_model(root, &format!("minecraft:{model}#main")) {
            models.insert(format!("{model}/main"), part);
        }
    }
    models
}

fn optimized(root: &Path) -> BTreeMap<String, Part> {
    let document = EntityModelDocument::load(root).ok();
    let boats = BoatModels::load(document.as_ref());
    let carts = CartModels::load(document.as_ref());
    let armor = ArmorStandAssets::load(root, document.as_ref());
    let mut models = BTreeMap::new();
    for (label, group) in [("boat", boats.models), ("chest_boat", boats.chest_models)] {
        for (wood, part) in group {
            models.insert(format!("{label}/{wood}"), part);
        }
    }
    for (kind, part) in carts.models {
        models.insert(
            format!("cart/{}", kind.id().trim_start_matches("minecraft:")),
            part,
        );
    }
    for ((slot, small), part) in armor.layers {
        let model = if small {
            "armor_stand_small"
        } else {
            "armor_stand"
        };
        models.insert(
            format!("{model}/{slot}"),
            std::sync::Arc::unwrap_or_clone(part),
        );
    }
    for (slot, part) in armor.player {
        models.insert(format!("player/{slot}"), part);
    }
    for (model, part) in [
        ("armor_stand", armor.main),
        ("armor_stand_small", armor.small),
    ] {
        if let Some(part) = part {
            models.insert(
                format!("{model}/main"),
                std::sync::Arc::unwrap_or_clone(part),
            );
        }
    }
    models
}

fn assert_equivalent(root: &Path) {
    // Debug includes every part name, pose, polygon normal/vertex and child.
    assert_eq!(
        format!("{:?}", baseline(root)),
        format!("{:?}", optimized(root))
    );
}

#[test]
fn entity_model_batches_preserve_parts_missing_layers_and_load_failures() {
    let root = FixtureRoot::new();
    assert_equivalent(&root.0); // missing file
    let file = root.0.join("entity-models.json");
    for bytes in [b"{".as_slice(), b"null", b"[]", b"{}"] {
        fs::write(&file, bytes).unwrap();
        assert_equivalent(&root.0);
    }
    let mut all = fixture();
    fs::write(&file, serde_json::to_vec(&all).unwrap()).unwrap();
    assert_equivalent(&root.0);
    assert_eq!(optimized(&root.0).len(), 47);
    all.as_object_mut()
        .unwrap()
        .remove("minecraft:armor_stand_small#main");
    all.as_object_mut()
        .unwrap()
        .remove("minecraft:boat/oak#main");
    fs::write(&file, serde_json::to_vec(&all).unwrap()).unwrap();
    assert_equivalent(&root.0);
    assert_eq!(optimized(&root.0).len(), 45);
    let document = EntityModelDocument::load(&root.0).unwrap();
    let mut first = document.model("minecraft:armor_stand#main").unwrap();
    first.children.clear();
    assert!(
        !document
            .model("minecraft:armor_stand#main")
            .unwrap()
            .children
            .is_empty()
    );
}

#[test]
fn armor_equipment_loads_without_an_entity_model_document() {
    let root = FixtureRoot::new();
    let equipment = root.0.join("assets/minecraft/equipment");
    fs::create_dir_all(&equipment).unwrap();
    fs::write(
        equipment.join("leather.json"),
        serde_json::to_vec(&json!({"layers": {"humanoid": [
            {"texture": "minecraft:leather", "dyeable": {"color_when_undyed": 0x123456}},
            {"texture": "minecraft:leather_overlay"}
        ]}}))
        .unwrap(),
    )
    .unwrap();
    fs::write(equipment.join("invalid.json"), b"{").unwrap();
    let assets = ArmorStandAssets::load(&root.0, None);
    assert!(assets.main.is_none());
    assert!(assets.layers.is_empty());
    assert_eq!(
        assets.equipment["minecraft:leather"]["humanoid"],
        vec![
            ("entity/equipment/humanoid/leather".into(), Some(0x123456)),
            ("entity/equipment/humanoid/leather_overlay".into(), None),
        ]
    );
    assert!(!assets.equipment.contains_key("minecraft:invalid"));
}

#[test]
fn armor_models_come_from_vanillas_inner_and_outer_layers() {
    // Vanilla 1.21.7 defines no per-slot armour layers: the prepared
    // `entity-models.json` carries only `player#inner_armor` /
    // `player#outer_armor` (and the `player_slim` / `armor_stand` pairs).
    // Every slot must still resolve — leggings from the inner model, the
    // rest from the outer model — or worn armour is silently skipped.
    let root = FixtureRoot::new();
    // Distinct parts per layer, so each slot can be traced to its source.
    let part = |n: f32| {
        json!({
            "pose": [n, 0, 0, 0, 0, 0, 1, 1, 1],
            "polygons": [[[0,0,0,0,0],[1,0,0,1,0],[1,1,0,1,1],[0,1,0,0,1]]],
            "normals": [[0,0,1]],
            "children": {}
        })
    };
    let mut all = serde_json::Map::new();
    for (model, n) in [
        ("player", 1.0),
        ("player_slim", 2.0),
        ("armor_stand", 3.0),
        ("armor_stand_small", 4.0),
    ] {
        all.insert(format!("minecraft:{model}#inner_armor"), part(n));
        all.insert(format!("minecraft:{model}#outer_armor"), part(n + 0.5));
    }
    fs::write(
        root.0.join("entity-models.json"),
        serde_json::to_vec(&Value::Object(all)).unwrap(),
    )
    .unwrap();
    // Vanilla 1.21.7's `assets/minecraft/equipment/iron.json`, verbatim.
    let equipment = root.0.join("assets/minecraft/equipment");
    fs::create_dir_all(&equipment).unwrap();
    fs::write(
        equipment.join("iron.json"),
        br#"{
  "layers": {
    "horse_body": [{"texture": "minecraft:iron"}],
    "humanoid": [{"texture": "minecraft:iron"}],
    "humanoid_leggings": [{"texture": "minecraft:iron"}]
  }
}"#,
    )
    .unwrap();
    let document = EntityModelDocument::load(&root.0).unwrap();
    let assets = ArmorStandAssets::load(&root.0, Some(&document));
    let debug = |key: &str| format!("{:?}", document.model(key).unwrap());
    // Every slot resolves, from the inner/outer layer vanilla defines.
    for slot in ["helmet", "chestplate", "leggings", "boots"] {
        assert!(assets.player.contains_key(slot), "missing player/{slot}");
        assert!(
            assets.player.contains_key(format!("slim_{slot}").as_str()),
            "missing player/slim_{slot}"
        );
    }
    assert_eq!(
        format!("{:?}", assets.player["leggings"]),
        debug("minecraft:player#inner_armor")
    );
    assert_eq!(
        format!("{:?}", assets.player["helmet"]),
        debug("minecraft:player#outer_armor")
    );
    assert_eq!(
        format!("{:?}", assets.player["chestplate"]),
        debug("minecraft:player#outer_armor")
    );
    assert_eq!(
        format!("{:?}", assets.player["boots"]),
        debug("minecraft:player#outer_armor")
    );
    assert_eq!(
        format!("{:?}", assets.player["slim_leggings"]),
        debug("minecraft:player_slim#inner_armor")
    );
    assert_eq!(
        format!("{:?}", assets.player["slim_helmet"]),
        debug("minecraft:player_slim#outer_armor")
    );
    // The armour-stand layers map the same way, for both sizes.
    assert_eq!(
        format!("{:?}", assets.layers[&("leggings", false)]),
        debug("minecraft:armor_stand#inner_armor")
    );
    assert_eq!(
        format!("{:?}", assets.layers[&("helmet", true)]),
        debug("minecraft:armor_stand_small#outer_armor")
    );
    // The equipment table reads the asset's layers.
    assert_eq!(
        assets.equipment["minecraft:iron"]["humanoid"],
        vec![("entity/equipment/humanoid/iron".to_string(), None)]
    );
    assert_eq!(
        assets.equipment["minecraft:iron"]["humanoid_leggings"],
        vec![("entity/equipment/humanoid_leggings/iron".to_string(), None)]
    );
}

#[test]
fn content_mods_equipment_loads_under_its_namespace() {
    let root = FixtureRoot::new();
    let equipment = root.0.join("assets/gems/equipment");
    fs::create_dir_all(&equipment).unwrap();
    fs::write(
        equipment.join("ruby.json"),
        serde_json::to_vec(&json!({"layers": {
            "humanoid": [{"texture": "gems:ruby"}],
            "humanoid_leggings": [{"texture": "gems:ruby"}]
        }}))
        .unwrap(),
    )
    .unwrap();
    // Not a content root: other namespaces stay unread, as before.
    assert!(ArmorStandAssets::load(&root.0, None).equipment.is_empty());
    fs::write(
        root.0.join(horizon_items::content_namespaces::MARKER),
        br#"["gems"]"#,
    )
    .unwrap();
    let assets = ArmorStandAssets::load(&root.0, None);
    assert_eq!(
        assets.equipment["gems:ruby"]["humanoid_leggings"],
        vec![("gems:entity/equipment/humanoid_leggings/ruby".into(), None)]
    );
}

#[test]
#[ignore = "opt-in benchmark; requires ENTITY_MODEL_BENCH_DOCUMENT"]
fn entity_model_document_benchmark() {
    let input = std::env::var("ENTITY_MODEL_BENCH_DOCUMENT").unwrap();
    let root = FixtureRoot::new();
    let bytes = fs::read(&input).unwrap();
    fs::write(root.0.join("entity-models.json"), &bytes).unwrap();
    assert_equivalent(&root.0);
    let model_count = optimized(&root.0).len();
    let repetitions: usize = std::env::var("ENTITY_MODEL_BENCH_REPETITIONS")
        .unwrap_or_else(|_| "15".into())
        .parse()
        .unwrap();
    assert!(repetitions > 0);
    for _ in 0..3 {
        black_box(baseline(black_box(&root.0)));
        black_box(optimized(black_box(&root.0)));
    }
    let mut before = Vec::new();
    let mut after = Vec::new();
    for i in 0..repetitions {
        for optimized_first in [i % 2 == 0, i % 2 != 0] {
            let started = Instant::now();
            if optimized_first {
                black_box(optimized(black_box(&root.0)));
                after.push(started.elapsed().as_secs_f64() * 1000.0);
            } else {
                black_box(baseline(black_box(&root.0)));
                before.push(started.elapsed().as_secs_f64() * 1000.0);
            }
        }
    }
    before.sort_by(f64::total_cmp);
    after.sort_by(f64::total_cmp);
    let old = before[repetitions / 2];
    let new = after[repetitions / 2];
    println!(
        "{}",
        json!({
            "document_bytes": bytes.len(), "selected_models": model_count,
            "warmup_pairs": 3, "repetitions": repetitions,
            "baseline_ms": before, "optimized_ms": after,
            "baseline_median_ms": old, "optimized_median_ms": new, "speedup": old / new
        })
    );
}
