use std::{env, path::PathBuf, sync::Once};

use yansi::Paint;

use crate::{
    bench::{Bench, BenchResult},
    profiler::CounterValues,
    stats::BenchStats,
};

/// Creates directory if it does not exist
pub fn get_output_directory() -> PathBuf {
    static INIT: Once = Once::new();
    static mut OUTPUT_DIRECTORY: Option<PathBuf> = None;
    unsafe {
        INIT.call_once(|| {
            let output_directory = if let Some(value) = env::var_os("BINGGAN_HOME") {
                PathBuf::from(value)
            } else if let Some(path) = env::var_os("CARGO_TARGET_DIR").map(PathBuf::from) {
                path.join("binggan")
            } else {
                PathBuf::from("target/binggan")
            };
            if !output_directory.exists() {
                let _ = std::fs::create_dir_all(&output_directory);
            }
            OUTPUT_DIRECTORY = Some(output_directory);
        });
        OUTPUT_DIRECTORY.clone().unwrap()
    }
}

pub(crate) fn report_group<'a>(
    bench_group_name: &Option<String>,
    benches: &mut [Box<dyn Bench<'a> + 'a>],
    report_memory: bool,
) {
    if benches.is_empty() {
        return;
    }

    let mut table_data: Vec<Vec<String>> = Vec::new();
    for bench in benches.iter_mut() {
        let result = bench.get_results(bench_group_name);
        add_result(&result, report_memory, &mut table_data);
        write_results_to_disk(&result);
    }
    print_table(table_data);
}

fn get_bench_file(result: &BenchResult) -> PathBuf {
    get_output_directory().join(&result.bench_id)
}

fn add_result(result: &BenchResult, report_memory: bool, table_data: &mut Vec<Vec<String>>) {
    let stats = &result.stats;
    let perf_counter = &result.perf_counter;

    // Filepath in target directory
    let filepath = get_bench_file(result);
    // Check if file exists and deserialize
    let mut old_stats: Option<BenchStats> = None;
    let mut old_counter: Option<CounterValues> = None;
    if filepath.exists() {
        let content = std::fs::read_to_string(&filepath).unwrap();
        let lines: Vec<_> = content.lines().collect();
        old_stats = miniserde::json::from_str(lines[0]).unwrap();
        old_counter = lines
            .get(1)
            .and_then(|line| miniserde::json::from_str(line).ok());
    };

    //bench.name
    let mut stats_columns = stats.to_columns(old_stats, result.input_size_in_bytes, report_memory);
    stats_columns.insert(0, result.bench_name.to_string());
    table_data.push(stats_columns);

    if let Some(perf_counter) = perf_counter.as_ref() {
        let mut columns = perf_counter.to_columns(old_counter);
        columns.insert(0, "".to_string());
        table_data.push(columns);
    }
}

pub fn write_results_to_disk(result: &BenchResult) {
    let perf_counter = &result.perf_counter;
    let stats = &result.stats;
    let filepath = get_bench_file(result);
    let mut out = miniserde::json::to_string(&stats);
    if let Some(perf_counter) = perf_counter {
        out.push('\n');
        let perf_out = miniserde::json::to_string(&perf_counter);
        out.push_str(&perf_out);
    }
    std::fs::write(filepath, out).unwrap();
}

fn print_table(table_data: Vec<Vec<String>>) {
    if table_data.is_empty() {
        return;
    }

    // Find the maximum number of columns in any row
    let num_cols = table_data.iter().map(|row| row.len()).max().unwrap_or(0);

    // Calculate the maximum width of each column
    let mut column_width = vec![0; num_cols];
    for row in &table_data {
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

        print_table(data);

        // Assertions would go here. In this case, since we're printing to the console,
        // we don't have a return value to assert on. Typically, you'd want to assert
        // on the function's output or side effects.
        //
        // However, in this case, manual verification of the printed table might be necessary
        // since the primary function of `print_table` is to format and print the table to the console.
    }
}
