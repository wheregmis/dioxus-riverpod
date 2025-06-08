#![doc = include_str!("../README.md")]

pub mod providers;

pub mod prelude {
    pub use crate::providers::*;

    // Re-export the unified provider attribute macro
    pub use dioxus_riverpod_macros::provider;
}
