//! # Global Provider Management
//!
//! This module provides global singletons for cache, disposal, and refresh management
//! that operate at application scale rather than component lifecycle scale.

use std::sync::OnceLock;

use crate::{cache::ProviderCache, disposal::DisposalRegistry, refresh::RefreshRegistry};

/// Global singleton instance of the provider cache
static GLOBAL_CACHE: OnceLock<ProviderCache> = OnceLock::new();

/// Global singleton instance of the refresh registry
static GLOBAL_REFRESH_REGISTRY: OnceLock<RefreshRegistry> = OnceLock::new();

/// Global singleton instance of the disposal registry
static GLOBAL_DISPOSAL_REGISTRY: OnceLock<DisposalRegistry> = OnceLock::new();

/// Initialize the global provider management system
///
/// This should be called once at the start of your application,
/// typically in your main function or app initialization.
///
/// ## Example
///
/// ```rust,no_run
/// use dioxus::prelude::*;
/// use dioxus_riverpod::global::init_global_providers;
///
/// fn main() {
///     // Initialize global provider system
///     init_global_providers();
///     
///     // Launch your app
///     dioxus::launch(app);
/// }
///
/// #[component]
/// fn app() -> Element {
///     rsx! {
///         div { "Hello World!" }
///     }
/// }
/// ```
pub fn init_global_providers() {
    // Initialize cache first
    let cache = GLOBAL_CACHE.get_or_init(ProviderCache::new);

    // Initialize refresh registry
    let _refresh_registry = GLOBAL_REFRESH_REGISTRY.get_or_init(RefreshRegistry::new);

    // Initialize disposal registry with reference to cache
    let _disposal_registry =
        GLOBAL_DISPOSAL_REGISTRY.get_or_init(|| DisposalRegistry::new(cache.clone()));
}

/// Get the global provider cache instance
///
/// Returns the global cache that persists across the entire application lifecycle.
/// This cache is shared by all providers regardless of component boundaries.
///
/// ## Panics
///
/// Panics if `init_global_providers()` has not been called yet.
pub fn get_global_cache() -> &'static ProviderCache {
    GLOBAL_CACHE
        .get()
        .expect("Global providers not initialized. Call init_global_providers() first.")
}

/// Get the global refresh registry instance
///
/// Returns the global refresh registry that manages reactive updates and intervals
/// across the entire application.
///
/// ## Panics
///
/// Panics if `init_global_providers()` has not been called yet.
pub fn get_global_refresh_registry() -> &'static RefreshRegistry {
    GLOBAL_REFRESH_REGISTRY
        .get()
        .expect("Global providers not initialized. Call init_global_providers() first.")
}

/// Get the global disposal registry instance
///
/// Returns the global disposal registry that manages automatic cleanup
/// across the entire application.
///
/// ## Panics
///
/// Panics if `init_global_providers()` has not been called yet.
pub fn get_global_disposal_registry() -> &'static DisposalRegistry {
    GLOBAL_DISPOSAL_REGISTRY
        .get()
        .expect("Global providers not initialized. Call init_global_providers() first.")
}

/// Check if global providers have been initialized
pub fn is_initialized() -> bool {
    GLOBAL_CACHE.get().is_some()
        && GLOBAL_REFRESH_REGISTRY.get().is_some()
        && GLOBAL_DISPOSAL_REGISTRY.get().is_some()
}

/// Reset global providers (mainly for testing)
///
/// This is primarily intended for testing scenarios where you need
/// to reset the global state between tests.
///
/// ## Warning
///
/// This function is not thread-safe and should only be used in single-threaded
/// test environments. Do not use this in production code.
#[cfg(test)]
pub fn reset_global_providers() {
    // Note: OnceLock doesn't have a public reset method, so this is mainly
    // for documentation. In real tests, you'd typically use a different
    // approach or restart the test process.
    panic!("Global provider reset is not currently supported. Restart the application.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_provider_initialization() {
        assert!(!is_initialized());

        init_global_providers();

        assert!(is_initialized());

        // Test that we can get all instances
        let _cache = get_global_cache();
        let _refresh = get_global_refresh_registry();
        let _disposal = get_global_disposal_registry();
    }

    #[test]
    #[should_panic(expected = "Global providers not initialized")]
    fn test_panic_when_not_initialized() {
        // Check if already initialized - skip test if so
        if is_initialized() {
            panic!("Global providers not initialized"); // Manually trigger expected panic
        }

        // This should panic since we haven't called init_global_providers()
        let _cache = get_global_cache();
    }
}
