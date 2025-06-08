use dioxus_lib::prelude::*;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

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

/// A type-erased cache entry that can hold any cloneable data
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

/// Global cache for provider results
#[derive(Clone, Default)]
pub struct ProviderCache {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl ProviderCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get<T: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<T> {
        let cache = self.cache.lock().ok()?;
        cache.get(key)?.get::<T>()
    }

    pub fn set<T: Clone + Send + Sync + 'static>(&self, key: String, value: T) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(key, CacheEntry::new(value));
        }
    }

    pub fn invalidate(&self, key: &str) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(key);
        }
    }

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
    let state = use_signal(|| AsyncState::Loading);

    // Get or create the provider cache using use_context_provider
    let cache = use_context_provider(ProviderCache::new);
    let cache_key = provider.id();

    // Use hook to run only once per component instance
    use_hook(move || {
        let cache = cache.clone();
        let cache_key = cache_key.clone();
        let provider = provider.clone();
        let mut state = state;

        // Check cache first
        if let Some(cached_result) = cache.get::<Result<P::Output, P::Error>>(&cache_key) {
            match cached_result {
                Ok(data) => state.set(AsyncState::Success(data)),
                Err(error) => state.set(AsyncState::Error(error)),
            }
        } else {
            // Set loading state
            state.set(AsyncState::Loading);

            // Spawn async task to fetch data
            spawn(async move {
                let result = provider.run().await;

                // Cache the result
                cache.set(cache_key.clone(), result.clone());

                // Update the state
                match result {
                    Ok(data) => state.set(AsyncState::Success(data)),
                    Err(error) => state.set(AsyncState::Error(error)),
                }
            });
        }
    });

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
    let state = use_signal(|| AsyncState::Loading);

    // Get or create the provider cache
    let cache = use_context_provider(ProviderCache::new);
    let cache_key = provider.id(&param);

    // Use hook to run only once per component instance
    use_hook(move || {
        let cache = cache.clone();
        let cache_key = cache_key.clone();
        let provider = provider.clone();
        let param = param.clone();
        let mut state = state;

        // Check cache first
        if let Some(cached_result) = cache.get::<Result<P::Output, P::Error>>(&cache_key) {
            match cached_result {
                Ok(data) => state.set(AsyncState::Success(data)),
                Err(error) => state.set(AsyncState::Error(error)),
            }
        } else {
            // Set loading state
            state.set(AsyncState::Loading);

            // Spawn async task to fetch data
            spawn(async move {
                let result = provider.run(param).await;

                // Cache the result
                cache.set(cache_key.clone(), result.clone());

                // Update the state
                match result {
                    Ok(data) => state.set(AsyncState::Success(data)),
                    Err(error) => state.set(AsyncState::Error(error)),
                }
            });
        }
    });

    state
}

/// Hook for using a future provider with suspense support
pub fn use_future_provider_suspense<P: FutureProvider>(provider: P) -> Result<P::Output, P::Error> {
    let state = use_future_provider(provider);

    let current_state = state.read();
    match &*current_state {
        AsyncState::Loading => {
            // For now, we'll handle suspense differently
            // In a real implementation, this would need proper suspense support
            panic!("Component should be wrapped in SuspenseBoundary - data is still loading")
        }
        AsyncState::Success(data) => Ok(data.clone()),
        AsyncState::Error(error) => Err(error.clone()),
    }
}

/// Hook for using a family provider with suspense support
pub fn use_family_provider_suspense<P, Param>(
    provider: P,
    param: Param,
) -> Result<P::Output, P::Error>
where
    P: FamilyProvider<Param>,
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    let state = use_family_provider(provider, param);

    let current_state = state.read();
    match &*current_state {
        AsyncState::Loading => {
            // For now, we'll handle suspense differently
            // In a real implementation, this would need proper suspense support
            panic!("Component should be wrapped in SuspenseBoundary - data is still loading")
        }
        AsyncState::Success(data) => Ok(data.clone()),
        AsyncState::Error(error) => Err(error.clone()),
    }
}

/// Hook to access the provider cache for manual cache management  
pub fn use_provider_cache() -> ProviderCache {
    use_context_provider(ProviderCache::new)
}

/// Hook to invalidate a specific provider cache entry
pub fn use_invalidate_provider<P: FutureProvider>(provider: P) {
    let cache = use_provider_cache();
    cache.invalidate(&provider.id());
}

/// Hook to invalidate a specific family provider cache entry
pub fn use_invalidate_family_provider<P, Param>(provider: P, param: Param)
where
    P: FamilyProvider<Param>,
    Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    let cache = use_provider_cache();
    cache.invalidate(&provider.id(&param));
}

/// Hook to clear the entire provider cache
pub fn use_clear_provider_cache() {
    let cache = use_provider_cache();
    cache.clear();
}
