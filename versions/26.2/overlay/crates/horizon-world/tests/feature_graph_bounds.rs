use horizon_core::random::{LegacyRandom, RandomSource};
use horizon_world::{
    heightmap::HeightmapType,
    registry::BlockStateId,
    worldgen::{
        anchor::WorldGenerationContext,
        feature::{FeatureBlocks, FeatureWorld},
        feature_graph::{FeatureDefinition, FeatureExecutionError, PlacedFeatureDefinition},
        placement::{PlacementContext, PlacementQueryError},
    },
};
struct World;
impl PlacementContext for World {
    fn generation(&self) -> WorldGenerationContext {
        WorldGenerationContext::new(0, 16, 8)
    }
    fn min_y(&mut self) -> Result<i32, PlacementQueryError> {
        Ok(0)
    }
    fn height(&mut self, _: HeightmapType, _: i32, _: i32) -> Result<i32, PlacementQueryError> {
        panic!("no-op graph queried height")
    }
}
impl FeatureWorld for World {
    type Error = std::convert::Infallible;
    fn block(&mut self, _: [i32; 3]) -> Result<BlockStateId, Self::Error> {
        panic!("no-op graph read blocks")
    }
    fn set_block(&mut self, _: [i32; 3], _: BlockStateId, _: i32) -> Result<bool, Self::Error> {
        panic!("no-op graph wrote blocks")
    }
}
fn blocks() -> FeatureBlocks {
    FeatureBlocks {
        air: BlockStateId(0),
        stone: BlockStateId(1),
        cobblestone: BlockStateId(2),
        end_stone: BlockStateId(3),
        obsidian: BlockStateId(4),
    }
}
#[test]
fn large_graphs_fail_at_a_shared_visit_bound_without_drawing_randomness() {
    let leaf = PlacedFeatureDefinition {
        feature: Box::new(FeatureDefinition::NoOp),
        placement: vec![],
    };
    let branch = PlacedFeatureDefinition {
        feature: Box::new(FeatureDefinition::Sequence(vec![leaf; 8192])),
        placement: vec![],
    };
    let tree = FeatureDefinition::Sequence(vec![branch; 9]);
    let mut random = LegacyRandom::new(5);
    let mut expected = random.clone();
    assert!(matches!(
        tree.place(&mut World, &mut random, [0; 3], blocks()),
        Err(FeatureExecutionError::WorkLimit)
    ));
    assert_eq!(random.next_long(), expected.next_long());
}
#[test]
fn nested_inline_decoding_rejects_excessive_depth() {
    let mut value = String::from(r#"{"type":"no_op"}"#);
    for _ in 0..70 {
        value =
            format!(r#"{{"type":"sequence","features":[{{"feature":{value},"placement":[]}}]}}"#);
    }
    assert!(FeatureDefinition::from_json(&value).is_err());
}

#[test]
fn modifier_expansion_without_leaf_calls_uses_the_shared_graph_budget() {
    use horizon_world::worldgen::{placement::PlacementModifier, value_provider::IntProvider};
    let feature = PlacedFeatureDefinition {
        feature: Box::new(FeatureDefinition::NoOp),
        placement: vec![
            PlacementModifier::Count(IntProvider::Constant(4096)),
            PlacementModifier::Count(IntProvider::Constant(4096)),
            PlacementModifier::Count(IntProvider::Constant(0)),
        ],
    };
    let mut random = LegacyRandom::new(5);
    let mut expected = random.clone();
    assert!(matches!(
        feature.place(&mut World, &mut random, [0; 3], blocks()),
        Err(FeatureExecutionError::WorkLimit)
    ));
    assert_eq!(random.next_long(), expected.next_long());
}

#[test]
fn wrapped_26_2_configured_features_decode_like_the_flat_form() {
    // 26.2 wraps every configured feature's parameters in `config`
    // (`{type, config}`); 26.3's record form is flat. Both must decode
    // to the same feature. (Empty feature lists are rejected by
    // design, so each list holds one no-op placed feature.)
    let leaf = r#"{"feature":{"type":"minecraft:no_op"},"placement":[]}"#;
    let wrapped_sequence = format!(
        r#"{{"type":"minecraft:sequence","config":{{"features":[{leaf}]}}}}"#
    );
    assert!(matches!(
        FeatureDefinition::from_json(&wrapped_sequence),
        Ok(FeatureDefinition::Sequence(features)) if features.len() == 1
    ));
    let flat_sequence = format!(r#"{{"type":"minecraft:sequence","features":[{leaf}]}}"#);
    assert!(matches!(
        FeatureDefinition::from_json(&flat_sequence),
        Ok(FeatureDefinition::Sequence(features)) if features.len() == 1
    ));
    let wrapped_selector = format!(
        r#"{{"type":"minecraft:random_selector","config":{{"features":[{{"chance":0.5,"feature":{leaf}}}],"default":{leaf}}}}}"#
    );
    assert!(matches!(
        FeatureDefinition::from_json(&wrapped_selector),
        Ok(FeatureDefinition::ChanceChoice { features, fallback })
            if features.len() == 1
                && matches!(&*fallback.feature, FeatureDefinition::NoOp)
    ));
    let flat_selector = format!(
        r#"{{"type":"minecraft:random_selector","features":[{{"chance":0.5,"feature":{leaf}}}],"default":{leaf}}}"#
    );
    assert!(matches!(
        FeatureDefinition::from_json(&flat_selector),
        Ok(FeatureDefinition::ChanceChoice { features, .. }) if features.len() == 1
    ));
}
