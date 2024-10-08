use binggan::{black_box, BenchRunner, PeakMemAlloc, INSTRUMENTED_SYSTEM};

#[global_allocator]
pub static GLOBAL: &PeakMemAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

pub fn factorial(mut n: usize) -> usize {
    let mut result = 1usize;
    while n > 0 {
        result = result.wrapping_mul(black_box(n));
        n -= 1;
    }
    result
}

fn bench_factorial() {
    let mut runner = BenchRunner::new();
    runner.set_alloc(GLOBAL); // Set the peak mem allocator. This will enable peak memory reporting.

    for val in [100, 400] {
        runner.bench_function(format!("factorial {}", val), move |_| {
            factorial(black_box(val));
            Some(())
        });
    }
}

fn main() {
    bench_factorial();
}
