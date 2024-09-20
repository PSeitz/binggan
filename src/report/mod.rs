//!
//! Module for reporting
//!
//! The `report` module contains the [report::Reporter] trait and the [report::PlainReporter] struct.
//! You can set the reporter with the [BenchRunner::set_reporter] method.
//!

/// The plain_reporter
mod plain_reporter;
/// The table_reporter
mod table_reporter;

pub use crate::stats::BenchStats;
pub use plain_reporter::PlainReporter;
pub use table_reporter::TableReporter;

use yansi::Paint;

use crate::{
    bench::{Bench, BenchResult},
    format::{bytes_to_string, format_duration},
    stats::compute_diff,
    write_results::fetch_previous_run_and_write_results_to_disk,
};

/// The trait for reporting the results of a benchmark run.
pub trait Reporter: ReporterClone {
    /// Report the results from a group (can be a single bench)
    fn report_results(&self, results: Vec<BenchResult>);
}

/// The trait to enable cloning on the Box reporter
pub trait ReporterClone {
    /// Clone the box
    fn clone_box(&self) -> Box<dyn Reporter>;
}

pub(crate) fn report_group<'a>(
    benches: &mut [Box<dyn Bench<'a> + 'a>],
    reporter: &dyn Reporter,
    report_memory: bool,
) {
    if benches.is_empty() {
        return;
    }

    let mut results = Vec::new();
    for bench in benches.iter_mut() {
        let mut result = bench.get_results(report_memory);
        fetch_previous_run_and_write_results_to_disk(&mut result);
        results.push(result);
    }
    reporter.report_results(results);
}

pub(crate) fn avg_median_str(
    stats: &BenchStats,
    input_size_in_bytes: Option<usize>,
    other: Option<BenchStats>,
) -> (String, String) {
    let avg_ns_diff = compute_diff(stats, input_size_in_bytes, other, |stats| stats.average_ns);
    let median_ns_diff = compute_diff(stats, input_size_in_bytes, other, |stats| stats.median_ns);

    // if input_size_in_bytes is set report the throughput, otherwise just use format_duration
    let avg_str = format!(
        "{} {}",
        format(stats.average_ns, input_size_in_bytes),
        avg_ns_diff,
    );
    let median_str = format!(
        "{} {}",
        format(stats.median_ns, input_size_in_bytes),
        median_ns_diff,
    );
    (avg_str, median_str)
}

pub(crate) fn min_max_str(stats: &BenchStats, input_size_in_bytes: Option<usize>) -> String {
    if input_size_in_bytes.is_none() {
        format!(
            "[{} .. {}]",
            format(stats.min_ns, None),
            format(stats.max_ns, None)
        )
    } else {
        format!(
            "[{} .. {}]",
            format(stats.max_ns, input_size_in_bytes), // flip min and max
            format(stats.min_ns, input_size_in_bytes)
        )
    }
}

pub(crate) fn memory_str(
    stats: &BenchStats,
    other: Option<BenchStats>,
    report_memory: bool,
) -> String {
    let mem_diff = compute_diff(stats, None, other, |stats| stats.avg_memory as u64);
    if !report_memory {
        return "".to_string();
    }
    format!(
        "Memory: {} {}",
        bytes_to_string(stats.avg_memory as u64)
            .bright_cyan()
            .bold(),
        mem_diff,
    )
}

fn format(duration_ns: u64, input_size_in_bytes: Option<usize>) -> String {
    if let Some(input_size_in_bytes) = input_size_in_bytes {
        let mut duration_ns: f64 = duration_ns as f64;
        let unit = unit_per_second(input_size_in_bytes, &mut duration_ns);
        format!("{:>6} {}", short(duration_ns), unit)
    } else {
        format_duration(duration_ns).to_string()
    }
}

/// Formats a floating-point number (`f64`) into a shorter, human-readable string
/// with varying precision depending on the value of the number.
///
/// # Parameters
/// - `n`: The floating-point number to format.
///
/// # Returns
/// A string representation of the number with different decimal precision based on its value:
/// - If `n` is less than 10, it will be formatted with 4 decimal places.
/// - If `n` is between 10 and 100, it will be formatted with 3 decimal places.
/// - If `n` is between 100 and 1000, it will be formatted with 2 decimal places.
/// - If `n` is between 1000 and 10000, it will be formatted with 1 decimal place.
/// - If `n` is greater than or equal to 10000, it will be formatted with no decimal places.
///
/// # Examples
/// ```
/// use binggan::report::short;
/// let value = 9.876543;
/// assert_eq!(short(value), "9.8765");
///
/// let value = 987.6543;
/// assert_eq!(short(value), "987.65");
///
/// let value = 12345.67;
/// assert_eq!(short(value), "12346");
/// ```
pub fn short(n: f64) -> String {
    if n < 10.0 {
        format!("{:.4}", n)
    } else if n < 100.0 {
        format!("{:.3}", n)
    } else if n < 1000.0 {
        format!("{:.2}", n)
    } else if n < 10000.0 {
        format!("{:.1}", n)
    } else {
        format!("{:.0}", n)
    }
}

/// Returns the unit and alters the passed parameter to match the unit
pub fn unit_per_second(bytes: usize, nanoseconds: &mut f64) -> &'static str {
    let bytes_per_second = bytes as f64 * (1e9 / *nanoseconds);
    let (denominator, unit) = if bytes_per_second < 1024.0 {
        (1.0, "  B/s")
    } else if bytes_per_second < 1024.0 * 1024.0 {
        (1024.0, "KiB/s")
    } else if bytes_per_second < 1024.0 * 1024.0 * 1024.0 {
        (1024.0 * 1024.0, "MiB/s")
    } else {
        (1024.0 * 1024.0 * 1024.0, "GiB/s")
    };

    let bytes_per_second = bytes as f64 * (1e9 / *nanoseconds);
    *nanoseconds = bytes_per_second / denominator;

    unit
}
