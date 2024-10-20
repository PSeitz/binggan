//! The plugin system works by registering to events.
//!
//! The `PluginEvents` enum contains all the events that can be emitted.
//! The `EventListener` trait is used to listen to these events.
//!
//! The `BenchRunner` has an `PluginManager` which can be used to add plugins.
//! The listeners can be used to track memory consumption, report results, etc.
//!
//! `name` is used to identify the listener.
//!
//! # Example
//! ```rust
//! use binggan::*;
//! use binggan::plugins::*;
//!
//! struct MyListener;
//!
//! impl EventListener for MyListener {
//!     fn name(&self) -> &'static str {
//!         "my_listener"
//!     }
//!     fn on_event(&mut self, event: PluginEvents) {
//!         match event {
//!             PluginEvents::GroupStart{runner_name, ..} => {
//!                 println!("Starting: {:?}", runner_name);
//!             }
//!             _ => {}
//!         }
//!     }
//!     fn as_any(&mut self) -> &mut dyn std::any::Any {
//!         self
//!     }
//! }
//! let mut runner = BenchRunner::new();
//! runner.get_plugin_manager().add_plugin(MyListener);
//!
//! ```
//!

pub(crate) mod alloc;

#[cfg(feature = "branch_predictor")]
mod bpu_trasher;
mod cache_trasher;
mod perf_counter;

pub mod events;

use rustc_hash::FxHashMap;

pub use alloc::*;
pub use cache_trasher::*;
pub use events::*;
pub use perf_counter::*;

#[cfg(feature = "branch_predictor")]
pub use bpu_trasher::*;

use crate::BenchId;

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
