use std::collections::HashMap;

use binggan::{black_box, plugins::CacheTrasher, BenchRunner, PeakMemAlloc, INSTRUMENTED_SYSTEM};

#[global_allocator]
pub static GLOBAL: &PeakMemAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

fn test_vec(data: &Vec<usize>) -> Vec<i32> {
    let mut vec = Vec::new();
    for idx in data {
        if vec.len() <= *idx {
            vec.resize(idx + 1, 0);
        }
        vec[*idx] += 1;
    }
    vec
}
fn test_hashmap(data: &Vec<usize>) -> HashMap<&usize, i32> {
    let mut map = std::collections::HashMap::new();
    for idx in data {
        *map.entry(idx).or_insert(0) += 1;
    }
    map
}

fn run_bench() {
    let inputs: Vec<(&str, Vec<usize>)> = vec![
        (
            "max id 100; 100 el all the same",
            std::iter::repeat(100).take(100).collect(),
        ),
        ("max id 100; 100 el all different", (0..100).collect()),
    ];
    let mut runner: BenchRunner = BenchRunner::new();
    runner.set_alloc(GLOBAL); // Set the peak mem allocator. This will enable peak memory reporting.

    runner.config().enable_perf();
    runner
        .get_plugin_manager()
        .add_plugin(CacheTrasher::default());

    for (input_name, data) in inputs.iter() {
        let mut group = runner.new_group();
        group.set_name(input_name);
        group.set_input_size(data.len() * std::mem::size_of::<usize>());
        group.register_with_input("vec", data, move |data| {
            let vec = black_box(test_vec(data));
            Some(vec.len() as u64)
        });
        group.register_with_input("hashmap", data, move |data| {
            let map = black_box(test_hashmap(data));
            Some(
                map.len() as u64
                    * (std::mem::size_of::<usize>() + std::mem::size_of::<i32>()) as u64,
            )
        });
        group.run();
    }
}

fn main() {
    run_bench();
}
