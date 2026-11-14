//! Existing pure bed geometry shared by gameplay and the reusable Mob layer.
use horizon_core::geometry::Vec3;

/// `LivingEntity.setPosToBed` (26.2): the middle of the sleeper's block,
/// 0.6875 above its base. 26.2 has no sleep-height check: `startSleeping`
/// is void, and the height is flat, not measured from the bed's shape.
/// 26.3's `AbstractBedBlock.getSleepHeight` and `StrawBedBlock` (a straw
/// bed's head slept on at its foot) do not exist in 26.2, and straw beds
/// are absent from the 26.2 data.
pub fn pos_to_bed(head: [i32; 3]) -> Vec3 {
    Vec3::new(
        f64::from(head[0]) + 0.5,
        f64::from(head[1]) + 0.6875,
        f64::from(head[2]) + 0.5,
    )
}

#[cfg(test)]
mod tests {
    use super::pos_to_bed;

    /// `setPosToBed`: the middle of the block, 0.6875 above its base, for
    /// every bed. No shape is measured and no straw-bed foot is consulted.
    #[test]
    fn the_sleeper_lies_at_the_flat_bed_height() {
        let at = pos_to_bed([10, 70, 5]);
        assert_eq!((at.x, at.y, at.z), (10.5, 70.6875, 5.5));
    }
}
