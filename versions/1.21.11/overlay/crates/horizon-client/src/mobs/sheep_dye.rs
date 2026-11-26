//! `DyeItem.interactLivingEntity`: the client hears it, the server changes wool.
use super::{Mob, Species, render::DYE_NAMES};
use crate::items::ItemStack;

/// The sixteen vanilla DyeItems dispatch this interaction. In 1.21.11 the
/// colour is the item's own (DyeItem's dyeColor field): there is no
/// `minecraft:dye` component to override or remove.
fn color(stack: &ItemStack) -> Option<u8> {
    if stack.count == 0 {
        return None;
    }
    let name = stack
        .item
        .strip_prefix("minecraft:")?
        .strip_suffix("_dye")?;
    DYE_NAMES.iter().position(|&d| d == name).map(|i| i as u8)
}

/// Whether this invocation plays DYE_USE. Client state is deliberately left
/// untouched, so repeated client clicks keep playing until metadata arrives.
/// Player.interactOn's creative count restoration belongs to the caller.
pub fn interact(mob: &mut Mob, stack: &mut ItemStack, client: bool) -> bool {
    if mob.species() != Species::Sheep || !mob.alive() || mob.extra.sheared {
        return false;
    }
    let Some(color) = color(stack).filter(|&c| c != mob.extra.color) else {
        return false;
    };
    if !client {
        mob.extra.color = color;
        stack.count -= 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_core::geometry::Vec3;
    use std::sync::Arc;

    fn sheep() -> Mob {
        let mut kind = crate::mobs::sensor_scan_tests::kind();
        Arc::make_mut(&mut kind).species = Species::Sheep;
        Mob::new(37, kind, Vec3::ZERO, 0.0)
    }

    #[test]
    fn client_repeats_without_consuming_or_predicting_the_color() {
        let mut mob = sheep();
        let mut stack = ItemStack::new("minecraft:red_dye", 2);
        for _ in 0..2 {
            assert!(interact(&mut mob, &mut stack, true));
            assert_eq!(mob.extra.color, 0);
            assert_eq!(stack.count, 2);
        }
        assert!(interact(&mut mob, &mut stack, false));
        assert_eq!(mob.extra.color, 14);
        assert_eq!(stack.count, 1);
        assert!(!interact(&mut mob, &mut stack, true));
        assert!(!interact(&mut mob, &mut stack, false));
    }

    #[test]
    fn the_color_is_the_items_own_name_not_a_component() {
        assert_eq!(color(&ItemStack::new("minecraft:red_dye", 1)), Some(14));
        assert_eq!(color(&ItemStack::new("minecraft:blue_dye", 1)), Some(11));
        assert_eq!(color(&ItemStack::new("minecraft:stone", 1)), None);
        assert_eq!(color(&ItemStack::new("minecraft:red_dye", 0)), None);
    }
}
