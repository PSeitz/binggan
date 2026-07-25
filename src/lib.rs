#![deny(
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_imports,
    unused_qualifications,
    missing_docs
)]

//! # Binggan (餅乾, bǐng gān) - A Benchmarking Library for Stable Rust
//!
//! Binggan is a benchmarking library designed for flexibility, providing fast and stable results.
//! It reports peak memory consumption and can integrate with `perf` for hardware performance counters.
//!
//! ## Main Components
//! [BenchRunner] is the main entry point for running individual benchmarks and creating
//! [BenchGroup]s. Use [BenchRunner::new_group] to run multiple benchmarks together.
//!
//! See <https://github.com/PSeitz/binggan/tree/main/benches> for examples.
//!
//! ## OutputValue
//! The typical benchmarking flow involves providing some input, processing it through a function, and obtaining an output.
//! Benchmarks return [OutputValue], which represents the result of the benchmark. This output can be particularly
//! useful in scenarios like compression benchmarks, where it reports the output size or other relevant metrics.
//!
//! ## Plugins
//! See the [plugins] module for more information on how to register custom plugins.
//!
//! ## Reporting
//! See the [report] module for more information on how to customize the benchmark result reporting.
//!
//! # Perf Integration
//! Binggan can integrate with perf to report hardware performance counters.
//! See [PerfCounterPlugin](crate::plugins::PerfCounterPlugin) for more information.
//!
//! # Example for BenchGroup
//!
//! ```
//! use std::collections::HashMap;
//!
//! use binggan::{black_box, plugins::*, BenchRunner, PeakMemAlloc, INSTRUMENTED_SYSTEM};
//!
//! #[global_allocator]
//! pub static GLOBAL: &PeakMemAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;
//!
//! fn test_vec(data: &Vec<usize>) -> Vec<i32> {
//!     let mut vec = Vec::new();
//!     for idx in data {
//!         if vec.len() <= *idx {
//!             vec.resize(idx + 1, 0);
//!         }
//!         vec[*idx] += 1;
//!     }
//!     vec
//! }
//! fn test_hashmap(data: &Vec<usize>) -> HashMap<&usize, i32> {
//!     let mut map = std::collections::HashMap::new();
//!     for idx in data {
//!         *map.entry(idx).or_insert(0) += 1;
//!     }
//!     map
//! }
//!
//! fn run_bench() {
//!     let inputs: Vec<(&str, Vec<usize>)> = vec![
//!         (
//!             "max id 100; 100 el all the same",
//!             std::iter::repeat_n(100, 100).collect(),
//!         ),
//!         ("max id 100; 100 el all different", (0..100).collect()),
//!     ];
//!     let mut runner: BenchRunner = BenchRunner::new();
//!
//!     runner
//!        // Set the peak mem allocator. This will enable peak memory reporting.
//!        .add_plugin(PeakMemAllocPlugin::new(GLOBAL))
//!        .add_plugin(CacheTrasher::default());
//!
//!     for (input_name, data) in inputs.iter() {
//!         let mut group = runner.new_group();
//!         group.set_name(input_name);
//!         group.set_input_size(data.len() * std::mem::size_of::<usize>());
//!         group.register_with_input("vec", data, move |data| {
//!             black_box(test_vec(data));
//!         });
//!         group.register_with_input("hashmap", data, move |data| {
//!             black_box(test_hashmap(data));
//!         });
//!         group.run();
//!     }
//! }
//!
//! fn main() {
//!     run_bench();
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "real_blackbox", feature(test))]

#[cfg(feature = "real_blackbox")]
extern crate test;

/// The module to define custom plugins
pub mod plugins;
/// The module to report benchmark results
pub mod report;

pub(crate) mod bench;
pub(crate) mod bench_id;
pub(crate) mod bench_runner;
pub(crate) mod filter;
pub(crate) mod output_value;
pub(crate) mod stats;
pub(crate) mod write_results;

mod bench_group;
mod config;

pub use bench::BenchResult;
pub use bench_group::BenchGroup;
pub use bench_id::BenchId;
pub use bench_runner::BenchRunner;
pub use config::Config;
pub use output_value::OutputValue;
pub use peakmem_alloc::*;

pub(crate) use config::parse_args;

/// A function that is opaque to the optimizer, used to prevent the compiler from
/// optimizing away computations in a benchmark.
pub use std::hint::black_box;
