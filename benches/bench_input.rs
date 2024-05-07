use binggan::{black_box, InputGroup, PeakMemAlloc, INSTRUMENTED_SYSTEM};

#[global_allocator]
pub static GLOBAL: &PeakMemAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

fn test_vec(data: &Vec<usize>) {
    let mut vec = Vec::new();
    for idx in data {
        if vec.len() <= *idx {
            vec.resize(idx + 1, 0);
        }
        vec[*idx] += 1;
    }
}
fn test_hashmap(data: &Vec<usize>) {
    let mut map = std::collections::HashMap::new();
    for idx in data {
        *map.entry(idx).or_insert(0) += 1;
    }
}

fn bench_group(mut runner: InputGroup<Vec<usize>>) {
    runner.set_alloc(GLOBAL); // Set the peak mem allocator. This will enable peak memory reporting.
    runner.enable_perf();
    runner.throughput(|input| input.len() * std::mem::size_of::<usize>());
    runner.register("vec", |data| {
        test_vec(data);
        black_box(());
    });
    runner.register("hashmap", move |data| {
        test_hashmap(data);
        black_box(());
    });
    runner.run();
}

fn main() {
    let data = vec![
        (
            "max id 100; 100 el all the same",
            std::iter::repeat(100).take(100).collect(),
        ),
        ("max id 100; 100 el all different", (0..100).collect()),
    ];
    bench_group(InputGroup::new_with_inputs(data));
}
