![Rust](https://github.com/PSeitz/binggan/workflows/Rust/badge.svg)
[![Docs](https://docs.rs/binggan/badge.svg)](https://docs.rs/crate/binggan/)
[![Crates.io](https://img.shields.io/crates/v/binggan.svg)](https://crates.io/crates/binggan)

## Binggan
![binggan logo](https://raw.githubusercontent.com/PSeitz/binggan/main/logo_s.png)

Binggan (餅乾, bǐng gān, means cookie in Chinese) is a benchmarking library for Rust.
It is designed to be simple to use and to provide a good overview of the performance of your code and its memory consumption.

### Features

* 📊 Peak Memory Usage
* 💎 Stack Offset Randomization
* 🔌 Plugin System
* 💖 Perf Integration (Linux)
* 🔄 Delta Comparison
* ⚡ BLAZINGLY Fast Execution
* 🔀 Interleaving Test Runs (More accurate results)
* 🏷️ Named Runs, Groups and Benchmarks
* 🧙 No Macros, No Magic (Just a regular API)
* 🦀 Runs on Stable Rust
* 📈 Custom Reporter
* 🧩 Report Output of Benchmarks
* 🎨 NOW with colored output!
* 🔍 Advanced Filtering (AND/OR/NOT and fields like `bench_name:my_bench`)

### Example

```rust
use binggan::{black_box, plugins::*, InputGroup, PeakMemAlloc, INSTRUMENTED_SYSTEM};

#[global_allocator]
pub static GLOBAL: &PeakMemAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;


fn test_vec(data: &Vec<usize>) {
    // ...
}
fn test_hashmap(data: &Vec<usize>) {
    // ...
}

fn bench_group(mut runner: InputGroup<Vec<usize>>) {
    runner
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
        // The return value of the function will be reported as the `OutputValue` 
        vec.len() as u64
    });
    runner.register("hashmap", move |data| {
        let map = black_box(test_hashmap(data));
        // The return value of the function will be reported as the `OutputValue` 
        map.len() as u64 * (std::mem::size_of::<usize>() + std::mem::size_of::<i32>()) as u64
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

```

### Example Output
```bash
cargo bench

turbo_buckets_vs_fxhashmap_full_unique
100k max id / 100k num elem
TurboBuckets         Memory: 786.4 KB     Avg: 1.6356 GB/s (+0.18%)    Median: 1.6397 GB/s (+0.83%)    [1.5530 GB/s .. 1.6740 GB/s]    Output: 100_000    
FlushVec             Memory: 200.0 KB     Avg: 8.7891 GB/s (-1.16%)    Median: 8.8631 GB/s (-0.40%)    [8.0207 GB/s .. 8.9986 GB/s]    Output: 100_000    
FlushVec With Val    Memory: 3.2 MB       Avg: 3.7802 GB/s (-0.31%)    Median: 3.7875 GB/s (-0.00%)    [3.5477 GB/s .. 3.9165 GB/s]    Output: 100_000    
TurboFlexBuckets     Memory: 786.5 KB     Avg: 1.2632 GB/s (+0.28%)    Median: 1.2653 GB/s (+0.40%)    [1.2282 GB/s .. 1.2810 GB/s]    Output: 100_000    
Vec with Val         Memory: 3.2 MB       Avg: 1.2488 GB/s (-1.16%)    Median: 1.2526 GB/s (-0.97%)    [1.1634 GB/s .. 1.3042 GB/s]    Output: 100_001    
500k max id / 500k num elem
TurboBuckets         Memory: 2.4 MB        Avg: 4.1036 GB/s (+0.48%)    Median: 4.0994 GB/s (-0.02%)    [3.9879 GB/s .. 4.2272 GB/s]    Output: 500_000    
FlushVec             Memory: 1000.0 KB     Avg: 8.8669 GB/s (+1.50%)    Median: 8.8787 GB/s (+0.67%)    [8.6667 GB/s .. 8.9674 GB/s]    Output: 500_000    
FlushVec With Val    Memory: 16.0 MB       Avg: 1.8976 GB/s (-1.03%)    Median: 1.9574 GB/s (+1.46%)    [1.1587 GB/s .. 2.0764 GB/s]    Output: 500_000    
TurboFlexBuckets     Memory: 2.4 MB        Avg: 2.1348 GB/s (+0.72%)    Median: 2.1412 GB/s (+0.55%)    [2.0800 GB/s .. 2.1732 GB/s]    Output: 500_000    
Vec with Val         Memory: 16.0 MB       Avg: 4.3664 GB/s (-2.64%)    Median: 4.5571 GB/s (+0.19%)    [2.1844 GB/s .. 4.8527 GB/s]    Output: 500_001    
```

### Peak Memory
To activate peak memory reporting, you need to wrap your allocator with the PeakMemAlloc and enable the PeakMemAllocPlugin (see example above).

While number of allocations are also interesting for performance analysis, peak memory will determine the memory requirements of the code.

### Filtering

Binggan has a filtering system built in, powered by `tantivy-query-grammar`. You can run a subset of benchmarks by providing a query string to the CLI:

```bash
cargo bench -- "bench_name:my_bench AND group_name:my_group"
cargo bench -- "my_bench OR other_bench"
cargo bench -- "NOT other_bench"
cargo bench -- "r:my_runner b:my_bench -g:my_group"
```

You can also use the `BINGGAN_FILTER` environment variable to set the filter:

```bash
BINGGAN_FILTER="my_bench OR other_bench" cargo bench
```

Available fields are `runner_name` (or `r`), `group_name` (or `g`), and `bench_name` (or `b`). If no field is specified, it will match against the full generated `BenchId`.

### Perf Integration
Perf may run into limitations where all counters are reported as zero. https://github.com/jimblandy/perf-event/issues/2
Disabling the NMI watchdog should help:

`sudo sh -c "echo '0' > /proc/sys/kernel/nmi_watchdog"`

### TODO

- [ ] Improve the reporter api. Currently the reporter gets preaggregated data.

#### Maybe Later Features:
* Charts
* Auto comparison of Histograms (e.g. if a benchmark has several bands in which it operates, it would be nice to compare them)

