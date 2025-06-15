#![doc = include_str!("../README.md")]

// Core modules
pub mod cache;
pub mod disposal;
pub mod hooks;
pub mod providers;
pub mod refresh;
pub mod types;

pub mod prelude {
    pub use crate::providers::*;

    // Re-export the unified provider attribute macro
    pub use dioxus_riverpod_macros::provider;
}
