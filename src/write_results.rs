use std::{env, path::PathBuf, sync::OnceLock};

use crate::{bench::BenchResult, bench_id::BenchId, plugins::PerfCounterValues, stats::BenchStats};

/// Creates directory if it does not exist
pub fn get_output_directory() -> &'static PathBuf {
    static OUTPUT_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();
    OUTPUT_DIRECTORY.get_or_init(|| {
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
        output_directory
    })
}

fn get_bench_file(bench_id: &BenchId) -> PathBuf {
    get_output_directory().join(bench_id.get_full_name())
}

pub(crate) struct PreviousRun {
    pub stats: BenchStats,
    pub perf_counter: Option<PerfCounterValues>,
    pub serialized_output_value: Option<String>,
}

pub(crate) fn fetch_previous_run(bench_id: &BenchId) -> Option<PreviousRun> {
    // Filepath in target directory
    let filepath = get_bench_file(bench_id);
    // Check if file exists and deserialize
    if filepath.exists() {
        let content = std::fs::read_to_string(&filepath).unwrap();
        let lines: Vec<_> = content.lines().collect();
        let stats = miniserde::json::from_str(lines[0]).unwrap();
        let perf_counter = lines
            .get(1)
            .and_then(|line| miniserde::json::from_str(line).ok());
        let serialized_output_value = lines.get(2).map(|line| (*line).to_string());
        return Some(PreviousRun {
            stats,
            perf_counter,
            serialized_output_value,
        });
    }
    None
}

pub(crate) fn write_results_to_disk(result: &BenchResult) {
    let mut out = miniserde::json::to_string(&result.stats);
    if let Some(perf_counter) = &result.perf_counter {
        out.push('\n');
        let perf_out = miniserde::json::to_string(perf_counter);
        out.push_str(&perf_out);
    }
    if let Some(output_value) = &result.serialized_output_value {
        if result.perf_counter.is_none() {
            // Keep the second line reserved for perf counters so old readers still either read
            // perf counters there or ignore the empty line.
            out.push('\n');
        }
        out.push('\n');
        out.push_str(output_value);
    }
    std::fs::write(get_bench_file(&result.bench_id), out).unwrap();
}
