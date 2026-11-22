use super::*;

/// `ThrownExperienceBottle.onHit` uses plain `award`: the orbs start at the
/// hit point with random motion, not flown off the face (1.21.6's
/// `awardWithDirection` starts them 0.25 out, away from what it hit).
#[test]
#[ignore = "needs prepared resources (HORIZON_TEST_RESOURCES)"]
fn bottle_orbs_start_at_the_hit() {
    let mut game = super::super::native_machines::tests::server_game();
    let bottle = ItemStack::new("minecraft:experience_bottle", 1);
    let at = Vec3::new(8.5, 70.5, 8.5);
    for face in [Some(([9, 70, 8], Direction::West)), None] {
        game.orbs.clear();
        game.splash(
            &bottle,
            at,
            at,
            None,
            face,
            &PotionOwner::default(),
            false,
        );
        assert!(!game.orbs.is_empty());
        for o in game.orbs.iter() {
            let p = [o.position.x, o.position.y, o.position.z];
            assert!(
                (p[0] - at.x).abs() < 1e-9
                    && (p[1] - at.y).abs() < 1e-9
                    && (p[2] - at.z).abs() < 1e-9,
                "{p:?}"
            );
        }
    }
}
