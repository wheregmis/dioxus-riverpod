//! # Dioxus Riverpod Providers
//!
//! This module provides a reactive state management system for Dioxus applications,
//! inspired by Riverpod from the Flutter ecosystem.
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

use dioxus_lib::prelude::*;
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    fmt::Debug,
    future::Future,
    hash::Hash,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};
use tokio::task::JoinHandle;
use dioxus_lib::prelude::Task;
use tracing::debug;

#[cfg(not(target_family = "wasm"))]
use std::time::Instant;
#[cfg(target_family = "wasm")]
use web_time::Instant;

#[cfg(not(target_family = "wasm"))]
use tokio::time;
#[cfg(target_family = "wasm")]
use wasmtimer::tokio as time;

//
// ============================================================================
// Type Aliases and Core Types
// ============================================================================

/// Type alias for reactive context storage
type ReactiveContextSet = Arc<Mutex<HashSet<ReactiveContext>>>;
type ReactiveContextRegistry = Arc<Mutex<HashMap<String, ReactiveContextSet>>>;
type IntervalTaskRegistry = Arc<Mutex<HashMap<String, (Duration, JoinHandle<()>)>>>;

//
// ============================================================================
// Refresh Registry - Manages reactive updates and interval tasks
// ============================================================================

/// Global registry for refresh signals that can trigger provider re-execution
#[derive(Clone, Default)]
pub struct RefreshRegistry {
    refresh_counters: Arc<Mutex<HashMap<String, u64>>>,
    reactive_contexts: ReactiveContextRegistry,
    interval_tasks: IntervalTaskRegistry,
    ongoing_revalidations: Arc<Mutex<HashSet<String>>>,
}

impl RefreshRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_refresh_count(&self, key: &str) -> u64 {
        if let Ok(counters) = self.refresh_counters.lock() {
            *counters.get(key).unwrap_or(&0)
        } else {
            0
        }
    }

    pub fn subscribe_to_refresh(&self, key: &str, reactive_context: ReactiveContext) {
        if let Ok(mut contexts) = self.reactive_contexts.lock() {
            let key_contexts = contexts
                .entry(key.to_string())
                .or_insert_with(|| Arc::new(Mutex::new(HashSet::new())));
            if let Ok(mut context_set) = key_contexts.lock() {
                context_set.insert(reactive_context);
            }
        }
    }

    pub fn trigger_refresh(&self, key: &str) {
        // Increment the counter
        if let Ok(mut counters) = self.refresh_counters.lock() {
            let counter = counters.entry(key.to_string()).or_insert(0);
            *counter += 1;
        }

        // Mark all reactive contexts as dirty
        if let Ok(contexts) = self.reactive_contexts.lock() {
            if let Some(key_contexts) = contexts.get(key) {
                if let Ok(context_set) = key_contexts.lock() {
                    for reactive_context in context_set.iter() {
                        reactive_context.mark_dirty();
                    }
                }
            }
        }
    }

    pub fn clear_all(&self) {
        if let Ok(counters) = self.refresh_counters.lock() {
            let keys: Vec<String> = counters.keys().cloned().collect();
            drop(counters);

            for key in keys {
                self.trigger_refresh(&key);
            }
        }
    }

    /// Start an interval task for automatic provider refresh
    pub fn start_interval_task<F>(&self, key: &str, interval: Duration, refresh_fn: F)
    where
        F: Fn() + Send + 'static,
    {
        if let Ok(mut tasks) = self.interval_tasks.lock() {
            // Cancel existing task if it exists and the new interval is shorter
            let should_create_new_task = match tasks.get(key) {
                None => true,
                Some((current_interval, current_task)) => {
                    if interval < *current_interval {
                        current_task.abort();
                        tasks.remove(key);
                        true
                    } else {
                        false // Keep existing shorter interval
                    }
                }
            };

            // WASM compatibility: disable background interval tasks in WASM
            #[cfg(not(target_family = "wasm"))]
            if should_create_new_task {
                let task = tokio::spawn(async move {
                    let mut interval_timer = time::interval(interval);
                    interval_timer.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

                    // Skip the first tick (immediate execution)
                    interval_timer.tick().await;

                    loop {
                        interval_timer.tick().await;
                        refresh_fn();
                    }
                });

                tasks.insert(key.to_string(), (interval, task));
            }
        }
    }

    /// Stop an interval task for a specific provider
    pub fn stop_interval_task(&self, key: &str) {
        if let Ok(mut tasks) = self.interval_tasks.lock() {
            if let Some((_, task)) = tasks.remove(key) {
                task.abort();
            }
        }
    }

    /// Check if a revalidation is already in progress for a given key
    pub fn is_revalidation_in_progress(&self, key: &str) -> bool {
        if let Ok(revalidations) = self.ongoing_revalidations.lock() {
            revalidations.contains(key)
        } else {
            false
        }
    }

    /// Mark a revalidation as started for a given key
    pub fn start_revalidation(&self, key: &str) -> bool {
        if let Ok(mut revalidations) = self.ongoing_revalidations.lock() {
            if revalidations.contains(key) {
                false // Already in progress
            } else {
                revalidations.insert(key.to_string());
                true // Successfully started
            }
        } else {
            false
        }
    }

    /// Mark a revalidation as completed for a given key
    pub fn complete_revalidation(&self, key: &str) {
        if let Ok(mut revalidations) = self.ongoing_revalidations.lock() {
            revalidations.remove(key);
        }
    }
}

//
// ============================================================================
// Async State Management
// ============================================================================

/// Represents the state of an async operation
#[derive(Clone, PartialEq)]
pub enum AsyncState<T, E> {
    /// The operation is currently loading
    Loading,
    /// The operation completed successfully with data
    Success(T),
    /// The operation failed with an error
    Error(E),
}

impl<T, E> AsyncState<T, E> {
    /// Returns true if the state is currently loading
    pub fn is_loading(&self) -> bool {
        matches!(self, AsyncState::Loading)
    }

    /// Returns true if the state contains successful data
    pub fn is_success(&self) -> bool {
        matches!(self, AsyncState::Success(_))
    }

    /// Returns true if the state contains an error
    pub fn is_error(&self) -> bool {
        matches!(self, AsyncState::Error(_))
    }

    /// Returns the data if successful, None otherwise
    pub fn data(&self) -> Option<&T> {
        match self {
            AsyncState::Success(data) => Some(data),
            _ => None,
        }
    }

    /// Returns the error if failed, None otherwise
    pub fn error(&self) -> Option<&E> {
        match self {
            AsyncState::Error(error) => Some(error),
            _ => None,
        }
    }
}

/// A type-erased cache entry for storing provider results with timestamp and reference counting
#[derive(Clone)]
struct CacheEntry {
    data: Arc<dyn Any + Send + Sync>,
    cached_at: Instant,
    reference_count: Arc<std::sync::atomic::AtomicU32>,
    last_accessed: Arc<Mutex<Instant>>,
}

impl CacheEntry {
    fn new<T: Clone + Send + Sync + 'static>(data: T) -> Self {
        let now = Instant::now();
        Self {
            data: Arc::new(data),
            cached_at: now,
            reference_count: Arc::new(AtomicU32::new(0)),
            last_accessed: Arc::new(Mutex::new(now)),
        }
    }

    fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        // Update last accessed time
        if let Ok(mut last_accessed) = self.last_accessed.lock() {
            *last_accessed = Instant::now();
        }
        self.data.downcast_ref::<T>().cloned()
    }

    fn is_expired(&self, expiration: Duration) -> bool {
        self.cached_at.elapsed() > expiration
    }

    fn is_stale(&self, stale_time: Duration) -> bool {
        self.cached_at.elapsed() > stale_time
    }

    /// Increment reference count when a provider hook starts using this entry
    fn add_reference(&self) {
        self.reference_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement reference count when a provider hook stops using this entry
    fn remove_reference(&self) {
        self.reference_count.fetch_sub(1, Ordering::SeqCst);
    }

    /// Get current reference count
    fn reference_count(&self) -> u32 {
        self.reference_count.load(Ordering::SeqCst)
    }
}

/// Global cache for provider results with automatic cleanup
#[derive(Clone, Default)]
pub struct ProviderCache {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl ProviderCache {
    /// Create a new provider cache
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a cached result by key, checking for expiration
    pub fn get<T: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<T> {
        self.cache.lock().ok()?.get(key)?.get::<T>()
    }

    /// Get a cached result by key, checking for expiration with a specific expiration duration
    pub fn get_with_expiration<T: Clone + Send + Sync + 'static>(
        &self,
        key: &str,
        expiration: Option<Duration>,
    ) -> Option<T> {
        // First, check if the entry exists and is expired
        let is_expired = {
            let cache_guard = self.cache.lock().ok()?;
            let entry = cache_guard.get(key)?;

            if let Some(exp_duration) = expiration {
                entry.is_expired(exp_duration)
            } else {
                false
            }
        };

        // If expired, remove the entry
        if is_expired {
            if let Ok(mut cache) = self.cache.lock() {
                cache.remove(key);
                debug!(
                    "🗑️ [CACHE EXPIRATION] Removing expired cache entry for key: {}",
                    key
                );
            }
            return None;
        }

        // Entry is not expired, return the data
        let cache_guard = self.cache.lock().ok()?;
        let entry = cache_guard.get(key)?;
        entry.get::<T>()
    }

    /// Get cached data with staleness information for SWR behavior
    pub fn get_with_staleness<T: Clone + Send + Sync + 'static>(
        &self,
        key: &str,
        stale_time: Option<Duration>,
        expiration: Option<Duration>,
    ) -> Option<(T, bool)> {
        let cache_guard = self.cache.lock().ok()?;
        let entry = cache_guard.get(key)?;

        // Check if expired first
        if let Some(exp_duration) = expiration {
            if entry.is_expired(exp_duration) {
                debug!(
                    "🗑️ [SWR DEBUG] Cache entry for key '{}' is expired after {:?}",
                    key, exp_duration
                );
                return None; // Expired, return None to trigger fresh fetch
            }
        }

        // Get the data
        let data = entry.get::<T>()?;

        // Check if stale
        let is_stale = if let Some(stale_duration) = stale_time {
            let is_stale = entry.is_stale(stale_duration);
            let elapsed = entry.cached_at.elapsed();
            debug!(
                "🔍 [SWR DEBUG] Cache entry for key '{}': age={:?}, stale_time={:?}, is_stale={}",
                key, elapsed, stale_duration, is_stale
            );
            is_stale
        } else {
            debug!("🔍 [SWR DEBUG] No stale_time configured for key '{}'", key);
            false
        };

        Some((data, is_stale))
    }

    /// Store a result in the cache
    pub fn set<T: Clone + Send + Sync + 'static>(&self, key: String, value: T) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(key, CacheEntry::new(value));
        }
    }

    /// Remove a specific cache entry
    pub fn invalidate(&self, key: &str) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(key);
        }
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
}

/// A unified trait for defining providers - async operations that return data
///
/// This trait supports both simple providers (no parameters) and parameterized providers.
/// Use `Provider<()>` for simple providers and `Provider<ParamType>` for parameterized providers.
pub trait Provider<Param = ()>: Clone + PartialEq + 'static
where
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    type Output: Clone + PartialEq + Send + Sync + 'static;
    type Error: Clone + Send + Sync + 'static;

    /// Execute the async operation
    fn run(&self, param: Param) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;

    /// Get a unique identifier for this provider (used for caching/invalidation)
    fn id(&self, param: &Param) -> String;

    /// Get the interval duration for automatic refresh (None means no interval)
    fn interval(&self) -> Option<Duration> {
        None
    }

    /// Get the cache expiration duration (None means no expiration)
    fn cache_expiration(&self) -> Option<Duration> {
        None
    }

    /// Get the stale time duration for stale-while-revalidate behavior (None means no SWR)
    fn stale_time(&self) -> Option<Duration> {
        None
    }

    /// Get whether this provider should auto-dispose when unused (false by default)
    fn auto_dispose(&self) -> bool {
        false
    }

    /// Get the dispose delay duration - how long to wait before disposing after last usage (None means default delay)
    fn dispose_delay(&self) -> Option<Duration> {
        None
    }
}

/// Hook to access the provider cache for manual cache management  
pub fn use_provider_cache() -> ProviderCache {
    use_context::<ProviderCache>()
}

/// Hook to invalidate a specific provider cache entry
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
pub fn use_clear_provider_cache() -> impl Fn() + Clone {
    let cache = use_provider_cache();
    let refresh_registry = use_context::<RefreshRegistry>();

    move || {
        cache.clear();
        refresh_registry.clear_all();
    }
}

/// Hook to access the disposal registry for auto-dispose management
pub fn use_disposal_registry() -> DisposalRegistry {
    use_context::<DisposalRegistry>()
}

//
// ============================================================================
// Unified Provider Hook - Works with all Provider types
// ============================================================================

/// Trait for unified provider usage - automatically handles providers with and without parameters
pub trait UseProvider<Args> {
    type Output: Clone + PartialEq + Send + Sync + 'static;
    type Error: Clone + Send + Sync + 'static;

    fn use_provider(self, args: Args) -> Signal<AsyncState<Self::Output, Self::Error>>;
}

/// Implementation for providers with no parameters (future providers)
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

            // Subscribe to refresh events for this cache key if we have a reactive context
            if let Some(reactive_context) = ReactiveContext::current() {
                refresh_registry.subscribe_to_refresh(&cache_key, reactive_context);
            }

            // Read the current refresh count (this makes the memo reactive to changes)
            let _current_refresh_count = refresh_registry.get_refresh_count(&cache_key);

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

/// Implementation for providers with parameters (family providers)
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

            // Subscribe to refresh events for this cache key if we have a reactive context
            if let Some(reactive_context) = ReactiveContext::current() {
                refresh_registry.subscribe_to_refresh(&cache_key, reactive_context);
            }

            // Read the current refresh count (this makes the memo reactive to changes)
            let _current_refresh_count = refresh_registry.get_refresh_count(&cache_key);

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

/// Unified hook for using any provider - automatically detects parameterized vs non-parameterized providers
///
/// ## Usage
///
/// ```rust,ignore
/// use dioxus::prelude::*;
/// use dioxus_riverpod::prelude::*;
///
/// // Provider with no parameters
/// let data = use_provider(fetch_data, ());
///
/// // Provider with parameters  
/// let user_data = use_provider(fetch_user, (user_id,));
/// ```
pub fn use_provider<P, Args>(provider: P, args: Args) -> Signal<AsyncState<P::Output, P::Error>>
where
    P: UseProvider<Args>,
{
    provider.use_provider(args)
}

//
// ============================================================================
// Disposal Registry - Manages auto-dispose functionality
// ============================================================================

/// Registry for managing provider disposal timers and cleanup
#[derive(Clone, Default)]
pub struct DisposalRegistry {
    disposal_timers: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    cache: Option<ProviderCache>,
}

impl DisposalRegistry {
    pub fn new(cache: ProviderCache) -> Self {
        Self {
            disposal_timers: Arc::new(Mutex::new(HashMap::new())),
            cache: Some(cache),
        }
    }

    /// Schedule disposal of a provider after the specified delay
    pub fn schedule_disposal(&self, cache_key: String, dispose_delay: Duration) {
        // WASM compatibility: disable auto-disposal in WASM
        #[cfg(not(target_family = "wasm"))]
        if let (Ok(mut timers), Some(cache)) = (self.disposal_timers.lock(), &self.cache) {
            // Cancel existing timer if present
            if let Some(existing_timer) = timers.remove(&cache_key) {
                existing_timer.abort();
            }

            let cache_clone = cache.clone();
            let cache_key_clone = cache_key.clone();

            let timer = tokio::spawn(async move {
                time::sleep(dispose_delay).await;

                // Check if the provider can still be disposed
                // After waiting dispose_delay, if entry exists and has no references, dispose it
                if let Ok(cache_guard) = cache_clone.cache.lock() {
                    if let Some(entry) = cache_guard.get(&cache_key_clone) {
                        // Only check reference count since we already waited the appropriate delay
                        if entry.reference_count() == 0 {
                            drop(cache_guard);
                            cache_clone.invalidate(&cache_key_clone);
                            debug!("🗑️ [AUTO-DISPOSE] Disposed provider: {}", cache_key_clone);
                        } else {
                            debug!(
                                "🔄 [AUTO-DISPOSE] Disposal skipped (provider in use): {}",
                                cache_key_clone
                            );
                        }
                    }
                }
            });

            timers.insert(cache_key, timer);
        }
        
        // In WASM, auto-disposal is disabled for compatibility
        #[cfg(target_family = "wasm")]
        {
            let _ = (cache_key, dispose_delay); // Silence unused variable warnings
        }
    }

    /// Cancel disposal timer for a provider (called when provider is accessed again)
    pub fn cancel_disposal(&self, cache_key: &str) {
        #[cfg(not(target_family = "wasm"))]
        if let Ok(mut timers) = self.disposal_timers.lock() {
            if let Some(timer) = timers.remove(cache_key) {
                timer.abort();
                debug!("🔄 [AUTO-DISPOSE] Cancelled disposal for: {}", cache_key);
            }
        }
        
        // In WASM, auto-disposal is disabled for compatibility
        #[cfg(target_family = "wasm")]
        {
            let _ = cache_key; // Silence unused variable warning
        }
    }

    /// Get the default disposal delay (30 seconds)
    pub fn default_dispose_delay() -> Duration {
        Duration::from_secs(30)
    }
}
