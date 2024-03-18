![binggan logo](https://raw.githubusercontent.com/PSeitz/binggan/master/logo_s.png)

Binggan is a benchmarking library for Rust.
It is designed to be simple to use and to provide a good overview of the performance of your code and its memory consumption.

It allows arbitrary named inputs to be passed to the benchmarks.

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
