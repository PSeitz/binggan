/// Linux specific code for perf counter integration.
use std::error::Error;

use crate::bench_id::BenchId;
use crate::plugins::{EventListener, PerBenchData, PluginEvents};
use perf_event::events::{Cache, CacheOp, CacheResult, Hardware, WhichCache};
use perf_event::Counter;
use perf_event::{Builder, Group};
use std::any::Any;

use super::CounterValues;

pub(crate) struct PerfCounters {
    branches: Counter,
    branches_missed: Counter,
    group: Group,
    // translation lookaside buffer
    tlbd_access_counter: Counter,
    tlbd_miss_counter: Counter,
    l1d_access_counter: Counter,
    l1d_miss_counter: Counter,
}
impl PerfCounters {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut group = Group::new()?;
        const L1D_ACCESS: Cache = Cache {
            which: WhichCache::L1D,
            operation: CacheOp::READ,
            result: CacheResult::ACCESS,
        };
        const L1D_MISS: Cache = Cache {
            result: CacheResult::MISS,
            ..L1D_ACCESS
        };
        let l1d_access_counter = Builder::new().group(&mut group).kind(L1D_ACCESS).build()?;
        let l1d_miss_counter = Builder::new().group(&mut group).kind(L1D_MISS).build()?;

        // TLB
        const TLBD_ACCESS: Cache = Cache {
            which: WhichCache::DTLB,
            operation: CacheOp::READ,
            result: CacheResult::ACCESS,
        };
        const TLBD_MISS: Cache = Cache {
            result: CacheResult::MISS,
            ..TLBD_ACCESS
        };
        let tlbd_access_counter = Builder::new().group(&mut group).kind(TLBD_ACCESS).build()?;
        let tlbd_miss_counter = Builder::new().group(&mut group).kind(TLBD_MISS).build()?;

        let branches = Builder::new()
            .group(&mut group)
            .kind(Hardware::BRANCH_INSTRUCTIONS)
            .build()?;
        let missed_branches = Builder::new()
            .group(&mut group)
            .kind(Hardware::BRANCH_MISSES)
            .build()?;
        group.disable()?;

        Ok(PerfCounters {
            group,
            tlbd_access_counter,
            tlbd_miss_counter,
            l1d_access_counter,
            l1d_miss_counter,
            branches,
            branches_missed: missed_branches,
        })
    }
}

impl PerfCounters {
    pub fn enable(&mut self) {
        self.group.enable().unwrap();
    }
    pub fn disable(&mut self) {
        self.group.disable().unwrap();
    }
    pub fn finish(&mut self, num_iter: u64) -> std::io::Result<CounterValues> {
        let num_iter = num_iter as f64;
        let l1d_access_count = self.l1d_access_counter.read()? as f64 / num_iter;
        let tlbd_access_count = self.tlbd_access_counter.read()? as f64 / num_iter;
        let tlbd_miss_count = self.tlbd_miss_counter.read()? as f64 / num_iter;
        let miss_count = self.l1d_miss_counter.read()? as f64 / num_iter;
        let branches_count = self.branches.read()? as f64 / num_iter;
        let missed_branches_count = self.branches_missed.read()? as f64 / num_iter;

        Ok(CounterValues {
            l1d_access_count,
            tlbd_access_count,
            tlbd_miss_count,
            l1d_miss_count: miss_count,
            branches_count,
            missed_branches_count,
        })
    }
}

/// Name of the event listener
pub static PERF_CNT_EVENT_LISTENER_NAME: &str = "_binggan_perf";

/// Integration via EventListener
/// One counter per bench id.
#[derive(Default)]
pub struct PerfCounterPlugin {
    perf_per_bench: PerBenchData<Option<PerfCounters>>,
}

impl PerfCounterPlugin {
    /// Get the perf counter for a bench id
    pub(crate) fn get_by_bench_id_mut(&mut self, bench_id: &BenchId) -> Option<&mut PerfCounters> {
        self.perf_per_bench
            .get_mut(bench_id)
            .and_then(Option::as_mut)
    }
}

impl EventListener for PerfCounterPlugin {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
    fn name(&self) -> &'static str {
        PERF_CNT_EVENT_LISTENER_NAME
    }
    fn on_event(&mut self, event: PluginEvents) {
        match event {
            PluginEvents::BenchStart { bench_id } => {
                self.perf_per_bench
                    .insert_if_absent(bench_id, || PerfCounters::new().ok());
                let perf = self.perf_per_bench.get_mut(bench_id).unwrap();
                if let Some(perf) = perf {
                    perf.enable();
                }
            }
            PluginEvents::BenchStop { bench_id, .. } => {
                let perf = self.perf_per_bench.get_mut(bench_id).unwrap();
                if let Some(perf) = perf {
                    perf.disable();
                }
            }
            _ => {}
        }
    }
}
