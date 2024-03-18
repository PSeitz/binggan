use crate::{
    bench::BenchResult,
    format::{bytes_to_string, format_duration},
};
//use csv_macro::CSV;
use miniserde::{Deserialize, Serialize};
use yansi::Paint;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BenchStats {
    min_ns: u64,
    max_ns: u64,
    average_ns: u64,
    median_ns: u64,
    avg_memory: usize,
}
impl BenchStats {
    pub fn to_columns(&self, other: Option<BenchStats>, include_memory: bool) -> Vec<String> {
        let avg_ns_diff = other
            .as_ref()
            .map(|other| {
                format_percentage(compute_percentage_diff(
                    self.average_ns as f64,
                    other.average_ns as f64,
                ))
            })
            .unwrap_or_default();
        let median_ns_diff = other
            .as_ref()
            .map(|other| {
                format_percentage(compute_percentage_diff(
                    self.median_ns as f64,
                    other.median_ns as f64,
                ))
            })
            .unwrap_or_default();

        let min_str = format_duration(self.min_ns);
        let max_str = format_duration(self.max_ns);
        let memory_string = if include_memory {
            let mem_diff = other
                .clone()
                .map(|other| {
                    format_percentage(compute_percentage_diff(
                        self.avg_memory as f64,
                        other.avg_memory as f64,
                    ))
                })
                .unwrap_or_default();
            format!(
                "Memory: {} {}",
                bytes_to_string(self.avg_memory as u64).bright_cyan().bold(),
                mem_diff,
            )
        } else {
            "".to_string()
        };

        vec![
            memory_string,
            format!("Avg: {} {}", format_duration(self.average_ns), avg_ns_diff,),
            format!(
                "Median: {} {}",
                format_duration(self.median_ns),
                median_ns_diff,
            ),
            min_str,
            max_str,
        ]
    }
}
pub fn compute_percentage_diff(a: f64, b: f64) -> f64 {
    (a / b - 1.0) * 100.0
}
pub fn format_percentage(diff: f64) -> String {
    if diff > 2.0 {
        format!(" (+{:.2}%)", diff).red().to_string()
    } else if diff < -2.0 {
        format!(" ({:.2}%)", diff).green().to_string()
    } else {
        format!(" ({:.2}%)", diff).resetting().to_string()
    }
}
pub fn compute_stats(results: &[BenchResult]) -> Option<BenchStats> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_res(duration_ns: u64, memory_consumption: usize) -> BenchResult {
        BenchResult {
            duration_ns,
            memory_consumption,
        }
    }

    #[test]
    fn test_compute_stats_median_odd() {
        let results = vec![create_res(10, 0), create_res(20, 0), create_res(30, 0)];
        let stats = compute_stats(&results).unwrap();
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
        let stats = compute_stats(&results).unwrap();
        assert_eq!(
            stats.median_ns, 25,
            "Median should be the average of the two middle elements for even count"
        );
    }
}
