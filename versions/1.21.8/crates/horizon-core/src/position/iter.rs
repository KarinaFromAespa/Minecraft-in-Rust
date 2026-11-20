//! Inclusive block boxes in vanilla's fixed visit order.
use super::BlockPos;
use crate::geometry::{Aabb, Axis, Vec3, floor_i32};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockIterationTooLarge;
impl std::fmt::Display for BlockIterationTooLarge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("block box exceeds the positive Java-int iteration domain")
    }
}
impl std::error::Error for BlockIterationTooLarge {}

/// Value iterator; each position is independent of later cursor updates.
///
/// Bounds are inclusive, including an AABB's floored maximum face. Boxes whose
/// true volume exceeds i32::MAX are rejected instead of reproducing Java's
/// overflowed counts. A caller can lazily limit a supported box with `take`.
#[derive(Clone, Debug)]
pub struct BlockBoxIter {
    start: [i32; 3],
    steps: [i32; 3],
    sizes: [usize; 3],
    // Innermost axis first.
    axes: [Axis; 3],
    index: usize,
    count: usize,
}
impl BlockBoxIter {
    fn new(a: BlockPos, b: BlockPos) -> Result<Self, BlockIterationTooLarge> {
        let a = [a.x, a.y, a.z];
        let b = [b.x, b.y, b.z];
        let min = std::array::from_fn::<_, 3, _>(|i| a[i].min(b[i]));
        let max = std::array::from_fn::<_, 3, _>(|i| a[i].max(b[i]));
        let sizes = std::array::from_fn::<_, 3, _>(|i| (max[i] as i64 - min[i] as i64 + 1) as u64);
        let count = sizes
            .into_iter()
            .try_fold(1u64, |total, size| {
                total.checked_mul(size).filter(|v| *v <= i32::MAX as u64)
            })
            .ok_or(BlockIterationTooLarge)?;
        // `betweenClosed` order: ascending, X innermost. 1.21.9's
        // direction-ordered `betweenCornersInDirection` does not exist in
        // 1.21.8 (item 178 removed it with the sweep rework).
        Ok(Self {
            start: min,
            steps: [1; 3],
            sizes: sizes.map(|v| v as usize),
            axes: [Axis::X, Axis::Y, Axis::Z],
            index: 0,
            count: count as usize,
        })
    }
}
impl Iterator for BlockBoxIter {
    type Item = BlockPos;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.count {
            return None;
        }
        let mut index = self.index;
        let mut point = self.start;
        for axis in self.axes {
            let i = axis as usize;
            let offset = index % self.sizes[i];
            index /= self.sizes[i];
            point[i] = point[i].wrapping_add(self.steps[i].wrapping_mul(offset as i32));
        }
        self.index += 1;
        Some(BlockPos::new(point[0], point[1], point[2]))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.count - self.index;
        (remaining, Some(remaining))
    }
}
impl ExactSizeIterator for BlockBoxIter {}
impl std::iter::FusedIterator for BlockBoxIter {}

impl BlockPos {
    #[inline]
    pub fn containing(point: Vec3) -> Self {
        Self::new(floor_i32(point.x), floor_i32(point.y), floor_i32(point.z))
    }
    pub fn between_closed(a: Self, b: Self) -> Result<BlockBoxIter, BlockIterationTooLarge> {
        BlockBoxIter::new(a, b)
    }
    pub fn between_closed_box(b: Aabb) -> Result<BlockBoxIter, BlockIterationTooLarge> {
        Self::between_closed(
            Self::containing(b.min_position()),
            Self::containing(b.max_position()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordering_inclusive_faces_and_value_independence() {
        let a = BlockPos::new(-1, 0, 0);
        let b = BlockPos::new(0, 1, 0);
        assert_eq!(
            BlockPos::between_closed(b, a).unwrap().collect::<Vec<_>>(),
            vec![a, BlockPos::new(0, 0, 0), BlockPos::new(-1, 1, 0), b]
        );
        assert_eq!(
            BlockPos::between_closed_box(Aabb::unit_cube(Vec3::ZERO))
                .unwrap()
                .len(),
            8
        );
        assert_eq!(
            BlockPos::between_closed(BlockPos::new(i32::MIN, 0, 0), BlockPos::new(i32::MAX, 0, 0))
                .unwrap_err(),
            BlockIterationTooLarge
        );
    }
}
