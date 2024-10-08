use crate::bench_id::BenchId;
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
