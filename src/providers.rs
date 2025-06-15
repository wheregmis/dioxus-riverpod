//! # Dioxus Riverpod Providers
//!
//! This module provides a unified API for the dioxus-riverpod reactive state management system,
//! inspired by Riverpod from the Flutter ecosystem.
//!
//! This module re-exports all the core functionality from the specialized modules:
//! - `cache`: Async state and cache management
//! - `hooks`: Provider trait and hooks
//! - `refresh`: Refresh registry and interval management  
//! - `disposal`: Auto-disposal logic
//! - `types`: Common types and aliases
//!
//! ## Core Features
//!
//! - **Providers**: Async operations that return data with automatic caching
//! - **Parameterized Providers**: Providers that accept parameters for dynamic data fetching
//! - **Interval Providers**: Auto-refreshing providers for live data
//! - **Cache Management**: Automatic caching with selective invalidation
//! - **Reactive Updates**: UI automatically updates when data changes
//!
//! ## Usage
//!
//! ```rust,no_run
//! use dioxus_riverpod::prelude::*;
//!
//! #[derive(Clone, Debug, PartialEq)]
//! struct User { name: String }
//!
//! #[derive(Clone, Debug, PartialEq)]
//! struct Data { value: i32 }
//!
//! #[derive(Clone, Debug, PartialEq)]
//! struct Post { title: String }
//!
//! // Simple provider
//! #[provider]
//! async fn fetch_user() -> Result<User, String> {
//!     Ok(User { name: "Alice".to_string() })
//! }
//!
//! // Provider with auto-refresh
//! #[provider(interval_secs = 30)]
//! async fn live_data() -> Result<Data, String> {
//!     Ok(Data { value: 42 })
//! }
//!
//! // Parameterized provider
//! #[provider]
//! async fn fetch_user_posts(user_id: u32) -> Result<Vec<Post>, String> {
//!     Ok(vec![Post { title: format!("Post by user {}", user_id) }])
//! }
//! ```

// Re-export all types and functionality from specialized modules
pub use crate::{
    cache::{AsyncState, ProviderCache},
    hooks::{
        Provider, UseProvider,
        use_provider, use_provider_cache, use_invalidate_provider, 
        use_clear_provider_cache
    },
    refresh::RefreshRegistry,
    types::{ReactiveContext, ReactiveContextSet, ReactiveContextRegistry, IntervalTaskRegistry},
};
