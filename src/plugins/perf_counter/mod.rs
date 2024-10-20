#[cfg(target_os = "linux")]
pub(crate) mod linux;

use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(not(target_os = "linux"))]
pub(crate) mod dummy;

#[cfg(not(target_os = "linux"))]
pub use dummy::*;

use crate::report::format::format_with_underscores_f64;
use crate::stats::*;
use miniserde::Deserialize;
use miniserde::Serialize;

/// Name of the event listener
pub static PERF_CNT_EVENT_LISTENER_NAME: &str = "_binggan_perf";

/// Enum representing different performance counters used in profiling.
///
/// ## Legend
/// - Br: Branches
/// - A: Accesses
/// - M: Missed
/// - TLB: Translation Lookaside Buffer
/// - d: Data
/// - Instr: Instructions
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PerfCounter {
    /// Count of total CPU cycles. May be more stable than wall-clock time for performance
    /// measurements.
    CpuCycles,
    /// Count of total branch instructions executed.
    Branches,
    /// Count of branch instructions that resulted in mispredictions.
    MissedBranches,
    /// Count of accesses to the Level 1 Data (L1d) cache.
    L1DCacheAccess,
    /// Count of misses in the Level 1 Data (L1d) cache, where the data was not found.
    L1DCacheMiss,
    /// Count of accesses to the Data Translation Lookaside Buffer (dTLB).
    TLBDataAccess,
    /// Count of misses in the Data Translation Lookaside Buffer (dTLB), where the virtual to physical address translation was not found.
    TLBDataMiss,
    /// Count of total instructions retired (completed) by the CPU.
    /// This will not count instructions that were not completed due to branch mispredictions.
    InstructionsRetired,
    /// Software event that counts the number of page faults.
    PageFaults,
    /// Minor page faults did not require disk I/O to handle.
    PageFaultsMinor,
    /// Major page faults required disk I/O to handle.
    PageFaultsMajor,
}

/// A static array of mappings between `PerfCounter` variants and their string identifiers.
const MAPPINGS: &[(&str, PerfCounter)] = &[
    ("Cycles", PerfCounter::CpuCycles),
    ("Br", PerfCounter::Branches),
    ("BrM", PerfCounter::MissedBranches),
    ("L1dA", PerfCounter::L1DCacheAccess),
    ("L1dM", PerfCounter::L1DCacheMiss),
    ("dTLBA", PerfCounter::TLBDataAccess),
    ("dTLBM", PerfCounter::TLBDataMiss),
    ("IRet", PerfCounter::InstructionsRetired),
    ("PGF", PerfCounter::PageFaults),
    ("PGFMin", PerfCounter::PageFaultsMinor),
    ("PGFMaj", PerfCounter::PageFaultsMajor),
];

impl Display for PerfCounter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let val = MAPPINGS
            .iter()
            .find(|(_, counter)| *counter == *self)
            .map(|(s, _)| *s)
            .expect("Invalid PerfCounter");
        write!(f, "{}", val)
    }
}
impl FromStr for PerfCounter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        MAPPINGS
            .iter()
            .find(|(key, _)| *key == s)
            .map(|(_, counter)| *counter)
            .ok_or_else(|| format!("Invalid PerfCounter: {}", s))
    }
}
/// Get the default performance counters.
pub fn default_perf_counters() -> &'static [PerfCounter] {
    &[
        PerfCounter::Branches,
        PerfCounter::MissedBranches,
        PerfCounter::L1DCacheAccess,
        PerfCounter::L1DCacheMiss,
        PerfCounter::CpuCycles,
    ]
}

/// Counter values from perf.
///
/// This struct is used to store the counter values from perf.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfCounterValues {
    values: Vec<(PerfCounter, f64)>,
}

/// Print Counter value
fn print_counter_value<F: Fn(f64) -> f64>(
    name: &str,
    value: f64,
    other: Option<f64>,
    f: F,
) -> String {
    let diff_str = other
        .map(|other_value| {
            if other_value == 0.0 || value == 0.0 || other_value == value {
                return "".to_string();
            }

            format_percentage(compute_percentage_diff(value, other_value), true)
        })
        .unwrap_or_default();

    format!(
        "{}: {} {}",
        name,
        format_with_underscores_f64(f(value)),
        diff_str,
    )
}

impl PerfCounterValues {
    /// Method to compare two `Vec<(PerfCounter, f64)>` instances and return formatted columns
    pub fn to_columns(&self, other_values: Option<&Self>) -> Vec<String> {
        let mut result = Vec::new();

        for (counter_enum, value) in self.values.iter() {
            // Find corresponding value in other, if present
            let other_value = other_values.and_then(|other| {
                other
                    .values
                    .iter()
                    .find(|(other_enum, _)| other_enum == counter_enum)
                    .map(|(_, v)| *v)
            });

            // Print the counter value and the optional difference
            result.push(print_counter_value(
                &format!("{}", counter_enum),
                *value,
                other_value,
                |val| val,
            ));
        }

        result
    }
}
