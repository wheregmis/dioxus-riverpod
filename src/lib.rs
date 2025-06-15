#![doc = include_str!("../README.md")]

// Core modules
pub mod cache;
pub mod disposal;
pub mod global;
pub mod hooks;
pub mod injection;
pub mod providers;
pub mod refresh;
pub mod types;

pub mod prelude {
    pub use crate::global::*;
    pub use crate::hooks::*;
    pub use crate::injection::*;
    pub use crate::providers::*;
    pub use crate::types::*;

    // Re-export the unified provider attribute macro
    pub use dioxus_riverpod_macros::provider;
}
