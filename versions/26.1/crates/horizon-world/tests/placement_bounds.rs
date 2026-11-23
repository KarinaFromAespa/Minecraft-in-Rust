use horizon_core::random::LegacyRandom;
use horizon_world::{
    heightmap::HeightmapType,
    worldgen::{
        anchor::WorldGenerationContext,
        placement::{
            LayerBlock, PlacementContext, PlacementError, PlacementModifier, PlacementQueryError,
        },
        value_provider::IntProvider,
    },
};

struct UnboundedColumn {
    reads: usize,
}
impl PlacementContext for UnboundedColumn {
    fn generation(&self) -> WorldGenerationContext {
        WorldGenerationContext::new(-64, 384, 63)
    }
    fn min_y(&mut self) -> Result<i32, PlacementQueryError> {
        Ok(-64)
    }
    fn height(&mut self, _: HeightmapType, _: i32, _: i32) -> Result<i32, PlacementQueryError> {
        Ok(i32::MAX)
    }
    fn layer_block(&mut self, _: [i32; 3]) -> Result<LayerBlock, PlacementQueryError> {
        self.reads += 1;
        Ok(LayerBlock::Empty)
    }
}
#[test]
fn layered_scans_bound_work_when_world_queries_return_extreme_heights() {
    let mut context = UnboundedColumn { reads: 0 };
    let result = PlacementModifier::CountOnEveryLayer(IntProvider::Constant(1)).modify_in(
        &mut context,
        &mut LegacyRandom::new(0),
        [0, 0, 0],
        |_, _| -> Result<(), ()> { panic!("empty column emitted") },
    );
    assert!(matches!(result, Err(PlacementError::WorkLimit)));
    assert!(context.reads > 0 && context.reads <= 65536);
}
