![Rust](https://github.com/PSeitz/binggan/workflows/Rust/badge.svg)
[![Docs](https://docs.rs/binggan/badge.svg)](https://docs.rs/crate/binggan/)
[![Crates.io](https://img.shields.io/crates/v/binggan.svg)](https://crates.io/crates/binggan)

## Binggan
![binggan logo](https://raw.githubusercontent.com/PSeitz/binggan/main/logo_s.png)

Binggan (餅乾, bǐng gān, means cookie in Chinese) is a benchmarking library for Rust.
It is designed to be simple to use and to provide a good overview of the performance of your code and its memory consumption.

It allows arbitrary named inputs to be passed to the benchmarks.

### Features

* 📊 Peak Memory Usage
* 💎 Stack Offset Randomization
* 💖 Perf Integration (Linux)
* 🔄 Delta Comparison
* ⚡ Fast Execution
* 🧩 Interleaving Test Runs (More accurate results)
* 🏷️ Named Benchmark Inputs
* 🧙 No Macros, No Magic (Just a regular API)
* 🎨 NOW with colored output!
* 🦀 Runs on Stable Rust

### Example

```rust
use binggan::{black_box, InputGroup, PeakMemAlloc, INSTRUMENTED_SYSTEM};

#[global_allocator]
pub static GLOBAL: &PeakMemAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

fn bench_group(mut runner: InputGroup<Vec<usize>>) {
    // Set the peak mem allocator. This will enable peak memory reporting.
    runner.set_alloc(GLOBAL); 

    // Enable perf integration. This will enable CPU performance counters and report them.
    runner.enable_perf();
    runner.register("vec", move |data| {
        black_box(test_vec(data));
    });
    runner.register("hashmap", move |data| {
        black_box(test_hashmap(data));
    });
    runner.run();
}

fn test_vec(data: &Vec<usize>) {
    // ...
}
fn test_hashmap(data: &Vec<usize>) {
    // ...
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
```

### Example Output
```bash
cargo bench

turbo_buckets_vs_fxhashmap_zipfs1%
100k max id / 100k num elem
TurboBuckets                 Memory: 786.4 KB      Avg: 0.3411ms  (-8.90%)     Median: 0.3394ms  (-9.51%)     0.3223ms    0.3741ms    
Vec                          Memory: 400.0 KB      Avg: 0.0503ms  (-10.27%)    Median: 0.0492ms  (-12.27%)    0.0463ms    0.0676ms    
FxHashMap                    Memory: 442.4 KB      Avg: 1.0560ms  (+26.89%)    Median: 1.1512ms  (+58.61%)    0.6558ms    1.1979ms    
FxHashMap Reserved Max Id    Memory: 1.2 MB        Avg: 0.5220ms  (-7.86%)     Median: 0.4988ms  (-11.40%)    0.4762ms    0.7515ms    
500k max id / 500k num elem
TurboBuckets                 Memory: 4.5 MB      Avg: 1.7766ms  (+24.15%)    Median: 1.6490ms  (+15.67%)    1.3477ms    2.7288ms     
Vec                          Memory: 2.0 MB      Avg: 0.3759ms  (0.75%)      Median: 0.3598ms  (0.50%)      0.2975ms    0.5415ms     
FxHashMap                    Memory: 1.8 MB      Avg: 3.7157ms  (+6.57%)     Median: 3.5566ms  (+2.38%)     3.1622ms    5.2814ms     
FxHashMap Reserved Max Id    Memory: 9.4 MB      Avg: 5.8076ms  (+39.56%)    Median: 5.3666ms  (+31.39%)    3.0705ms    15.8945ms    

```
### Peak Memory
To activate peak memory reporting, you need to wrap your allocator with the PeakMemAlloc and call `set_alloc` in the group.

While number of allocations are also interesting for performance analysis, 
peak memory will determine the memory requirements of the code.

### TODO

- [ ] Customize Reporting (e.g. write your own reporter)

#### Maybe Later Features:
* Charts
* Auto comparison of Histograms (e.g. if a benchmark has several bands in which it operates, it would be nice to compare them)

