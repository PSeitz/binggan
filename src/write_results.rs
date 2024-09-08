use std::{env, path::PathBuf, sync::Once};

use crate::bench::BenchResult;

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

fn get_bench_file(result: &BenchResult) -> PathBuf {
    get_output_directory().join(result.bench_id.get_full_name())
}

pub fn fetch_previous_run_and_write_results_to_disk(result: &mut BenchResult) {
    // Filepath in target directory
    let filepath = get_bench_file(result);
    // Check if file exists and deserialize
    if filepath.exists() {
        let content = std::fs::read_to_string(&filepath).unwrap();
        let lines: Vec<_> = content.lines().collect();
        result.old_stats = miniserde::json::from_str(lines[0]).unwrap();
        result.old_perf_counter = lines
            .get(1)
            .and_then(|line| miniserde::json::from_str(line).ok());
    }

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
