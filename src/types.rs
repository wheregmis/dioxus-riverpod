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

/// Common trait bounds for provider parameters
pub trait ProviderParamBounds:
    Clone + PartialEq + std::hash::Hash + std::fmt::Debug + Send + Sync + 'static
{
}
impl<T> ProviderParamBounds for T where
    T: Clone + PartialEq + std::hash::Hash + std::fmt::Debug + Send + Sync + 'static
{
}

/// Common trait bounds for provider output types
pub trait ProviderOutputBounds: Clone + PartialEq + Send + Sync + 'static {}
impl<T> ProviderOutputBounds for T where T: Clone + PartialEq + Send + Sync + 'static {}

/// Common trait bounds for provider error types
pub trait ProviderErrorBounds: Clone + PartialEq + Send + Sync + 'static {}
impl<T> ProviderErrorBounds for T where T: Clone + PartialEq + Send + Sync + 'static {}
