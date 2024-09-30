//! The plugin system works by registering to events.
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
//!     fn on_event(&mut self, event: BingganEvents) {
//!         match event {
//!             BingganEvents::GroupStart{runner_name, ..} => {
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
//! let events = runner.get_event_manager();
//! events.add_listener_if_absent(MyListener);
//!
//! ```
//!

pub(crate) mod alloc;
pub mod events;
pub(crate) mod profiler;
pub use events::*;
