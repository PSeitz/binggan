use crate::{
    bench_group::BenchResult,
    format::{bytes_to_string, format_duration},
};
use std::fmt;
use yansi::Paint;

pub struct BenchStats {
    min_ns: u64,
    max_ns: u64,
    average_ns: u64,
    median_ns: u64,
    avg_memory: usize,
}
pub fn compute_stats(results: &[&BenchResult]) -> Option<BenchStats> {
    if results.is_empty() {
        return None;
    }
    // Avg memory consumption
    let total_memory: usize = results.iter().map(|res| res.memory_consumption).sum();
    let avg_memory = total_memory / results.len();

    let mut sorted_results: Vec<u64> = results.iter().map(|res| res.duration_ns).collect();
    sorted_results.sort();

    // Calculate minimum and maximum
    let min_ns = *sorted_results.first().unwrap();
    let max_ns = *sorted_results.last().unwrap();

    // Calculate average
    let total_duration: u64 = sorted_results.iter().sum();
    let average_ns = (total_duration as f64 / sorted_results.len() as f64) as u64;

    // Calculate median
    let mid = sorted_results.len() / 2;
    let median_ns = if sorted_results.len() % 2 == 0 {
        (sorted_results[mid - 1] + sorted_results[mid]) / 2
    } else {
        sorted_results[mid]
    };

    // Return the struct with all statistics
    Some(BenchStats {
        min_ns,
        max_ns,
        average_ns,
        median_ns,
        avg_memory,
    })
}
impl fmt::Display for BenchStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let min_str = format_duration(self.min_ns).to_string();
        let max_str = format_duration(self.max_ns).to_string();
        let avg_str = format_duration(self.average_ns).to_string();
        let median_str = format_duration(self.median_ns).to_string();

        write!(
            f,
            "Memory: {:12} Average: {:12} Median: {:12} (Min: {:12} Max: {:12})",
            bytes_to_string(self.avg_memory as u64).bright_cyan().bold(),
            min_str,
            max_str,
            avg_str,
            median_str,
        )
    }
}
