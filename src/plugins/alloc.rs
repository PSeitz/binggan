use std::any::Any;

use peakmem_alloc::PeakMemAllocTrait;

use crate::{
    bench_id::BenchId,
    plugins::{BingganEvents, EventListener, PerBenchData},
};

/// Integration via EventListener
/// tracks the max memory consumption per bench
pub(crate) struct AllocPerBench {
    alloc_per_bench: PerBenchData<Vec<usize>>,
    alloc: &'static dyn PeakMemAllocTrait,
}
impl AllocPerBench {
    /// Creates a new instance of `AllocPerBench`.
    /// The `alloc` parameter is the allocator that will be used to track memory consumption.
    ///
    pub fn new(alloc: &'static dyn PeakMemAllocTrait) -> Self {
        Self {
            alloc_per_bench: PerBenchData::new(),
            alloc,
        }
    }
    pub fn get_by_bench_id(&self, bench_id: &BenchId) -> Option<&Vec<usize>> {
        self.alloc_per_bench.get(bench_id)
    }
}

pub static ALLOC_EVENT_LISTENER_NAME: &str = "_binggan_alloc";

impl EventListener for AllocPerBench {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
    fn name(&self) -> &'static str {
        ALLOC_EVENT_LISTENER_NAME
    }
    fn on_event(&mut self, event: BingganEvents) {
        match event {
            BingganEvents::BenchStart { bench_id } => {
                self.alloc_per_bench.insert_if_absent(bench_id, Vec::new);
                self.alloc.reset_peak_memory();
            }
            BingganEvents::BenchStop { bench_id, .. } => {
                let perf = self.alloc_per_bench.get_mut(bench_id).unwrap();
                perf.push(self.alloc.get_peak_memory());
            }
            _ => {}
        }
    }
}
