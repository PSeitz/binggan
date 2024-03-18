#![deny(
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_imports,
    unused_qualifications,
    missing_docs
)]

//! Binggan is a benchmarking library for Rust.
//! It is designed to be simple to use and to provide a good overview of the performance of your code and its memory consumption.
//!
//! It allows arbitrary named inputs to be passed to the benchmarks.
//!
//! # Example
//! ```rust
//! use binggan::{black_box, BenchGroup, Binggan, INSTRUMENTED_SYSTEM, PeakMemAlloc};
//!
//! #[global_allocator]
//! pub static GLOBAL: &PeakMemAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;
//!
//! fn fibonacci(n: u64) -> u64 {
//!     match n {
//!          0 | 1 => 1,
//!         n => fibonacci(n - 1) + fibonacci(n - 2),
//!     }
//! }
//!
//! fn bench_fibonacci_group<I>(mut runner: BenchGroup<I>) {
//!     // Set the peak mem allocator. This will enable peak memory reporting.
//!     runner.set_alloc(GLOBAL);
//!     runner.register("fibonacci", move |_| {
//!         fibonacci(black_box(10));
//!     });
//!     runner.register("fibonacci_alt", move |_| {
//!        // unimplemented!()
//!     });
//!     runner.run();
//! }
//!
//! fn main() {
//!     let mut runner = Binggan::new();
//!     bench_fibonacci_group(runner.new_group("fibonacci_plain"));
//!     bench_fibonacci_group(
//!        runner.new_group_with_inputs("fibonacci_input", vec![("10", 10), ("15", 15)]),
//!     );
//! }
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
pub trait BenchInputSize {
    /// The size of the input, if it is known.
    /// It is used to calculate the throughput of the benchmark.
    fn input_size(&self) -> Option<usize> {
        None
    }
}
impl<T: ?Sized> BenchInputSize for T {}

/// The main struct to create benchmarks.
///
/// It actually does nothing currently, but it is the entry point to create benchmarks.
#[derive(Default, Debug, Clone, Copy)]
pub struct Binggan {}

/// The options to configure the benchmarking.
#[derive(Debug, Default)]
pub struct Options {
    /// The options to configure the benchmarking.
    pub interleave: bool,
    /// Run perf profiler
    pub exact: bool,
    /// The options to configure the benchmarking.
    pub filter: Option<String>,
    /// Enable perf integration
    enable_perf: bool,
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
impl Binggan {
    /// Create a new instance of Binggan.
    pub fn new() -> Self {
        Binggan {}
    }
    /// Create a new benchmark group.
    pub fn new_group(&mut self, name: &str) -> BenchGroup {
        bench_group::BenchGroup::new(name.to_string(), parse_args())
    }
    /// Create a new benchmark group with named inputs.
    /// # Example
    /// ```rust
    /// use binggan::{black_box, BenchGroup, Binggan, INSTRUMENTED_SYSTEM, PeakMemAlloc};
    /// let mut runner = Binggan::new();
    /// runner.new_group_with_inputs("krasser_index", vec![("zipf 1%", 10), ("zipf 10%", 15)]);
    /// ```
    pub fn new_group_with_inputs<I, S: Into<String>>(
        &mut self,
        name: impl Into<String>,
        inputs: Vec<(S, I)>,
    ) -> BenchGroup<I> {
        parse_args();
        bench_group::BenchGroup::new_with_inputs(name.into(), inputs, parse_args())
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
