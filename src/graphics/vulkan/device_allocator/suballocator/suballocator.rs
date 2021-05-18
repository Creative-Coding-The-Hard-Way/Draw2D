use crate::graphics::vulkan::device_allocator::{Allocation, DeviceAllocator};

use super::region::{MergeResult, Region};

use anyhow::Result;

use super::Suballocator;

impl Suballocator {
    pub fn new(allocation: Allocation) -> Self {
        Self {
            free_regions: vec![Region::new(
                allocation.offset,
                allocation.byte_size,
            )],
            block: allocation,
        }
    }

    pub unsafe fn free_block(
        &mut self,
        allocator: &mut impl DeviceAllocator,
    ) -> Result<()> {
        allocator.free(&self.block)?;
        self.block = Allocation::null();
        Ok(())
    }

    /// Find and take a region with the requested size from the free regions.
    ///
    /// If no region is large enough, or no regions are remaining, then None is
    /// returned.
    pub fn allocate_region(&mut self, size: u64) -> Option<Region> {
        for i in 0..self.free_regions.len() {
            if size == self.free_regions[i].size {
                return Some(self.free_regions.remove(i));
            } else if size < self.free_regions[i].size {
                return Some(self.free_regions[i].take_subregion(size));
            }
        }
        None
    }

    /// Free a subregion back into the set of free regions.
    /// Regions are automatically joined to minimize fragmentation.
    pub fn free_region(&mut self, region: Region) -> Result<()> {
        let mut was_merged = false;
        let mut i = 0;

        while i < self.free_regions.len() && !was_merged {
            if self.free_regions[i] == region
                || self.free_regions[i].is_overlapping(&region)
            {
                anyhow::bail!(
                    "Attempting to free a suballocation twice will lead to
                     data inconsistency!"
                );
            } else if let MergeResult::Contiguous(merged) =
                region.merge(&self.free_regions[i])
            {
                let mut to_insert = merged;

                // check if the new merged region can fuse with the next free
                // region too. If it can, then build the fully merged region
                // and remove one entry from the free region vector.
                if i + 1 < self.free_regions.len() {
                    if let MergeResult::Contiguous(merged) =
                        to_insert.merge(&self.free_regions[i + 1])
                    {
                        to_insert = merged;
                        self.free_regions.remove(i + 1);
                    }
                }

                self.free_regions[i] = to_insert;
                was_merged = true;
            } else {
                if region.end() < self.free_regions[i].start() {
                    break;
                }
                i += 1;
            }
        }

        // The region is not contiguous with any other region in the
        // free_region vector. Insert it wherever the merge loop stopped so
        // that free_regions stay consecutive.
        if !was_merged {
            self.free_regions.insert(i, region);
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.free_regions.len() == 1
            && self.free_regions[0].offset == 0
            && self.free_regions[0].size == self.block.byte_size
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::graphics::vulkan::device_allocator::Allocation;

    #[test]
    pub fn test_allocate_region() {
        let allocation = fake_allocation(1024);
        let mut suballocator = Suballocator::new(allocation);

        assert_eq!(suballocator.free_regions, vec![Region::new(0, 1024)]);

        let region = suballocator.allocate_region(256);
        assert_eq!(region, Some(Region::new(0, 256)));
        assert_eq!(suballocator.free_regions, vec![Region::new(256, 768)]);

        let remaining = suballocator.allocate_region(768);
        assert_eq!(remaining, Some(Region::new(256, 768)));
        assert_eq!(suballocator.free_regions, vec![]);
    }

    #[test]
    pub fn test_free_whole_region() {
        let mut sub = Suballocator::new(fake_allocation(1024));

        let region = sub.allocate_region(1024).unwrap();
        assert_eq!(region, Region::new(0, 1024));
        assert_eq!(sub.free_regions, vec![]);

        sub.free_region(region);
        assert_eq!(sub.free_regions, vec![Region::new(0, 1024)]);
    }

    #[test]
    pub fn test_split_region() {
        let mut sub = Suballocator::new(fake_allocation(1024));

        let region = sub.allocate_region(512).unwrap();
        assert_eq!(region, Region::new(0, 512));
        assert_eq!(sub.free_regions, vec![Region::new(512, 512)]);

        sub.free_region(region);
        assert_eq!(sub.free_regions, vec![Region::new(0, 1024)]);
    }

    #[test]
    pub fn test_merge_front_and_back() {
        let mut sub = Suballocator::new(fake_allocation(1024));

        let a = sub.allocate_region(256).unwrap();
        let b = sub.allocate_region(512).unwrap();
        let c = sub.allocate_region(256).unwrap();

        assert_eq!(sub.free_regions, vec![]);

        sub.free_region(c);
        assert_eq!(sub.free_regions, vec![Region::new(768, 256)]);

        sub.free_region(a);
        assert_eq!(
            sub.free_regions,
            vec![Region::new(0, 256), Region::new(768, 256)]
        );

        sub.free_region(b);
        assert_eq!(sub.free_regions, vec![Region::new(0, 1024)]);
    }

    #[test]
    pub fn test_merge_front_with_leading() {
        let mut sub = Suballocator::new(fake_allocation(1024));

        let a = sub.allocate_region(256).unwrap();
        let b = sub.allocate_region(256).unwrap();
        let c = sub.allocate_region(256).unwrap();
        let d = sub.allocate_region(256).unwrap();

        assert_eq!(sub.free_regions, vec![]);

        sub.free_region(a);
        assert_eq!(sub.free_regions, vec![Region::new(0, 256)]);

        sub.free_region(d);
        assert_eq!(
            sub.free_regions,
            vec![Region::new(0, 256), Region::new(768, 256)]
        );

        sub.free_region(c);
        assert_eq!(
            sub.free_regions,
            vec![Region::new(0, 256), Region::new(512, 512)]
        );

        sub.free_region(b);
        assert_eq!(sub.free_regions, vec![Region::new(0, 1024)]);
    }

    #[test]
    pub fn test_merge_back_with_trailing() {
        let mut sub = Suballocator::new(fake_allocation(1024));

        let a = sub.allocate_region(256).unwrap();
        let b = sub.allocate_region(256).unwrap();
        let c = sub.allocate_region(256).unwrap();
        let d = sub.allocate_region(256).unwrap();

        assert_eq!(sub.free_regions, vec![]);

        sub.free_region(a);
        assert_eq!(sub.free_regions, vec![Region::new(0, 256)]);

        sub.free_region(d);
        assert_eq!(
            sub.free_regions,
            vec![Region::new(0, 256), Region::new(768, 256)]
        );

        sub.free_region(b);
        assert_eq!(
            sub.free_regions,
            vec![Region::new(0, 512), Region::new(768, 256)]
        );

        sub.free_region(c);
        assert_eq!(sub.free_regions, vec![Region::new(0, 1024)]);
    }

    #[should_panic]
    #[test]
    pub fn test_double_free() {
        let mut sub = Suballocator::new(fake_allocation(1024));
        let a = sub.allocate_region(512).unwrap();
        let b = sub.allocate_region(512).unwrap();
        sub.free_region(b).unwrap();
        sub.free_region(a).unwrap();
        sub.free_region(a).unwrap();
    }

    fn fake_allocation(size: u64) -> Allocation {
        let mut allocation = Allocation::null();
        allocation.byte_size = size;
        allocation
    }
}
