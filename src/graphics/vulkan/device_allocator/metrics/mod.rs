mod console_markdown_report;
mod metrics;

pub use self::{
    console_markdown_report::ConsoleMarkdownReport, metrics::Metrics,
};

use super::{Allocation, DeviceAllocator};

use anyhow::Result;
use ash::vk;
use std::collections::HashMap;

/// Types which implement this trait can be used by the Metrics Allocator to
/// render a report on memory allocations.
pub trait MetricsReport {
    /// Render the metrics report.
    ///
    /// The output is implementation-defined (console, file, format, etc..).
    fn render(
        &self,
        name: &str,
        total: &Metrics,
        metrics_by_type: &HashMap<u32, Metrics>,
    );
}

/// A device allocator decorator which records the number of allocations and
/// other metrics. A summary of results is printed when the allocator is
/// destroyed.
pub struct MetricsAllocator<Alloc: DeviceAllocator> {
    allocator: Alloc,
    name: String,
    by_type: HashMap<u32, Metrics>,
    total: Metrics,
    report: Box<dyn MetricsReport>,
}

impl<Alloc: DeviceAllocator> MetricsAllocator<Alloc> {
    /// Decorate an existing allocator with support for metrics.
    pub fn new<Report: 'static + MetricsReport>(
        name: impl Into<String>,
        report: Report,
        allocator: Alloc,
    ) -> Self {
        Self {
            name: name.into(),
            allocator,
            by_type: HashMap::new(),
            total: Metrics::default(),
            report: Box::new(report),
        }
    }

    fn record_allocation(&mut self, allocation: &Allocation) {
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
}

impl<Alloc: DeviceAllocator> DeviceAllocator for MetricsAllocator<Alloc> {
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
    ) -> Result<Allocation> {
        let allocation = self.allocator.allocate(allocate_info)?;
        self.record_allocation(&allocation);
        Ok(allocation)
    }

    unsafe fn free(&mut self, allocation: &Allocation) -> Result<()> {
        self.record_free(allocation);
        self.allocator.free(allocation)
    }
}

impl<T: DeviceAllocator> Drop for MetricsAllocator<T> {
    fn drop(&mut self) {
        self.report.render(&self.name, &self.total, &self.by_type);
    }
}
