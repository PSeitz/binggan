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
mod cache_trasher;
pub mod events;

pub(crate) mod perf_counter;

pub use perf_counter::*;

pub use cache_trasher::*;
pub use events::*;
