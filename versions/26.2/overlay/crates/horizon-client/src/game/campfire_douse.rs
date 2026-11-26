//! `ShovelItem.useOn` on a campfire: a shovel put on a lit campfire puts it
//! out. (26.3 moved the douse into `CampfireBlock.useItemOn` behind the
//! `#minecraft:douses_campfires` tag; 26.2 has neither the tag nor the
//! block-side branch — the shovel's own `useOn` does it.)
use super::Game;
use horizon_client::actions::Action;
use horizon_client::{Result, animate_tick::make_campfire_smoke, items::ItemStack};
use horizon_core::position::BlockPos;
use horizon_world::chunk::BlockState;

impl Game<'_> {
    /// The lit campfire at `pos` that `held` puts out, when the item is a
    /// shovel (`ShovelItem.useOn`). Clicking the campfire's underside passes
    /// (`getClickedFace() == DOWN`).
    fn campfire_to_douse(
        &self,
        pos: [i32; 3],
        held: Option<&ItemStack>,
        face: [i32; 3],
    ) -> Option<BlockState> {
        if face == [0, -1, 0] {
            return None;
        }
        held.filter(|s| {
            self.res
                .items()
                .item_tags
                .get("minecraft:shovels")
                .is_some_and(|members| members.contains(&*s.item))
        })?;
        self.pack
            .assets
            .registry
            .state(self.world.block(pos[0], pos[1], pos[2]))
            .filter(|s| matches!(&*s.name, "minecraft:campfire" | "minecraft:soul_campfire"))
            .filter(|s| s.properties.get("lit").map(String::as_str) == Some("true"))
            .cloned()
    }

    /// `ShovelItem.useOn` as this client runs it before the server's
    /// answer (`MultiPlayerGameMode.useItemOn`): the twenty smoke puffs of
    /// `CampfireBlock.dowse` and the campfire gone out, kept as predicted
    /// under `seq`. The level event's sound comes from the server. True when
    /// it doused (SUCCESS), which ends the click for both hands.
    pub(super) fn predict_campfire_douse(
        &mut self,
        pos: [i32; 3],
        held: Option<&ItemStack>,
        face: [i32; 3],
        seq: i32,
    ) -> bool {
        // Player.isSecondaryUseActive: a held item means both hands count.
        if self.keys.held(Action::Sneak) {
            return false;
        }
        let Some(mut state) = self.campfire_to_douse(pos, held, face) else {
            return false;
        };
        let signal = state.properties.get("signal_fire").map(String::as_str) == Some("true");
        for _ in 0..20 {
            make_campfire_smoke(&mut self.particles, pos, signal, true);
        }
        state.properties.insert("lit".into(), "false".into());
        if let Ok(unlit) = self.pack.assets.registry.resolve(&state) {
            let at = BlockPos::new(pos[0], pos[1], pos[2]);
            self.predict_blocks(&vec![(at, unlit)], seq);
        }
        true
    }

    /// `ShovelItem.useOn` with `held` in the hand that acts, the block
    /// at `pos` clicked on `face`: true when it doused a lit campfire (SUCCESS).
    pub(super) fn use_campfire_douse(
        &mut self,
        pos: [i32; 3],
        held: Option<&ItemStack>,
        face: [i32; 3],
    ) -> Result<bool> {
        let (Some(used), Some(mut state)) =
            (held.cloned(), self.campfire_to_douse(pos, held, face))
        else {
            return Ok(false);
        };
        let registry = self.pack.assets.registry;
        // Level event 1009 (SOUND_EXTINGUISH_FIRE) goes to every player.
        self.level_event_at(1009, pos, 0);
        // CampfireBlock.dowse: the acting client's own smoke (twenty puffs),
        // then the block change game event from the player.
        if self.lan_acting.is_none() {
            let signal = state.properties.get("signal_fire").map(String::as_str) == Some("true");
            for _ in 0..20 {
                make_campfire_smoke(&mut self.particles, pos, signal, true);
            }
        }
        self.post_block_game_event("minecraft:block_change", pos, self.sculk_actor(), None);
        state.properties.insert("lit".into(), "false".into());
        if let Ok(unlit) = registry.resolve(&state) {
            self.level_set(pos, unlit)?;
        }
        self.damage_held(1);
        // ServerPlayerGameMode.useItemOn: the action was consumed.
        self.trigger_block("minecraft:item_used_on_block", pos, &used);
        Ok(true)
    }
}
