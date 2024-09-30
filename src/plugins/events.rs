//! Event manager for Binggan.
//! The event manager is responsible for managing event listeners and emitting events.
//! It is used to notify listeners about events that occur during the benchmark run.
//!
//! See the `BingganEvents` enum for the list of events that can be emitted.
//! Any type that implements the `EventListener` trait can be added to the event manager.
//!

use crate::{bench::BenchResult, bench_id::BenchId};
use rustc_hash::FxHashMap;
use std::any::Any;

/// Events that can be emitted by the benchmark runner.
#[derive(Debug, Clone, Copy)]
pub enum BingganEvents<'a> {
    /// Profiling of the group started
    GroupStart {
        /// The name of the runner
        runner_name: Option<&'a str>,
        /// The name of the group
        group_name: Option<&'a str>,
        /// The name of the column of the output value.
        output_value_column_title: &'static str,
    },
    /// Profiling of the group finished.
    GroupStop {
        /// The name of the runner
        runner_name: Option<&'a str>,
        /// The name of the group
        group_name: Option<&'a str>,
        /// The results of the group
        /// This will include the results of all the benchmarks in the group.
        /// It also contains delta information of the last run if available
        results: &'a [BenchResult],
        /// The name of the column of the output value.
        output_value_column_title: &'static str,
    },
    /// A benchmark in a group is started. Note that a benchmark can be run multiple times for higher
    /// accuracy. BenchStart and BenchStop are not called for each iteration.
    ///
    /// A group is iterated multiple times. This will be called for every iteration in the group.
    BenchStart {
        /// The bench id
        bench_id: &'a BenchId,
    },
    /// A benchmark in a group is stopped.
    BenchStop {
        /// The bench id
        bench_id: &'a BenchId,
        /// The duration of the benchmark
        duration: u64,
    },
}

/// The trait for listening to events emitted by the benchmark runner.
pub trait EventListener: Any {
    /// The name of the event listener.
    fn name(&self) -> &'static str;
    /// Handle an event.
    /// See the [BingganEvents] enum for the list of events that can be emitted.
    fn on_event(&mut self, event: BingganEvents);
    /// Downcast the listener to `Any`.
    fn as_any(&mut self) -> &mut dyn Any;
}

/// The event manager is responsible for managing event listeners and emitting events.
/// It is used to notify listeners about events that occur during the benchmark run.
///
/// See the `BingganEvents` enum for the list of events that can be emitted.
/// Any type that implements the `EventListener` trait can be added to the event manager.
pub struct EventManager {
    listeners: Vec<(String, Box<dyn EventListener>)>,
}
impl EventManager {
    /// Create a new instance of `EventManager`.
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

    /// Downcast a listener to a specific type.
    pub fn downcast_listener<T: 'static>(&mut self, name: &str) -> Option<&mut T> {
        self.get_listener(name)?.as_any().downcast_mut::<T>()
    }

    /// Remove a listener by name.
    pub fn remove_listener_by_name(&mut self, name: &str) {
        self.listeners.retain(|(n, _)| n != name);
    }

    /// Emit an event to all listeners.
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
    /// Create a new instance of `PerBenchData`.
    pub fn new() -> Self {
        Self {
            per_bench_data: FxHashMap::default(),
        }
    }
    /// Get a mutable reference to the data for a specific bench id.
    pub fn get_mut(&mut self, bench_id: &BenchId) -> Option<&mut T> {
        self.per_bench_data.get_mut(bench_id)
    }
    /// Get a reference to the data for a specific bench id.
    pub fn get(&self, bench_id: &BenchId) -> Option<&T> {
        self.per_bench_data.get(bench_id)
    }
    /// Insert data for a specific bench id if it is not already present.
    pub fn insert_if_absent<F: FnOnce() -> T>(&mut self, bench_id: &BenchId, data: F) {
        if !self.per_bench_data.contains_key(bench_id) {
            self.per_bench_data.insert(bench_id.clone(), data());
        }
    }
}
