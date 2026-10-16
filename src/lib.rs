//! Default faction source and identity data. Programs depend only on the public game SDK.
mod catalog;
pub use catalog::*;
include!(concat!(env!("OUT_DIR"), "/programs.rs"));
include!(concat!(env!("OUT_DIR"), "/examples.rs"));
