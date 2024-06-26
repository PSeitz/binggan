#![deny(
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_imports,
    unused_qualifications,
    missing_docs
)]

//! Binggan (餅乾, bǐng gān, means cookie in Chinese) is a benchmarking library for Rust.
//! It is designed to provide fast and stable results, report peak memory consumption and integrate with perf.
//!
//! It allows arbitrary named inputs to be passed to the benchmarks.
//!
//! # Benchmarking
//! There are 2 main entry points [BenchRunner] and [InputGroup].
//! If you want to run benchmarks with the same inputs, use [InputGroup].
//! Otherwise if you need more flexibility you can use [BenchGroup] via [BenchRunner::new_group_with_name](crate::BenchRunner::new_group_with_name).
//!
//! See <https://github.com/PSeitz/binggan/tree/main/benches> for examples.
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
//! fn bench_group(mut runner: InputGroup<Vec<usize>>) {
//!     runner.set_alloc(GLOBAL); // Set the peak mem allocator. This will enable peak memory reporting.
//!     runner.config().enable_perf(); // Enable perf integration. This only works on linux.
//!     runner.register("vec", move |data| {
//!         test_vec(data);
//!     });
//!     runner.register("hashmap", move |data| {
//!         test_hashmap(data);
//!     });
//!    runner.run();
//! }
//!
//! fn test_vec(data: &Vec<usize>) {
//!     let mut vec = Vec::new();
//!     for idx in data {
//!         if vec.len() <= *idx {
//!             vec.resize(idx + 1, 0);
//!         }
//!         vec[*idx] += 1;
//!     }
//!     black_box(vec);
//! }
//! fn test_hashmap(data: &Vec<usize>) {
//!     let mut map = std::collections::HashMap::new();
//!     for idx in data {
//!         *map.entry(idx).or_insert(0) += 1;
//!     }
//!     black_box(map);
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
//!         });
//!         group.register_with_input("hashmap", data, move |data| {
//!             black_box(test_hashmap(data));
//!         });
//!     }
//!     group.run();
//! }
//!
//! fn main() {
//!     run_bench();
//! }
//! ```
//!

#![cfg_attr(feature = "real_blackbox", feature(test))]

#[cfg(feature = "real_blackbox")]
extern crate test;

pub use peakmem_alloc::*;

pub(crate) mod bench;
pub(crate) mod bench_runner;
pub(crate) mod format;
pub(crate) mod profiler;
pub(crate) mod report;
pub(crate) mod stats;

mod bench_group;
mod bench_input_group;
mod config;

pub use bench_group::BenchGroup;
pub use bench_input_group::InputGroup;
pub use bench_runner::BenchRunner;
pub use bench_runner::NamedInput;
pub use config::Config;

pub(crate) use config::parse_args;

/// A function that is opaque to the optimizer, used to prevent the compiler from
/// optimizing away computations in a benchmark.
pub use std::hint::black_box;
