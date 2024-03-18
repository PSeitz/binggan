
# TODO

- [] Filter benchmark handling via args
- [] Pass Allocator

### Features (mostly done):
* Fast Compile Times. (Looking at you criterion ;)
* Fast Execution
* Stack offset randomization
* Interleaving test runs between benches in a group (configurable)
* Named benchmark inputs
* Stats (low overhead)
* Perf Integration
* Memory Usage (peak memory usage)
* Linux Centric
* No Macros, no magic. (no guessing, just a regular well-documented API)
* Easy Benchmark Generation. (It's just code)
* Runs on stable Rust

#### Maybe Later Features:
* Charts
* Delta Comparison
* Auto comparison of Histograms (e.g. if a benchmark has several bands in which it operates, it would be nice to compare them)

### Memory Usage (peak memory usage)

This measures the peak memory usage of the benchmarked code. While number of allocations are also interesting for performance analysis, what you ususally care about is the 
peak memory consumption of the code. Since that is what will determine the memory requirements of the code.
