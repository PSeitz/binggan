use crate::{
    bench::RunResult,
    format::{bytes_to_string, format_duration},
};
use miniserde::{Deserialize, Serialize};
use yansi::Paint;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BenchStats {
    min_ns: u64,
    max_ns: u64,
    average_ns: u64,
    median_ns: u64,
    avg_memory: usize,
}

/// Compute diff from two values of BenchStats
fn compute_diff<F: Fn(&BenchStats) -> u64>(
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

impl BenchStats {
    pub fn to_columns(
        self,
        other: Option<BenchStats>,
        input_size_in_bytes: Option<usize>,
        output_value: Option<u64>,
        report_memory: bool,
    ) -> Vec<String> {
        let avg_ns_diff = compute_diff(&self, input_size_in_bytes, other, |stats| stats.average_ns);
        let median_ns_diff =
            compute_diff(&self, input_size_in_bytes, other, |stats| stats.median_ns);

        // if input_size_in_bytes is set report the throughput, otherwise just use format_duration
        let format = |duration_ns: u64| {
            if let Some(input_size_in_bytes) = input_size_in_bytes {
                let mut duration_ns: f64 = duration_ns as f64;
                let unit = unit_per_second(input_size_in_bytes, &mut duration_ns);
                format!("{:>6} {}", short(duration_ns), unit)
            } else {
                format_duration(duration_ns).to_string()
            }
        };

        let avg_str = format!("Avg: {} {}", format(self.average_ns), avg_ns_diff,);
        let median_str = format!("Median: {} {}", format(self.median_ns), median_ns_diff,);

        let min_max = if input_size_in_bytes.is_some() {
            format!("[{} .. {}]", format(self.max_ns), format(self.min_ns))
        } else {
            format!("[{} .. {}]", format(self.min_ns), format(self.max_ns))
        };
        let memory_string = if report_memory {
            let mem_diff = compute_diff(&self, None, other, |stats| stats.avg_memory as u64);
            format!(
                "Memory: {} {}",
                bytes_to_string(self.avg_memory as u64).bright_cyan().bold(),
                mem_diff,
            )
        } else {
            "".to_string()
        };
        if let Some(output_value) = output_value {
            vec![
                memory_string,
                avg_str,
                median_str,
                min_max,
                format!("OutputValue: {}", output_value.to_string()),
            ]
        } else {
            vec![memory_string, avg_str, median_str, min_max]
        }
    }
}

//fn format_throughput(bytes: usize, mut nanoseconds: f64) -> String {
//let unit = bytes_per_second(bytes, &mut nanoseconds);
//format!("{:>6} {}", short(nanoseconds), unit)
//}

/// Returns the unit and alters the passed parameter to match the unit
fn unit_per_second(bytes: usize, nanoseconds: &mut f64) -> &'static str {
    let bytes_per_second = bytes as f64 * (1e9 / *nanoseconds);
    let (denominator, unit) = if bytes_per_second < 1024.0 {
        (1.0, "  B/s")
    } else if bytes_per_second < 1024.0 * 1024.0 {
        (1024.0, "KiB/s")
    } else if bytes_per_second < 1024.0 * 1024.0 * 1024.0 {
        (1024.0 * 1024.0, "MiB/s")
    } else {
        (1024.0 * 1024.0 * 1024.0, "GiB/s")
    };

    let bytes_per_second = bytes as f64 * (1e9 / *nanoseconds);
    *nanoseconds = bytes_per_second / denominator;

    unit
}

pub fn short(n: f64) -> String {
    if n < 10.0 {
        format!("{:.4}", n)
    } else if n < 100.0 {
        format!("{:.3}", n)
    } else if n < 1000.0 {
        format!("{:.2}", n)
    } else if n < 10000.0 {
        format!("{:.1}", n)
    } else {
        format!("{:.0}", n)
    }
}

pub fn compute_percentage_diff(a: f64, b: f64) -> f64 {
    (a / b - 1.0) * 100.0
}
pub fn format_percentage(diff: f64, smaller_is_better: bool) -> String {
    let diff_str = if diff >= 0.0 {
        format!("(+{:.2}%)", diff)
    } else {
        format!("({:.2}%)", diff)
    };
    if smaller_is_better {
        if diff > 2.0 {
            diff_str.red().to_string()
        } else if diff < -2.0 {
            diff_str.green().to_string()
        } else {
            diff_str.resetting().to_string()
        }
    } else if diff > 2.0 {
        diff_str.green().to_string()
    } else if diff < -2.0 {
        diff_str.red().to_string()
    } else {
        diff_str.resetting().to_string()
    }
}
pub fn compute_stats(results: &[RunResult], _num_iter: usize) -> BenchStats {
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

    fn create_res(duration_ns: u64, memory_consumption: usize) -> RunResult {
        RunResult {
            output: None,
            duration_ns,
            memory_consumption,
        }
    }

    #[test]
    fn test_compute_stats_median_odd() {
        let results = vec![create_res(10, 0), create_res(20, 0), create_res(30, 0)];
        let stats = compute_stats(&results, 32);
        assert_eq!(
            stats.median_ns, 20,
            "Median should be the middle element for odd count"
        );
    }

    #[test]
    fn test_compute_stats_median_even() {
        let results = vec![
            create_res(10, 0),
            create_res(20, 0),
            create_res(30, 0),
            create_res(40, 0),
        ];
        let stats = compute_stats(&results, 32);
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
