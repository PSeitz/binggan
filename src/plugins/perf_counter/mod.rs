#[cfg(target_os = "linux")]
pub(crate) mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;

use crate::stats::*;
use miniserde::Deserialize;
use miniserde::Serialize;

use yansi::Paint;

/// Name of the event listener
pub static PERF_CNT_EVENT_LISTENER_NAME: &str = "_binggan_perf";

/// Counter values from perf.
///
/// This struct is used to store the counter values from perf.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct CounterValues {
    /// Number of branches
    pub branches_count: f64,
    /// Number of missed branches
    pub missed_branches_count: f64,
    /// Level 1 Data Cache Accesses
    pub l1d_access_count: f64,
    /// Level 1 Data Cache Misses
    pub l1d_miss_count: f64,
    /// TLB Data Cache Accesses
    pub tlbd_access_count: f64,
    /// TLB Data Cache Misses
    pub tlbd_miss_count: f64,
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

    format!("{}: {} {}", name, format_number(f(stats)), diff_str,)
}

fn format_number(n: f64) -> String {
    let max_digits = 5;
    let integer_part = n.trunc() as i64;
    let integer_length = if integer_part != 0 {
        integer_part.abs().to_string().len() as i32
    } else if n == 0.0 {
        1 // Special handling for 0 to consider the digit before the decimal point
    } else {
        0 // For numbers less than 1 but not zero
    };

    let precision = (max_digits - integer_length).max(0) as usize;
    format!("{:.*}", precision, n)
}

impl CounterValues {
    #[allow(dead_code)]
    /// Print the legend for the counter values
    pub fn print_legend() {
        println!(
            "{:16} {:16} {:16} {:16}",
            "L1DAccess: L1dA".red(),
            "L1DMiss: L1dM".green(),
            "Branches: Br".blue(),
            "Missed Branches: MBr".red()
        );
    }

    /// Method to compare two `CounterValues` instances and return formatted columns
    pub fn to_columns(self, other: Option<CounterValues>) -> Vec<String> {
        vec![
            print_counter_value("Br", &self, other, |stats| stats.branches_count),
            print_counter_value("MBr", &self, other, |stats| stats.missed_branches_count),
            print_counter_value("L1dA", &self, other, |stats| stats.l1d_access_count),
            print_counter_value("L1dM", &self, other, |stats| stats.l1d_miss_count),
            print_counter_value("TLBdA", &self, other, |stats| stats.tlbd_access_count),
            print_counter_value("TLBdM", &self, other, |stats| stats.tlbd_miss_count),
            print_counter_value("L1dA", &self, other, |stats| stats.l1d_access_count),
        ]
    }
}
