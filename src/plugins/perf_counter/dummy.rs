use super::PERF_CNT_EVENT_LISTENER_NAME;
use crate::plugins::*;
use std::any::Any;

///
/// Perf Counter Plugin.
///
/// Stores one counter group per bench id.
#[derive(Default)]
#[allow(missing_copy_implementations)]
pub struct PerfCounterPlugin {}

impl PerfCounterPlugin {
    /// Create a new instance of the plugin with the specified counters
    ///
    pub fn new(perf_counters: Vec<PerfCounter>) -> Self {
        PerfCounterPlugin {}
    }
}

impl EventListener for PerfCounterPlugin {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
    fn name(&self) -> &'static str {
        PERF_CNT_EVENT_LISTENER_NAME
    }
    fn on_event(&mut self, _event: PluginEvents) {}
}
