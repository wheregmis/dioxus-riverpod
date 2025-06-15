//! # Refresh Registry
//!
//! This module provides the refresh registry that manages reactive updates and interval tasks
//! for providers. It handles triggering re-execution of providers when their dependencies change
//! and manages automatic refresh intervals for live data providers.
//!
//! ## Key Features
//!
//! - **Refresh Tracking**: Maintains counters for provider refresh events
//! - **Reactive Context Management**: Subscribes and notifies reactive contexts when data changes
//! - **Interval Tasks**: Manages background tasks for auto-refreshing providers
//! - **Revalidation Control**: Prevents duplicate revalidations and manages ongoing operations
//!
//! ## Cross-Platform Compatibility
//!
//! This module uses cross-platform abstractions:
//! - `dioxus::spawn` for background tasks (works on both web and desktop)
//! - `wasmtimer` for web timing and `tokio` for desktop timing
//! - Automatic task cleanup when components unmount

use dioxus_lib::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};

#[cfg(not(target_family = "wasm"))]
use tokio::time;
#[cfg(target_family = "wasm")]
use wasmtimer::tokio as time;

/// Type alias for reactive context storage
type ReactiveContextSet = Arc<Mutex<HashSet<ReactiveContext>>>;
type ReactiveContextRegistry = Arc<Mutex<HashMap<String, ReactiveContextSet>>>;

/// Task type for different periodic operations
#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    /// Interval refresh task that re-executes providers at regular intervals
    IntervalRefresh,
    /// Stale checking task that monitors for stale data and triggers revalidation
    StaleCheck,
}

/// Registry for periodic tasks (intervals and stale checks)
type PeriodicTaskRegistry = Arc<Mutex<HashMap<String, (TaskType, Duration, ())>>>;

/// Global registry for refresh signals that can trigger provider re-execution
///
/// The `RefreshRegistry` manages the reactive update system for providers. It tracks
/// which reactive contexts are subscribed to which providers, maintains refresh counters,
/// and manages periodic tasks for both auto-refreshing and stale-checking.
///
/// ## Thread Safety
///
/// All internal state is protected by mutexes to ensure thread-safe access across
/// different contexts and background tasks.
#[derive(Clone, Default)]
pub struct RefreshRegistry {
    /// Counters for tracking how many times each provider has been refreshed
    refresh_counters: Arc<Mutex<HashMap<String, u64>>>,
    /// Registry of reactive contexts subscribed to each provider key
    reactive_contexts: ReactiveContextRegistry,
    /// Registry of periodic tasks (both interval refresh and stale checking)
    periodic_tasks: PeriodicTaskRegistry,
    /// Set of provider keys that are currently being revalidated
    ongoing_revalidations: Arc<Mutex<HashSet<String>>>,
}

impl RefreshRegistry {
    /// Create a new refresh registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current refresh count for a provider key
    ///
    /// Returns the number of times the provider has been refreshed, or 0 if not found.
    pub fn get_refresh_count(&self, key: &str) -> u64 {
        if let Ok(counters) = self.refresh_counters.lock() {
            *counters.get(key).unwrap_or(&0)
        } else {
            0
        }
    }

    /// Subscribe a reactive context to refresh events for a provider key
    ///
    /// When the provider is refreshed, the reactive context will be marked as dirty,
    /// causing any components using it to re-render.
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

    /// Trigger a refresh for a provider key
    ///
    /// This increments the refresh counter and marks all subscribed reactive contexts
    /// as dirty, causing components to re-render and providers to re-execute.
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

    /// Clear all cached data and trigger refresh for all providers
    ///
    /// This is useful for global cache invalidation scenarios.
    pub fn clear_all(&self) {
        if let Ok(counters) = self.refresh_counters.lock() {
            let keys: Vec<String> = counters.keys().cloned().collect();
            drop(counters);

            for key in keys {
                self.trigger_refresh(&key);
            }
        }
    }

    /// Start a periodic task for automatic provider operations
    ///
    /// Creates a background task that will call the provided function at regular
    /// intervals. Supports both interval refresh and stale checking operations.
    /// If an existing task exists with a longer interval, it will be replaced.
    /// Tasks with shorter intervals are preserved to avoid unnecessary re-creation.
    ///
    /// ## Cross-Platform Implementation
    ///
    /// Uses `dioxus::spawn` to create tasks that work on both web and desktop platforms.
    /// Tasks are automatically cancelled when the component unmounts.
    pub fn start_periodic_task<F>(
        &self,
        key: &str,
        task_type: TaskType,
        interval: Duration,
        task_fn: F,
    ) where
        F: Fn() + Send + 'static,
    {
        if let Ok(mut tasks) = self.periodic_tasks.lock() {
            let task_key = format!("{}:{:?}", key, task_type);

            // For stale check tasks, don't create multiple tasks for the same provider
            if task_type == TaskType::StaleCheck
                && tasks.iter().any(|(k, (t, _, _))| {
                    k.starts_with(&format!("{}:", key)) && *t == TaskType::StaleCheck
                })
            {
                return;
            }

            // Cancel existing task if it exists and the new interval is shorter (for interval tasks)
            let should_create_new_task = match tasks.get(&task_key) {
                None => true,
                Some((_, current_interval, _)) => {
                    if task_type == TaskType::IntervalRefresh && interval < *current_interval {
                        tasks.remove(&task_key);
                        true
                    } else if task_type == TaskType::StaleCheck {
                        false // Don't replace stale check tasks
                    } else {
                        false // Keep existing shorter interval
                    }
                }
            };

            if should_create_new_task {
                // Adjust interval for stale checking - check every stale_time/4
                let actual_interval = if task_type == TaskType::StaleCheck {
                    Duration::max(
                        Duration::min(interval / 4, Duration::from_secs(30)),
                        Duration::from_secs(1),
                    )
                } else {
                    interval
                };

                spawn(async move {
                    let mut interval_timer = time::interval(actual_interval);
                    interval_timer.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

                    // Skip the first tick (immediate execution)
                    interval_timer.tick().await;

                    loop {
                        interval_timer.tick().await;
                        task_fn();
                    }
                });

                tasks.insert(task_key, (task_type, interval, ()));
            }
        }
    }

    /// Start an interval task for automatic provider refresh
    ///
    /// Convenience method that creates a periodic task for interval refresh.
    pub fn start_interval_task<F>(&self, key: &str, interval: Duration, refresh_fn: F)
    where
        F: Fn() + Send + 'static,
    {
        self.start_periodic_task(key, TaskType::IntervalRefresh, interval, refresh_fn);
    }

    /// Start a stale-checking task for automatic SWR revalidation
    ///
    /// Convenience method that creates a periodic task for stale checking.
    pub fn start_stale_check_task<F>(&self, key: &str, stale_time: Duration, stale_check_fn: F)
    where
        F: Fn() + Send + 'static,
    {
        self.start_periodic_task(key, TaskType::StaleCheck, stale_time, stale_check_fn);
    }

    /// Stop a periodic task for a specific provider and task type
    ///
    /// Removes the task from the registry. Since we use `dioxus::spawn`, the actual
    /// task will be automatically cancelled when the component unmounts.
    pub fn stop_periodic_task(&self, key: &str, task_type: TaskType) {
        if let Ok(mut tasks) = self.periodic_tasks.lock() {
            let task_key = format!("{}:{:?}", key, task_type);
            tasks.remove(&task_key);
        }
    }

    /// Stop an interval task for a specific provider
    ///
    /// Removes the task from the registry. Since we use `dioxus::spawn`, the actual
    /// task will be automatically cancelled when the component unmounts.
    pub fn stop_interval_task(&self, key: &str) {
        self.stop_periodic_task(key, TaskType::IntervalRefresh);
    }

    /// Stop a stale-checking task for a specific provider
    ///
    /// Removes the task from the registry. Since we use `dioxus::spawn`, the actual
    /// task will be automatically cancelled when the component unmounts.
    pub fn stop_stale_check_task(&self, key: &str) {
        self.stop_periodic_task(key, TaskType::StaleCheck);
    }

    /// Check if a revalidation is already in progress for a given key
    ///
    /// This prevents duplicate revalidation operations that could cause race conditions
    /// or unnecessary work.
    pub fn is_revalidation_in_progress(&self, key: &str) -> bool {
        if let Ok(revalidations) = self.ongoing_revalidations.lock() {
            revalidations.contains(key)
        } else {
            false
        }
    }

    /// Mark a revalidation as started for a given key
    ///
    /// Returns `true` if the revalidation was successfully started (no other revalidation
    /// was in progress), or `false` if a revalidation is already ongoing.
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
    ///
    /// This should be called when a revalidation operation finishes, whether it
    /// succeeded or failed.
    pub fn complete_revalidation(&self, key: &str) {
        if let Ok(mut revalidations) = self.ongoing_revalidations.lock() {
            revalidations.remove(key);
        }
    }
}
