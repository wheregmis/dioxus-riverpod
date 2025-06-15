//! # Global Provider Hooks
//!
//! This module provides the main hooks and traits for working with global providers in Dioxus.
//! It includes the core Provider trait, the unified hook system for both parameterized
//! and non-parameterized providers, and utility hooks for global cache management.
//!
//! **⚠️ Global Initialization Required**: All hooks in this module require
//! `dioxus_riverpod::global::init_global_providers()` to be called at application startup.
//!
//! ## Key Features
//!
//! - **Global State Management**: Application-wide provider state
//! - **Provider Trait**: Unified interface for all provider types
//! - **Automatic Caching**: Built-in global caching with configurable expiration
//! - **Stale-While-Revalidate (SWR)**: Background updates while serving stale data
//! - **Auto-Dispose**: Automatic cleanup of unused providers
//! - **Interval Refresh**: Automatic refresh at specified intervals
//! - **Cross-Platform**: Works on both web and desktop using dioxus tasks
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use dioxus::prelude::*;
//! use dioxus_riverpod::{prelude::*, global::init_global_providers};
//!
//! fn main() {
//!     // REQUIRED: Initialize global providers
//!     init_global_providers();
//!     dioxus::launch(App);
//! }
//!
//! #[derive(Clone, PartialEq)]
//! struct DataProvider;
//!
//! impl Provider<()> for DataProvider {
//!     type Output = String;
//!     type Error = String;
//!
//!     async fn run(&self, _: ()) -> Result<Self::Output, Self::Error> {
//!         Ok("data".to_string())
//!     }
//!
//!     fn id(&self, _: &()) -> String {
//!         "data".to_string()
//!     }
//! }
//!
//! #[component]
//! fn App() -> Element {
//!     rsx! { MyComponent {} }
//! }
//!
//! #[component]
//! fn MyComponent() -> Element {
//!     // Use provider directly - global cache handles everything
//!     let data = use_provider(DataProvider, ());
//!
//!     match *data.read() {
//!         AsyncState::Loading => rsx! { div { "Loading..." } },
//!         AsyncState::Success(ref value) => rsx! { div { "{value}" } },
//!         AsyncState::Error(_) => rsx! { div { "Error occurred" } },
//!     }
//! }
//! ```

use dioxus_lib::prelude::*;
use std::{fmt::Debug, future::Future, hash::Hash, time::Duration};
use tracing::debug;

use crate::{
    cache::{AsyncState, ProviderCache},
    disposal::DisposalRegistry,
    global::{get_global_cache, get_global_disposal_registry, get_global_refresh_registry},
    refresh::{RefreshRegistry, TaskType},
};

/// A unified trait for defining providers - async operations that return data
///
/// This trait supports both simple providers (no parameters) and parameterized providers.
/// Use `Provider<()>` for simple providers and `Provider<ParamType>` for parameterized providers.
///
/// ## Features
///
/// - **Async Execution**: All providers are async by default
/// - **Configurable Caching**: Optional cache expiration times
/// - **Stale-While-Revalidate**: Serve stale data while revalidating in background
/// - **Auto-Refresh**: Optional automatic refresh at intervals
/// - **Auto-Dispose**: Automatic cleanup when providers are no longer used
///
/// ## Cross-Platform Compatibility
///
/// The Provider trait is designed to work across platforms using Dioxus's spawn system:
/// - Uses `dioxus::spawn` for async execution (no Send + Sync required for most types)
/// - Parameters may need Send + Sync if shared across contexts
/// - Output and Error types only need Clone since they stay within Dioxus context
///
/// ## Example
///
/// ```rust,no_run
/// use dioxus_riverpod::prelude::*;
/// use std::time::Duration;
///
/// #[derive(Clone, PartialEq)]
/// struct DataProvider;
///
/// impl Provider<()> for DataProvider {
///     type Output = String;
///     type Error = String;
///
///     async fn run(&self, _: ()) -> Result<Self::Output, Self::Error> {
///         // Fetch data from API
///         Ok("Hello, World!".to_string())
///     }
///
///     fn id(&self, _: &()) -> String {
///         "data_provider".to_string()
///     }
///
///     // Optional: Cache for 5 minutes
///     fn cache_expiration(&self) -> Option<Duration> {
///         Some(Duration::from_secs(300))
///     }
///
///     // Optional: Consider stale after 1 minute
///     fn stale_time(&self) -> Option<Duration> {
///         Some(Duration::from_secs(60))
///     }
/// }
/// ```
pub trait Provider<Param = ()>: Clone + PartialEq + 'static
where
    Param: Clone + PartialEq + Hash + Debug + 'static,
{
    /// The type of data returned on success
    type Output: Clone + PartialEq + Send + Sync + 'static;
    /// The type of error returned on failure  
    type Error: Clone + Send + Sync + 'static;

    /// Execute the async operation
    ///
    /// This method performs the actual work of the provider, such as fetching data
    /// from an API, reading from a database, or computing a value.
    fn run(&self, param: Param) -> impl Future<Output = Result<Self::Output, Self::Error>>;

    /// Get a unique identifier for this provider instance with the given parameters
    ///
    /// This ID is used for caching and invalidation. It should be unique for each
    /// provider and parameter combination. For parameterized providers, include
    /// the parameter values in the ID.
    fn id(&self, param: &Param) -> String;

    /// Get the interval duration for automatic refresh (None means no interval)
    ///
    /// When set, the provider will automatically refresh its data at the specified
    /// interval, even if no component is actively watching it.
    fn interval(&self) -> Option<Duration> {
        None
    }

    /// Get the cache expiration duration (None means no expiration)
    ///
    /// When set, cached data will be considered expired after this duration and
    /// will be removed from the cache, forcing a fresh fetch on the next access.
    fn cache_expiration(&self) -> Option<Duration> {
        None
    }

    /// Get the stale time duration for stale-while-revalidate behavior (None means no SWR)
    ///
    /// When set, data older than this duration will be considered stale and will
    /// trigger a background revalidation while still serving the stale data to the UI.
    fn stale_time(&self) -> Option<Duration> {
        None
    }

    /// Get whether this provider should auto-dispose when unused (false by default)
    ///
    /// **DEPRECATED**: Auto-dispose based on component unmounting is not recommended
    /// in a global cache system. Consider using cache_expiration() instead for
    /// time-based cleanup, which is more appropriate for shared global state.
    ///
    /// When enabled, this feature tracks component usage but disposal is ultimately
    /// based on access patterns rather than component lifecycle.
    fn auto_dispose(&self) -> bool {
        false // Disabled by default - use cache_expiration instead
    }

    /// Get the dispose delay duration - how long to wait before disposing after last usage
    ///
    /// **DEPRECATED**: Use cache_expiration() instead for time-based cache cleanup.
    /// Only relevant when auto_dispose() returns true. If None, uses the default delay.
    fn dispose_delay(&self) -> Option<Duration> {
        None
    }
}

/// Get the provider cache - requires global providers to be initialized
fn get_provider_cache() -> ProviderCache {
    get_global_cache().clone()
}

/// Get the refresh registry - requires global providers to be initialized
fn get_refresh_registry() -> RefreshRegistry {
    get_global_refresh_registry().clone()
}

/// Get the disposal registry - requires global providers to be initialized
fn get_disposal_registry() -> Option<DisposalRegistry> {
    Some(get_global_disposal_registry().clone())
}

/// Hook to access the provider cache for manual cache management
///
/// This hook provides direct access to the global provider cache for manual
/// invalidation, clearing, and other cache operations.
///
/// ## Global Providers Required
///
/// You must call `init_global_providers()` at application startup before using any provider hooks.
///
/// ## Setup
///
/// ```rust,no_run
/// use dioxus_riverpod::{prelude::*, global::init_global_providers};
///
/// fn main() {
///     init_global_providers();
///     dioxus::launch(App);
/// }
///
/// #[component]
/// fn App() -> Element {
///     rsx! {
///         MyComponent {}
///     }
/// }
/// ```
///
/// ## Example
///
/// ```rust,no_run
/// use dioxus::prelude::*;
/// use dioxus_riverpod::prelude::*;
///
/// #[component]
/// fn MyComponent() -> Element {
///     let cache = use_provider_cache();
///     
///     // Manually invalidate a specific cache entry
///     cache.invalidate("my_provider_key");
///     
///     rsx! {
///         div { "Cache operations example" }
///     }
/// }
/// ```
pub fn use_provider_cache() -> ProviderCache {
    get_provider_cache()
}

/// Hook to invalidate a specific provider cache entry
///
/// Returns a function that, when called, will invalidate the cache entry for the
/// specified provider and parameters, and trigger a refresh of all components
/// using that provider.
///
/// Requires global providers to be initialized with `init_global_providers()`.
///
/// ## Example
///
/// ```rust,no_run
/// use dioxus::prelude::*;
/// use dioxus_riverpod::prelude::*;
///
/// #[derive(Clone, PartialEq)]
/// struct UserProvider;
///
/// impl Provider<u32> for UserProvider {
///     type Output = String;
///     type Error = String;
///
///     async fn run(&self, user_id: u32) -> Result<Self::Output, Self::Error> {
///         Ok(format!("User {}", user_id))
///     }
///
///     fn id(&self, user_id: &u32) -> String {
///         format!("user_{}", user_id)
///     }
/// }
///
/// #[component]
/// fn MyComponent() -> Element {
///     let user_id = 1;
///     let invalidate_user = use_invalidate_provider(UserProvider, user_id);
///     
///     rsx! {
///         button {
///             onclick: move |_| invalidate_user(),
///             "Refresh User Data"
///         }
///     }
/// }
/// ```
pub fn use_invalidate_provider<P, Param>(provider: P, param: Param) -> impl Fn() + Clone
where
    P: Provider<Param>,
    Param: Clone + PartialEq + Hash + Debug + 'static,
{
    let cache = get_provider_cache();
    let refresh_registry = get_refresh_registry();
    let cache_key = provider.id(&param);

    move || {
        cache.invalidate(&cache_key);
        refresh_registry.trigger_refresh(&cache_key);
    }
}

/// Hook to clear the entire provider cache
///
/// Returns a function that, when called, will clear all cached provider data
/// and trigger a refresh of all providers currently in use.
///
/// Requires global providers to be initialized with `init_global_providers()`.
///
/// ## Example
///
/// ```rust,no_run
/// use dioxus::prelude::*;
/// use dioxus_riverpod::prelude::*;
///
/// #[component]
/// fn MyComponent() -> Element {
///     let clear_cache = use_clear_provider_cache();
///     
///     rsx! {
///         button {
///             onclick: move |_| clear_cache(),
///             "Clear All Cache"
///         }
///     }
/// }
/// ```
pub fn use_clear_provider_cache() -> impl Fn() + Clone {
    let cache = get_provider_cache();
    let refresh_registry = get_refresh_registry();

    move || {
        cache.clear();
        refresh_registry.clear_all();
    }
}

/// Hook to access the disposal registry for auto-dispose management
///
/// **DEPRECATED**: Auto-dispose based on component unmounting is not recommended
/// in a global cache system. Use cache_expiration() for time-based cleanup instead.
///
/// This provides access to the disposal registry, allowing for manual control
/// over the auto-dispose functionality.
///
/// Requires global providers to be initialized with `init_global_providers()`.
///
/// ## Example
///
/// ```rust,no_run
/// use dioxus::prelude::*;
/// use dioxus_riverpod::prelude::*;
///
/// #[component]
/// fn MyComponent() -> Element {
///     if let Some(disposal_registry) = use_disposal_registry() {
///         // Manually cancel disposal for a specific provider
///         disposal_registry.cancel_disposal("my_provider_key");
///     }
///     
///     rsx! {
///         div { "Disposal management example" }
///     }
/// }
/// ```
#[deprecated(note = "Use cache_expiration() instead of auto_dispose for better global cache behavior")]
pub fn use_disposal_registry() -> Option<DisposalRegistry> {
    get_disposal_registry()
}

/// Trait for unified provider usage - automatically handles providers with and without parameters
///
/// This trait is implemented for all Provider types and provides a unified interface
/// for using providers regardless of whether they take parameters or not.
pub trait UseProvider<Args> {
    /// The type of data returned on success
    type Output: Clone + PartialEq + Send + Sync + 'static;
    /// The type of error returned on failure
    type Error: Clone + Send + Sync + 'static;

    /// Use the provider with the given arguments
    fn use_provider(self, args: Args) -> Signal<AsyncState<Self::Output, Self::Error>>;
}

// Helper functions for common provider operations

/// Sets up auto-dispose cleanup for a provider
fn setup_auto_dispose_cleanup<P, Param>(
    provider: &P,
    cache_key: &str,
    cache: &ProviderCache,
    disposal_registry: &Option<DisposalRegistry>,
) where
    P: Provider<Param> + Clone,
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    if let Some(disposal_reg) = disposal_registry {
        let cache_key_for_cleanup = cache_key.to_string();
        let provider_for_cleanup = provider.clone();
        let disposal_registry_for_cleanup = disposal_reg.clone();
        let cache_for_cleanup = cache.clone();

        use_drop(move || {
            // Find and decrement reference count for the cache entry
            if let Ok(cache_lock) = cache_for_cleanup.cache.lock() {
                if let Some(entry) = cache_lock.get(&cache_key_for_cleanup) {
                    entry.remove_reference();
                    debug!(
                        "🔄 [AUTO-DISPOSE] Removed reference for: {} (refs: {})",
                        cache_key_for_cleanup,
                        entry.reference_count()
                    );
                }
            }

            // Schedule disposal after the specified delay
            let dispose_delay = provider_for_cleanup
                .dispose_delay()
                .unwrap_or_else(DisposalRegistry::default_dispose_delay);
            disposal_registry_for_cleanup
                .schedule_disposal(cache_key_for_cleanup.clone(), dispose_delay);
        });
    }
}

/// Handles auto-dispose reference counting for cache entries
fn handle_auto_dispose_reference(
    cache_key: &str,
    cache: &ProviderCache,
    disposal_registry: &Option<DisposalRegistry>,
    is_new_entry: bool,
) {
    if let Some(disposal_reg) = disposal_registry {
        disposal_reg.cancel_disposal(cache_key);

        // Add reference count for this component using the cache entry
        if let Ok(cache_lock) = cache.cache.lock() {
            if let Some(entry) = cache_lock.get(cache_key) {
                entry.add_reference();
                let action = if is_new_entry { "new entry" } else { "" };
                debug!(
                    "🔄 [AUTO-DISPOSE] Added reference for {}: {} (refs: {})",
                    action,
                    cache_key,
                    entry.reference_count()
                );
            }
        }
    }
}

/// Performs SWR staleness checking and triggers background revalidation if needed
fn check_and_handle_swr_core<P, Param>(
    provider: &P,
    param: &Param,
    cache_key: &str,
    cache: &ProviderCache,
    refresh_registry: &RefreshRegistry,
) where
    P: Provider<Param> + Clone,
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    let stale_time = provider.stale_time();
    let cache_expiration = provider.cache_expiration();

    if let Some(stale_duration) = stale_time {
        if let Ok(cache_lock) = cache.cache.lock() {
            if let Some(entry) = cache_lock.get(cache_key) {
                if entry.is_stale(stale_duration)
                    && !entry.is_expired(cache_expiration.unwrap_or(Duration::from_secs(3600)))
                    && !refresh_registry.is_revalidation_in_progress(cache_key)
                {
                    // Data is stale but not expired and no revalidation in progress - trigger background revalidation
                    if refresh_registry.start_revalidation(cache_key) {
                        debug!(
                            "🔄 [SWR] Data is stale for key: {} - triggering background revalidation",
                            cache_key
                        );

                        let cache = cache.clone();
                        let cache_key_clone = cache_key.to_string();
                        let provider = provider.clone();
                        let param = param.clone();
                        let refresh_registry_clone = refresh_registry.clone();

                        spawn(async move {
                            let result = provider.run(param).await;
                            cache.set(cache_key_clone.clone(), result);

                            // Mark revalidation as complete and trigger refresh
                            refresh_registry_clone.complete_revalidation(&cache_key_clone);
                            refresh_registry_clone.trigger_refresh(&cache_key_clone);
                            debug!(
                                "✅ [SWR] Background revalidation completed for key: {}",
                                cache_key_clone
                            );
                        });
                    }
                }
            }
        }
    }
}

/// Sets up interval refresh task for a provider
fn setup_interval_task_core<P, Param>(
    provider: &P,
    param: &Param,
    cache_key: &str,
    cache: &ProviderCache,
    refresh_registry: &RefreshRegistry,
) where
    P: Provider<Param> + Clone + Send,
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    if let Some(interval) = provider.interval() {
        let cache_clone = cache.clone();
        let provider_clone = provider.clone();
        let param_clone = param.clone();
        let cache_key_clone = cache_key.to_string();
        let refresh_registry_clone = refresh_registry.clone();

        refresh_registry.start_interval_task(cache_key, interval, move || {
            // Re-execute the provider and update cache in background
            let cache_for_task = cache_clone.clone();
            let provider_for_task = provider_clone.clone();
            let param_for_task = param_clone.clone();
            let cache_key_for_task = cache_key_clone.clone();
            let refresh_registry_for_task = refresh_registry_clone.clone();

            spawn(async move {
                let result = provider_for_task.run(param_for_task).await;
                cache_for_task.set(cache_key_for_task.clone(), result);

                // Trigger refresh to mark reactive contexts as dirty and update UI
                refresh_registry_for_task.trigger_refresh(&cache_key_for_task);
            });
        });
    }
}

/// Sets up automatic stale-checking task for SWR providers
fn setup_stale_check_task_core<P, Param>(
    provider: &P,
    param: &Param,
    cache_key: &str,
    cache: &ProviderCache,
    refresh_registry: &RefreshRegistry,
) where
    P: Provider<Param> + Clone + Send,
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    if let Some(stale_time) = provider.stale_time() {
        let cache_clone = cache.clone();
        let provider_clone = provider.clone();
        let param_clone = param.clone();
        let cache_key_clone = cache_key.to_string();
        let refresh_registry_clone = refresh_registry.clone();

        refresh_registry.start_stale_check_task(cache_key, stale_time, move || {
            // Check if data is stale and trigger revalidation if needed
            check_and_handle_swr_core(
                &provider_clone,
                &param_clone,
                &cache_key_clone,
                &cache_clone,
                &refresh_registry_clone,
            );
        });
    }
}

/// Core provider implementation that handles all the common logic
fn use_provider_core<P, Param>(provider: P, param: Param) -> Signal<AsyncState<P::Output, P::Error>>
where
    P: Provider<Param> + Send + Clone,
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    let mut state = use_signal(|| AsyncState::Loading);
    let cache = get_provider_cache();
    let refresh_registry = get_refresh_registry();

    let cache_key = provider.id(&param);
    let cache_expiration = provider.cache_expiration();

    // Setup intelligent cache management (replaces old auto-dispose system)
    setup_intelligent_cache_management(&provider, &cache_key, &cache, &refresh_registry);

    // Check cache expiration before the memo - this happens on every render
    check_and_handle_cache_expiration(cache_expiration, &cache_key, &cache, &refresh_registry);

    // SWR staleness checking - runs on every render to check for stale data
    check_and_handle_swr_core(&provider, &param, &cache_key, &cache, &refresh_registry);

    // Use memo with reactive dependencies to track changes automatically
    let _execution_memo = use_memo(use_reactive!(|(provider, param)| {
        let cache_key = provider.id(&param);
        let cache_expiration = provider.cache_expiration();

        debug!(
            "🔄 [USE_PROVIDER] Memo executing for key: {} with param: {:?}",
            cache_key, param
        );

        // Subscribe to refresh events for this cache key if we have a reactive context
        if let Some(reactive_context) = ReactiveContext::current() {
            refresh_registry.subscribe_to_refresh(&cache_key, reactive_context);
        }

        // Read the current refresh count (this makes the memo reactive to changes)
        let _current_refresh_count = refresh_registry.get_refresh_count(&cache_key);

        // Cache expiration check inside reactive memo - this runs on every reactive update
        check_and_handle_cache_expiration(cache_expiration, &cache_key, &cache, &refresh_registry);

        // Set up interval task if provider has interval configured
        setup_interval_task_core(&provider, &param, &cache_key, &cache, &refresh_registry);

        // Set up stale check task if provider has stale time configured
        setup_stale_check_task_core(&provider, &param, &cache_key, &cache, &refresh_registry);

        // Check cache first - serve any available data (fresh or stale)
        if let Some(cached_result) = cache.get::<Result<P::Output, P::Error>>(&cache_key) {
            // Access tracking is automatically handled by cache.get() updating last_accessed time
            debug!("📊 [CACHE-HIT] Serving cached data for: {}", cache_key);

            match cached_result {
                Ok(data) => {
                    let _ = spawn(async move {
                        state.set(AsyncState::Success(data));
                    });
                }
                Err(error) => {
                    let _ = spawn(async move {
                        state.set(AsyncState::Error(error));
                    });
                }
            }
            return;
        }

        // Cache miss - set loading and spawn async task
        let _ = spawn(async move {
            state.set(AsyncState::Loading);
        });

        let cache = cache.clone();
        let cache_clone = cache.clone();
        let cache_key_clone = cache_key.clone();
        let provider = provider.clone();
        let param = param.clone();
        let mut state_for_async = state;

        spawn(async move {
            let result = provider.run(param).await;
            cache_clone.set(cache_key_clone.clone(), result.clone());

            // Access tracking is automatically handled when data is stored and accessed
            debug!("📊 [CACHE-STORE] Stored new data for: {}", cache_key_clone);

            match result {
                Ok(data) => state_for_async.set(AsyncState::Success(data)),
                Err(error) => state_for_async.set(AsyncState::Error(error)),
            }
        });
    }));

    state
}

/// Implementation for providers with no parameters (simple providers)
impl<P> UseProvider<()> for P
where
    P: Provider<()> + Send,
{
    type Output = P::Output;
    type Error = P::Error;

    fn use_provider(self, _args: ()) -> Signal<AsyncState<Self::Output, Self::Error>> {
        use_provider_core(self, ())
    }
}

/// Implementation for providers with parameters (parameterized providers)
impl<P, Param> UseProvider<(Param,)> for P
where
    P: Provider<Param> + Send,
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    type Output = P::Output;
    type Error = P::Error;

    fn use_provider(self, args: (Param,)) -> Signal<AsyncState<Self::Output, Self::Error>> {
        let param = args.0;
        use_provider_core(self, param)
    }
}

/// Implementation for providers with a single u32 parameter (without tuple wrapper)
impl<P> UseProvider<u32> for P
where
    P: Provider<u32> + Send,
{
    type Output = P::Output;
    type Error = P::Error;

    fn use_provider(self, args: u32) -> Signal<AsyncState<Self::Output, Self::Error>> {
        use_provider_core(self, args)
    }
}

/// Implementation for providers with a single String parameter (without tuple wrapper)
impl<P> UseProvider<String> for P
where
    P: Provider<String> + Send,
{
    type Output = P::Output;
    type Error = P::Error;

    fn use_provider(self, args: String) -> Signal<AsyncState<Self::Output, Self::Error>> {
        use_provider_core(self, args)
    }
}

/// Implementation for providers with a single i32 parameter (without tuple wrapper)
impl<P> UseProvider<i32> for P
where
    P: Provider<i32> + Send,
{
    type Output = P::Output;
    type Error = P::Error;

    fn use_provider(self, args: i32) -> Signal<AsyncState<Self::Output, Self::Error>> {
        use_provider_core(self, args)
    }
}

/// Shared cache expiration logic
fn check_and_handle_cache_expiration(
    cache_expiration: Option<Duration>,
    cache_key: &str,
    cache: &ProviderCache,
    refresh_registry: &RefreshRegistry,
) {
    if let Some(expiration) = cache_expiration {
        if let Ok(mut cache_lock) = cache.cache.lock() {
            if let Some(entry) = cache_lock.get(cache_key) {
                if entry.is_expired(expiration) {
                    debug!(
                        "🗑️ [CACHE EXPIRATION] Removing expired cache entry for key: {}",
                        cache_key
                    );
                    cache_lock.remove(cache_key);
                    // Trigger a refresh to re-execute the provider
                    refresh_registry.trigger_refresh(cache_key);
                }
            }
        }
    }
}

/// Unified hook for using any provider - automatically detects parameterized vs non-parameterized providers
///
/// This is the main hook for consuming providers in Dioxus components. It automatically
/// handles both simple providers (no parameters) and parameterized providers, providing
/// a consistent interface for all provider types.
///
/// ## Features
///
/// - **Automatic Caching**: Results are cached based on provider configuration
/// - **Reactive Updates**: Components automatically re-render when data changes
/// - **Loading States**: Provides loading, success, and error states
/// - **Background Refresh**: Supports interval refresh and stale-while-revalidate
/// - **Auto-Dispose**: Automatically cleans up unused providers
///
/// ## Usage
///
/// ```rust,no_run
/// use dioxus::prelude::*;
/// use dioxus_riverpod::prelude::*;
///
/// #[derive(Clone, PartialEq)]
/// struct DataProvider;
///
/// impl Provider<()> for DataProvider {
///     type Output = String;
///     type Error = String;
///
///     async fn run(&self, _: ()) -> Result<Self::Output, Self::Error> {
///         Ok("Hello World".to_string())
///     }
///
///     fn id(&self, _: &()) -> String {
///         "data".to_string()
///     }
/// }
///
/// #[component]
/// fn MyComponent() -> Element {
///     // Provider with no parameters
///     let data = use_provider(DataProvider, ());
///     
///     match *data.read() {
///         AsyncState::Loading => rsx! { div { "Loading..." } },
///         AsyncState::Success(ref value) => rsx! { div { "{value}" } },
///         AsyncState::Error(ref _err) => rsx! { div { "Error occurred" } },
///     }
/// }
/// ```
pub fn use_provider<P, Args>(provider: P, args: Args) -> Signal<AsyncState<P::Output, P::Error>>
where
    P: UseProvider<Args>,
{
    provider.use_provider(args)
}

/// Sets up intelligent cache management for a provider
///
/// This replaces the old component-unmount auto-dispose with a better system:
/// 1. Access-time tracking for LRU management
/// 2. Periodic cleanup of unused entries based on cache_expiration
/// 3. Cache size limits with LRU eviction
/// 4. Automatic background cleanup tasks
fn setup_intelligent_cache_management<P, Param>(
    provider: &P,
    cache_key: &str,
    cache: &ProviderCache,
    refresh_registry: &RefreshRegistry,
) where
    P: Provider<Param> + Clone,
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    // Set up periodic cleanup task for this provider if cache_expiration is configured
    if let Some(cache_expiration) = provider.cache_expiration() {
        let cleanup_interval = std::cmp::max(
            cache_expiration / 4, // Clean up 4x more frequently than expiration
            Duration::from_secs(30), // But at least every 30 seconds
        );
        
        let cache_clone = cache.clone();
        let unused_threshold = cache_expiration * 2; // Remove entries unused for 2x expiration time
        let cleanup_key = format!("{}_cleanup", cache_key);

        refresh_registry.start_periodic_task(
            &cleanup_key,
            TaskType::CacheCleanup,
            cleanup_interval,
            move || {
                // Remove entries that haven't been accessed recently
                let removed = cache_clone.cleanup_unused_entries(unused_threshold);
                if removed > 0 {
                    debug!("🧹 [SMART-CLEANUP] Removed {} unused cache entries", removed);
                }

                // Enforce cache size limits (configurable - could be made dynamic)
                const MAX_CACHE_SIZE: usize = 1000;
                let evicted = cache_clone.evict_lru_entries(MAX_CACHE_SIZE);
                if evicted > 0 {
                    debug!("🗑️ [LRU-EVICT] Evicted {} entries due to cache size limit", evicted);
                }
            },
        );

        debug!("📊 [SMART-CACHE] Intelligent cache management enabled for: {} (cleanup every {:?})", cache_key, cleanup_interval);
    }
}
