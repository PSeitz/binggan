use std::time::{Duration, Instant};

use binggan::{
    plugins::{CacheTrasher, PerfCounterPlugin},
    BenchRunner, PeakMemAlloc, INSTRUMENTED_SYSTEM,
};

#[global_allocator]
pub static GLOBAL: &PeakMemAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

fn run_bench() {
    let mut runner: BenchRunner = BenchRunner::new();
    runner.set_alloc(GLOBAL); // Set the peak mem allocator. This will enable peak memory reporting.

    runner
        .get_plugin_manager()
        .add_plugin(CacheTrasher::default())
        .add_plugin(PerfCounterPlugin::default());
    runner.config().set_num_iter_for_group(128);

    let mut group = runner.new_group();
    group.set_input_size(10_000);
    group.register_with_input("1 MB/s", &(), move |_data| {
        let start = Instant::now();
        // Busy loop for approximately 10 milliseconds. This is more precise than sleep.
        while start.elapsed() < Duration::from_millis(10) {}
        Some(())
    });
    group.run();
}

fn main() {
    run_bench();
}
