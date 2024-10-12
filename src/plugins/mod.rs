//! The plugin system works by registering to events.
//!
//! The `BingganEvents` enum contains all the events that can be emitted.
//! The `EventListener` trait is used to listen to these events.
//!
//! The `BenchRunner` has an `EventManager` which can be used to add listeners.
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
