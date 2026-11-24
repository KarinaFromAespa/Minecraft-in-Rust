//! Survival rules of the world generator's feature placement: each block's rule from the class
//! vanilla gives it and the class of its placement check (tools/test_block_survival_parity.py's
//! `bind_definitions` and its tables).

use crate::pyjson::Value;

const CLASS_RULES: &[(&str, &str)] = &[
    ("FireBlock", "Fire"),
    ("CactusBlock", "Cactus"),
    ("PathBlock", "Path"),
    ("MushroomBlock", "Mushroom"),
    ("MultifaceBlock", "Multiface"),
    ("VineBlock", "Vine"),
    ("HangingMossBlock", "HangingMoss"),
    ("SeaPickleBlock", "SeaPickle"),
    ("SnowLayerBlock", "SnowLayer"),
    ("BellBlock", "Bell"),
    ("ShelfMushroomBlock", "FullBehind"),
    ("GrowingPlantBlock", "GrowingPlant"),
    ("MangrovePropaguleBlock", "MangrovePropagule"),
    ("SpeleothemBlock", "Speleothem"),
    ("ScaffoldingBlock", "Scaffolding"),
    ("PistonHeadBlock", "PistonHead"),
    ("ChorusPlantBlock", "ChorusPlant"),
    ("ChorusFlowerBlock", "ChorusFlower"),
    ("PitcherCropBlock", "PitcherCrop"),
    ("BigDripleafBlock", "BigDripleaf"),
    ("BigDripleafStemBlock", "BigDripleafStem"),
    // 1.21.10's class is `FarmBlock` (26.1 renamed it `FarmlandBlock`);
    // without the 1.21.10 name farmland binds no survival rule.
    ("FarmBlock", "Farmland"),
    ("CocoaBlock", "Cocoa"),
    ("SugarCaneBlock", "SugarCane"),
    ("SporeBlossomBlock", "SporeBlossom"),
    ("FrogspawnBlock", "Frogspawn"),
    ("SmallDripleafBlock", "SmallDripleaf"),
    ("TallSeagrassBlock", "TallSeagrass"),
    ("DoorBlock", "Door"),
    ("BaseRailBlock", "RigidBelow"),
    ("LeafLitterBlock", "FullBelow"),
    ("WallSignBlock", "SolidBehind"),
    ("GrindstoneBlock", "Always"),
    ("RedstoneWallTorchBlock", "FullBehind"),
    ("BubbleColumnBlock", "BubbleColumn"),
    ("RedstoneWireBlock", "RedstoneWire"),
    ("SoulFireBlock", "TagBelow:minecraft:soul_fire_base_blocks"),
    ("BlockBehaviour", "Always"),
    ("CakeBlock", "SolidBelow"),
    ("CandleCakeBlock", "SolidBelow"),
    ("BannerBlock", "SolidBelow"),
    ("StandingSignBlock", "SolidBelow"),
    ("WallBannerBlock", "SolidBehind"),
    ("CarpetBlock", "NonAirBelow"),
    ("MossyCarpetBlock", "MossCarpet"),
    ("BaseTorchBlock", "CenterBelow"),
    ("CandleBlock", "CenterBelow"),
    ("LanternBlock", "Lantern"),
    ("HangingRootsBlock", "FullAbove"),
    ("LadderBlock", "FullBehind"),
    ("AmethystClusterBlock", "FullBehind"),
    ("TripWireHookBlock", "FullBehind"),
    ("BaseCoralWallFanBlock", "FullBehind"),
    ("WallTorchBlock", "FullBehind"),
    ("BasePressurePlateBlock", "RigidOrCenterBelow"),
    ("DiodeBlock", "RigidBelow"),
    ("BaseCoralPlantTypeBlock", "FullBelow"),
    ("CeilingHangingSignBlock", "CenterAbove"),
    ("FaceAttachedHorizontalDirectionalBlock", "FaceAttached"),
    ("BambooSaplingBlock", "SupportsBamboo"),
    ("BambooStalkBlock", "SupportsBamboo"),
];

const VEGETATION_RULES: &[(&str, &str)] = &[
    ("WaterlilyBlock", "LilyPad"),
    ("VegetationBlock", "Vegetation"),
    ("AzaleaBlock", "TagBelow:minecraft:supports_azalea"),
    (
        "DryVegetationBlock",
        // 1.21.10's tag is `dry_vegetation_may_place_on`; 26.1 renamed it
        // `supports_dry_vegetation` (and changed its contents).
        "TagBelow:minecraft:dry_vegetation_may_place_on",
    ),
    (
        "NetherSproutsBlock",
        "TagBelow:minecraft:supports_nether_sprouts",
    ),
    ("NetherWartBlock", "TagBelow:minecraft:supports_nether_wart"),
    // 1.21.10's wither rose stands on dirt, farmland, netherrack, soul
    // sand or soul soil (26.1 reads the `supports_wither_rose` tag
    // instead, gone in 1.21.10); no single tag says it.
    ("WitherRoseBlock", "WitherRose"),
    ("CactusFlowerBlock", "CactusFlower"),
    ("SeagrassBlock", "Seagrass"),
];

fn lookup(table: &[(&str, &'static str)], key: &str) -> Option<&'static str> {
    table.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

/// `{block: rule}` from the exporter's `{block: class}`, `{block: placement class}` and
/// `{block: {field: support tag}}`, in the order Python's dict built it.
pub fn bind_definitions(
    classes: &Value,
    placement: &Value,
    support_tags: &Value,
) -> Result<Value, String> {
    let classes = classes
        .as_object()
        .ok_or("survival-classes.json is not an object")?;
    let mut definitions = Value::Object(Vec::new());
    for (name, owner) in classes {
        if let Some(rule) = owner.as_str().and_then(|o| lookup(CLASS_RULES, o)) {
            definitions.insert(name, Value::str(rule));
        }
    }
    for (name, owner) in classes {
        let owner = owner.as_str().unwrap_or("");
        let placed = placement.get(name).and_then(Value::as_str);
        if owner == "CropBlock" && placed == Some("CropBlock") {
            definitions.insert(name, Value::str("Crop"));
        }
        if owner == "DoublePlantBlock" && placed == Some("VegetationBlock") {
            definitions.insert(name, Value::str("DoublePlant"));
        }
        if owner != "VegetationBlock" {
            continue;
        }
        let method = placed.ok_or_else(|| format!("No placement class for {name}"))?;
        if let Some(rule) = lookup(VEGETATION_RULES, method) {
            definitions.insert(name, Value::str(rule));
        } else if [
            "AttachedStemBlock",
            "StemBlock",
            "NetherFungusBlock",
            "NetherRootsBlock",
        ]
        .contains(&method)
        {
            let field = if method == "StemBlock" {
                "stemSupportBlocks"
            } else {
                "supportBlocks"
            };
            // Later versions carry the support set in a TagKey field the exporter
            // reads. 1.21.11's stem classes have no such field: their mayPlaceOn
            // hardcodes the block check (farmland only), which the BlockBelow
            // rule form carries. The fungus/root classes hardcode theirs too
            // (item 757); they get no bound rule here rather than a wrong one.
            if let Some(tag) = support_tags
                .get(name)
                .and_then(|t| t.get(field))
                .and_then(Value::as_str)
            {
                definitions.insert(name, Value::Str(format!("TagBelow:{tag}")));
            } else if method == "StemBlock" || method == "AttachedStemBlock" {
                definitions.insert(name, Value::str("BlockBelow:minecraft:farmland"));
            }
        }
    }
    Ok(definitions)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1.21.10's placement class for the lily pad is `WaterlilyBlock`
    /// (26.1 renamed it `LilyPadBlock`); without the 1.21.10 name the
    /// pad binds no survival rule.
    #[test]
    fn lily_pad_binds_its_rule_from_the_1_21_11_placement_class() {
        let classes = Value::Object(vec![(
            "minecraft:lily_pad".to_owned(),
            Value::str("VegetationBlock"),
        )]);
        let placement = Value::Object(vec![(
            "minecraft:lily_pad".to_owned(),
            Value::str("WaterlilyBlock"),
        )]);
        let definitions = bind_definitions(&classes, &placement, &Value::Object(vec![])).unwrap();
        assert_eq!(
            definitions
                .get("minecraft:lily_pad")
                .and_then(Value::as_str),
            Some("LilyPad")
        );
    }

    /// 1.21.10's farmland class is `FarmBlock` (26.1 renamed it
    /// `FarmlandBlock`); without the 1.21.10 name farmland binds no
    /// survival rule.
    #[test]
    fn farmland_binds_its_rule_from_the_1_21_11_class() {
        let classes = Value::Object(vec![(
            "minecraft:farmland".to_owned(),
            Value::str("FarmBlock"),
        )]);
        let definitions =
            bind_definitions(&classes, &Value::Object(vec![]), &Value::Object(vec![])).unwrap();
        assert_eq!(
            definitions
                .get("minecraft:farmland")
                .and_then(Value::as_str),
            Some("Farmland")
        );
    }

    /// 1.21.10's wither rose stands on dirt, farmland, netherrack, soul
    /// sand or soul soil; 26.1 reads the `supports_wither_rose` tag
    /// instead, which does not exist in 1.21.10, so the old name bound an
    /// empty tag and wither roses could never survive.
    #[test]
    fn wither_rose_binds_its_own_rule_not_an_empty_tag() {
        let classes = Value::Object(vec![(
            "minecraft:wither_rose".to_owned(),
            Value::str("VegetationBlock"),
        )]);
        let placement = Value::Object(vec![(
            "minecraft:wither_rose".to_owned(),
            Value::str("WitherRoseBlock"),
        )]);
        let definitions = bind_definitions(&classes, &placement, &Value::Object(vec![])).unwrap();
        assert_eq!(
            definitions
                .get("minecraft:wither_rose")
                .and_then(Value::as_str),
            Some("WitherRose")
        );
    }

    /// 1.21.10's dry-vegetation support tag is
    /// `dry_vegetation_may_place_on`; 26.1 renamed it
    /// `supports_dry_vegetation`, which does not exist in 1.21.10, so the
    /// old name bound an empty tag and dry grass could never survive.
    #[test]
    fn dry_vegetation_binds_the_1_21_11_support_tag() {
        let classes = Value::Object(vec![(
            "minecraft:short_dry_grass".to_owned(),
            Value::str("VegetationBlock"),
        )]);
        let placement = Value::Object(vec![(
            "minecraft:short_dry_grass".to_owned(),
            Value::str("DryVegetationBlock"),
        )]);
        let definitions = bind_definitions(&classes, &placement, &Value::Object(vec![])).unwrap();
        assert_eq!(
            definitions
                .get("minecraft:short_dry_grass")
                .and_then(Value::as_str),
            Some("TagBelow:minecraft:dry_vegetation_may_place_on")
        );
    }
}
