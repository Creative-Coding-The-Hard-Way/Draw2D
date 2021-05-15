use crate::graphics::vulkan::device_allocator::Allocation;

#[derive(Debug, Copy, Clone)]
pub struct Metrics {
    pub total_allocations: u32,
    pub max_concurrent_allocations: u32,
    pub current_allocations: u32,
    pub mean_allocation_byte_size: u64,
    pub biggest_allocation: u64,
    pub smallest_allocation: u64,
}

impl Default for Metrics {
    /// Create an empty metrics object with all counters set to zero or their
    /// reasonable defaults.
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
    /// Update counters within the metrics data structure in response to a new
    /// allocation being acquired.
    pub fn measure_alloctaion(&mut self, allocation: &Allocation) {
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

    /// Update counters within the metrics data structure in response to an
    /// allocation being freed.
    pub fn measure_free(&mut self) {
        self.current_allocations -= 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mean_allocation_size() {
        let mut metrics = Metrics::default();
        metrics.measure_alloctaion(&allocation_with_size(100));
        metrics.measure_alloctaion(&allocation_with_size(50));
        metrics.measure_alloctaion(&allocation_with_size(75));

        assert_eq!(metrics.mean_allocation_byte_size, 75);
    }

    #[test]
    fn test_allocation_counters() {
        let mut metrics = Metrics::default();
        metrics.measure_alloctaion(&allocation_with_size(100));
        metrics.measure_alloctaion(&allocation_with_size(50));
        metrics.measure_alloctaion(&allocation_with_size(75));
        metrics.measure_free();

        assert_eq!(metrics.current_allocations, 2);
        assert_eq!(metrics.smallest_allocation, 50);
        assert_eq!(metrics.biggest_allocation, 100);
        assert_eq!(metrics.max_concurrent_allocations, 3);
    }

    fn allocation_with_size(size: u64) -> Allocation {
        let mut allocation = Allocation::null();
        allocation.byte_size = size;
        allocation
    }
}
