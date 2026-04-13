use std::any::Any;
use yansi::Paint;

use peakmem_alloc::PeakMemAllocTrait;

use crate::{
    bench_id::BenchId,
    plugins::{EventListener, PerBenchData, PluginEvents},
};

/// Plugin to track peak memory consumption.
pub struct PeakMemAllocPlugin {
    alloc_per_bench: PerBenchData<Vec<usize>>,
    alloc: &'static dyn PeakMemAllocTrait,
}
impl PeakMemAllocPlugin {
    /// Creates a new instance of `AllocPerBench`.
    /// The `alloc` parameter is the allocator that will be used to track memory consumption.
    ///
    pub fn new(alloc: &'static dyn PeakMemAllocTrait) -> Self {
        Self {
            alloc_per_bench: PerBenchData::new(),
            alloc,
        }
    }
    /// Returns the peak memory consumptions for each group run for the given bench id.
    pub fn get_by_bench_id(&self, bench_id: &BenchId) -> Option<&Vec<usize>> {
        self.alloc_per_bench.get(bench_id)
    }
}

/// The plugin name for PeakAllocPlugin.
pub static ALLOC_EVENT_LISTENER_NAME: &str = "_binggan_alloc";

impl EventListener for PeakMemAllocPlugin {
    fn prio(&self) -> u32 {
        u32::MAX - 2
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
    fn name(&self) -> &'static str {
        ALLOC_EVENT_LISTENER_NAME
    }
    fn on_event(&mut self, event: PluginEvents) {
        match event {
            PluginEvents::BenchStart { bench_id } => {
                self.alloc_per_bench.insert_if_absent(bench_id, Vec::new);
                self.alloc.reset_peak_memory();
            }
            PluginEvents::BenchStop { bench_id, .. } => {
                let perf = self.alloc_per_bench.get_mut(bench_id).unwrap();
                perf.push(self.alloc.get_peak_memory());
            }
            _ => {}
        }
    }

    fn custom_metrics(&self, bench_id: &BenchId, metrics: &mut Vec<(&'static str, f64)>) {
        if let Some(perf) = self.get_by_bench_id(bench_id) {
            let total_memory: usize = perf.iter().copied().sum();
            let avg_memory = if perf.is_empty() {
                0
            } else {
                total_memory / perf.len()
            };
            metrics.push(("Memory", avg_memory as f64));
        }
    }

    fn custom_metric_keys(&self) -> &[&'static str] {
        &["Memory"]
    }

    fn format_custom_metrics(
        &self,
        stats: &BenchStats,
        other: Option<&crate::stats::BenchStats>,
    ) -> Vec<(&'static str, String)> {
        let avg_memory = stats
            .custom_metrics
            .iter()
            .find(|(k, _)| *k == "Memory")
            .map(|(_, v)| *v)
            .unwrap_or(0.0) as u64;
        let mem_diff = crate::stats::compute_diff(stats, None, other, |stats| {
            stats
                .custom_metrics
                .iter()
                .find(|(k, _)| *k == "Memory")
                .map(|(_, v)| *v)
                .unwrap_or(0.0) as u64
        });

        let s = format!(
            "Memory: {} {}",
            crate::report::format::bytes_to_string(avg_memory)
                .bright_cyan()
                .bold(),
            mem_diff,
        );
        vec![("Memory", s)]
    }
}
