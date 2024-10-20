//! [PluginManager] for Binggan.
//!
//! The plugin manager is responsible for managing plugins and emitting events to them
//! that occur during the benchmark run.
//!
//! See the [PluginEvents] enum for the list of events that can be emitted.
//! Any type that implements the [EventListener] trait can be added to [PluginManager].
//!

use crate::{bench::BenchResult, bench_id::BenchId};
use std::any::Any;

/// Events that can be emitted by the benchmark runner.
#[derive(Debug, Clone, Copy)]
pub enum PluginEvents<'a> {
    /// The number of iterations for the benches in a group has been set.
    /// The previous event was `GroupStart`.
    GroupBenchNumIters {
        /// The number of iterations for each bench in the group. The whole group has the same
        /// number of iterations to be a fair comparison between the benches in the group.
        num_iter: usize,
    },
    /// The number of iterations for the bench group.
    /// The previous event was `GroupStart`.
    GroupNumIters {
        /// Unlike GroupBenchNumIters, this is the number of iterations for the group.
        /// So each bench is run `num_iter` * `num_group_iter` times.
        num_iter: usize,
    },
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
    /// The priority of the event listener.
    /// If the event listener has a higher priority, it will be called first.
    /// E.g. perf counter plugin should be called after the cache trasher in the [PluginEvents::BenchStart] event,
    /// so that the cache trashing is not included in the perf counter.
    ///
    /// The default priority is `u32::MAX / 2`.
    ///
    /// Note: In the [PluginEvents::BenchStop] event, the order is reversed. The listener with the highest priority
    /// is called last. This is to endure symmetry.
    fn prio(&self) -> u32 {
        u32::MAX / 2
    }

    /// The name of the event listener.
    fn name(&self) -> &'static str;
    /// Handle an event.
    /// See the [PluginEvents] enum for the list of events that can be emitted.
    fn on_event(&mut self, event: PluginEvents);
    /// Downcast the listener to `Any`.
    fn as_any(&mut self) -> &mut dyn Any;
}

/// [PluginManager] is responsible for managing plugins and emitting events.
///
/// See the [PluginEvents] enum for the list of events that can be emitted.
/// Any type that implements the `EventListener` trait can be added to the plugin manager.
pub struct PluginManager {
    listeners: Vec<(String, Box<dyn EventListener>)>,
}
impl PluginManager {
    /// Create a new instance of [PluginManager].
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    /// Removes any plugins with the same name and sets the new listener.
    pub fn replace_plugin<L: EventListener + 'static>(&mut self, listener: L) -> &mut Self {
        self.remove_plugin_by_name(listener.name());
        self.add_plugin(listener);
        self
    }

    /// Add a new plugin. Note that this will not remove listeners with the
    /// same name.
    pub fn add_plugin<L: EventListener + 'static>(&mut self, listener: L) -> &mut Self {
        self.listeners
            .push((listener.name().to_owned(), Box::new(listener)));
        self.listeners.sort_by_key(|(_, l)| l.prio());
        self
    }

    /// Add a new plugin to the plugin manager if it is not already present by name.
    pub fn add_plugin_if_absent<L: EventListener + 'static>(&mut self, listener: L) -> &mut Self {
        if self.get_plugins(listener.name()).is_some() {
            return self;
        }
        self.add_plugin(listener);
        self
    }

    /// Get the first plugin that matches the name.
    pub fn get_plugins(&mut self, name: &str) -> Option<&mut Box<dyn EventListener>> {
        self.listeners
            .iter_mut()
            .find(|(n, _)| n == name)
            .map(|(_, l)| l)
    }

    /// Downcast a plugin to a specific type.
    pub fn downcast_plugin<T: 'static>(&mut self, name: &str) -> Option<&mut T> {
        self.get_plugins(name)?.as_any().downcast_mut::<T>()
    }

    /// Remove a plugin by name.
    pub fn remove_plugin_by_name(&mut self, name: &str) {
        self.listeners.retain(|(n, _)| n != name);
    }

    /// Emit an event to all plugin.
    pub fn emit(&mut self, event: PluginEvents) {
        if matches!(event, PluginEvents::BenchStop { .. }) {
            for (_, listener) in self.listeners.iter_mut().rev() {
                listener.on_event(event);
            }
        } else {
            for (_, listener) in self.listeners.iter_mut() {
                listener.on_event(event);
            }
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
