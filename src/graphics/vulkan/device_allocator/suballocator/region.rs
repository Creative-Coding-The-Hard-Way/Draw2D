use std::ops::{Bound, Range, RangeBounds};

use crate::graphics::vulkan::device_allocator::Allocation;

/// A region represents a range within an allocation.
///
/// Regions within an allocation never overlap.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Region {
    pub offset: u64,
    pub size: u64,
}

/// This enum represents the result of attempting to merge two regions.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MergeResult {
    /// Indicates that the regions cannot be merged because they are not
    /// contiguous.
    NonContiguous,

    /// Indicactes that the regions were merged into a new contiguous region.
    Contiguous(Region),
}

impl Region {
    pub fn new(offset: u64, size: u64) -> Self {
        Self { offset, size }
    }

    /// Create a region which occupies an entire allocation.
    pub fn new_whole_region(allocation: &Allocation) -> Self {
        Self::new(allocation.offset, allocation.byte_size)
    }

    /// The address where this Region begins within the parent allocation.
    pub fn start(&self) -> u64 {
        self.offset
    }

    /// The address where this Region ends within the parent allocation.
    pub fn end(&self) -> u64 {
        self.offset + self.size
    }

    /// Returns true when this region and the other region are touching with
    /// no space between.
    pub fn is_contiguous(&self, other: &Self) -> bool {
        self.start() == other.end() || self.end() == other.start()
    }

    /// Attempt to merge this region with another.
    pub fn merge(&self, other: &Self) -> MergeResult {
        if self.is_contiguous(other) {
            // SAFE - the line above checks that the regions are contiguous
            MergeResult::Contiguous(unsafe { self.merge_unsafe(other) })
        } else {
            MergeResult::NonContiguous
        }
    }

    /// Take a subregion from this region, updating the size and offset to
    /// match.
    pub fn take_subregion(&mut self, size: u64) -> Region {
        let new_region = Region::new(self.offset, size);
        self.offset += size;
        self.size -= size;
        new_region
    }

    /// Returns true when any part of this region is contained within another.
    ///
    /// NOTE: this is different from `is_contiguous` because that method only
    /// checks that the edges are touching. Overlapping regions are not just
    /// touching, but truly overlapping - this indicates the user is attempting
    /// to free memory incorrectly.
    pub fn is_overlapping(&self, other: &Self) -> bool {
        if other.range().contains(&self.start())
            || other.range().contains(&self.end())
        {
            return true;
        }

        if self.range().contains(&other.start())
            || self.range().contains(&other.end())
        {
            return true;
        }

        false
    }

    fn range(&self) -> impl RangeBounds<u64> {
        (Bound::Excluded(self.start()), Bound::Excluded(self.end()))
    }

    /// Merge this region with another region.
    ///
    /// # Unsafe Because
    ///
    /// - this method does not check that the regions are adjacent before
    ///   merging
    unsafe fn merge_unsafe(&self, other: &Self) -> Self {
        Self {
            offset: self.offset.min(other.offset),
            size: self.size + other.size,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::graphics::vulkan::device_allocator::Allocation;

    #[test]
    pub fn start_end() {
        assert_eq!(Region::new(8, 123).start(), 8);
        assert_eq!(Region::new(8, 123).end(), 131);
    }

    #[test]
    pub fn test_overlapping() {
        let big = Region::new(10, 10);
        let small = Region::new(15, 2);

        assert!(big.is_overlapping(&small));
        assert!(small.is_overlapping(&big));
        assert!(big.is_contiguous(&small) == false);
        assert!(small.is_contiguous(&big) == false);

        let left = Region::new(7, 4);
        assert!(left.is_contiguous(&big) == false);
        assert!(left.is_overlapping(&big));
        assert!(big.is_overlapping(&left));

        let right = Region::new(18, 8);
        assert!(right.is_contiguous(&big) == false);
        assert!(right.is_overlapping(&big));
        assert!(big.is_overlapping(&right));

        assert!(left.is_overlapping(&right) == false);
    }

    #[test]
    pub fn new_region_for_whole_allocation_test() {
        let allocation = dummy_allocation(23, 1234);
        let region = Region::new_whole_region(&allocation);
        assert_eq!(region.offset, 23);
        assert_eq!(region.size, 1234);
    }

    #[test]
    pub fn is_contiguous_test() {
        let a = Region::new(0, 512);
        let b = Region::new(512, 256);
        assert!(a.is_contiguous(&b));
        assert!(a.is_overlapping(&b) == false);
        assert!(a.is_contiguous(&Region::new(513, 20)) == false);
    }

    #[test]
    pub fn merge_contiguous() {
        let a = Region::new(0, 512);
        let b = Region::new(512, 256);
        assert_eq!(a.merge(&b), MergeResult::Contiguous(Region::new(0, 768)));
        assert_eq!(b.merge(&a), MergeResult::Contiguous(Region::new(0, 768)));
    }

    #[test]
    pub fn merge_non_contiguous() {
        let a = Region::new(0, 256);
        let b = Region::new(512, 256);
        assert_eq!(a.merge(&b), MergeResult::NonContiguous);
        assert_eq!(b.merge(&a), MergeResult::NonContiguous);
    }

    #[test]
    pub fn take_subregion_test() {
        let mut a = Region::new(0, 256);
        let b = a.take_subregion(64);
        assert_eq!(Region::new(0, 64), b);
        assert_eq!(Region::new(64, 192), a);
    }

    fn dummy_allocation(offset: u64, size: u64) -> Allocation {
        let mut allocation = Allocation::null();
        allocation.offset = offset;
        allocation.byte_size = size;
        allocation
    }
}
