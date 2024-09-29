use std::any::Any;

use crate::stats::*;
use miniserde::*;

use crate::bench_id::BenchId;
use crate::plugins::{BingganEvents, EventListener, PerBenchData};

#[cfg(not(target_os = "linux"))]
pub(crate) mod dummy_profiler;
#[cfg(target_os = "linux")]
pub(crate) mod perf_profiler;

#[cfg(not(target_os = "linux"))]
pub(crate) use dummy_profiler::*;
#[cfg(target_os = "linux")]
pub(crate) use perf_profiler::*;

use yansi::Paint;

pub trait Profiler {
    fn enable(&mut self);
    fn disable(&mut self);
    fn finish(&mut self, num_iter: u64) -> std::io::Result<CounterValues>;
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
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

pub static PERF_CNT_EVENT_LISTENER_NAME: &str = "_binggan_perf";

/// Integration via EventListener
/// One counter per bench id.
#[derive(Default)]
pub struct PerfCounterPerBench {
    perf_per_bench: PerBenchData<PerfCounters>,
}

impl PerfCounterPerBench {
    pub fn get_by_bench_id_mut(&mut self, bench_id: &BenchId) -> Option<&mut PerfCounters> {
        self.perf_per_bench.get_mut(bench_id)
    }
}

impl EventListener for PerfCounterPerBench {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
    fn name(&self) -> &'static str {
        PERF_CNT_EVENT_LISTENER_NAME
    }
    fn on_event(&mut self, event: BingganEvents) {
        match event {
            BingganEvents::BenchStart(bench_id) => {
                self.perf_per_bench
                    .insert_if_absent(bench_id, || PerfCounters::new().unwrap());
                let perf = self.perf_per_bench.get_mut(bench_id).unwrap();
                perf.enable();
            }
            BingganEvents::BenchStop(bench_id, _) => {
                let perf = self.perf_per_bench.get_mut(bench_id).unwrap();
                perf.disable();
            }
            _ => {}
        }
    }
}
