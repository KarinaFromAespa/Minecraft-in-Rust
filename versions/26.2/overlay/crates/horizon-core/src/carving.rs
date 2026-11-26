//! Packed carving masks and vanilla's ordered inclusive column segments.
mod ellipsoid;
pub use ellipsoid::{
    CarveError, CarverOutput, can_reach, carve_ellipsoid, carve_ellipsoid_with_budget,
};

/// A contiguous carved interval in a local chunk column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarvingColumn {
    pub x: i32,
    pub z: i32,
    pub min_y: i32,
    pub max_y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CarvingMaskError {
    InvalidBounds,
    NegativeIndex,
    Capacity,
}
impl std::fmt::Display for CarvingMaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidBounds => "carving mask height must be between 1 and 4064",
            Self::NegativeIndex => "carving mask index is negative",
            Self::Capacity => "carving mask exceeds its bounded bit capacity",
        })
    }
}
impl std::error::Error for CarvingMaskError {}

/// Bits are contiguous in Y, then Z, then X. Coordinates retain Java's wrapping
/// index arithmetic and positive out-of-range aliases. Growth is capped at
/// 2^24 bits; valid chunk coordinates use at most 256 * 4064 bits.
#[derive(Clone, Debug)]
pub struct CarvingMask {
    min_y: i32,
    max_y: i32,
    height: i32,
    words: Vec<u64>,
    used_words: usize,
}
impl CarvingMask {
    pub fn new(min_y: i32, max_y: i32) -> Result<Self, CarvingMaskError> {
        let height = i64::from(max_y) - i64::from(min_y) + 1;
        if !(1..=4064).contains(&height) {
            return Err(CarvingMaskError::InvalidBounds);
        }
        Ok(Self {
            min_y,
            max_y,
            height: height as i32,
            words: vec![0; (height as usize * 256).div_ceil(64)],
            used_words: 0,
        })
    }
    pub fn min_y(&self) -> i32 {
        self.min_y
    }
    pub fn max_y(&self) -> i32 {
        self.max_y
    }
    pub fn is_empty(&self) -> bool {
        self.used_words == 0
    }
    pub fn carve(&mut self, x: i32, y: i32, z: i32) -> Result<(), CarvingMaskError> {
        let index = y
            .wrapping_sub(self.min_y)
            .wrapping_add(z.wrapping_add(x.wrapping_shl(4)).wrapping_mul(self.height));
        if index < 0 {
            return Err(CarvingMaskError::NegativeIndex);
        }
        if index >= 1 << 24 {
            return Err(CarvingMaskError::Capacity);
        }
        let index = index as usize;
        let word = index / 64;
        if word >= self.words.len() {
            self.words.resize(word + 1, 0);
        }
        self.words[word] |= 1 << (index & 63);
        self.used_words = self.used_words.max(word + 1);
        Ok(())
    }
    /// Whether the cell is already carved. Out-of-range coordinates read as
    /// uncarved; vanilla's 26.2 ellipsoid checks the mask before each write,
    /// so a claimed cell is never written twice.
    pub fn get(&self, x: i32, y: i32, z: i32) -> bool {
        let index = y
            .wrapping_sub(self.min_y)
            .wrapping_add(z.wrapping_add(x.wrapping_shl(4)).wrapping_mul(self.height));
        if index < 0 || index >= 1 << 24 {
            return false;
        }
        let index = index as usize;
        let word = index / 64;
        word < self.words.len() && self.words[word] & (1 << (index & 63)) != 0
    }
    /// The bit index of an in-range cell, for callers that keep per-cell data
    /// alongside the mask (ellipsoid tags). Out-of-range coordinates, and
    /// coordinates the mask's wrapping arithmetic aliases, return `None`.
    pub fn index(&self, x: i32, y: i32, z: i32) -> Option<u32> {
        if !(0..16).contains(&x) || !(0..16).contains(&z) || y < self.min_y || y > self.max_y {
            return None;
        }
        Some(
            (y.wrapping_sub(self.min_y)
                .wrapping_add(z.wrapping_add(x.wrapping_shl(4)).wrapping_mul(self.height)))
                as u32,
        )
    }
    /// Traverse ascending packed runs, splitting each run at column boundaries.
    /// No per-cell or per-column allocation is required.
    pub fn columns(&self) -> impl Iterator<Item = CarvingColumn> + '_ {
        let mut cursor = Cursor::default();
        std::iter::from_fn(move || cursor.next(self))
    }
    /// Like vanilla's callback traversal, changes made by the visitor affect the
    /// next run search, but do not change the already selected run's boundaries.
    /// An error stops traversal immediately and retains visitor mutations.
    pub fn try_visit_mut<E>(
        &mut self,
        mut visitor: impl FnMut(&mut Self, CarvingColumn) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut cursor = Cursor::default();
        while let Some(column) = cursor.next(self) {
            visitor(self, column)?;
        }
        Ok(())
    }
    fn next_set(&self, from: usize) -> Option<usize> {
        let mut word = from / 64;
        if word >= self.used_words {
            return None;
        }
        let mut bits = self.words[word] & (u64::MAX << (from & 63));
        loop {
            if bits != 0 {
                return Some(word * 64 + bits.trailing_zeros() as usize);
            }
            word += 1;
            if word == self.used_words {
                return None;
            }
            bits = self.words[word];
        }
    }
    fn next_clear(&self, from: usize) -> usize {
        let mut word = from / 64;
        let mut bits = !self.words[word] & (u64::MAX << (from & 63));
        loop {
            if bits != 0 {
                return word * 64 + bits.trailing_zeros() as usize;
            }
            word += 1;
            if word == self.used_words {
                return word * 64;
            }
            bits = !self.words[word];
        }
    }
}

#[derive(Default)]
struct Cursor {
    search: usize,
    run: Option<(usize, usize, usize)>,
}
impl Cursor {
    fn next(&mut self, mask: &CarvingMask) -> Option<CarvingColumn> {
        let height = mask.height as usize;
        let (start, end, column) = match self.run {
            Some(run) => run,
            None => {
                let start = mask.next_set(self.search)?;
                let end = mask.next_clear(start) - 1;
                self.search = end + 1;
                (start, end, start / height)
            }
        };
        self.run = (column < end / height).then_some((start, end, column + 1));
        let base = column * height;
        Some(CarvingColumn {
            x: ((column >> 4) & 15) as i32,
            z: (column & 15) as i32,
            min_y: mask.min_y.wrapping_add(start.saturating_sub(base) as i32),
            max_y: mask.min_y.wrapping_add((end - base).min(height - 1) as i32),
        })
    }
}

/// The ellipsoid that first claimed each carved cell, keyed by the mask's bit
/// index (`CarvingMask::index`). Vanilla 26.2 marks fluid blocks during the
/// worm trace, in ellipsoid order, so the apply pass needs to know which
/// ellipsoid each cell belongs to; the mask alone (visited in packed column
/// order) cannot reproduce that order. Only cells the mask holds have entries.
#[derive(Clone, Debug, Default)]
pub struct EllipsoidTags {
    tags: std::collections::HashMap<u32, u32>,
}
impl EllipsoidTags {
    pub fn get(&self, index: u32) -> Option<u32> {
        self.tags.get(&index).copied()
    }
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }
}

/// A carving mask that records, for each cell, the ellipsoid that first
/// claimed it. The sequence advances on every `begin_ellipsoid` (one per
/// ellipsoid the carvers visit that can reach the chunk), so the tags order
/// the cells the way vanilla 26.2's worm trace marks them. A cell claimed by
/// an earlier ellipsoid keeps its first tag: vanilla's mask check skips
/// already-carved cells.
#[derive(Debug)]
pub struct SequencedMask {
    mask: CarvingMask,
    tags: EllipsoidTags,
    sequence: u32,
}
impl SequencedMask {
    pub fn new(min_y: i32, max_y: i32) -> Result<Self, CarvingMaskError> {
        Ok(Self {
            mask: CarvingMask::new(min_y, max_y)?,
            tags: EllipsoidTags::default(),
            sequence: 0,
        })
    }
    pub fn into_parts(self) -> (CarvingMask, EllipsoidTags) {
        (self.mask, self.tags)
    }
}
impl CarverOutput for SequencedMask {
    type Error = CarvingMaskError;
    fn min_y(&self) -> i32 {
        self.mask.min_y()
    }
    fn max_y(&self) -> i32 {
        self.mask.max_y()
    }
    fn carve(&mut self, x: i32, y: i32, z: i32) -> Result<(), Self::Error> {
        if !self.mask.get(x, y, z) {
            if let Some(index) = self.mask.index(x, y, z) {
                self.tags.tags.insert(index, self.sequence);
            }
            self.mask.carve(x, y, z)?;
        }
        Ok(())
    }
    fn begin_ellipsoid(&mut self) {
        self.sequence = self.sequence.wrapping_add(1);
    }
}
