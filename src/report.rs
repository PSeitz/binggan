//!
//! Module for reporting
//!
//! The `report` module contains the [report::Reporter] trait and the [report::PlainReporter] struct.
//! You can set the reporter with the [BenchRunner::set_reporter] method.

use yansi::Paint;

use crate::{
    bench::{Bench, BenchResult},
    write_results::fetch_previous_run_and_write_results_to_disk,
};

/// The trait for reporting the results of a benchmark run.
pub trait Reporter: ReporterClone {
    /// Report the results from a group (can be a single bench)
    fn report_results(&self, results: Vec<BenchResult>);
}

/// The trait to enable cloning on the Box reporter
pub trait ReporterClone {
    /// Clone the box
    fn clone_box(&self) -> Box<dyn Reporter>;
}

pub(crate) fn report_group<'a>(
    benches: &mut [Box<dyn Bench<'a> + 'a>],
    reporter: &dyn Reporter,
    report_memory: bool,
) {
    if benches.is_empty() {
        return;
    }

    let mut results = Vec::new();
    for bench in benches.iter_mut() {
        let mut result = bench.get_results(report_memory);
        fetch_previous_run_and_write_results_to_disk(&mut result);
        results.push(result);
    }
    reporter.report_results(results);
}

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
            self.add_result(&result, &mut table_data);
        }
        self.print_table(&table_data);
    }
}
impl PlainReporter {
    /// Create a new PlainReporter
    pub fn new() -> Self {
        Self {}
    }

    fn add_result(&self, result: &BenchResult, table_data: &mut Vec<Vec<String>>) {
        let stats = &result.stats;
        let perf_counter = &result.perf_counter;

        let mut stats_columns = stats.to_columns(
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
