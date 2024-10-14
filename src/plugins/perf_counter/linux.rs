/// Linux specific code for perf counter integration.
use std::error::Error;

use crate::bench_id::BenchId;
use crate::plugins::{EventListener, PerBenchData, PluginEvents};
use perf_event::events::{Cache, CacheOp, CacheResult, Hardware, WhichCache};
use perf_event::Counter;
use perf_event::{Builder, Group};
use std::any::Any;

use super::{CounterValues, PERF_CNT_EVENT_LISTENER_NAME};

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

///
/// Plugin to report perf counters.
///
/// The numbers are reported with the following legend:
/// ```bash
/// Br: Branches
/// MBr: Missed Branches
/// L1dA: L1 Data Access
/// L1dM: L1 Data Access Misses
/// TLBdA: Translation Lookaside Buffer Data Access
/// TLBdM: Translation Lookaside Buffer Data Access Misses
/// ```
/// e.g.
/// ```bash
/// fibonacci    Memory: 0 B       Avg: 135ns      Median: 136ns     132ns          140ns    
///              L1dA: 809.310     L1dM: 0.002     Br: 685.059       MBr: 0.010     
/// baseline     Memory: 0 B       Avg: 1ns        Median: 1ns       1ns            1ns      
///              L1dA: 2.001       L1dM: 0.000     Br: 6.001         MBr: 0.000     
/// ```
///
/// # Note:
/// This is only available on Linux. On other OSs this does nothing.
///
/// Perf may run into limitations where all counters are reported as zero. <https://github.com/jimblandy/perf-event/issues/2>.
/// Disabling the NMI watchdog should help:
///
/// `sudo sh -c "echo '0' > /proc/sys/kernel/nmi_watchdog"`
///
/// ## Usage Example
/// ```rust
/// use binggan::{*, plugins::*}
///
/// let mut runner = BenchRunner::new();
/// runner
///    .get_plugin_manager()
///    .add_plugin(PerfCounterPlugin::default());
/// ```

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
