use crate::plugins::profiler::Profiler;
use std::error::Error;

use super::CounterValues;

pub(crate) struct PerfProfiler {}
impl PerfProfiler {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {})
    }
}
impl Profiler for PerfProfiler {
    fn enable(&mut self) {}
    fn disable(&mut self) {}
    fn finish(&mut self, _num_iter: u64) -> std::io::Result<CounterValues> {
        Ok(CounterValues::default())
    }
}

// Plugin
pub static PERF_CNT_EVENT_LISTENER_NAME: &str = "_binggan_perf";

#[derive(Default)]
pub struct PerfCounterPerBench {}

impl PerfCounterPerBench {
    pub fn get_by_bench_id_mut(&mut self, bench_id: &BenchId) -> Option<&mut PerfCounters> {
        None
    }
}

impl EventListener for PerfCounterPerBench {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
    fn name(&self) -> &'static str {
        PERF_CNT_EVENT_LISTENER_NAME
    }
    fn on_event(&mut self, event: BingganEvents) {}
}
