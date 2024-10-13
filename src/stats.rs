use crate::bench::RunResult;
use miniserde::{Deserialize, Serialize};
use yansi::Paint;

/// `BenchStats` holds statistical data for benchmarking performance,
/// including timing and memory usage.
///
/// The data is already aggregated.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BenchStats {
    /// The minimum time taken for an operation, in nanoseconds.
    pub min_ns: u64,

    /// The maximum time taken for an operation, in nanoseconds.
    pub max_ns: u64,

    /// The average time taken for an operation, in nanoseconds.
    pub average_ns: u64,

    /// The median time taken for an operation, in nanoseconds.
    pub median_ns: u64,

    /// The average memory used during the operation, in bytes.
    pub avg_memory: usize,
}

/// Compute diff from two values of BenchStats
pub fn compute_diff<F: Fn(&BenchStats) -> u64>(
    stats: &BenchStats,
    input_size_in_bytes: Option<usize>,
    other: Option<BenchStats>,
    f: F,
) -> String {
    other
        .as_ref()
        .map(|other| {
            if f(other) == 0 || f(stats) == 0 || f(other) == f(stats) {
                return "".to_string();
            }
            // Diff on throughput
            if let Some(input_size_in_bytes) = input_size_in_bytes {
                let val = bytes_per_second(input_size_in_bytes, f(stats) as f64);
                let val_other = bytes_per_second(input_size_in_bytes, f(other) as f64);
                let diff = compute_percentage_diff(val, val_other);
                format_percentage(diff, false)
            } else {
                let diff = compute_percentage_diff(f(stats) as f64, f(other) as f64);
                format_percentage(diff, true)
            }
        })
        .unwrap_or_default()
}

fn bytes_per_second(input_size_in_bytes: usize, ns: f64) -> f64 {
    (input_size_in_bytes as f64) / (ns / 1e9)
}

//fn format_throughput(bytes: usize, mut nanoseconds: f64) -> String {
//let unit = bytes_per_second(bytes, &mut nanoseconds);
//format!("{:>6} {}", short(nanoseconds), unit)
//}

pub fn compute_percentage_diff(a: f64, b: f64) -> f64 {
    (a / b - 1.0) * 100.0
}
pub fn format_percentage(diff: f64, smaller_is_better: bool) -> String {
    const COLOR_THRESHOLD: f64 = 2.0;
    let diff_str = if diff >= 0.0 {
        format!("(+{:.2}%)", diff)
    } else {
        format!("({:.2}%)", diff)
    };
    if diff > COLOR_THRESHOLD {
        if smaller_is_better {
            diff_str.red().to_string()
        } else {
            diff_str.green().to_string()
        }
    } else if diff < -COLOR_THRESHOLD {
        if smaller_is_better {
            diff_str.green().to_string()
        } else {
            diff_str.red().to_string()
        }
    } else {
        diff_str.resetting().to_string()
    }
}
pub fn compute_stats<O>(
    results: &[RunResult<O>],
    memory_consumption: Option<&Vec<usize>>,
) -> BenchStats {
    // Avg memory consumption
    let avg_memory = memory_consumption
        .map(|memory_consumption| {
            let total_memory: usize = memory_consumption.iter().copied().sum();
            total_memory / memory_consumption.len()
        })
        .unwrap_or(0);

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
    BenchStats {
        min_ns,
        max_ns,
        average_ns,
        median_ns,
        avg_memory,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_res(duration_ns: u64) -> RunResult<u64> {
        RunResult {
            output: None,
            duration_ns,
        }
    }

    #[test]
    fn test_compute_stats_median_odd() {
        let results = vec![create_res(10), create_res(20), create_res(30)];
        let stats = compute_stats(&results, None);
        assert_eq!(
            stats.median_ns, 20,
            "Median should be the middle element for odd count"
        );
    }

    #[test]
    fn test_compute_stats_median_even() {
        let results = vec![
            create_res(10),
            create_res(20),
            create_res(30),
            create_res(40),
        ];
        let stats = compute_stats(&results, None);
        assert_eq!(
            stats.median_ns, 25,
            "Median should be the average of the two middle elements for even count"
        );
    }

    #[test]
    fn test_compute_diff_average_ns_with_input_size() {
        let stats = BenchStats {
            min_ns: 0,
            max_ns: 0,
            average_ns: 150,
            median_ns: 0,
            avg_memory: 24,
        };

        let other_stats = BenchStats {
            min_ns: 0,
            max_ns: 0,
            average_ns: 100, // different average_ns to see the difference in the output
            median_ns: 0,
            avg_memory: 0,
        };

        // Example usage: Using average_ns field for comparison.
        let diff = compute_diff(&stats, Some(1000), Some(other_stats), |x| x.average_ns);

        // Check the output
        assert_eq!(diff, "(-33.33%)".red().to_string());
    }
}
