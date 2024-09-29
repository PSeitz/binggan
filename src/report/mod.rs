//!
//! Module for reporting
//!
//! The `report` module contains the [report::Reporter] trait and the [report::PlainReporter] struct.
//! You can set the reporter with the [BenchRunner::set_reporter] method.
//!

/// Helper methods to format benchmark results
pub mod format;
/// The plain_reporter
mod plain_reporter;
/// The table_reporter
mod table_reporter;

pub use crate::stats::BenchStats;
pub use plain_reporter::PlainReporter;
pub use table_reporter::TableReporter;

use yansi::Paint;

use format::{bytes_to_string, format_duration_or_throughput};

use crate::{
    bench::{Bench, BenchResult},
    plugins::{BingganEvents, EventManager},
    stats::compute_diff,
    write_results::fetch_previous_run_and_write_results_to_disk,
};

/// The trait for reporting the results of a benchmark run.
pub trait Reporter: ReporterClone {
    /// Report the results from a group (can be a single bench)
    fn report_results(&self, results: Vec<BenchResult>, output_value_column_title: &'static str);
}

/// The trait to enable cloning on the Box reporter
pub trait ReporterClone {
    /// Clone the box
    fn clone_box(&self) -> Box<dyn Reporter>;
}

pub(crate) fn report_group<'a>(
    group_name: Option<&str>,
    benches: &mut [Box<dyn Bench<'a> + 'a>],
    reporter: &dyn Reporter,
    output_value_column_title: &'static str,
    events: &mut EventManager,
) {
    if benches.is_empty() {
        return;
    }

    let mut results = Vec::new();
    for bench in benches.iter_mut() {
        let mut result = bench.get_results(events);
        fetch_previous_run_and_write_results_to_disk(&mut result);
        results.push(result);
    }
    events.emit(BingganEvents::GroupStop {
        name: group_name,
        results: &results,
        output_value_column_title,
    });
    reporter.report_results(results, output_value_column_title);
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
        format_duration_or_throughput(stats.average_ns, input_size_in_bytes),
        avg_ns_diff,
    );
    let median_str = format!(
        "{} {}",
        format_duration_or_throughput(stats.median_ns, input_size_in_bytes),
        median_ns_diff,
    );
    (avg_str, median_str)
}

pub(crate) fn min_max_str(stats: &BenchStats, input_size_in_bytes: Option<usize>) -> String {
    if input_size_in_bytes.is_none() {
        format!(
            "[{} .. {}]",
            format_duration_or_throughput(stats.min_ns, None),
            format_duration_or_throughput(stats.max_ns, None)
        )
    } else {
        format!(
            "[{} .. {}]",
            format_duration_or_throughput(stats.max_ns, input_size_in_bytes), // flip min and max
            format_duration_or_throughput(stats.min_ns, input_size_in_bytes)
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
