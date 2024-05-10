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
    pub l1d_access_count: f64,
    pub l1d_miss_count: f64,
    pub branches_count: f64,
    pub missed_branches_count: f64,
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
        let l1d_access_count_diff = other
            .as_ref()
            .map(|other| {
                format_percentage(
                    compute_percentage_diff(self.l1d_access_count, other.l1d_access_count),
                    true,
                )
            })
            .unwrap_or_default();
        let l1d_miss_count_diff = other
            .as_ref()
            .map(|other| {
                format_percentage(
                    compute_percentage_diff(self.l1d_miss_count, other.l1d_miss_count),
                    true,
                )
            })
            .unwrap_or_default();
        let branches_count_diff = other
            .as_ref()
            .map(|other| {
                format_percentage(
                    compute_percentage_diff(self.branches_count, other.branches_count),
                    true,
                )
            })
            .unwrap_or_default();
        let missed_branches_count_diff = other
            .as_ref()
            .map(|other| {
                format_percentage(
                    compute_percentage_diff(
                        self.missed_branches_count,
                        other.missed_branches_count,
                    ),
                    true,
                )
            })
            .unwrap_or_default();

        let l1da = format!(
            "L1dA: {:.3} {}",
            self.l1d_access_count, l1d_access_count_diff,
        );
        let l1dm = format!("L1dM: {:.3} {}", self.l1d_miss_count, l1d_miss_count_diff);
        let branches = format!("Br: {:.3} {}", self.branches_count, branches_count_diff);
        let branches_missed = format!(
            "BrM: {:.3} {}",
            self.missed_branches_count, missed_branches_count_diff
        );
        vec![
            l1da.red().to_string(),
            l1dm.green().to_string(),
            branches.blue().to_string(),
            branches_missed.red().to_string(),
        ]
    }
}
