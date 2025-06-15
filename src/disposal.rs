//! # Provider Disposal Management
//!
//! This module manages automatic disposal of unused providers to prevent memory leaks
//! and maintain optimal performance.

use crate::cache::ProviderCache;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tracing::debug;

// Platform-specific imports
#[cfg(not(target_family = "wasm"))]
use tokio::{task::JoinHandle, time};
#[cfg(target_family = "wasm")]
use {dioxus_lib::prelude::spawn, wasmtimer::tokio as time};

/// Registry for managing provider disposal timers and cleanup
#[derive(Clone, Default)]
pub struct DisposalRegistry {
    #[cfg(not(target_family = "wasm"))]
    disposal_timers: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    #[cfg(target_family = "wasm")]
    disposal_timers: Arc<Mutex<HashMap<String, ()>>>, // Dummy type for WASM
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
        if let (Ok(mut timers), Some(cache)) = (self.disposal_timers.lock(), &self.cache) {
            // Cross-platform disposal scheduling
            #[cfg(not(target_family = "wasm"))]
            {
                // Cancel existing timer if present
                if let Some(existing_timer) = timers.remove(&cache_key) {
                    existing_timer.abort();
                }

                let cache_clone = cache.clone();
                let cache_key_clone = cache_key.clone();

                let timer = tokio::spawn(async move {
                    time::sleep(dispose_delay).await;

                    // Check if the provider can still be disposed
                    if let Ok(cache_guard) = cache_clone.cache.lock() {
                        if let Some(entry) = cache_guard.get(&cache_key_clone) {
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

            #[cfg(target_family = "wasm")]
            {
                // For WASM, use dioxus spawn for disposal timers
                let cache_clone = cache.clone();
                let cache_key_clone = cache_key.clone();

                spawn(async move {
                    time::sleep(dispose_delay).await;

                    // Check if the provider can still be disposed
                    if let Ok(cache_guard) = cache_clone.cache.lock() {
                        if let Some(entry) = cache_guard.get(&cache_key_clone) {
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

                // Store dummy entry for consistency
                timers.insert(cache_key, ());
            }
        }
    }

    /// Cancel disposal timer for a provider (called when provider is accessed again)
    pub fn cancel_disposal(&self, cache_key: &str) {
        if let Ok(mut timers) = self.disposal_timers.lock() {
            if let Some(timer) = timers.remove(cache_key) {
                #[cfg(not(target_family = "wasm"))]
                {
                    timer.abort();
                    debug!("🔄 [AUTO-DISPOSE] Cancelled disposal for: {}", cache_key);
                }
                
                // In WASM, we can't cancel individual disposal tasks
                // They will complete but check if the provider is still unused
                #[cfg(target_family = "wasm")]
                {
                    let _ = timer; // Silence unused variable warning
                    debug!("🔄 [AUTO-DISPOSE] Noted disposal cancellation for: {}", cache_key);
                }
            }
        }
    }

    /// Get the default disposal delay (30 seconds)
    pub fn default_dispose_delay() -> Duration {
        Duration::from_secs(30)
    }
}
