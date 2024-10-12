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
//! Binggan has two primary entry points for running benchmarks:
//!
//! - **[BenchRunner]**: A main runner for. Useful for single benchmarks or to create groups.
//! - **[InputGroup]**: Use this when running a group of benchmarks with the same inputs, where ownership of inputs can be transferred.
//!
//! Otherwise if you need more flexibility you can use [BenchGroup] via [BenchRunner::new_group](crate::BenchRunner::new).
//!
//! See <https://github.com/PSeitz/binggan/tree/main/benches> for examples. `benches/bench_group.rs` and
//! `benches/bench_input_group.rs` are different ways to produce the same output.
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
//! See [Config::enable_perf](crate::Config::enable_perf) for more information.
//!
//! # Example for InputGroup
//! ```rust
//! use binggan::{black_box, InputGroup, PeakMemAlloc, INSTRUMENTED_SYSTEM};
//!
//! #[global_allocator]
//! pub static GLOBAL: &PeakMemAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;
//!
//! fn main() {
//!     // Tuples of name and data for the inputs
//!     let data = vec![
//!         (
//!             "max id 100; 100 el all the same",
//!             std::iter::repeat(100).take(100).collect(),
//!         ),
//!         (   
//!             "max id 100; 100 el all different",
//!             (0..100).collect()
//!         ),
//!     ];
//!     bench_group(InputGroup::new_with_inputs(data));
//! }
//!
//! // Run the benchmark for the group with input `Vec<usize>`
//! fn bench_group(mut runner: InputGroup<Vec<usize>, u64>) {
//!     runner.set_alloc(GLOBAL); // Set the peak mem allocator. This will enable peak memory reporting.
//!     runner.config().enable_perf(); // Enable perf integration. This only works on linux.
//!     runner.register("vec", move |data| {
//!         let vec = test_vec(data);
//!         Some(vec.len() as u64)
//!     });
//!     runner.register("hashmap", move |data| {
//!         let map = test_hashmap(data);
//!         Some(map.len() as u64)
//!     });
//!    runner.run();
//! }
//!
//! fn test_vec(data: &Vec<usize>) -> Vec<usize> {
//!     let mut vec = Vec::new();
//!     for idx in data {
//!         if vec.len() <= *idx {
//!             vec.resize(idx + 1, 0);
//!         }
//!         vec[*idx] += 1;
//!     }
//!     black_box(vec)
//! }
//! fn test_hashmap(data: &Vec<usize>) -> std::collections::HashMap<usize, i32> {
//!     let mut map = std::collections::HashMap::new();
//!     for idx in data {
//!         *map.entry(*idx).or_insert(0) += 1;
//!     }
//!     black_box(map)
//! }
//!
//! ```
//!
//! # Example for BenchGroup
//!
//! ```
//! use std::collections::HashMap;
//!
//! use binggan::{black_box, BenchRunner, PeakMemAlloc, INSTRUMENTED_SYSTEM};
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
//!             std::iter::repeat(100).take(100).collect(),
//!         ),
//!         ("max id 100; 100 el all different", (0..100).collect()),
//!     ];
//!     let mut runner: BenchRunner = BenchRunner::new();
//!     runner.set_alloc(GLOBAL); // Set the peak mem allocator. This will enable peak memory reporting.
//!
//!     runner.config().enable_perf();
//!     runner.config().set_cache_trasher(true);
//!
//!     let mut group = runner.new_group();
//!     for (input_name, data) in inputs.iter() {
//!         group.set_input_size(data.len() * std::mem::size_of::<usize>());
//!         group.register_with_input("vec", data, move |data| {
//!             black_box(test_vec(data));
//!             Some(())
//!         });
//!         group.register_with_input("hashmap", data, move |data| {
//!             black_box(test_hashmap(data));
//!             Some(())
//!         });
//!     }
//!     group.run();
//! }
//!
//! fn main() {
//!     run_bench();
//! }
//! ```

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
pub(crate) mod output_value;
pub(crate) mod stats;
pub(crate) mod write_results;

mod bench_group;
mod bench_input_group;
mod config;

pub use bench::BenchResult;
pub use bench_group::BenchGroup;
pub use bench_id::BenchId;
pub use bench_input_group::InputGroup;
pub use bench_runner::BenchRunner;
pub use config::Config;
pub use output_value::OutputValue;
pub use peakmem_alloc::*;

pub(crate) use config::parse_args;

/// A function that is opaque to the optimizer, used to prevent the compiler from
/// optimizing away computations in a benchmark.
pub use std::hint::black_box;
