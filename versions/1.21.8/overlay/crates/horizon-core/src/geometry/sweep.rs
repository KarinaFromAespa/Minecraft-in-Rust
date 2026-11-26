//! `BlockGetter.forEachBlockIntersectedBetween`'s movement-path visits, in
//! 1.21.8's form: the tiny-movement threshold is `(0.99999)^2` (1.21.9
//! narrowed it to `(1e-5)^2`), the start and final boxes are visited with
//! `betweenClosed` (1.21.9's direction-ordered iteration does not exist in
//! 1.21.8), and the ray walks from the final box's min corner minus the
//! movement to the min corner, capped at `MAX_BLOCK_ITERATIONS_ALONG_TRAVEL`
//! (16) ray steps (1.21.9 walks from the furthest corner with no cap).
use super::{Aabb, TraversalError, Vec3, clamp, floor_i32};
use crate::{fast_hash::FxHashSet, position::BlockPos};

/// `BlockGetter.MAX_BLOCK_ITERATIONS_ALONG_TRAVEL`.
const MAX_BLOCK_ITERATIONS_ALONG_TRAVEL: i32 = 16;

struct Sweep<'v, V> {
    visitor: &'v mut V,
    visited: FxHashSet<i64>,
    work: usize,
    limit: usize,
}
impl<V: FnMut(BlockPos, i32) -> bool> Sweep<'_, V> {
    fn spend(&mut self) -> Result<(), TraversalError> {
        if self.work == self.limit {
            return Err(TraversalError::VisitLimit);
        }
        self.work += 1;
        Ok(())
    }
    fn visit(
        &mut self,
        positions: impl Iterator<Item = BlockPos>,
        step: i32,
    ) -> Result<bool, TraversalError> {
        for pos in positions {
            self.spend()?;
            if !self.visited.insert(pos.pack()) {
                continue;
            }
            if !(self.visitor)(pos, step) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Visit blocks and step indices for a moving box, stopping if the callback
/// returns false. `final_box` is the box at `end`, not at `start`.
///
/// Only finite inputs/computed geometry and positive Java-int-sized block boxes
/// are supported. `max_work` limits iterator candidates and ray steps, including
/// suppressed duplicate visits. Prior callbacks remain applied on an error.
/// Block/entity effects and their state transitions are separate from visitation.
pub fn sweep_intersected_blocks(
    start: Vec3,
    end: Vec3,
    final_box: Aabb,
    max_work: usize,
    mut visitor: impl FnMut(BlockPos, i32) -> bool,
) -> Result<bool, TraversalError> {
    if !start.is_finite()
        || !end.is_finite()
        || !final_box.min_position().is_finite()
        || !final_box.max_position().is_finite()
    {
        return Err(TraversalError::NonFinite);
    }
    let movement = end.subtract(start);
    if !movement.is_finite() {
        return Err(TraversalError::NonFinite);
    }
    if movement.length_squared() < (0.99999_f32 * 0.99999_f32) as f64 {
        let blocks =
            BlockPos::between_closed_box(final_box).map_err(|_| TraversalError::InvalidBounds)?;
        // This branch has no deduplication in vanilla. Avoid allocating a set
        // or copying visited positions for stationary/tiny movements.
        for (work, pos) in blocks.enumerate() {
            if work == max_work {
                return Err(TraversalError::VisitLimit);
            }
            if !visitor(pos, 0) {
                return Ok(false);
            }
        }
        return Ok(true);
    }
    let mut sweep = Sweep {
        visitor: &mut visitor,
        // Only asked whether it holds a position: the fast hash, sized for a
        // short move's boxes.
        visited: FxHashSet::with_capacity_and_hasher(32, Default::default()),
        work: 0,
        limit: max_work,
    };
    // 1.21.8's `addCollisionsAlongTravel` walks from the final box's min
    // corner minus the movement, to the min corner; there is no separate
    // start-box visit (1.21.9 visited the moved box first, direction-ordered).
    let min = final_box.min_position();
    let origin = min.subtract(movement);
    if !origin.is_finite() || !min.is_finite() {
        return Err(TraversalError::NonFinite);
    }
    let size = final_box.size().components();
    if size.iter().any(|v| !v.is_finite()) {
        return Err(TraversalError::NonFinite);
    }
    // The direction vanilla derives: the min corner minus the walk's origin
    // (not `movement` itself, which can differ by an ulp).
    let direction = min.subtract(origin);
    let mut position = origin.components().map(floor_i32);
    let delta = direction.components();
    let sign = delta.map(|v| {
        if v == 0.0 {
            0i32
        } else if v > 0.0 {
            1
        } else {
            -1
        }
    });
    let increment = std::array::from_fn::<_, 3, _>(|i| {
        if sign[i] == 0 {
            f64::MAX
        } else {
            sign[i] as f64 / delta[i]
        }
    });
    let mut boundary = std::array::from_fn::<_, 3, _>(|i| {
        let value = origin.components()[i];
        let fraction = value - (value.floor() as i64) as f64;
        increment[i]
            * if sign[i] > 0 {
                1.0 - fraction
            } else {
                fraction
            }
    });
    let mut step = 0i32;
    while boundary.iter().any(|v| *v <= 1.0) {
        sweep.spend()?;
        let axis = if boundary[0] < boundary[1] {
            if boundary[0] < boundary[2] { 0 } else { 2 }
        } else if boundary[1] < boundary[2] {
            1
        } else {
            2
        };
        position[axis] = position[axis].wrapping_add(sign[axis]);
        boundary[axis] += increment[axis];
        // The count increments on every ray step, not only on a clip hit, and
        // the walk stops past `MAX_BLOCK_ITERATIONS_ALONG_TRAVEL`:
        // vanilla's `if (count++ > 16) break` visits steps 1..=17 and returns
        // 18, so the final box takes step 19.
        step = step.wrapping_add(1);
        if step > MAX_BLOCK_ITERATIONS_ALONG_TRAVEL + 1 {
            break;
        }
        let pos = BlockPos::new(position[0], position[1], position[2]);
        // The static Java clip takes raw endpoints, including wrapped +1 ints.
        let block_box = Aabb {
            min: Vec3::from_block(pos),
            max: Vec3::from_block(pos.offset(1, 1, 1)),
        };
        let Some(point) = block_box.clip(origin, min) else {
            continue;
        };
        let clamped = std::array::from_fn::<_, 3, _>(|i| {
            clamp(
                point.components()[i],
                position[i] as f64 + 1.0e-5_f32 as f64,
                (position[i] as f64 + 1.0) - 1.0e-5_f32 as f64,
            )
        });
        // The far corner is the clamped hit plus the box size (1.21.9
        // subtracted the size times the furthest corner's signs), and the
        // box is visited in plain ascending order, not direction-ordered.
        let far =
            std::array::from_fn::<_, 3, _>(|i| floor_i32(clamped[i] + size[i]));
        let blocks = BlockPos::between_closed(
            pos,
            BlockPos::new(far[0], far[1], far[2]),
        )
        .map_err(|_| TraversalError::InvalidBounds)?;
        if !sweep.visit(blocks, step)? {
            return Ok(false);
        }
    }
    let final_blocks = BlockPos::between_closed_box(final_box)
        .map_err(|_| TraversalError::InvalidBounds)?;
    sweep.visit(final_blocks, step.wrapping_add(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tiny_movement_includes_maximum_faces_and_stops_immediately() {
        let box_ = Aabb::unit_cube(Vec3::ZERO);
        let mut visits = Vec::new();
        assert_eq!(
            sweep_intersected_blocks(Vec3::ZERO, Vec3::ZERO, box_, 20, |p, step| {
                visits.push((p, step));
                true
            }),
            Ok(true)
        );
        assert_eq!(visits.len(), 8);
        assert!(visits.iter().all(|(_, step)| *step == 0));
        assert_eq!(
            sweep_intersected_blocks(Vec3::ZERO, Vec3::ZERO, box_, 20, |_, _| false),
            Ok(false)
        );
        assert_eq!(
            sweep_intersected_blocks(Vec3::ZERO, Vec3::ZERO, box_, 1, |_, _| true),
            Err(TraversalError::VisitLimit)
        );
    }
}
