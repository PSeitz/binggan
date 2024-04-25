use binggan::{black_box, BenchGroup, PeakMemAlloc, INSTRUMENTED_SYSTEM};

#[global_allocator]
pub static GLOBAL: &PeakMemAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn bench_fibonacci_group<I>(mut runner: BenchGroup<I>) {
    runner.set_alloc(GLOBAL); // Set the peak mem allocator. This will enable peak memory reporting.
    runner.enable_perf();
    runner.register("fibonacci", move |_| {
        fibonacci(black_box(10));
    });
    runner.register("fibonacci_alt", move |_| {});
    runner.run();
}

fn main() {
    bench_fibonacci_group(BenchGroup::new().name("fibonacci_plain"));
    bench_fibonacci_group(
        BenchGroup::new_with_inputs(vec![("10", 10), ("15", 15)]).name("fibonacci_input"),
    );
}
