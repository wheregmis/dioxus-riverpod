//! Cache management and async state types for dioxus-riverpod

use std::{
    any::Any,
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};
use tracing::debug;

// Platform-specific time imports
#[cfg(not(target_family = "wasm"))]
use std::time::Instant;
#[cfg(target_family = "wasm")]
use web_time::Instant;

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
pub struct CacheEntry {
    data: Arc<dyn Any + Send + Sync>,
    cached_at: Instant,
    reference_count: Arc<AtomicU32>,
    last_accessed: Arc<Mutex<Instant>>,
}

impl CacheEntry {
    pub fn new<T: Clone + Send + Sync + 'static>(data: T) -> Self {
        let now = Instant::now();
        Self {
            data: Arc::new(data),
            cached_at: now,
            reference_count: Arc::new(AtomicU32::new(0)),
            last_accessed: Arc::new(Mutex::new(now)),
        }
    }

    pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        // Update last accessed time
        if let Ok(mut last_accessed) = self.last_accessed.lock() {
            *last_accessed = Instant::now();
        }
        self.data.downcast_ref::<T>().cloned()
    }

    pub fn is_expired(&self, expiration: Duration) -> bool {
        self.cached_at.elapsed() > expiration
    }

    pub fn is_stale(&self, stale_time: Duration) -> bool {
        self.cached_at.elapsed() > stale_time
    }

    /// Increment reference count when a provider hook starts using this entry
    pub fn add_reference(&self) {
        self.reference_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement reference count when a provider hook stops using this entry
    pub fn remove_reference(&self) {
        self.reference_count.fetch_sub(1, Ordering::SeqCst);
    }

    /// Get current reference count
    pub fn reference_count(&self) -> u32 {
        self.reference_count.load(Ordering::SeqCst)
    }
}

/// Global cache for provider results with automatic cleanup
#[derive(Clone, Default)]
pub struct ProviderCache {
    pub cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
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
                return None;
            }
        }

        // Check if stale
        let is_stale = if let Some(stale_duration) = stale_time {
            entry.is_stale(stale_duration)
        } else {
            false
        };

        entry.get::<T>().map(|data| (data, is_stale))
    }

    /// Set a value in the cache
    pub fn set<T: Clone + Send + Sync + 'static>(&self, key: String, value: T) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(key, CacheEntry::new(value));
        }
    }

    /// Remove a value from the cache
    pub fn remove(&self, key: &str) -> bool {
        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(key).is_some()
        } else {
            false
        }
    }

    /// Invalidate (remove) a specific cache entry - alias for remove
    pub fn invalidate(&self, key: &str) {
        self.remove(key);
    }

    /// Clear all cached values
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
}
