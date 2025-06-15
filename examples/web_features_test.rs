use dioxus::prelude::*;
use dioxus_riverpod::prelude::*;
use std::time::Duration;
use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(not(target_family = "wasm"))]
use tokio::time::sleep;
#[cfg(target_family = "wasm")]
use wasmtimer::tokio::sleep;

static CALL_COUNTER: AtomicU32 = AtomicU32::new(0);

// Test provider with interval (auto-refresh every 3 seconds)
#[provider(interval = "3s")]
async fn fetch_live_data() -> Result<String, String> {
    sleep(Duration::from_millis(500)).await;
    
    let call_id = CALL_COUNTER.fetch_add(1, Ordering::SeqCst);
    Ok(format!("Live data call #{}", call_id))
}

// Test provider with auto-dispose (disposes after 5 seconds of no use)
#[provider(auto_dispose = true, dispose_delay = "5s")]
async fn fetch_auto_dispose_data() -> Result<String, String> {
    sleep(Duration::from_millis(300)).await;
    Ok("Auto-dispose data loaded".to_string())
}

// Test provider with SWR (stale-while-revalidate after 2 seconds)
#[provider(stale_time = "2s")]
async fn fetch_swr_data() -> Result<String, String> {
    sleep(Duration::from_millis(800)).await;
    
    let call_id = CALL_COUNTER.fetch_add(1, Ordering::SeqCst);
    Ok(format!("SWR data call #{}", call_id))
}

// Test provider with cache expiration (expires after 4 seconds)
#[provider(cache_expiration = "4s")]
async fn fetch_cached_data() -> Result<String, String> {
    sleep(Duration::from_millis(600)).await;
    
    let call_id = CALL_COUNTER.fetch_add(1, Ordering::SeqCst);
    Ok(format!("Cached data call #{}", call_id))
}

#[component]
fn FeatureTest() -> Element {
    let live_data = use_provider(fetch_live_data, ());
    let auto_dispose_data = use_provider(fetch_auto_dispose_data, ());
    let swr_data = use_provider(fetch_swr_data, ());
    let cached_data = use_provider(fetch_cached_data, ());
    
    // Refresh functions
    let refresh_live = use_invalidate_provider(fetch_live_data, ());
    let refresh_swr = use_invalidate_provider(fetch_swr_data, ());
    let refresh_cached = use_invalidate_provider(fetch_cached_data, ());

    rsx! {
        div { style: "font-family: system-ui; max-width: 800px; margin: 0 auto; padding: 20px;",
            h1 { "🚀 All Features Web Test" }
            p { "Testing interval, auto-dispose, SWR, and cache expiration on web platform" }
            
            div { style: "display: grid; gap: 16px; margin: 20px 0;",
                // Interval Provider Test
                div { style: "border: 1px solid #ddd; padding: 16px; border-radius: 8px;",
                    h3 { "📡 Interval Provider (auto-refresh every 3s)" }
                    match &*live_data.read() {
                        AsyncState::Loading => rsx! { p { "⏳ Loading live data..." } },
                        AsyncState::Success(data) => rsx! { p { "✅ {data}" } },
                        AsyncState::Error(e) => rsx! { p { "❌ Error: {e}" } },
                    }
                    button { 
                        onclick: move |_| refresh_live(),
                        "🔄 Manual Refresh"
                    }
                }
                
                // Auto-dispose Provider Test
                div { style: "border: 1px solid #ddd; padding: 16px; border-radius: 8px;",
                    h3 { "🗑️ Auto-dispose Provider (disposes after 5s)" }
                    match &*auto_dispose_data.read() {
                        AsyncState::Loading => rsx! { p { "⏳ Loading auto-dispose data..." } },
                        AsyncState::Success(data) => rsx! { p { "✅ {data}" } },
                        AsyncState::Error(e) => rsx! { p { "❌ Error: {e}" } },
                    }
                    p { style: "font-size: 0.9em; color: #666;",
                        "Data will auto-dispose 5 seconds after last access"
                    }
                }
                
                // SWR Provider Test
                div { style: "border: 1px solid #ddd; padding: 16px; border-radius: 8px;",
                    h3 { "🔄 SWR Provider (stale after 2s)" }
                    match &*swr_data.read() {
                        AsyncState::Loading => rsx! { p { "⏳ Loading SWR data..." } },
                        AsyncState::Success(data) => rsx! { p { "✅ {data}" } },
                        AsyncState::Error(e) => rsx! { p { "❌ Error: {e}" } },
                    }
                    button { 
                        onclick: move |_| refresh_swr(),
                        "🔄 Refresh SWR"
                    }
                    p { style: "font-size: 0.9em; color: #666;",
                        "Shows stale data immediately, revalidates in background after 2s"
                    }
                }
                
                // Cache Expiration Provider Test
                div { style: "border: 1px solid #ddd; padding: 16px; border-radius: 8px;",
                    h3 { "💾 Cache Expiration Provider (expires after 4s)" }
                    match &*cached_data.read() {
                        AsyncState::Loading => rsx! { p { "⏳ Loading cached data..." } },
                        AsyncState::Success(data) => rsx! { p { "✅ {data}" } },
                        AsyncState::Error(e) => rsx! { p { "❌ Error: {e}" } },
                    }
                    button { 
                        onclick: move |_| refresh_cached(),
                        "🔄 Refresh Cached"
                    }
                    p { style: "font-size: 0.9em; color: #666;",
                        "Shows loading when cache expires after 4s"
                    }
                }
            }
            
            div { style: "margin-top: 24px; padding: 16px; background: #f0f9ff; border-radius: 8px;",
                h3 { "🔍 What to observe:" }
                ul {
                    li { "📡 Interval provider should update every 3 seconds automatically" }
                    li { "🗑️ Auto-dispose data should clear from memory after 5s of no use" }
                    li { "🔄 SWR should serve stale data instantly after 2s, then update in background" }
                    li { "💾 Cache expiration should show loading state when data expires after 4s" }
                    li { "✅ All features should work identically on web and desktop" }
                }
            }
        }
    }
}

fn app() -> Element {
    // Setup all required contexts for dioxus-riverpod with auto-dispose features
    rsx! {
        // Provider setup component
        ContextProvider {}
    }
}

#[component] 
fn ContextProvider() -> Element {
    // Provide contexts in the right order
    use_context_provider(dioxus_riverpod::providers::ProviderCache::new);
    use_context_provider(dioxus_riverpod::providers::RefreshRegistry::new);
    
    // Create DisposalRegistry with the cache
    let cache = use_context::<dioxus_riverpod::providers::ProviderCache>();
    use_context_provider(move || dioxus_riverpod::providers::DisposalRegistry::new(cache.clone()));

    rsx! {
        FeatureTest {}
    }
}

fn main() {
    launch(app);
}
