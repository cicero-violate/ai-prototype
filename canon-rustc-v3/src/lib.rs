#![cfg_attr(feature = "rustc-driver", feature(rustc_private))]

#[cfg(feature = "rustc-driver")]
extern crate rustc_abi;
#[cfg(feature = "rustc-driver")]
extern crate rustc_ast;
#[cfg(feature = "rustc-driver")]
extern crate rustc_driver;
#[cfg(feature = "rustc-driver")]
extern crate rustc_hir;
#[cfg(feature = "rustc-driver")]
extern crate rustc_interface;
#[cfg(feature = "rustc-driver")]
extern crate rustc_middle;
#[cfg(feature = "rustc-driver")]
extern crate rustc_span;

pub mod emit;
pub mod facts;
pub mod flags;
pub mod graph;

#[cfg(feature = "rustc-driver")]
pub mod hir;
#[cfg(feature = "rustc-driver")]
pub mod index;
#[cfg(feature = "rustc-driver")]
pub mod mir;
#[cfg(feature = "rustc-driver")]
pub mod wrapper;
