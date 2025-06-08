use dioxus_lib::prelude::*;
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    fmt::Debug,
    future::Future,
    hash::Hash,
    sync::{Arc, Mutex},
};

/// Type alias for reactive context storage
type ReactiveContextSet = Arc<Mutex<HashSet<ReactiveContext>>>;
type ReactiveContextRegistry = Arc<Mutex<HashMap<String, ReactiveContextSet>>>;

/// Global registry for refresh signals that can trigger provider re-execution
#[derive(Clone, Default)]
pub struct RefreshRegistry {
    refresh_counters: Arc<Mutex<HashMap<String, u64>>>,
    reactive_contexts: ReactiveContextRegistry,
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
            let key_contexts = contexts.entry(key.to_string()).or_insert_with(|| Arc::new(Mutex::new(HashSet::new())));
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
}

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

/// A type-erased cache entry for storing provider results
#[derive(Clone)]
struct CacheEntry {
    data: Arc<dyn Any + Send + Sync>,
}

impl CacheEntry {
    fn new<T: Clone + Send + Sync + 'static>(data: T) -> Self {
        Self {
            data: Arc::new(data),
        }
    }

    fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.data.downcast_ref::<T>().cloned()
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

    /// Get a cached result by key
    pub fn get<T: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<T> {
        self.cache.lock().ok()?.get(key)?.get::<T>()
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
}

/// Hook for using a future provider in a Dioxus component
pub fn use_future_provider<P: FutureProvider>(
    provider: P,
) -> Signal<AsyncState<P::Output, P::Error>> {
    let mut state = use_signal(|| AsyncState::Loading);
    let cache = use_context::<ProviderCache>();
    let refresh_registry = use_context::<RefreshRegistry>();
    
    // Use memo with reactive dependencies to track changes automatically
    let _execution_memo = use_memo(use_reactive!(|provider| {
        let cache_key = provider.id();
        
        // Subscribe to refresh events for this cache key if we have a reactive context
        if let Some(reactive_context) = ReactiveContext::current() {
            refresh_registry.subscribe_to_refresh(&cache_key, reactive_context);
        }
        
        // Read the current refresh count (this makes the memo reactive to changes)
        let _current_refresh_count = refresh_registry.get_refresh_count(&cache_key);
        
        // Check cache first
        if let Some(cached_result) = cache.get::<Result<P::Output, P::Error>>(&cache_key) {
            match cached_result {
                Ok(data) => {
                    let _ = spawn(async move {
                        state.set(AsyncState::Success(data));
                    });
                },
                Err(error) => {
                    let _ = spawn(async move {
                        state.set(AsyncState::Error(error));
                    });
                },
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

/// Hook for using a family provider in a Dioxus component
pub fn use_family_provider<P, Param>(
    provider: P,
    param: Param,
) -> Signal<AsyncState<P::Output, P::Error>>
where
    P: FamilyProvider<Param>,
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    let mut state = use_signal(|| AsyncState::Loading);
    let cache = use_context::<ProviderCache>();
    let refresh_registry = use_context::<RefreshRegistry>();
    
    // Use memo with reactive dependencies to track changes automatically
    let _execution_memo = use_memo(use_reactive!(|provider, param| {
        let cache_key = provider.id(&param);
        
        // Subscribe to refresh events for this cache key if we have a reactive context
        if let Some(reactive_context) = ReactiveContext::current() {
            refresh_registry.subscribe_to_refresh(&cache_key, reactive_context);
        }
        
        // Read the current refresh count (this makes the memo reactive to changes)
        let _current_refresh_count = refresh_registry.get_refresh_count(&cache_key);
        
        // Check cache first
        if let Some(cached_result) = cache.get::<Result<P::Output, P::Error>>(&cache_key) {
            match cached_result {
                Ok(data) => {
                    let _ = spawn(async move {
                        state.set(AsyncState::Success(data));
                    });
                },
                Err(error) => {
                    let _ = spawn(async move {
                        state.set(AsyncState::Error(error));
                    });
                },
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

/// Invalidate a specific provider cache entry
pub fn invalidate_provider<P: FutureProvider>(cache: &ProviderCache, provider: P) {
    cache.invalidate(&provider.id());
}

/// Invalidate a specific family provider cache entry
pub fn invalidate_family_provider<P, Param>(cache: &ProviderCache, provider: P, param: &Param)
where
    P: FamilyProvider<Param>,
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    cache.invalidate(&provider.id(param));
}
