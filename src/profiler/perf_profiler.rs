use std::error::Error;

use crate::profiler::CounterValues;
use crate::profiler::Profiler;
use perf_event::events::{Cache, CacheOp, CacheResult, Hardware, WhichCache};
use perf_event::Counter;
use perf_event::{Builder, Group};

pub(crate) struct PerfCounters {
    group: Group,
    // translation lookaside buffer
    tlbd_access_counter: Counter,
    tlbd_miss_counter: Counter,
    l1d_access_counter: Counter,
    l1d_miss_counter: Counter,
    branches: Counter,
    branch_misses: Counter,
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
            branch_misses: missed_branches,
        })
    }
}

impl Profiler for PerfCounters {
    fn enable(&mut self) {
        self.group.enable().unwrap();
    }
    fn disable(&mut self) {
        self.group.disable().unwrap();
    }
    fn finish(&mut self, num_iter: u64) -> std::io::Result<CounterValues> {
        let num_iter = num_iter as f64;
        let l1d_access_count = self.l1d_access_counter.read()? as f64 / num_iter;
        let tlbd_access_count = self.tlbd_access_counter.read()? as f64 / num_iter;
        let tlbd_miss_count = self.tlbd_miss_counter.read()? as f64 / num_iter;
        let miss_count = self.l1d_miss_counter.read()? as f64 / num_iter;
        let branches_count = self.branches.read()? as f64 / num_iter;
        let missed_branches_count = self.branch_misses.read()? as f64 / num_iter;

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
