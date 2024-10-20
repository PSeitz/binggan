/// Linux specific code for perf counter integration.
use std::error::Error;
use std::io;

use crate::bench_id::BenchId;
use crate::plugins::{EventListener, PerBenchData, PluginEvents};
use perf_event::events::{Cache, CacheOp, CacheResult, Hardware, Software, WhichCache};
use perf_event::Counter;
use perf_event::{Builder, Group};
use std::any::Any;
use yansi::Paint;

use super::{default_perf_counters, PerfCounter, PerfCounterValues, PERF_CNT_EVENT_LISTENER_NAME};

impl PerfCounter {
    /// Maps each enum variant to the appropriate `Hardware` or `Cache` type
    fn build(self, group: &mut Group) -> io::Result<Counter> {
        let builder = Builder::new().group(group);
        match self {
            PerfCounter::CpuCycles => builder.kind(Hardware::CPU_CYCLES).build(),
            PerfCounter::PageFaultsMinor => builder.kind(Software::PAGE_FAULTS_MIN).build(),
            PerfCounter::PageFaultsMajor => builder.kind(Software::PAGE_FAULTS_MAJ).build(),
            PerfCounter::PageFaults => builder.kind(Software::PAGE_FAULTS).build(),
            PerfCounter::Branches => builder.kind(Hardware::BRANCH_INSTRUCTIONS).build(),
            PerfCounter::MissedBranches => builder.kind(Hardware::BRANCH_MISSES).build(),
            PerfCounter::L1DCacheAccess => builder
                .kind(Cache {
                    which: WhichCache::L1D,
                    operation: CacheOp::READ,
                    result: CacheResult::ACCESS,
                })
                .build(),
            PerfCounter::L1DCacheMiss => builder
                .kind(Cache {
                    which: WhichCache::L1D,
                    operation: CacheOp::READ,
                    result: CacheResult::MISS,
                })
                .build(),
            PerfCounter::TLBDataAccess => builder
                .kind(Cache {
                    which: WhichCache::DTLB,
                    operation: CacheOp::READ,
                    result: CacheResult::ACCESS,
                })
                .build(),
            PerfCounter::TLBDataMiss => builder
                .kind(Cache {
                    which: WhichCache::DTLB,
                    operation: CacheOp::READ,
                    result: CacheResult::MISS,
                })
                .build(),
            PerfCounter::InstructionsRetired => builder.kind(Hardware::INSTRUCTIONS).build(),
        }
    }
}

pub(crate) struct PerfCounterGroup {
    group: Group,
    counters: Vec<(PerfCounter, Counter)>, // Store enum and corresponding counter
}

impl PerfCounterGroup {
    pub fn new(counters_enum: &[PerfCounter]) -> Result<Self, Box<dyn Error>> {
        let mut group = Group::new()?;

        let mut counters: Vec<_> = Vec::new();
        for counter in counters_enum {
            match counter.build(&mut group) {
                Ok(built_counter) => counters.push((*counter, built_counter)),
                Err(e) => {
                    let warn = "Some counter combinations are incompatible".bold().red();
                    println!(
                        "{}. Disabling PerfCounter: {} \nError: {:?}",
                        warn, counter, e
                    );
                }
            }
        }

        group.disable()?;

        Ok(PerfCounterGroup { group, counters })
    }
}

impl PerfCounterGroup {
    pub fn enable(&mut self) {
        self.group.enable().unwrap();
    }
    pub fn disable(&mut self) {
        self.group.disable().unwrap();
    }
    pub fn finish(&mut self, num_iter: u64) -> io::Result<PerfCounterValues> {
        let num_iter = num_iter as f64;
        let mut values = Vec::new();

        for (counter_enum, counter) in &mut self.counters {
            let count = counter.read().unwrap() as f64 / num_iter;
            values.push((*counter_enum, count)); // Store in HashMap
        }

        Ok(PerfCounterValues { values })
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
/// use binggan::{*, plugins::*};
///
/// let mut runner = BenchRunner::new();
/// runner.add_plugin(PerfCounterPlugin::default());
/// ```

pub struct PerfCounterPlugin {
    perf_per_bench: PerBenchData<Option<PerfCounterGroup>>,
    enabled_perf_counters: Vec<PerfCounter>,
}

impl Default for PerfCounterPlugin {
    fn default() -> Self {
        PerfCounterPlugin {
            perf_per_bench: PerBenchData::default(),
            enabled_perf_counters: default_perf_counters().to_vec(),
        }
    }
}

impl PerfCounterPlugin {
    /// Create a new instance of the plugin with the specified counters
    ///
    pub fn new(perf_counters: Vec<PerfCounter>) -> Self {
        PerfCounterPlugin {
            perf_per_bench: PerBenchData::default(),
            enabled_perf_counters: perf_counters,
        }
    }
    /// Get the perf counter for a bench id
    pub(crate) fn get_by_bench_id_mut(
        &mut self,
        bench_id: &BenchId,
    ) -> Option<&mut PerfCounterGroup> {
        self.perf_per_bench
            .get_mut(bench_id)
            .and_then(Option::as_mut)
    }
}

impl EventListener for PerfCounterPlugin {
    fn prio(&self) -> u32 {
        0
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
    fn name(&self) -> &'static str {
        PERF_CNT_EVENT_LISTENER_NAME
    }
    fn on_event(&mut self, event: PluginEvents) {
        match event {
            PluginEvents::BenchStart { bench_id } => {
                self.perf_per_bench.insert_if_absent(bench_id, || {
                    PerfCounterGroup::new(&self.enabled_perf_counters).ok()
                });
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
