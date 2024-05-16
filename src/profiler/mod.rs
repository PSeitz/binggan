use crate::stats::*;
use miniserde::*;

#[cfg(not(feature = "perf_event"))]
pub(crate) mod dummy_profiler;
#[cfg(feature = "perf_event")]
pub(crate) mod perf_profiler;

#[cfg(not(feature = "perf_event"))]
pub(crate) use dummy_profiler::*;
#[cfg(feature = "perf_event")]
pub(crate) use perf_profiler::*;

use yansi::Paint;

pub trait Profiler {
    fn enable(&mut self);
    fn disable(&mut self);
    fn finish(&mut self, num_iter: u64) -> std::io::Result<CounterValues>;
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct CounterValues {
    /// Level 1 Data Cache Accesses
    pub l1d_access_count: f64,
    /// Level 1 Data Cache Misses
    pub l1d_miss_count: f64,
    /// TLB Data Cache Accesses
    pub tlbd_access_count: f64,
    /// TLB Data Cache Misses
    pub tlbd_miss_count: f64,
    pub branches_count: f64,
    pub missed_branches_count: f64,
}

/// Print Counter value
fn print_counter_value<F: Fn(&CounterValues) -> f64>(
    name: &str,
    stats: &CounterValues,
    other: Option<CounterValues>,
    f: F,
) -> String {
    let diff_str = other
        .as_ref()
        .map(|other| {
            if f(other) == 0.0 || f(stats) == 0.0 || f(other) == f(stats) {
                return "".to_string();
            }

            let val = f(stats);
            let other = f(other);
            format_percentage(compute_percentage_diff(val, other), true)
        })
        .unwrap_or_default();

    format!("{}: {:.3} {}", name, f(stats), diff_str,)
}

impl CounterValues {
    #[allow(dead_code)]
    pub fn print_legend() {
        println!(
            "{:16} {:16} {:16} {:16}",
            "L1DAccess: L1dA".red(),
            "L1DMiss: L1dM".green(),
            "Branches: Br".blue(),
            "Missed Branches: MBr".red()
        );
    }

    // Method to compare two `CounterValues` instances and return columns
    pub fn to_columns(self, other: Option<CounterValues>) -> Vec<String> {
        vec![
            print_counter_value("L1dA", &self, other, |stats| stats.l1d_access_count)
                .red()
                .to_string(),
            print_counter_value("L1dM", &self, other, |stats| stats.l1d_miss_count)
                .green()
                .to_string(),
            print_counter_value("TLBdA", &self, other, |stats| stats.tlbd_access_count)
                .red()
                .to_string(),
            print_counter_value("TLBdM", &self, other, |stats| stats.tlbd_miss_count)
                .red()
                .to_string(),
            print_counter_value("L1dA", &self, other, |stats| stats.l1d_access_count)
                .red()
                .to_string(),
            print_counter_value("Br", &self, other, |stats| stats.branches_count)
                .blue()
                .to_string(),
            print_counter_value("MBr", &self, other, |stats| stats.missed_branches_count)
                .red()
                .to_string(),
        ]
    }
}