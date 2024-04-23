![binggan logo](https://raw.githubusercontent.com/PSeitz/binggan/main/logo_s.png)

Binggan is a benchmarking library for Rust.
It is designed to be simple to use and to provide a good overview of the performance of your code and its memory consumption.

It allows arbitrary named inputs to be passed to the benchmarks.

```rust
use binggan::{black_box, BenchGroup, Binggan, PeakMemAlloc, INSTRUMENTED_SYSTEM};

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
    let mut runner = Binggan::new();
    bench_fibonacci_group(runner.new_group("fibonacci_plain"));
    bench_fibonacci_group(
        runner.new_group_with_inputs("fibonacci_input", vec![("10", 10), ("15", 15)]),
    );
}
```


### Features:

* Peak Memory Usage
* Stack Offset Randomization
* Perf Integration
* Delta Comparison
* Fast Execution
* Interleaving Test Runs Between Benches in a Group
* Named Benchmark Inputs
* Fast Compile Time (3s for release build)
* No Macros, No Magic (Just a regular API)
* Easy Benchmark Generation
* Runs on Stable Rust

# TODO

- [] Throughput

#### Maybe Later Features:
* Charts
* Auto comparison of Histograms (e.g. if a benchmark has several bands in which it operates, it would be nice to compare them)

### Memory Usage (peak memory usage)

This measures the peak memory usage of the benchmarked code.
While number of allocations are also interesting for performance analysis, 
peak memory will determine the memory requirements of the code.
