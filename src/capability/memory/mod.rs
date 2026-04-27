//! Memory capability.
//!
//! Memory is an indexed prior-run store. It does not mutate the kernel by
//! itself; it produces deterministic lookup records that context, judgment, and
//! planning can consume as evidence inputs.

pub mod store;

pub use self::store::{MemoryFact, MemoryIndex, MemoryLookupRecord};