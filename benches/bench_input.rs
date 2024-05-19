use std::collections::HashMap;

use binggan::{black_box, InputGroup, PeakMemAlloc, INSTRUMENTED_SYSTEM};

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

fn bench_group(mut runner: InputGroup<Vec<usize>>) {
    runner.set_alloc(GLOBAL); // Set the peak mem allocator. This will enable peak memory reporting.

    // Enables the perf integration. Only on Linux, noop on other OS.
    runner.config().enable_perf();
    // Trashes the CPU cache between runs
    runner.config().set_cache_trasher(true);
    // Enables throughput reporting
    runner.throughput(|input| input.len() * std::mem::size_of::<usize>());
    runner.register("vec", |data| {
        black_box(test_vec(data));
    });
    runner.register("hashmap", move |data| {
        black_box(test_hashmap(data));
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
