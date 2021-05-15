mod metrics;

pub use self::metrics::Metrics;

use super::{Allocation, DeviceAllocator};

use anyhow::Result;
use ash::vk;
use std::collections::HashMap;

/// A device allocator decorator which records the number of allocations and
/// other metrics. A summary of results is printed when the allocator is
/// destroyed.
pub struct MetricsAllocator<Alloc: DeviceAllocator> {
    allocator: Alloc,
    name: String,
    by_type: HashMap<u32, Metrics>,
    total: Metrics,
}

impl<Alloc: DeviceAllocator> MetricsAllocator<Alloc> {
    /// Decorate an existing allocator with support for metrics.
    pub fn new(name: impl Into<String>, allocator: Alloc) -> Self {
        Self {
            name: name.into(),
            allocator,
            by_type: HashMap::new(),
            total: Metrics::default(),
        }
    }

    fn record_allocation(
        &mut self,
        _memory_requirements: vk::MemoryRequirements,
        allocation: &Allocation,
    ) {
        self.total.measure_alloctaion(&allocation);
        self.by_type
            .entry(allocation.memory_type_index)
            .or_default()
            .measure_alloctaion(allocation);
    }

    fn record_free(&mut self, allocation: &Allocation) {
        if allocation.is_null() {
            return;
        }
        self.total.measure_free();
        self.by_type
            .entry(allocation.memory_type_index)
            .and_modify(|metrics| metrics.measure_free());
    }

    fn build_report(&self) -> String {
        let mut report = indoc::formatdoc!(
            "

            # {} - Memory Report

            ",
            self.name
        );

        report += indoc::formatdoc!(
            "
            ## Total Across All Memory Types

              - max concurrent allocations | {}
              -          total allocations | {}
              -         mean size in bytes | {}b
              -         largest allocation | {}b
              -        smallest allocation | {}b
              -         leaked allocations | {}

            ",
            self.total.max_concurrent_allocations,
            self.total.total_allocations,
            self.total.mean_allocation_byte_size,
            self.total.biggest_allocation,
            self.total.smallest_allocation,
            self.total.current_allocations
        )
        .as_ref();

        for (memory_type_index, metrics) in &self.by_type {
            report += indoc::formatdoc!(
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
            )
            .as_ref();
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
