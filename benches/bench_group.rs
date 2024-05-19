use std::collections::HashMap;

use binggan::{black_box, BenchRunner, PeakMemAlloc, INSTRUMENTED_SYSTEM};

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
    runner.config().set_cache_trasher(true);

    let mut group = runner.new_group();
    for (input_name, data) in inputs.iter() {
        group.set_input_size(data.len() * std::mem::size_of::<usize>());
        group.register_with_input("vec", input_name, data, move |data| {
            black_box(test_vec(data));
        });
        group.register_with_input("hashmap", input_name, data, move |data| {
            black_box(test_hashmap(data));
        });
    }
    group.run();
}

fn main() {
    run_bench();
}
