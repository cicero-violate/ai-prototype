//! Action host compatibility facade.
//!
//! Host-backed action execution is migrating out of the legacy tooling/native
//! namespace. This facade lets action code depend on execution terminology while
//! the concrete host implementation remains in the compatibility module.

pub use crate::capability::tooling::native::{
    execute_native_tool, execute_recorded_shell, NativeToolHost as ActionHost,
};
