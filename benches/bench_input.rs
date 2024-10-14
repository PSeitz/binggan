use std::collections::HashMap;

use binggan::{black_box, plugins::*, InputGroup, PeakMemAlloc, INSTRUMENTED_SYSTEM};

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
fn test_hashmap(data: &Vec<usize>) -> HashMap<usize, i32> {
    let mut map = std::collections::HashMap::new();
    for idx in data {
        *map.entry(*idx).or_insert(0) += 1;
    }
    map
}

fn bench_group(mut runner: InputGroup<Vec<usize>, u64>) {
    runner
        .get_plugin_manager()
        // Trashes the CPU cache between runs
        .add_plugin(CacheTrasher::default())
        // Set the peak mem allocator. This will enable peak memory reporting.
        .add_plugin(PeakMemAllocPlugin::new(GLOBAL))
        // Enables the perf integration. Only on Linux, noop on other OS.
        .add_plugin(PerfCounterPlugin::default());
    // Enables throughput reporting
    runner.throughput(|input| input.len() * std::mem::size_of::<usize>());
    runner.register("vec", |data| {
        let vec = black_box(test_vec(data));
        Some(vec.len() as u64) // The return value of the function will be reported as the `OutputValue` if it is `Some`.
    });
    runner.register("hashmap", move |data| {
        let map = black_box(test_hashmap(data));
        // The return value of the function will be reported as the `OutputValue` if it is `Some`.
        Some(map.len() as u64 * (std::mem::size_of::<usize>() + std::mem::size_of::<i32>()) as u64)
    });
    runner.run();
}

fn main() {
    // Tuples of name and data for the inputs
    let data = vec![
        (
            "max id 100; 100 ids all the same",
            std::iter::repeat(100).take(100).collect(),
        ),
        ("max id 100; 100 ids all different", (0..100).collect()),
    ];
    bench_group(InputGroup::new_with_inputs(data));
}
