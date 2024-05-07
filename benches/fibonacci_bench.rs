use binggan::{black_box, InputGroup, PeakMemAlloc, INSTRUMENTED_SYSTEM};

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

fn bench_fibonacci_group<I: 'static>(mut runner: InputGroup<I>) {
    runner.set_alloc(GLOBAL); // Set the peak mem allocator. This will enable peak memory reporting.
    runner.register("factorial", move |_| {
        factorial(black_box(400));
    });
    //runner.register("fibonacci_alt", move |_| {});
    runner.run();
}

fn main() {
    bench_fibonacci_group(InputGroup::new().name("fibonacci_plain"));
    bench_fibonacci_group(
        InputGroup::new_with_inputs(vec![("10", 10), ("15", 15)]).name("fibonacci_input"),
    );
}
