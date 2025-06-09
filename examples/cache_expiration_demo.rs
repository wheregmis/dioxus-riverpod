use dioxus::prelude::*;
use dioxus_riverpod::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

// Provider with 2-second cache expiration for testing
#[provider(cache_expiration = "2s")]
async fn fetch_expiring_data() -> Result<String, String> {
    println!("⏰ [EXECUTING] Fetching data - cache expires in 2 seconds");
    sleep(Duration::from_millis(200)).await;
    Ok(format!(
        "Data fetched at: {}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ))
}

// Family provider with 3-second cache expiration
#[provider(cache_expiration = "3s")]
async fn fetch_user_data(user_id: u32) -> Result<String, String> {
    println!(
        "⏰ [EXECUTING] Fetching user {} data - cache expires in 3 seconds",
        user_id
    );
    sleep(Duration::from_millis(150)).await;
    Ok(format!(
        "User {} data fetched at: {}",
        user_id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ))
}

#[component]
fn DataSection(trigger: Signal<i32>) -> Element {
    // Use the trigger signal to force re-evaluation without component recreation
    let _trigger_value = trigger.read();

    rsx! {
        div {
            h3 { "📦 Future Provider with 2s Cache Expiration" }
            div { style: "padding: 15px; border: 1px solid #dc3545; border-radius: 5px; margin: 10px 0;",
                match &*use_provider(fetch_expiring_data, ()).read() {
                    AsyncState::Loading => rsx! {
                        p { style: "color: #orange;", "⏳ Loading..." }
                    },
                    AsyncState::Success(data) => rsx! {
                        p { style: "color: #dc3545; font-weight: bold;", "✅ {data}" }
                    },
                    AsyncState::Error(e) => rsx! {
                        p { style: "color: red;", "❌ Error: {e}" }
                    },
                }
            }
            h3 { "👤 Family Provider with 3s Cache Expiration (User 1)" }
            div { style: "padding: 15px; border: 1px solid #28a745; border-radius: 5px; margin: 10px 0;",
                match &*use_provider(fetch_user_data, (1u32,)).read() {
                    AsyncState::Loading => rsx! {
                        p { style: "color: #orange;", "⏳ Loading user data..." }
                    },
                    AsyncState::Success(data) => rsx! {
                        p { style: "color: #28a745; font-weight: bold;", "✅ {data}" }
                    },
                    AsyncState::Error(e) => rsx! {
                        p { style: "color: red;", "❌ Error: {e}" }
                    },
                }
            }
        }
    }
}

#[component]
fn App() -> Element {
    // Provide the necessary contexts
    use_context_provider(ProviderCache::new);
    use_context_provider(RefreshRegistry::new);

    let mut trigger_refresh = use_signal(|| 0);

    rsx! {
        div { style: "padding: 20px; font-family: Arial, sans-serif;",
            h1 { "🚀 Cache Expiration Demo" }
            div { style: "margin: 20px 0;",
                button {
                    onclick: move |_| {
                        let current = *trigger_refresh.read();
                        trigger_refresh.set(current + 1);
                        println!(
                            "🔄 [USER ACTION] Refresh button clicked - triggering component re-render (count: {})",
                            current + 1,
                        );
                    },
                    style: "padding: 10px 20px; background: #007cba; color: white; border: none; border-radius: 4px; cursor: pointer; margin-right: 10px;",
                    "🔄 Refresh Components"
                }
                span { style: "color: #666;", "Refresh count: {trigger_refresh.read()}" }
            }
            // DataSection component - refresh counter triggers re-evaluation but preserves cache
            DataSection { trigger: trigger_refresh }
            div { style: "margin: 20px 0; padding: 15px; background: #f8f9fa; border-radius: 5px; border-left: 4px solid #007cba;",
                h4 { "📋 Test Instructions:" }
                ol {
                    li { "Initial page load executes both providers (cache miss)" }
                    li {
                        "Click 'Refresh Components' immediately - should use cached data (no execution logs)"
                    }
                    li {
                        "Wait 2+ seconds, then click 'Refresh' - first provider should re-execute (cache expired)"
                    }
                    li {
                        "Wait 3+ seconds, then click 'Refresh' - second provider should re-execute (cache expired)"
                    }
                    li {
                        "Watch the console logs to see when providers actually execute vs. when cached data is used"
                    }
                }
            }
            div { style: "margin: 20px 0; padding: 15px; background: #e9ecef; border-radius: 5px;",
                p { style: "margin: 0;",
                    "💡 **Cache Expiration vs Interval**: Cache expiration is passive - data only refreshes when accessed after expiration. Interval refresh is proactive - data refreshes automatically in the background."
                }
            }
        }
    }
}

fn main() {
    println!("🚀 Starting Cache Expiration Demo");
    println!("📋 Instructions:");
    println!("   1. Watch for provider execution logs when page loads");
    println!("   2. Use the refresh button to test cache behavior");
    println!("   3. Notice that providers only re-execute after cache expires AND when accessed");
    println!();

    dioxus::launch(App);
}
