use super::{EventListener, PluginEvents};
use bpu_trasher::trash_bpu;

/// Trashes the branch predictor between benchmarks.
///
#[derive(Clone, Copy, Default)]
pub struct BPUTrasher {}

impl EventListener for BPUTrasher {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn name(&self) -> &'static str {
        "bpu_trasher"
    }
    fn on_event(&mut self, event: PluginEvents) {
        if let PluginEvents::BenchStart { bench_id: _ } = event {
            trash_bpu();
        }
    }
}
