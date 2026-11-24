use super::*;

/// `ThrownExperienceBottle.onHit` awards plain (`ExperienceOrb.award`):
/// every fresh orb starts exactly where the bottle hit, whatever face
/// it hit — 1.21.6's `awardWithDirection` (orbs 0.25 out from the face,
/// flying off along the bottle's motion) does not exist here.
#[test]
#[ignore = "needs prepared resources (HORIZON_TEST_RESOURCES)"]
fn bottle_orbs_start_where_the_bottle_hit() {
    let mut game = super::super::native_machines::tests::server_game();
    let bottle = ItemStack::new("minecraft:experience_bottle", 1);
    let at = Vec3::new(8.5, 70.5, 8.5);
    for face in [
        Some(([9, 70, 8], Direction::West)),
        Some(([8, 69, 8], Direction::Up)),
        None,
    ] {
        game.orbs.clear();
        // The bottle's position at the hit is the hit point itself
        // (`ThrowableProjectile.tick` sets it there before `onHit`).
        game.splash(&bottle, at, at, None, face, &PotionOwner::default(), false);
        assert!(!game.orbs.is_empty());
        for o in game.orbs.iter() {
            assert_eq!(
                [o.position.x, o.position.y, o.position.z],
                [at.x, at.y, at.z],
                "an orb starts at the hit position, not offset from a face"
            );
        }
    }
}
