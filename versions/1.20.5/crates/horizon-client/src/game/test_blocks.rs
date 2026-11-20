//! Test blocks on the server: the mode a placed test block starts with.
//! 1.21.4 has no set-test-block packet, so no guest handler lives here.
use super::Game;
use horizon_client::{
    containers::BlockEntity,
    test_block::{Effects, TestBlock, TestBlockMode},
};
use horizon_core::position::BlockPos;

type Pos = (i32, i32, i32);

impl Game<'_> {
    fn test_block_mut(&mut self, pos: Pos) -> Option<&mut TestBlock> {
        match self.world.block_entities.get_mut(&pos)? {
            BlockEntity::TestBlock(b) => Some(b),
            _ => None,
        }
    }

    /// The `mode` of the block state at `pos`.
    pub(super) fn test_block_state_mode(&self, pos: Pos) -> Option<TestBlockMode> {
        let state = self.world.block(pos.0, pos.1, pos.2);
        let mode = self
            .pack
            .assets
            .registry
            .state(state)?
            .properties
            .get("mode")?;
        TestBlockMode::of_serialized_name(mode)
    }

    /// `new TestBlockEntity(pos, state)`: the mode is the state's.
    pub(super) fn test_block_placed(&mut self, pos: Pos) {
        let Some(mode) = self.test_block_state_mode(pos) else {
            return;
        };
        if let Some(b) = self.test_block_mut(pos) {
            b.mode = mode;
        }
    }

    fn test_block_at(&self, pos: Pos) -> Option<&TestBlock> {
        match self.world.block_entities.get(&pos)? {
            BlockEntity::TestBlock(b) => Some(b),
            _ => None,
        }
    }

    /// `TestBlockEntity.hasTriggered`.
    pub(super) fn test_block_triggered(&self, pos: Pos) -> bool {
        self.test_block_at(pos).is_some_and(|b| b.triggered)
    }

    /// `TestBlockEntity.getMessage`.
    pub(super) fn test_block_message(&self, pos: Pos) -> String {
        self.test_block_at(pos)
            .map_or_else(String::new, |b| b.message.clone())
    }

    /// What a `TestBlockEntity` call asked of the level: its neighbours
    /// update (`level.updateNeighborsAt`) and the log line.
    fn apply_test_block_effects(&mut self, pos: Pos, effects: Effects) {
        if effects.update_neighbors {
            let at = BlockPos::new(pos.0, pos.1, pos.2);
            let state = self.world.block(pos.0, pos.1, pos.2);
            self.with_level(|b, level, ticks| {
                let block = b.block(state);
                b.update_neighbors_at(level, ticks, at, block);
            });
        }
        if effects.log
            && let Some(b) = self.test_block_at(pos)
            && !b.message.trim().is_empty()
        {
            log::info!(
                "Test {} (at {}, {}, {}): {}",
                b.mode.serialized_name(),
                pos.0,
                pos.1,
                pos.2,
                b.message
            );
        }
    }

    /// `TestBlockEntity.trigger`.
    pub(super) fn trigger_test_block(&mut self, pos: Pos) {
        let Some(effects) = self.test_block_mut(pos).map(TestBlock::trigger) else {
            return;
        };
        self.apply_test_block_effects(pos, effects);
    }

    /// `TestBlockEntity.reset`, which `TestBlock.tick` runs.
    pub(super) fn reset_test_block(&mut self, pos: Pos) {
        let Some(effects) = self.test_block_mut(pos).map(TestBlock::reset) else {
            return;
        };
        self.apply_test_block_effects(pos, effects);
    }

    /// `TestBlock.neighborChanged`: a rising signal triggers the block.
    pub(super) fn test_block_neighbor_changed(&mut self, pos: Pos) {
        if self.test_block_at(pos).is_none() {
            return;
        }
        let at = BlockPos::new(pos.0, pos.1, pos.2);
        let signal = self.with_level(|b, level, ticks| b.has_neighbor_signal(level, ticks, at));
        let Some(effects) = self
            .test_block_mut(pos)
            .and_then(|b| b.neighbor_changed(signal))
        else {
            return;
        };
        self.apply_test_block_effects(pos, effects);
    }
}
