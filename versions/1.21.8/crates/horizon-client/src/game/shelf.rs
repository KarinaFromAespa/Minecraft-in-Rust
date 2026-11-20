//! Clicking a shelf's front face (`ShelfBlockEntity`): which slot the hit
//! lands in, and swapping the held item with it. 1.21.8 has no shelf block,
//! so none of this can run here; the `ShelfRenderer` (three items face out)
//! is gone with item 425.
use super::{Game, Hit, machines::dir_from_name};
use horizon_client::containers::BlockEntity;
use horizon_core::{
    geometry::{Direction, Vec3},
    position::BlockPos,
};
use horizon_world::level::hit_slot;

impl Game<'_> {
    /// Slot and facing for a click on a shelf's front face.
    pub(super) fn shelf_hit(&self, hit: &Hit) -> Option<(usize, Direction, bool)> {
        let p = hit.pos;
        if !matches!(
            self.world.block_entities.get(&(p[0], p[1], p[2])),
            Some(BlockEntity::Shelf(_))
        ) {
            return None;
        }
        let state = self.world.block(p[0], p[1], p[2]);
        let data = self.pack.assets.registry.state(state)?;
        let facing = data
            .properties
            .get("facing")
            .and_then(|s| dir_from_name(s))?;
        let face = Direction::ALL
            .into_iter()
            .find(|d| d.normal() == hit.face)?;
        let slot = hit_slot(
            facing,
            face,
            BlockPos::new(p[0], p[1], p[2]),
            Vec3::new(hit.point[0], hit.point[1], hit.point[2]),
            1,
            3,
        )?;
        Some((
            slot,
            facing,
            data.properties.get("powered").is_some_and(|s| s == "true"),
        ))
    }

    /// `SideChainPartBlock.getAllBlocksConnectedTo`: left to right, at most three.
    fn shelf_chain(&self, pos: [i32; 3], facing: Direction) -> Vec<[i32; 3]> {
        let left = match facing {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
            _ => return Vec::new(),
        };
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let behaviours = self.res.behaviours();
        if !behaviours
            .tag("minecraft:wooden_shelves")
            .is_some_and(|tag| tag.contains(behaviours.block(state)))
        {
            return Vec::new();
        }
        let mut result = vec![pos];
        for (dir, end, prepend) in [(left, "left", true), (left.opposite(), "right", false)] {
            let normal = dir.normal();
            for step in 1..3 {
                let p = [pos[0] + normal[0] * step, pos[1], pos[2] + normal[2] * step];
                let state = self.world.block(p[0], p[1], p[2]);
                let Some(info) = self.pack.assets.registry.state(state) else {
                    break;
                };
                let behaviours = self.res.behaviours();
                let connectable = behaviours
                    .tag("minecraft:wooden_shelves")
                    .is_some_and(|tag| tag.contains(behaviours.block(state)))
                    && info.properties.get("powered").is_some_and(|s| s == "true")
                    && info.properties.get("facing").and_then(|s| dir_from_name(s)) == Some(facing);
                if !connectable {
                    break;
                }
                let part = info
                    .properties
                    .get("side_chain")
                    .map(String::as_str)
                    .unwrap_or("unconnected");
                if part == "center" || part == end {
                    if prepend {
                        result.insert(0, p);
                    } else {
                        result.push(p);
                    }
                }
                if part != "center" {
                    break;
                }
            }
        }
        result
    }

    /// `ShelfBlock.useItemOn`: a single swap, or a connected hotbar swap while powered.
    pub(super) fn use_shelf(&mut self, hit: &Hit) -> Option<bool> {
        use horizon_client::first_person::Hand;
        if self.hand != Hand::Main {
            return None;
        }
        let (slot, facing, powered) = self.shelf_hit(hit)?;
        let pos = hit.pos;
        let state = self.world.block(pos[0], pos[1], pos[2]);
        if powered {
            let chain = self.shelf_chain(pos, facing);
            let mut swapped = false;
            let count = chain.len();
            for (i, p) in chain.into_iter().enumerate() {
                let key = (p[0], p[1], p[2]);
                let Some(BlockEntity::Shelf(shelf)) = self.world.block_entities.get_mut(&key)
                else {
                    continue;
                };
                for n in 0..3 {
                    let index = 9_i32 - (count as i32 - i as i32) * 3 + n as i32;
                    if !(0..9).contains(&index) {
                        continue;
                    }
                    let index = index as usize;
                    swapped |= self.inventory[index].is_some() || shelf.items[n].is_some();
                    std::mem::swap(&mut self.inventory[index], &mut shelf.items[n]);
                }
                self.container_changed(key);
                let s = self.world.block(p[0], p[1], p[2]);
                self.post_block_game_event("minecraft:entity_interact", p, None, Some(s));
            }
            if !swapped {
                return Some(false);
            }
            self.audio.play(
                "minecraft:block.shelf.multi_swap",
                Some(pos.map(|n| f64::from(n) + 0.5)),
                1.0,
                1.0,
            );
            Some(true)
        } else {
            let held = self.inventory[self.selected].clone();
            let key = (pos[0], pos[1], pos[2]);
            let Some(BlockEntity::Shelf(shelf)) = self.world.block_entities.get_mut(&key) else {
                return None;
            };
            let removed = std::mem::replace(&mut shelf.items[slot], held.clone());
            self.inventory[self.selected] = if self.creative && removed.is_none() {
                held.clone()
            } else {
                removed.clone()
            };
            self.container_changed(key);
            let vibrations = self.inventory[self.selected].as_ref().is_none_or(|stack| {
                let default = self
                    .res
                    .items()
                    .get(&stack.item)
                    .is_none_or(|item| item.interact_vibrations);
                let Some(data) = stack.data() else {
                    return default;
                };
                if data
                    .other
                    .iter()
                    .any(|(key, _)| key == "!minecraft:use_effects")
                {
                    return true;
                }
                data.other
                    .iter()
                    .find(|(key, _)| key == "minecraft:use_effects")
                    .map_or(default, |(_, tag)| {
                        horizon_client::save::number(tag.get("interact_vibrations")).unwrap_or(1)
                            != 0
                    })
            });
            if vibrations {
                self.post_block_game_event(
                    "minecraft:item_interact_finish",
                    pos,
                    None,
                    Some(state),
                );
            }
            let sound = match (held.is_some(), removed.is_some()) {
                (false, false) => return None,
                (true, false) => "minecraft:block.shelf.place_item",
                (false, true) => "minecraft:block.shelf.take_item",
                (true, true) => "minecraft:block.shelf.single_swap",
            };
            self.audio
                .play(sound, Some(pos.map(|n| f64::from(n) + 0.5)), 1.0, 1.0);
            Some(true)
        }
    }
}
