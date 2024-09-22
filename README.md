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
* 💖 Perf Integration (Linux)
* 🔄 Delta Comparison
* ⚡ Fast Execution
* 🔀 Interleaving Test Runs (More accurate results)
* 🏷️ Named Runs, Groups and Benchmarks
* 🧙 No Macros, No Magic (Just a regular API)
* 🎨 NOW with colored output!
* 🦀 Runs on Stable Rust
* 📈 Custom reporter
* 🧩 Custom OutputValues

### Example

```rust
use binggan::{black_box, InputGroup, PeakMemAlloc, INSTRUMENTED_SYSTEM};

#[global_allocator]
pub static GLOBAL: &PeakMemAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;


fn test_vec(data: &Vec<usize>) {
    // ...
}
fn test_hashmap(data: &Vec<usize>) {
    // ...
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
        Some(()) // anything that implements OutputValue
    })
    runner.register("hashmap", move |data| {
        black_box(test_hashmap(data));
        Some(())
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
TurboBuckets           Memory: 786.4 KB     Avg: 2.1107 GiB/s (+0.19%)    Median: 2.1288 GiB/s (+0.69%)    [1.9055 GiB/s .. 2.1464 GiB/s]    
FxHashMap              Memory: 1.8 MB       Avg: 1.1116 GiB/s (-0.65%)    Median: 1.1179 GiB/s (-0.90%)    [1020.2 MiB/s .. 1.1363 GiB/s]    
500k max id / 500k num elem
TurboBuckets           Memory: 2.4 MB       Avg: 5.7073 GiB/s (-0.29%)    Median: 5.7633 GiB/s (-0.55%)    [5.1313 GiB/s .. 6.1104 GiB/s]    
FxHashMap              Memory: 14.2 MB      Avg: 521.50 MiB/s (-1.81%)    Median: 523.42 MiB/s (-1.75%)    [465.28 MiB/s .. 562.83 MiB/s]    
1m max id / 1m num elem
TurboBuckets           Memory: 4.5 MB       Avg: 6.2922 GiB/s (+5.48%)    Median: 6.3850 GiB/s (+6.56%)    [4.9580 GiB/s .. 6.7989 GiB/s]    
FxHashMap              Memory: 28.3 MB      Avg: 403.52 MiB/s (+0.00%)    Median: 396.74 MiB/s (+0.97%)    [355.83 MiB/s .. 473.37 MiB/s]    
```

### Peak Memory
To activate peak memory reporting, you need to wrap your allocator with the PeakMemAlloc and call `set_alloc` in the group.

While number of allocations are also interesting for performance analysis, peak memory will determine the memory requirements of the code.

### Perf Integration
Perf may run into limitations where all counters are reported as zero. https://github.com/jimblandy/perf-event/issues/2
Disabling the NMI watchdog should help:

`sudo sh -c "echo '0' > /proc/sys/kernel/nmi_watchdog"`

### TODO

- [ ] Allow Hooks (perf integration as plugin)
- [ ] Improve the reporter api. Currently the reporter gets preaggregated data.

#### Maybe Later Features:
* Charts
* Auto comparison of Histograms (e.g. if a benchmark has several bands in which it operates, it would be nice to compare them)

