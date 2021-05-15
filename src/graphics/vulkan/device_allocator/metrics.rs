use super::{Allocation, DeviceAllocator};

use std::collections::HashMap;

use anyhow::Result;
use ash::vk;

#[derive(Debug, Copy, Clone)]
struct Metrics {
    total_allocations: u32,
    max_concurrent_allocations: u32,
    current_allocations: u32,
    mean_allocation_byte_size: u64,
    biggest_allocation: u64,
    smallest_allocation: u64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            total_allocations: 0,
            max_concurrent_allocations: 0,
            current_allocations: 0,
            mean_allocation_byte_size: 0,
            biggest_allocation: 0,
            smallest_allocation: u64::MAX,
        }
    }
}

impl Metrics {
    pub fn add_alloctation(&mut self, allocation: &Allocation) {
        self.current_allocations += 1;

        // weighted average of allocation sizes
        self.mean_allocation_byte_size = ((self.mean_allocation_byte_size
            * self.total_allocations as u64)
            + allocation.byte_size)
            / (self.total_allocations + 1) as u64;

        self.total_allocations += 1;

        self.max_concurrent_allocations = self
            .max_concurrent_allocations
            .max(self.current_allocations);
        self.biggest_allocation =
            self.biggest_allocation.max(allocation.byte_size);
        self.smallest_allocation =
            self.smallest_allocation.min(allocation.byte_size);
    }

    pub fn remove_allocation(&mut self) {
        self.current_allocations -= 1;
    }
}

/// A device allocator decorator which records the number of allocations and
/// other metrics. A summary of results is printed when the allocator is
/// destroyed.
pub struct MetricsAllocator<Alloc: DeviceAllocator> {
    allocator: Alloc,
    name: String,
    by_type: HashMap<u32, Metrics>,
}

impl<Alloc: DeviceAllocator> MetricsAllocator<Alloc> {
    /// Decorate an existing allocator with support for metrics.
    pub fn new(name: impl Into<String>, allocator: Alloc) -> Self {
        Self {
            name: name.into(),
            allocator,
            by_type: HashMap::new(),
        }
    }

    fn record_allocation(
        &mut self,
        memory_requirements: vk::MemoryRequirements,
        allocation: &Allocation,
    ) {
        if !self.by_type.contains_key(&allocation.memory_type_index) {
            self.by_type
                .insert(allocation.memory_type_index, Metrics::default());
        }
        self.by_type
            .get_mut(&allocation.memory_type_index)
            .unwrap()
            .add_alloctation(allocation);
    }

    fn record_free(&mut self, allocation: &Allocation) {
        if !self.by_type.contains_key(&allocation.memory_type_index) {
            return;
        }
        self.by_type
            .get_mut(&allocation.memory_type_index)
            .unwrap()
            .remove_allocation();
    }

    fn build_report(&self) -> String {
        let mut report = indoc::formatdoc!(
            "

            # {} - Memory Report

            ",
            self.name
        );

        for (memory_type_index, metrics) in &self.by_type {
            let entry = indoc::formatdoc!(
                "
                ## Metrics For Memory Type Index {}

                  - max concurrent allocations | {}
                  -          total allocations | {}
                  -         mean size in bytes | {}b
                  -         largest allocation | {}b
                  -        smallest allocation | {}b
                  -         leaked allocations | {}

                ",
                memory_type_index,
                metrics.max_concurrent_allocations,
                metrics.total_allocations,
                metrics.mean_allocation_byte_size,
                metrics.biggest_allocation,
                metrics.smallest_allocation,
                metrics.current_allocations
            );
            report += entry.as_ref();
        }

        report
    }
}

impl<Alloc: DeviceAllocator> DeviceAllocator for MetricsAllocator<Alloc> {
    unsafe fn allocate(
        &mut self,
        memory_requirements: vk::MemoryRequirements,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Allocation> {
        let allocation = self
            .allocator
            .allocate(memory_requirements, property_flags)?;
        self.record_allocation(memory_requirements, &allocation);
        Ok(allocation)
    }

    unsafe fn free(&mut self, allocation: &Allocation) -> Result<()> {
        self.record_free(allocation);
        self.allocator.free(allocation)
    }

    fn managed_by_me(&self, allocation: &Allocation) -> bool {
        self.allocator.managed_by_me(allocation)
    }
}

impl<T: DeviceAllocator> Drop for MetricsAllocator<T> {
    fn drop(&mut self) {
        log::debug!("{}", self.build_report());
    }
}
