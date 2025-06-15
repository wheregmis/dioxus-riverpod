//! # Cross-Platform Web Features Test
//! 
//! This example demonstrates all dioxus-riverpod features working
//! seamlessly on both web (WASM) and desktop platforms:
//! - Interval-based auto-refresh
//! - Auto-dispose for memory management  
//! - Stale-while-revalidate (SWR) pattern
//! - Cache expiration policies

use dioxus::prelude::*;
use dioxus_riverpod::prelude::*;
use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

// Cross-platform sleep function
#[cfg(not(target_family = "wasm"))]
use tokio::time::sleep;
#[cfg(target_family = "wasm")]
use wasmtimer::tokio::sleep;

/// Global counter for simulating unique API calls
static CALL_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Test provider with interval (auto-refresh every 3 seconds)
#[provider(interval = "3s")]
async fn fetch_live_data() -> Result<String, String> {
    sleep(Duration::from_millis(500)).await;
    let call_id = CALL_COUNTER.fetch_add(1, Ordering::SeqCst);
    Ok(format!("Live data call #{}", call_id))
}

/// Test provider with auto-dispose (disposes after 5 seconds of no use)
#[provider(auto_dispose = true, dispose_delay = "5s")]
async fn fetch_auto_dispose_data() -> Result<String, String> {
    sleep(Duration::from_millis(300)).await;
    Ok("Auto-dispose data loaded".to_string())
}

/// Test provider with SWR (stale-while-revalidate after 2 seconds)
#[provider(stale_time = "2s")]
async fn fetch_swr_data() -> Result<String, String> {
    sleep(Duration::from_millis(800)).await;
    let call_id = CALL_COUNTER.fetch_add(1, Ordering::SeqCst);
    Ok(format!("SWR data call #{}", call_id))
}

/// Test provider with cache expiration (expires after 4 seconds)
#[provider(cache_expiration = "4s")]
async fn fetch_cached_data() -> Result<String, String> {
    sleep(Duration::from_millis(600)).await;
    let call_id = CALL_COUNTER.fetch_add(1, Ordering::SeqCst);
    Ok(format!("Cached data call #{}", call_id))
}

/// Main feature test component demonstrating all provider types
#[component]
fn FeatureTest() -> Element {
    // Initialize all providers
    let live_data = use_provider(fetch_live_data, ());
    let auto_dispose_data = use_provider(fetch_auto_dispose_data, ());
    let swr_data = use_provider(fetch_swr_data, ());
    let cached_data = use_provider(fetch_cached_data, ());
    
    // Get refresh functions for manual testing
    let refresh_live = use_invalidate_provider(fetch_live_data, ());
    let refresh_swr = use_invalidate_provider(fetch_swr_data, ());
    let refresh_cached = use_invalidate_provider(fetch_cached_data, ());

    rsx! {
        div { class: "container",
            h1 { "🚀 Cross-Platform Features Demo" }
            p { "Testing interval, auto-dispose, SWR, and cache expiration" }
            div { class: "feature-grid",
                FeatureCard {
                    title: "📡 Interval Provider",
                    subtitle: "Auto-refresh every 3s",
                    data: live_data,
                    show_refresh: true,
                    refresh: refresh_live,
                    description: "Updates automatically in background",
                }
                FeatureCard {
                    title: "🗑️ Auto-dispose Provider",
                    subtitle: "Disposes after 5s",
                    data: auto_dispose_data,
                    show_refresh: false,
                    refresh: move |_| {},
                    description: "Clears from memory when unused",
                }
                FeatureCard {
                    title: "🔄 SWR Provider",
                    subtitle: "Stale after 2s",
                    data: swr_data,
                    show_refresh: true,
                    refresh: refresh_swr,
                    description: "Serves stale data, revalidates in background",
                }
                FeatureCard {
                    title: "💾 Cache Expiration",
                    subtitle: "Expires after 4s",
                    data: cached_data,
                    show_refresh: true,
                    refresh: refresh_cached,
                    description: "Shows loading when cache expires",
                }
            }
            ObservationGuide {}
        }
    }
}

/// Reusable card component for displaying provider state
#[component]
fn FeatureCard(
    title: String,
    subtitle: String,
    data: ReadOnlySignal<AsyncState<String, String>>,
    show_refresh: bool,
    refresh: EventHandler<()>,
    description: String,
) -> Element {
    rsx! {
        div { class: "feature-card",
            h3 { "{title}" }
            p { class: "subtitle", "{subtitle}" }
            div { class: "status",
                match &*data.read() {
                    AsyncState::Loading => rsx! {
                        p { "⏳ Loading..." }
                    },
                    AsyncState::Success(data) => rsx! {
                        p { class: "success", "✅ {data}" }
                    },
                    AsyncState::Error(e) => rsx! {
                        p { class: "error", "❌ Error: {e}" }
                    },
                }
            }
            if show_refresh {
                button { onclick: move |_| refresh.call(()), "🔄 Refresh" }
            }
            p { class: "description", "{description}" }
        }
    }
}

/// Guide component explaining what to observe
#[component] 
fn ObservationGuide() -> Element {
    rsx! {
        div { class: "observation-guide",
            h3 { "🔍 What to observe:" }
            ul {
                li { "📡 Interval provider updates every 3 seconds automatically" }
                li { "🗑️ Auto-dispose clears from memory after 5s of no use" }
                li { "🔄 SWR serves stale data instantly, updates in background after 2s" }
                li { "💾 Cache expiration shows loading when data expires after 4s" }
                li { "✅ All features work identically on web and desktop" }
            }
        }
    }
}

/// Application root with context providers setup
fn app() -> Element {
    rsx! {
        style { {include_str!("./assets/style.css")} }
        RiverpodProvider { FeatureTest {} }
    }
}

/// Provider component that sets up all required dioxus-riverpod contexts
#[component] 
fn RiverpodProvider(children: Element) -> Element {
    // Setup contexts in correct order
    use_context_provider(dioxus_riverpod::providers::ProviderCache::new);
    use_context_provider(dioxus_riverpod::providers::RefreshRegistry::new);
    
    // Create disposal registry with cache reference
    let cache = use_context::<dioxus_riverpod::providers::ProviderCache>();
    use_context_provider(move || dioxus_riverpod::providers::DisposalRegistry::new(cache.clone()));

    rsx! {
        {children}
    }
}

fn main() {
    launch(app);
}
