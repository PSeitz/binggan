use std::error::Error;

use perf_event::Counter;
use perf_event::{Builder, Group};

use miniserde::{Deserialize, Serialize};

pub trait Profiler {
    fn enable(&mut self);
    fn disable(&mut self);
    fn finish(&mut self, num_iter: u64) -> std::io::Result<CounterValues>;
}

use perf_event::events::{Cache, CacheOp, CacheResult, Hardware, WhichCache};
use yansi::Paint;

use crate::stats::{compute_percentage_diff, format_percentage};
pub(crate) struct PerfProfiler {
    group: Group,
    l1d_access_counter: Counter,
    l1d_miss_counter: Counter,
    branches: Counter,
    branch_misses: Counter,
}
impl PerfProfiler {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut group = Group::new()?;
        const L1D_ACCESS: Cache = Cache {
            which: WhichCache::L1D,
            operation: CacheOp::READ,
            result: CacheResult::ACCESS,
        };
        const L1D_MISS: Cache = Cache {
            result: CacheResult::MISS,
            ..L1D_ACCESS
        };
        let l1d_access_counter = Builder::new().group(&mut group).kind(L1D_ACCESS).build()?;
        let l1d_miss_counter = Builder::new().group(&mut group).kind(L1D_MISS).build()?;
        let branches = Builder::new()
            .group(&mut group)
            .kind(Hardware::BRANCH_INSTRUCTIONS)
            .build()?;
        let missed_branches = Builder::new()
            .group(&mut group)
            .kind(Hardware::BRANCH_MISSES)
            .build()?;
        group.disable()?;

        Ok(PerfProfiler {
            group,
            l1d_access_counter,
            l1d_miss_counter,
            branches,
            branch_misses: missed_branches,
        })
    }
}
impl Profiler for PerfProfiler {
    fn enable(&mut self) {
        self.group.enable().unwrap();
    }
    fn disable(&mut self) {
        self.group.disable().unwrap();
    }
    fn finish(&mut self, num_iter: u64) -> std::io::Result<CounterValues> {
        let num_iter = num_iter as f64;
        let l1d_access_count = self.l1d_access_counter.read()? as f64 / num_iter;
        let miss_count = self.l1d_miss_counter.read()? as f64 / num_iter;
        let branches_count = self.branches.read()? as f64 / num_iter;
        let missed_branches_count = self.branch_misses.read()? as f64 / num_iter;

        Ok(CounterValues {
            l1d_access_count,
            l1d_miss_count: miss_count,
            branches_count,
            missed_branches_count,
        })
    }
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
    pub fn to_columns(&self, other: Option<CounterValues>) -> Vec<String> {
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
