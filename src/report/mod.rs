//!
//! Module for reporting
//!
//! The `report` module contains reporters that use the plugin system via the [EventListener](crate::plugins::EventListener)
//! trait.
//! You can set the reporter by registering at [BenchRunner::get_plugin_manager] .
//! Use [REPORTER_PLUGIN_NAME](crate::report::REPORTER_PLUGIN_NAME) as the name of a reporter, to overwrite the existing
//!

/// Helper methods to format benchmark results
pub mod format;
/// The plain_reporter
mod plain_reporter;
/// The table_reporter
#[cfg(feature = "table_reporter")]
mod table_reporter;

pub use crate::stats::BenchStats;
pub use plain_reporter::PlainReporter;

#[cfg_attr(docsrs, doc(cfg(feature = "table_reporter")))]
#[cfg(feature = "table_reporter")]
pub use table_reporter::TableReporter;

use yansi::Paint;

use format::{bytes_to_string, format_duration_or_throughput};

use crate::{
    bench::Bench,
    plugins::{PluginEvents, PluginManager},
    stats::compute_diff,
    write_results::fetch_previous_run_and_write_results_to_disk,
};

/// The default reporter name. Choose this in `EventListener` to make sure there's only one
/// reporter.
pub const REPORTER_PLUGIN_NAME: &str = "reporter";

pub(crate) fn report_group<'a>(
    runner_name: Option<&str>,
    group_name: Option<&str>,
    benches: &mut [Box<dyn Bench<'a> + 'a>],
    output_value_column_title: &'static str,
    events: &mut PluginManager,
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
    events.emit(PluginEvents::GroupStop {
        runner_name,
        group_name,
        results: &results,
        output_value_column_title,
    });
}

pub(crate) fn avg_median_str(
    stats: &BenchStats,
    input_size_in_bytes: Option<usize>,
    other: Option<BenchStats>,
) -> (String, String) {
    let avg_ns_diff = compute_diff(stats, input_size_in_bytes, other, |stats| stats.average_ns);
    let median_ns_diff = compute_diff(stats, input_size_in_bytes, other, |stats| stats.median_ns);

    // if input_size_in_bytes is set, report the throughput, otherwise just use format_duration
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

use std::{
    ops::Deref,
    sync::{Arc, Once},
};

/// Helper to print name only once.
///
/// The bench runners name is like a header and should only be printed if there are tests to be
/// run. Since this information is available at the time of creation, it will be handled when
/// executing the benches instead.
#[derive(Clone)]
pub struct PrintOnce {
    inner: Arc<PrintOnceInner>,
}

impl Deref for PrintOnce {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.inner.name
    }
}
struct PrintOnceInner {
    name: String,
    print_once: Once,
}

/// Check and print the name. This will only print the name once.
/// If the past named differs (include None), sets the new name to be printed once
pub fn check_and_print(print_once: &mut Option<PrintOnce>, name: &str) {
    if let Some(print_once) = print_once {
        print_once.check_print(name);
        return;
    }
    // set and print
    *print_once = Some(PrintOnce::new(name.to_owned()));
    print_once.as_ref().unwrap().print_name();
}

impl PrintOnce {
    /// Create a new PrintOnce instance
    pub fn new(name: String) -> Self {
        PrintOnce {
            inner: Arc::new(PrintOnceInner {
                name,
                print_once: Once::new(),
            }),
        }
    }

    /// Check and print the name. This will only print the name once.
    ///
    /// If the past named differs, sets the new name to be printed once
    pub fn check_print(&mut self, name: &str) {
        if self.get_name() != name {
            self.inner = Arc::new(PrintOnceInner {
                name: name.to_owned(),
                print_once: Once::new(),
            });
        }
        self.print_name();
    }

    /// Print the name. This will only print the name once.
    pub fn print_name(&self) {
        self.inner.print_once.call_once(|| {
            println!("{}", self.get_name().black().on_red().invert().bold());
        });
    }
    /// Get the name
    pub fn get_name(&self) -> &str {
        &self.inner.name
    }
}
