use yansi::Paint;

use super::{avg_median_str, memory_str, min_max_str, BenchStats, Reporter, ReporterClone};
use crate::bench::BenchResult;

#[derive(Clone, Copy)]
/// The PlainReporter prints the results in a plain text table.
/// This is the default reporter.
///
/// e.g.
/// ```text
/// factorial 100    Avg: 33ns     Median: 32ns     [32ns .. 45ns]    
/// factorial 400    Avg: 107ns    Median: 107ns    [107ns .. 109ns]    
/// ```
pub struct PlainReporter {}
impl ReporterClone for PlainReporter {
    fn clone_box(&self) -> Box<dyn Reporter> {
        Box::new(*self)
    }
}

impl Reporter for PlainReporter {
    fn report_results(&self, results: Vec<BenchResult>) {
        let mut table_data: Vec<Vec<String>> = Vec::new();

        for result in results {
            let perf_counter = &result.perf_counter;

            let mut stats_columns = self.to_columns(
                result.stats,
                result.old_stats,
                result.input_size_in_bytes,
                result.output_value,
                result.tracked_memory,
            );
            stats_columns.insert(0, result.bench_id.bench_name.to_string());
            table_data.push(stats_columns);

            if let Some(perf_counter) = perf_counter.as_ref() {
                let mut columns = perf_counter.to_columns(result.old_perf_counter);
                columns.insert(0, "".to_string());
                table_data.push(columns);
            }
        }
        self.print_table(&table_data);
    }
}

impl PlainReporter {
    /// Create a new PlainReporter
    pub fn new() -> Self {
        Self {}
    }

    pub(crate) fn to_columns(
        self,
        stats: BenchStats,
        other: Option<BenchStats>,
        input_size_in_bytes: Option<usize>,
        output_value: Option<u64>,
        report_memory: bool,
    ) -> Vec<String> {
        let (avg_str, median_str) = avg_median_str(&stats, input_size_in_bytes, other);
        let avg_str = format!("Avg: {}", avg_str);
        let median_str = format!("Median: {}", median_str);

        let min_max = min_max_str(&stats, input_size_in_bytes);
        let memory_string = memory_str(&stats, other, report_memory);
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

    fn print_table(&self, table_data: &Vec<Vec<String>>) {
        if table_data.is_empty() {
            return;
        }

        // Find the maximum number of columns in any row
        let num_cols = table_data.iter().map(|row| row.len()).max().unwrap_or(0);

        // Calculate the maximum width of each column
        let mut column_width = vec![0; num_cols];
        for row in table_data {
            for (i, cell) in row.iter().enumerate() {
                let cell = cell.resetting().to_string();
                column_width[i] = column_width[i].max(cell.count_characters() + 4);
            }
        }

        // Print each row with padded cells for alignment
        for row in table_data {
            for (i, cell) in row.iter().enumerate() {
                let padding = column_width[i] - cell.resetting().to_string().count_characters();
                print!("{}{}", cell, " ".repeat(padding),);
            }
            println!(); // Newline at the end of each row
        }
    }
}

impl Default for PlainReporter {
    fn default() -> Self {
        Self::new()
    }
}

fn count_characters(input: &str) -> usize {
    let mut count = 0;
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch == '\x1B' {
            // Skip over the escape character and the '['
            chars.next();
            chars.next();

            // Continue skipping characters until we find a letter (which ends the ANSI code)
            while let Some(&ch) = chars.peek() {
                chars.next();
                if ch.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            count += 1;
            chars.next();
        }
    }

    count
}
trait LenWithoutControl {
    fn count_characters(&self) -> usize;
}
impl LenWithoutControl for str {
    fn count_characters(&self) -> usize {
        count_characters(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // fails
    #[test]
    fn width_test() {
        assert_eq!(
            "Memory: \u{1b}[1;96m786.5 KB\u{1b}[0m".count_characters(),
            "Memory: 786.5 KB".count_characters()
        );
    }

    #[test]
    fn test_print_table() {
        let data = vec![
            vec![
                "TurboBuckets",
                "Memory: \u{1b}[1;96m786.5 KB\u{1b}[0m",
                "Avg: 3.4791ms \u{1b}[31m (+18.96%)\u{1b}[0m",
                "Median: 3.5334ms \u{1b}[31m (+24.46%)\u{1b}[0m",
                "2.0247ms",
                "5.0919ms",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
            vec![
                "",
                "\u{1b}[31mL1dA: 6926924.148  (0.87%)\u{1b}[0m\u{1b}[0m",
                "\u{1b}[32mL1dM: 75340.273  (0.16%)\u{1b}[0m\u{1b}[0m",
                "\u{1b}[34mBr: 2004614.883  (0.00%)\u{1b}[0m\u{1b}[0m",
                "\u{1b}[31mBrM: 13.812 \u{1b}[31m (+113.53%)\u{1b}[0m\u{1b}[0m",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        ];
        let reporter = PlainReporter::new();

        reporter.print_table(&data);
    }
}
