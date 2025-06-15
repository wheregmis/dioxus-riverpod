//! # Provider Hooks
//!
//! This module provides the main hooks and traits for working with providers in Dioxus.
//! It includes the core Provider trait, the unified hook system for both parameterized
//! and non-parameterized providers, and utility hooks for cache management.
//!
//! ## Key Features
//!
//! - **Provider Trait**: Unified interface for all provider types
//! - **Automatic Caching**: Built-in caching with configurable expiration
//! - **Stale-While-Revalidate (SWR)**: Background updates while serving stale data
//! - **Auto-Dispose**: Automatic cleanup of unused providers
//! - **Interval Refresh**: Automatic refresh at specified intervals
//! - **Cross-Platform**: Works on both web and desktop using dioxus tasks
//!
//! ## Usage
//!
//! ```rust,no_run
//! use dioxus::prelude::*;
//! use dioxus_riverpod::prelude::*;
//!
//! // Simple provider (no parameters)
//! let data = use_provider(fetch_data, ());
//!
//! // Parameterized provider
//! let user = use_provider(fetch_user, (user_id,));
//! ```

use dioxus_lib::prelude::*;
use std::{fmt::Debug, future::Future, hash::Hash, time::Duration};
use tracing::debug;

use crate::{
    cache::{AsyncState, ProviderCache},
    disposal::DisposalRegistry,
    refresh::RefreshRegistry,
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
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    /// The type of data returned on success
    type Output: Clone + PartialEq + Send + Sync + 'static;
    /// The type of error returned on failure
    type Error: Clone + Send + Sync + 'static;

    /// Execute the async operation
    ///
    /// This method performs the actual work of the provider, such as fetching data
    /// from an API, reading from a database, or computing a value.
    fn run(&self, param: Param) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;

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
    /// When enabled, the provider's cache entry will be automatically removed after
    /// a configurable delay when no components are using it.
    fn auto_dispose(&self) -> bool {
        false
    }

    /// Get the dispose delay duration - how long to wait before disposing after last usage
    ///
    /// Only relevant when auto_dispose() returns true. If None, uses the default delay.
    fn dispose_delay(&self) -> Option<Duration> {
        None
    }
}

/// Hook to access the provider cache for manual cache management
///
/// This provides direct access to the underlying cache, allowing for manual
/// invalidation, clearing, and other cache operations.
///
/// ## Example
///
/// ```rust,no_run
/// use dioxus::prelude::*;
/// use dioxus_riverpod::prelude::*;
///
/// fn MyComponent(cx: Scope) -> Element {
///     let cache = use_provider_cache();
///     
///     // Manually invalidate a specific cache entry
///     cache.invalidate("my_provider_key");
///     
///     cx.render(rsx! {
///         div { "Cache operations example" }
///     })
/// }
/// ```
pub fn use_provider_cache() -> ProviderCache {
    use_context::<ProviderCache>()
}

/// Hook to invalidate a specific provider cache entry
///
/// Returns a function that, when called, will invalidate the cache entry for the
/// specified provider and parameters, and trigger a refresh of all components
/// using that provider.
///
/// ## Example
///
/// ```rust,no_run
/// use dioxus::prelude::*;
/// use dioxus_riverpod::prelude::*;
///
/// fn MyComponent(cx: Scope) -> Element {
///     let invalidate_user = use_invalidate_provider(fetch_user_provider, user_id);
///     
///     cx.render(rsx! {
///         button {
///             onclick: move |_| invalidate_user(),
///             "Refresh User Data"
///         }
///     })
/// }
/// ```
pub fn use_invalidate_provider<P, Param>(provider: P, param: Param) -> impl Fn() + Clone
where
    P: Provider<Param>,
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    let cache = use_provider_cache();
    let refresh_registry = use_context::<RefreshRegistry>();
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
/// ## Example
///
/// ```rust,no_run
/// use dioxus::prelude::*;
/// use dioxus_riverpod::prelude::*;
///
/// fn MyComponent(cx: Scope) -> Element {
///     let clear_cache = use_clear_provider_cache();
///     
///     cx.render(rsx! {
///         button {
///             onclick: move |_| clear_cache(),
///             "Clear All Cache"
///         }
///     })
/// }
/// ```
pub fn use_clear_provider_cache() -> impl Fn() + Clone {
    let cache = use_provider_cache();
    let refresh_registry = use_context::<RefreshRegistry>();

    move || {
        cache.clear();
        refresh_registry.clear_all();
    }
}

/// Hook to access the disposal registry for auto-dispose management
///
/// This provides access to the disposal registry, allowing for manual control
/// over the auto-dispose functionality.
pub fn use_disposal_registry() -> DisposalRegistry {
    use_context::<DisposalRegistry>()
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

/// Implementation for providers with no parameters (simple providers)
impl<P> UseProvider<()> for P
where
    P: Provider<()> + Send,
{
    type Output = P::Output;
    type Error = P::Error;

    fn use_provider(self, _args: ()) -> Signal<AsyncState<Self::Output, Self::Error>> {
        let provider = self;
        let mut state = use_signal(|| AsyncState::Loading);
        let cache = use_context::<ProviderCache>();
        let refresh_registry = use_context::<RefreshRegistry>();

        // Auto-dispose functionality
        let disposal_registry = if provider.auto_dispose() {
            Some(use_context::<DisposalRegistry>())
        } else {
            None
        };

        // Check cache expiration before the memo - this happens on every render
        let cache_key = provider.id(&());
        let cache_expiration = provider.cache_expiration();

        // Store cache key and disposal registry for cleanup
        let cache_key_for_cleanup = cache_key.clone();
        let provider_for_cleanup = provider.clone();
        let disposal_registry_for_cleanup = disposal_registry.clone();
        let cache_for_cleanup = cache.clone();

        // Component unmount cleanup - remove reference and schedule disposal
        use_drop(move || {
            if let Some(disposal_reg) = &disposal_registry_for_cleanup {
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
                disposal_reg.schedule_disposal(cache_key_for_cleanup.clone(), dispose_delay);
            }
        });

        // If cache expiration is enabled, check if current cache entry is expired and remove it
        if let Some(expiration) = cache_expiration {
            if let Ok(mut cache_lock) = cache.cache.lock() {
                if let Some(entry) = cache_lock.get(&cache_key) {
                    if entry.is_expired(expiration) {
                        debug!(
                            "🗑️ [CACHE EXPIRATION] Removing expired cache entry for key: {}",
                            cache_key
                        );
                        cache_lock.remove(&cache_key);
                        // Trigger a refresh to re-execute the provider
                        refresh_registry.trigger_refresh(&cache_key);
                    }
                }
            }
        }

        // SWR staleness checking - runs on every render to check for stale data
        let stale_time = provider.stale_time();
        if let Some(stale_duration) = stale_time {
            if let Ok(cache_lock) = cache.cache.lock() {
                if let Some(entry) = cache_lock.get(&cache_key) {
                    if entry.is_stale(stale_duration)
                        && !entry.is_expired(cache_expiration.unwrap_or(Duration::from_secs(3600)))
                        && !refresh_registry.is_revalidation_in_progress(&cache_key)
                    {
                        // Data is stale but not expired and no revalidation in progress - trigger background revalidation
                        if refresh_registry.start_revalidation(&cache_key) {
                            debug!(
                                "🔄 [SWR] Data is stale for key: {} - triggering background revalidation",
                                cache_key
                            );

                            let cache = cache.clone();
                            let cache_key_clone = cache_key.clone();
                            let provider = provider.clone();
                            let refresh_registry_clone = refresh_registry.clone();

                            spawn(async move {
                                let result = provider.run(()).await;
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

        // Use memo with reactive dependencies to track changes automatically
        let _execution_memo = use_memo(use_reactive!(|provider| {
            let cache_key = provider.id(&());
            let cache_expiration = provider.cache_expiration();

            // Subscribe to refresh events for this cache key if we have a reactive context
            if let Some(reactive_context) = ReactiveContext::current() {
                refresh_registry.subscribe_to_refresh(&cache_key, reactive_context);
            }

            // Read the current refresh count (this makes the memo reactive to changes)
            let _current_refresh_count = refresh_registry.get_refresh_count(&cache_key);

            // MOVED: Cache expiration check inside reactive memo - this runs on every reactive update
            check_and_handle_cache_expiration(
                cache_expiration,
                &cache_key,
                &cache,
                &refresh_registry,
            );

            // Set up interval task if provider has interval configured
            if let Some(interval) = provider.interval() {
                let cache_clone = cache.clone();
                let provider_clone = provider.clone();
                let cache_key_clone = cache_key.clone();
                let refresh_registry_clone = refresh_registry.clone();

                refresh_registry.start_interval_task(&cache_key, interval, move || {
                    // Re-execute the provider and update cache in background
                    let cache_for_task = cache_clone.clone();
                    let provider_for_task = provider_clone.clone();
                    let cache_key_for_task = cache_key_clone.clone();
                    let refresh_registry_for_task = refresh_registry_clone.clone();

                    spawn(async move {
                        let result = provider_for_task.run(()).await;
                        cache_for_task.set(cache_key_for_task.clone(), result);

                        // Trigger refresh to mark reactive contexts as dirty and update UI
                        refresh_registry_for_task.trigger_refresh(&cache_key_for_task);
                    });
                });
            }

            // Check cache first - serve any available data (fresh or stale)
            let _cache_expiration = provider.cache_expiration();
            if let Some(cached_result) = cache.get::<Result<P::Output, P::Error>>(&cache_key) {
                // Add reference count and cancel disposal if auto-dispose is enabled
                if let Some(ref disposal_reg) = disposal_registry {
                    disposal_reg.cancel_disposal(&cache_key);

                    // Add reference count for this component using the cache entry
                    if let Ok(cache_lock) = cache.cache.lock() {
                        if let Some(entry) = cache_lock.get(&cache_key) {
                            entry.add_reference();
                            debug!(
                                "🔄 [AUTO-DISPOSE] Added reference for: {} (refs: {})",
                                cache_key,
                                entry.reference_count()
                            );
                        }
                    }
                }

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
            let cache_key = cache_key.clone();
            let provider = provider.clone();
            let disposal_registry_for_async = disposal_registry.clone();
            let mut state_for_async = state;

            spawn(async move {
                let result = provider.run(()).await;
                cache.set(cache_key.clone(), result.clone());

                // Add reference count for auto-dispose after cache entry is created
                if let Some(_disposal_reg) = disposal_registry_for_async {
                    if let Ok(cache_lock) = cache.cache.lock() {
                        if let Some(entry) = cache_lock.get(&cache_key) {
                            entry.add_reference();
                            debug!(
                                "🔄 [AUTO-DISPOSE] Added reference for new entry: {} (refs: {})",
                                cache_key,
                                entry.reference_count()
                            );
                        }
                    }
                }

                match result {
                    Ok(data) => state_for_async.set(AsyncState::Success(data)),
                    Err(error) => state_for_async.set(AsyncState::Error(error)),
                }
            });
        }));

        state
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
        let provider = self;
        let param = args.0;
        let mut state = use_signal(|| AsyncState::Loading);
        let cache = use_context::<ProviderCache>();
        let refresh_registry = use_context::<RefreshRegistry>();

        // Auto-dispose functionality
        let disposal_registry = if provider.auto_dispose() {
            Some(use_context::<DisposalRegistry>())
        } else {
            None
        };

        // Check cache expiration before the memo - this happens on every render
        let cache_key = provider.id(&param);
        let cache_expiration = provider.cache_expiration();

        // Store cache key and disposal registry for cleanup
        let cache_key_for_cleanup = cache_key.clone();
        let provider_for_cleanup = provider.clone();
        let disposal_registry_for_cleanup = disposal_registry.clone();
        let cache_for_cleanup = cache.clone();

        // Component unmount cleanup - remove reference and schedule disposal
        use_drop(move || {
            if let Some(disposal_reg) = &disposal_registry_for_cleanup {
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
                disposal_reg.schedule_disposal(cache_key_for_cleanup.clone(), dispose_delay);
            }
        });

        // If cache expiration is enabled, check if current cache entry is expired and remove it
        if let Some(expiration) = cache_expiration {
            if let Ok(mut cache_lock) = cache.cache.lock() {
                if let Some(entry) = cache_lock.get(&cache_key) {
                    if entry.is_expired(expiration) {
                        debug!(
                            "🗑️ [CACHE EXPIRATION] Removing expired cache entry for key: {}",
                            cache_key
                        );
                        cache_lock.remove(&cache_key);
                        // Trigger a refresh to re-execute the provider
                        refresh_registry.trigger_refresh(&cache_key);
                    }
                }
            }
        }

        // SWR staleness checking - runs on every render to check for stale data
        let stale_time = provider.stale_time();
        if let Some(stale_duration) = stale_time {
            if let Ok(cache_lock) = cache.cache.lock() {
                if let Some(entry) = cache_lock.get(&cache_key) {
                    if entry.is_stale(stale_duration)
                        && !entry.is_expired(cache_expiration.unwrap_or(Duration::from_secs(3600)))
                        && !refresh_registry.is_revalidation_in_progress(&cache_key)
                    {
                        // Data is stale but not expired and no revalidation in progress - trigger background revalidation
                        if refresh_registry.start_revalidation(&cache_key) {
                            debug!(
                                "🔄 [SWR] Data is stale for key: {} - triggering background revalidation",
                                cache_key
                            );

                            let cache = cache.clone();
                            let cache_key_clone = cache_key.clone();
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

        // Use memo with reactive dependencies to track changes automatically
        let _execution_memo = use_memo(use_reactive!(|provider, param| {
            let cache_key = provider.id(&param);
            let cache_expiration = provider.cache_expiration();

            // Subscribe to refresh events for this cache key if we have a reactive context
            if let Some(reactive_context) = ReactiveContext::current() {
                refresh_registry.subscribe_to_refresh(&cache_key, reactive_context);
            }

            // Read the current refresh count (this makes the memo reactive to changes)
            let _current_refresh_count = refresh_registry.get_refresh_count(&cache_key);

            // MOVED: Cache expiration check inside reactive memo - this runs on every reactive update
            check_and_handle_cache_expiration(
                cache_expiration,
                &cache_key,
                &cache,
                &refresh_registry,
            );

            // Set up interval task if provider has interval configured
            if let Some(interval) = provider.interval() {
                let cache_clone = cache.clone();
                let provider_clone = provider.clone();
                let param_clone = param.clone();
                let cache_key_clone = cache_key.clone();
                let refresh_registry_clone = refresh_registry.clone();

                refresh_registry.start_interval_task(&cache_key, interval, move || {
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

            // Check cache first, with SWR support
            let cache_expiration = provider.cache_expiration();
            let stale_time = provider.stale_time();

            if let Some((cached_result, is_stale)) = cache
                .get_with_staleness::<Result<P::Output, P::Error>>(
                    &cache_key,
                    stale_time,
                    cache_expiration,
                )
            {
                // Add reference count and cancel disposal if auto-dispose is enabled
                if let Some(ref disposal_reg) = disposal_registry {
                    disposal_reg.cancel_disposal(&cache_key);

                    // Add reference count for this component using the cache entry
                    if let Ok(cache_lock) = cache.cache.lock() {
                        if let Some(entry) = cache_lock.get(&cache_key) {
                            entry.add_reference();
                            debug!(
                                "🔄 [AUTO-DISPOSE] Added reference for: {} (refs: {})",
                                cache_key,
                                entry.reference_count()
                            );
                        }
                    }
                }

                // Serve cached data immediately
                match cached_result.clone() {
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

                // If data is stale, trigger background revalidation
                if is_stale
                    && !refresh_registry.is_revalidation_in_progress(&cache_key)
                    && refresh_registry.start_revalidation(&cache_key)
                {
                    debug!(
                        "🔄 [SWR] Data is stale for key: {} - triggering background revalidation",
                        cache_key
                    );
                    let cache = cache.clone();
                    let cache_key_clone = cache_key.clone();
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

                return;
            }

            // Cache miss - set loading and spawn async task
            let _ = spawn(async move {
                state.set(AsyncState::Loading);
            });

            let cache = cache.clone();
            let cache_key = cache_key.clone();
            let provider = provider.clone();
            let param = param.clone();
            let disposal_registry_for_async = disposal_registry.clone();
            let mut state_for_async = state;

            spawn(async move {
                let result = provider.run(param).await;
                cache.set(cache_key.clone(), result.clone());

                // Add reference count for auto-dispose after cache entry is created
                if let Some(_disposal_reg) = disposal_registry_for_async {
                    if let Ok(cache_lock) = cache.cache.lock() {
                        if let Some(entry) = cache_lock.get(&cache_key) {
                            entry.add_reference();
                            debug!(
                                "🔄 [AUTO-DISPOSE] Added reference for new entry: {} (refs: {})",
                                cache_key,
                                entry.reference_count()
                            );
                        }
                    }
                }

                match result {
                    Ok(data) => state_for_async.set(AsyncState::Success(data)),
                    Err(error) => state_for_async.set(AsyncState::Error(error)),
                }
            });
        }));

        state
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
/// fn MyComponent() -> Element {
///     // Provider with no parameters
///     let data = use_provider(fetch_data, ());
///     
///     // Provider with parameters
///     let user_data = use_provider(fetch_user, (user_id,));
///     
///     match *data.read() {
///         AsyncState::Loading => rsx! { div { "Loading..." } },
///         AsyncState::Success(ref value) => rsx! { div { "{value}" } },
///         AsyncState::Error(ref err) => rsx! { div { "Error: {err}" } },
///     }
/// }
/// ```
pub fn use_provider<P, Args>(provider: P, args: Args) -> Signal<AsyncState<P::Output, P::Error>>
where
    P: UseProvider<Args>,
{
    provider.use_provider(args)
}
