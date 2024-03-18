#![cfg_attr(feature = "real_blackbox", feature(test))]

#[cfg(feature = "real_blackbox")]
extern crate test;

use peakmem_alloc::*;
use std::alloc::System;

#[global_allocator]
pub static GLOBAL: &PeakMemAlloc<System> = &INSTRUMENTED_SYSTEM;

mod bench_group;
pub mod format;
pub mod stats;
pub use bench_group::BenchGroup;

#[derive(Default)]
pub struct GibKekseJetzt {
    //bench_groups: Vec<BenchGroup>,
}

impl GibKekseJetzt {
    pub fn new_group(&mut self, name: &str) -> bench_group::BenchGroup<()> {
        bench_group::BenchGroup::new(name.to_string())
    }
    pub fn new_group_with_inputs<I, S: Into<String>>(
        &mut self,
        name: &str,
        inputs: Vec<(S, I)>,
    ) -> bench_group::BenchGroup<I> {
        bench_group::BenchGroup::new_with_inputs(name.to_string(), inputs)
    }
    pub fn new() -> Self {
        GibKekseJetzt {
            ..Default::default()
        }
    }
    //pub fn register<F: FnMut() -> () + 'static, S: Into<String>>(&mut self, name: S, fun: F) {
    //self.bench_groups[0].register(name, fun);
    //}

    //pub fn register_with_size<F: FnMut() -> () + 'static, S: Into<String>>(
    //&mut self,
    //name: S,
    //size: u64,
    //fun: F,
    //) {
    //self.bench_groups[0].register_with_size(name, size, fun);
    //}

    //pub fn run(&mut self) {
    //for group in &mut self.bench_groups {
    //group.run();
    //}
    //}

    //pub fn report(&mut self) {
    //for group in &mut self.bench_groups {
    //group.report();
    //}
    //}
    ////println!("{}: {}", self.name, self.func());
}

mod macros;

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
