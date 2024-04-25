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
//! It is designed to be simple to use and to provide a good overview of the performance of your code and its memory consumption.
//!
//! It allows arbitrary named inputs to be passed to the benchmarks.
//!
//! # Example
//! ```rust
//! use binggan::{black_box, BenchGroup, PeakMemAlloc, INSTRUMENTED_SYSTEM};
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
//!     bench_group(BenchGroup::new_with_inputs(data));
//! }
//!
//! // Run the benchmark for the group with input `Vec<usize>`
//! fn bench_group(mut runner: BenchGroup<Vec<usize>>) {
//!     runner.set_alloc(GLOBAL); // Set the peak mem allocator. This will enable peak memory reporting.
//!     runner.enable_perf();
//!     runner.register("vec", move |data| {
//!         black_box(test_vec(data));
//!     });
//!     runner.register("hashmap", move |data| {
//!         black_box(test_hashmap(data));
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
//! }
//! fn test_hashmap(data: &Vec<usize>) {
//!     let mut map = std::collections::HashMap::new();
//!     for idx in data {
//!         *map.entry(idx).or_insert(0) += 1;
//!     }
//! }
//!
//! ```
//!

#![cfg_attr(feature = "real_blackbox", feature(test))]

#[cfg(feature = "real_blackbox")]
extern crate test;

pub use peakmem_alloc::*;

pub(crate) mod bench;
mod bench_group;
pub(crate) mod format;
pub(crate) mod profiler;
pub(crate) mod report;
pub(crate) mod stats;
pub use bench_group::BenchGroup;
use rustop::opts;

/// Reports the size in bytes a input.
/// Unused currently.
pub trait BenchInputSize {
    /// The size of the input, if it is known.
    /// It is used to calculate the throughput of the benchmark.
    fn input_size(&self) -> Option<usize> {
        None
    }
}
impl<T: ?Sized> BenchInputSize for T {}

/// The options to configure the benchmarking.
/// The can be set on `BenchGroup`.
#[derive(Debug, Default)]
pub struct Options {
    /// Interleave benchmarks
    pub interleave: bool,
    /// Filter should match exact
    pub exact: bool,
    /// The filter for the benchmarks
    /// This is read from the command line by default.
    pub filter: Option<String>,
    /// Enable/disable perf integration
    pub enable_perf: bool,
}

fn parse_args() -> Options {
    let res = opts! {
        synopsis "";
        opt bench:bool, desc:"bench flag passed by rustc";
        opt interleave:bool=true, desc:"The benchmarks run interleaved by default, i.e. one iteration of each bench after another
                         This may lead to better results, it may also lead to worse results.
                         It very much depends on the benches and the environment you would like to simulate. ";
        opt exact:bool, desc:"Filter benchmarks by exact name rather than by pattern.";
        param filter:Option<String>, desc:"run only bench containing name."; // an optional positional parameter
    }
    .parse();
    if let Ok((args, _rest)) = res {
        Options {
            interleave: args.interleave,
            exact: args.exact,
            filter: args.filter,
            ..Default::default()
        }
    } else if let Err(rustop::Error::Help(help)) = res {
        println!("{}", help);
        std::process::exit(0);
    } else if let Err(e) = res {
        println!("{}", e);
        std::process::exit(1);
    } else {
        unreachable!();
    }
}

/// A function that is opaque to the optimizer, used to prevent the compiler from
/// optimizing away computations in a benchmark.
///
/// This variant is backed by the (unstable) test::black_box function.
#[cfg(feature = "real_blackbox")]
pub fn black_box<T>(dummy: T) -> T {
    test::black_box(dummy)
}

/// A function that is opaque to the optimizer, used to prevent the compiler from
/// optimizing away computations in a benchmark.
///
/// This variant is stable-compatible, but it may cause some performance overhead
/// or fail to prevent code from being eliminated.
#[cfg(not(feature = "real_blackbox"))]
pub fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}
