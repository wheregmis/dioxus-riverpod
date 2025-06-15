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
            div { class: "header",
                h1 { class: "main-title", "🚀 Dioxus Riverpod Features" }
                p { class: "subtitle", "Cross-Platform State Management Demo" }
            }
            div { class: "features-grid",
                div { class: "features-grid",
                    FeatureCard {
                        title: "Interval Provider".to_string(),
                        description: "Automatically refreshes data every 3 seconds in the background.".to_string(),
                        observation: "Watch the counter increment every 3 seconds automatically.".to_string(),
                        data: live_data, // Fixed signal name
                        show_refresh: Some(false),
                    }
                    FeatureCard {
                        title: "Auto-Dispose Provider".to_string(),
                        description: "Automatically cleans up from memory when not in use for 5 seconds.".to_string(),
                        observation: "Toggle visibility and watch memory cleanup after 5s.".to_string(),
                        data: auto_dispose_data, // Fixed signal name
                        show_refresh: Some(false),
                    }
                    FeatureCard {
                        title: "SWR Provider".to_string(),
                        description: "Serves stale data instantly while revalidating in background.".to_string(),
                        observation: "Fresh data shows instantly, then updates after 2s stale time.".to_string(),
                        data: swr_data, // Fixed signal name
                        show_refresh: Some(true),
                    }
                    FeatureCard {
                        title: "Cache Expiration Provider".to_string(),
                        description: "Traditional cache that expires after 4 seconds.".to_string(),
                        observation: "Shows loading state when cache expires after 4s.".to_string(),
                        data: cached_data, // Fixed signal name
                        show_refresh: Some(true),
                    }
                }
            }
            div { class: "footer-section",
                div { class: "platform-note",
                    h3 { "✅ Cross-Platform Compatible" }
                    p { "All features work identically on web and desktop platforms" }
                }
                div { class: "footer-branding",
                    p {
                        "Built with ❤️ using "
                        strong { "Dioxus Riverpod" }
                    }
                    p { class: "feature-tags",
                        span { class: "tag", "Cross-platform" }
                        span { class: "tag", "Reactive" }
                        span { class: "tag", "Type-safe" }
                    }
                }
            }
        }
    }
}

/// Reusable card component for displaying provider state
#[component]
fn FeatureCard(
    title: String,
    description: String,
    observation: String,
    data: Signal<AsyncState<String, String>>,
    show_refresh: Option<bool>,
) -> Element {
    let emoji = match title.as_str() {
        "Interval Provider" => "📡",
        "Auto-Dispose Provider" => "🗑️", 
        "SWR Provider" => "🔄",
        "Cache Expiration Provider" => "💾",
        _ => "⚡"
    };
    
    let card_id = title.chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_lowercase();
    
    let status_class = match &*data.read() {
        AsyncState::Loading => "status loading",
        AsyncState::Error(_) => "status error", 
        AsyncState::Success(_) => "status success",
    };

    rsx! {
        div { class: "feature-card",
            div { class: "card-header",
                h3 { class: "feature-title",
                    span { class: "feature-emoji", "{emoji}" }
                    "{title}"
                }
                div { class: "{status_class}" }
            }
            div { class: "card-content",
                p { class: "feature-description", "{description}" }
                div { class: "observation-tip",
                    strong { "🔍 What to observe:" }
                    p { "{observation}" }
                }
                div { class: "data-display",
                    match &*data.read() {
                        AsyncState::Loading => rsx! {
                            div { class: "loading-spinner" }
                            span { "Loading..." }
                        },
                        AsyncState::Error(e) => rsx! {
                            span { class: "error-text", "Error: {e}" }
                        },
                        AsyncState::Success(value) => rsx! {
                            div { class: "success-data",
                                strong { "Data: " }
                                span { "{value}" }
                            }
                        },
                    }
                }
                if let Some(true) = show_refresh {
                    div { class: "card-actions",
                        button { class: "refresh-btn", onclick: move |_| {}, "🔄 Refresh" }
                    }
                }
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
