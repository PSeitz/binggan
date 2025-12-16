use std::{collections::HashMap, time::Duration};

use binggan::{
    plugins::{CacheTrasher, PeakMemAllocPlugin, PerfCounterPlugin},
    BenchRunner, PeakMemAlloc, INSTRUMENTED_SYSTEM,
};
use quanta::Instant;

#[global_allocator]
pub static GLOBAL: &PeakMemAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

fn run_bench_throughput() {
    let mut runner: BenchRunner = BenchRunner::new();

    runner
        .add_plugin(CacheTrasher::default())
        // Enable the peak mem allocator. This will enable peak memory reporting.
        .add_plugin(PeakMemAllocPlugin::new(GLOBAL))
        .add_plugin(PerfCounterPlugin::default());

    let mut group = runner.new_group();
    group.set_input_size(10_000);
    group.register_with_input("1 MB/s", &(), move |_data| {
        let start = Instant::now();
        // Busy loop for approximately 10 milliseconds. This is more precise than sleep.
        while start.elapsed() < Duration::from_millis(10) {}
    });
    group.run();
}

fn run_bench_lifetime() {
    let inputs: Vec<(&str, Vec<usize>)> = vec![
        (
            "max id 100; 100 el all the same",
            std::iter::repeat(100).take(100).collect(),
        ),
        ("max id 100; 100 el all different", (0..100).collect()),
    ];
    let mut runner: BenchRunner = BenchRunner::new();

    for (_input_name, data) in inputs.iter() {
        let infos: HashMap<String, u64> = HashMap::new();
        let mut group = runner.new_group();
        group.register_with_input("vec", data, |_data| {
            let entry = infos.get("test").cloned();
            entry
        });
        group.run();
    }
}

fn main() {
    run_bench_throughput();
    run_bench_lifetime();
}
