//! Common types and aliases used throughout dioxus-provider

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};

/// Type alias for reactive context storage
pub type ReactiveContextSet = Arc<Mutex<HashSet<ReactiveContext>>>;

/// Type alias for reactive context registry
pub type ReactiveContextRegistry = Arc<Mutex<HashMap<String, ReactiveContextSet>>>;

/// Type alias for interval task registry
/// Since we're using dioxus spawn for both platforms, we only store interval duration
pub type IntervalTaskRegistry = Arc<Mutex<HashMap<String, (Duration, ())>>>;

/// Represents a reactive context that can be marked as dirty when providers update
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ReactiveContext {
    pub id: String,
}

impl ReactiveContext {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}
