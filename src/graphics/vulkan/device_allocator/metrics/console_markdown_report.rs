use super::{Metrics, MetricsReport};

use std::collections::HashMap;

/// Build a human-friendly markdown report which is printed directly to the
/// console.
///
/// Reports Look similar to the following:-
///
/// ```markdown
/// # Device Allocator - Memory Report Totals
///
///   |                Metric Name | Value        |
///   | -------------------------- | ------------ |
///   | max concurrent allocations | 6            |
///   |     total allocation count | 7            |
///   |    leaked allocation count | 0            |
///   |       mean allocation size | 25.07 KiB    |
///   |         biggest allocation | 87 KiB       |
///   |        smallest allocation | 256 B        |
///
///
/// ## Metrics By Memory Type Index
///
/// ### Memory Type 7
///
///   |                Metric Name | Value        |
///   | -------------------------- | ------------ |
///   | max concurrent allocations | 2            |
///   |     total allocation count | 2            |
///   |    leaked allocation count | 0            |
///   |       mean allocation size | 43.75 KiB    |
///   |         biggest allocation | 87 KiB       |
///   |        smallest allocation | 512 B        |
///
/// ### Memory Type 8
///
///   |                Metric Name | Value        |
///   | -------------------------- | ------------ |
///   | max concurrent allocations | 4            |
///   |     total allocation count | 5            |
///   |    leaked allocation count | 0            |
///   |       mean allocation size | 17.599 KiB   |
///   |         biggest allocation | 85.5 KiB     |
///   |        smallest allocation | 256 B        |
/// ```
///
pub struct ConsoleMarkdownReport {}

impl ConsoleMarkdownReport {
    const BASE: u64 = 1024;
    const UNITS: [&'static str; 4] = ["B", "KiB", "MiB", "GiB"];

    pub fn new() -> Self {
        Self {}
    }

    fn formatted_metrics_list(metrics: &Metrics) -> String {
        indoc::formatdoc!(
            "
            {indent}|                Metric Name | {header:<12} |
            {indent}| -------------------------- | {underline:<12} |
            {indent}| max concurrent allocations | {:<12} |
            {indent}|     total allocation count | {:<12} |
            {indent}|    leaked allocation count | {:<12} |
            {indent}|       mean allocation size | {:<12} |
            {indent}|         biggest allocation | {:<12} |
            {indent}|        smallest allocation | {:<12} |
            ",
            metrics.max_concurrent_allocations,
            metrics.total_allocations,
            metrics.current_allocations,
            Self::pretty_print_bytes(metrics.mean_allocation_byte_size),
            Self::pretty_print_bytes(metrics.biggest_allocation),
            Self::pretty_print_bytes(metrics.smallest_allocation),
            header = "Value",
            underline = "------------",
            indent = "  ",
        )
    }

    fn pretty_print_bytes(byte_size: u64) -> String {
        let order_of_magnitude = Self::order_of_magnitude(byte_size);
        let divisor = Self::BASE.pow(order_of_magnitude as u32);

        // This weird bit of gymnastics ensures that there are at *most* 3
        // numbers after the decimal, but only if the precision is actually
        // needed. (e.g. 256.000 will render as just 256 in the final report)
        let full_print = format!("{:.3}", byte_size as f32 / divisor as f32);
        let truncated: f32 = full_print.parse().unwrap();

        format!(
            "{size} {unit}",
            size = truncated,
            unit = Self::UNITS[order_of_magnitude]
        )
    }

    /// Compute the order of magnitude in units of 1024.
    ///
    /// This could probably be improved with some logarithms (division in a
    /// loop, I'm looking at you!), but the while loop works just fine.
    fn order_of_magnitude(byte_size: u64) -> usize {
        let mut order_of_magnitude = 0;
        let mut remaining_precision = byte_size as f64;
        while remaining_precision >= 1.0 {
            remaining_precision /= 1024.0;
            order_of_magnitude += 1;
        }
        if order_of_magnitude > 0 {
            order_of_magnitude -= 1;
        }
        order_of_magnitude.min(Self::UNITS.len())
    }
}

impl MetricsReport for ConsoleMarkdownReport {
    fn render(
        &self,
        name: &str,
        total: &Metrics,
        metrics_by_type: &HashMap<u32, Metrics>,
    ) {
        let mut report = indoc::formatdoc!(
            "

            # {name} - Memory Report Totals

            {totals}

            ## Metrics By Memory Type Index

            ",
            name = name,
            totals = Self::formatted_metrics_list(total)
        );

        for (memory_type_index, metrics) in metrics_by_type {
            report += indoc::formatdoc!(
                "### Memory Type {type}

                {metrics}
                ",
                type = memory_type_index,
                metrics = Self::formatted_metrics_list(&metrics)
            )
            .as_ref();
        }

        log::info!("{}", report);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_pretty_print_bytes() {
        use ConsoleMarkdownReport as CMR;

        assert_eq!(CMR::pretty_print_bytes(1), "1 B");
        assert_eq!(CMR::pretty_print_bytes(1024), "1 KiB");
        assert_eq!(CMR::pretty_print_bytes(1024 * 1024), "1 MiB");
        assert_eq!(CMR::pretty_print_bytes(1024 * 1024 * 1024), "1 GiB");

        assert_eq!(CMR::pretty_print_bytes(1000), "1000 B");
        assert_eq!(CMR::pretty_print_bytes(1034), "1.01 KiB");
        assert_eq!(CMR::pretty_print_bytes(2048), "2 KiB");
        assert_eq!(CMR::pretty_print_bytes(4096), "4 KiB");
        assert_eq!(CMR::pretty_print_bytes(34235), "33.433 KiB");
    }
}
