//! # Dioxus Riverpod Providers
//!
//! This module provides a reactive state management system for Dioxus applications,
//! inspired by Riverpod from the Flutter ecosystem.
//!
//! ## Core Features
//!
//! - **Future Providers**: Async operations that return data with automatic caching
//! - **Family Providers**: Parameterized providers for dynamic data fetching
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
//! // Family provider with parameters
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
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::task::JoinHandle;

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

            if should_create_new_task {
                let task = tokio::spawn(async move {
                    let mut interval_timer = tokio::time::interval(interval);
                    interval_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

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

/// A type-erased cache entry for storing provider results with timestamp
#[derive(Clone)]
struct CacheEntry {
    data: Arc<dyn Any + Send + Sync>,
    cached_at: std::time::Instant,
}

impl CacheEntry {
    fn new<T: Clone + Send + Sync + 'static>(data: T) -> Self {
        Self {
            data: Arc::new(data),
            cached_at: std::time::Instant::now(),
        }
    }

    fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.data.downcast_ref::<T>().cloned()
    }

    fn is_expired(&self, expiration: Duration) -> bool {
        self.cached_at.elapsed() > expiration
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
                println!(
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

/// A trait for defining future providers - async operations that return data
pub trait FutureProvider: Clone + PartialEq + 'static {
    type Output: Clone + PartialEq + Send + Sync + 'static;
    type Error: Clone + Send + Sync + 'static;

    /// Execute the async operation
    fn run(&self) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;

    /// Get a unique identifier for this provider (used for caching/invalidation)
    fn id(&self) -> String;

    /// Get the interval duration for automatic refresh (None means no interval)
    fn interval(&self) -> Option<Duration> {
        None
    }

    /// Get the cache expiration duration (None means no expiration)
    fn cache_expiration(&self) -> Option<Duration> {
        None
    }
}

/// A trait for defining family providers - parameterized async operations
pub trait FamilyProvider<Param>: Clone + PartialEq + 'static
where
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    type Output: Clone + PartialEq + Send + Sync + 'static;
    type Error: Clone + Send + Sync + 'static;

    /// Execute the async operation with the given parameter
    fn run(&self, param: Param) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;

    /// Get a unique identifier for this provider with the parameter
    fn id(&self, param: &Param) -> String;

    /// Get the interval duration for automatic refresh (None means no interval)
    fn interval(&self) -> Option<Duration> {
        None
    }

    /// Get the cache expiration duration (None means no expiration)
    fn cache_expiration(&self) -> Option<Duration> {
        None
    }
}

/// Hook to access the provider cache for manual cache management  
pub fn use_provider_cache() -> ProviderCache {
    use_context::<ProviderCache>()
}

/// Hook to invalidate a specific future provider cache entry
pub fn use_invalidate_provider<P: FutureProvider>(provider: P) -> impl Fn() + Clone {
    let cache = use_provider_cache();
    let refresh_registry = use_context::<RefreshRegistry>();
    let cache_key = provider.id();

    move || {
        cache.invalidate(&cache_key);
        refresh_registry.trigger_refresh(&cache_key);
    }
}

/// Hook to invalidate a specific family provider cache entry
pub fn use_invalidate_family_provider<P, Param>(provider: P, param: Param) -> impl Fn() + Clone
where
    P: FamilyProvider<Param>,
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

//
// ============================================================================
// Unified Provider Hook - Works with both Future and Family Providers
// ============================================================================

/// Trait for unified provider usage - automatically handles both future and family providers
pub trait UseProvider<Args> {
    type Output: Clone + PartialEq + Send + Sync + 'static;
    type Error: Clone + Send + Sync + 'static;

    fn use_provider(self, args: Args) -> Signal<AsyncState<Self::Output, Self::Error>>;
}

/// Implementation for future providers (no parameters)
impl<P> UseProvider<()> for P
where
    P: FutureProvider + Send,
{
    type Output = P::Output;
    type Error = P::Error;

    fn use_provider(self, _args: ()) -> Signal<AsyncState<Self::Output, Self::Error>> {
        let provider = self;
        let mut state = use_signal(|| AsyncState::Loading);
        let cache = use_context::<ProviderCache>();
        let refresh_registry = use_context::<RefreshRegistry>();

        // Check cache expiration before the memo - this happens on every render
        let cache_key = provider.id();
        let cache_expiration = provider.cache_expiration();

        // If cache expiration is enabled, check if current cache entry is expired and remove it
        if let Some(expiration) = cache_expiration {
            if let Ok(mut cache_lock) = cache.cache.lock() {
                if let Some(entry) = cache_lock.get(&cache_key) {
                    if entry.is_expired(expiration) {
                        println!(
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

        // Use memo with reactive dependencies to track changes automatically
        let _execution_memo = use_memo(use_reactive!(|provider| {
            let cache_key = provider.id();

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

                    tokio::spawn(async move {
                        let result = provider_for_task.run().await;
                        cache_for_task.set(cache_key_for_task.clone(), result);

                        // Trigger refresh to mark reactive contexts as dirty and update UI
                        refresh_registry_for_task.trigger_refresh(&cache_key_for_task);
                    });
                });
            }

            // Check cache first, with expiration if specified
            let cache_expiration = provider.cache_expiration();
            if let Some(cached_result) =
                cache.get_with_expiration::<Result<P::Output, P::Error>>(&cache_key, cache_expiration)
            {
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
            let mut state_for_async = state;

            spawn(async move {
                let result = provider.run().await;
                cache.set(cache_key, result.clone());

                match result {
                    Ok(data) => state_for_async.set(AsyncState::Success(data)),
                    Err(error) => state_for_async.set(AsyncState::Error(error)),
                }
            });
        }));

        state
    }
}

/// Implementation for family providers (with parameters)
impl<P, Param> UseProvider<(Param,)> for P
where
    P: FamilyProvider<Param> + Send,
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

        // Check cache expiration before the memo - this happens on every render
        let cache_key = provider.id(&param);
        let cache_expiration = provider.cache_expiration();

        // If cache expiration is enabled, check if current cache entry is expired and remove it
        if let Some(expiration) = cache_expiration {
            if let Ok(mut cache_lock) = cache.cache.lock() {
                if let Some(entry) = cache_lock.get(&cache_key) {
                    if entry.is_expired(expiration) {
                        println!(
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

                    tokio::spawn(async move {
                        let result = provider_for_task.run(param_for_task).await;
                        cache_for_task.set(cache_key_for_task.clone(), result);

                        // Trigger refresh to mark reactive contexts as dirty and update UI
                        refresh_registry_for_task.trigger_refresh(&cache_key_for_task);
                    });
                });
            }

            // Check cache first, with expiration if specified
            let cache_expiration = provider.cache_expiration();
            if let Some(cached_result) =
                cache.get_with_expiration::<Result<P::Output, P::Error>>(&cache_key, cache_expiration)
            {
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
            let param = param.clone();
            let mut state_for_async = state;

            spawn(async move {
                let result = provider.run(param).await;
                cache.set(cache_key, result.clone());

                match result {
                    Ok(data) => state_for_async.set(AsyncState::Success(data)),
                    Err(error) => state_for_async.set(AsyncState::Error(error)),
                }
            });
        }));

        state
    }
}

/// Unified hook for using any provider - automatically detects future vs family providers
///
/// ## Usage
///
/// ```rust,ignore
/// use dioxus::prelude::*;
/// use dioxus_riverpod::prelude::*;
/// 
/// // Future provider (no parameters)
/// let data = use_provider(fetch_data, ());
///
/// // Family provider (with parameters)  
/// let user_data = use_provider(fetch_user, (user_id,));
/// ```
pub fn use_provider<P, Args>(provider: P, args: Args) -> Signal<AsyncState<P::Output, P::Error>>
where
    P: UseProvider<Args>,
{
    provider.use_provider(args)
}


