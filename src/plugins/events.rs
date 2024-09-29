use std::any::Any;

use rustc_hash::FxHashMap;

use crate::{bench::BenchResult, bench_id::BenchId};

/// Events that can be emitted by the benchmark runner.
#[derive(Debug, Clone, Copy)]
pub enum BingganEvents<'a> {
    /// Parameter is the name of the run
    StartRun(&'a str),
    /// Parameter is the name of the benchmark group
    GroupStart(&'a str),
    GroupStop {
        name: Option<&'a str>,
        results: &'a [BenchResult],
        output_value_column_title: &'static str,
    },
    /// The benchmark is started. Note that a benchmark can be run multiple times for higher
    /// accuracy. BenchStart and BenchStop are not called for each iteration.
    ///
    BenchStart(&'a BenchId),
    BenchStop(&'a BenchId, u64),
}

pub trait EventListener: Any {
    fn name(&self) -> &'static str;
    fn on_event(&mut self, event: BingganEvents);
    fn as_any(&mut self) -> &mut dyn Any;
}

/// The event manager is responsible for managing event listeners and emitting events.
/// It is used to notify listeners about events that occur during the benchmark run.
///
pub struct EventManager {
    listeners: Vec<(String, Box<dyn EventListener>)>,
}
impl EventManager {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    /// Add a new listener to the event manager if it is not already present by name.
    pub fn add_listener_if_absent<L: EventListener + 'static>(&mut self, listener: L) {
        if self.get_listener(listener.name()).is_some() {
            return;
        }
        self.listeners
            .push((listener.name().to_owned(), Box::new(listener)));
    }

    /// Get a listener by name.
    pub fn get_listener(&mut self, name: &str) -> Option<&mut Box<dyn EventListener>> {
        self.listeners
            .iter_mut()
            .find(|(n, _)| n == name)
            .map(|(_, l)| l)
    }

    pub fn downcast_listener<T: 'static>(&mut self, name: &str) -> Option<&mut T> {
        self.get_listener(name)?.as_any().downcast_mut::<T>()
    }

    /// Remove a listener by name.
    pub fn remove_listener_by_name(&mut self, name: &str) {
        self.listeners.retain(|(n, _)| n != name);
    }

    pub fn emit(&mut self, event: BingganEvents) {
        for (_, listener) in self.listeners.iter_mut() {
            listener.on_event(event);
        }
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper struct to store data per bench id
pub struct PerBenchData<T> {
    per_bench_data: FxHashMap<BenchId, T>,
}
impl<T> Default for PerBenchData<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> PerBenchData<T> {
    pub fn new() -> Self {
        Self {
            per_bench_data: FxHashMap::default(),
        }
    }
    pub fn get_mut(&mut self, bench_id: &BenchId) -> Option<&mut T> {
        self.per_bench_data.get_mut(bench_id)
    }
    pub fn get(&self, bench_id: &BenchId) -> Option<&T> {
        self.per_bench_data.get(bench_id)
    }
    pub fn insert_if_absent<F: FnOnce() -> T>(&mut self, bench_id: &BenchId, data: F) {
        if !self.per_bench_data.contains_key(bench_id) {
            self.per_bench_data.insert(bench_id.clone(), data());
        }
    }
}
