use dioxus::prelude::*;
use dioxus_riverpod::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::time::sleep;

/// Global counter to track API call count for demonstration
static API_CALL_COUNT: AtomicU32 = AtomicU32::new(0);

/// Simulated API response data
#[derive(Debug, Clone, PartialEq)]
struct UserData {
    id: u32,
    name: String,
    fetch_time: String,
    api_call_number: u32,
    fetched_at: std::time::Instant,
}

/// Error type for our providers
#[derive(Debug, Clone)]
struct ApiError(String);

// ============================================================================
// PATTERN 1: Pure SWR - Never shows loading states after first load
// ============================================================================

#[provider(stale_time = "5s")]
async fn fetch_user_swr_only() -> Result<UserData, ApiError> {
    let call_number = API_CALL_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
    println!("🌐 [SWR] API call #{} starting...", call_number);

    sleep(Duration::from_millis(1500)).await;

    let now = {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let hours = (timestamp / 3600) % 24;
        let minutes = (timestamp / 60) % 60;
        let seconds = timestamp % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    };
    println!("✅ [SWR] API call #{} completed at {}", call_number, now);

    Ok(UserData {
        id: 1,
        name: "SWR User".to_string(),
        fetch_time: now,
        api_call_number: call_number,
        fetched_at: std::time::Instant::now(),
    })
}

// ============================================================================
// PATTERN 2: Traditional Cache Expiration - Shows loading states when expired
// ============================================================================

#[provider(cache_expiration = "8s")]
async fn fetch_user_traditional() -> Result<UserData, ApiError> {
    let call_number = API_CALL_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
    println!("🗄️ [TRADITIONAL] API call #{} starting...", call_number);

    sleep(Duration::from_millis(1500)).await;

    let now = {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let hours = (timestamp / 3600) % 24;
        let minutes = (timestamp / 60) % 60;
        let seconds = timestamp % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    };
    println!(
        "✅ [TRADITIONAL] API call #{} completed at {}",
        call_number, now
    );

    Ok(UserData {
        id: 2,
        name: "Traditional User".to_string(),
        fetch_time: now,
        api_call_number: call_number,
        fetched_at: std::time::Instant::now(),
    })
}

// ============================================================================
// PATTERN 3: No Caching - Always fresh, always shows loading
// ============================================================================

#[provider]
async fn fetch_user_no_cache() -> Result<UserData, ApiError> {
    let call_number = API_CALL_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
    println!("🚫 [NO CACHE] API call #{} starting...", call_number);

    sleep(Duration::from_millis(1500)).await;

    let now = {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let hours = (timestamp / 3600) % 24;
        let minutes = (timestamp / 60) % 60;
        let seconds = timestamp % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    };
    println!(
        "✅ [NO CACHE] API call #{} completed at {}",
        call_number, now
    );

    Ok(UserData {
        id: 3,
        name: "No Cache User".to_string(),
        fetch_time: now,
        api_call_number: call_number,
        fetched_at: std::time::Instant::now(),
    })
}

/// Component that displays user data with clear behavior indicators
#[component]
fn UserCard(
    title: String,
    description: String,
    user_state: Signal<AsyncState<UserData, ApiError>>,
    behavior: String,
) -> Element {
    let (status_emoji, status_color) = match &*user_state.read() {
        AsyncState::Loading => ("🔄", "#3182ce"),
        AsyncState::Success(_) => ("✅", "#38a169"),
        AsyncState::Error(_) => ("❌", "#e53e3e"),
    };

    rsx! {
        div { style: "border: 2px solid #e2e8f0; border-radius: 8px; padding: 16px; margin: 8px; background: white;",
            h3 { style: "margin: 0 0 8px 0; color: #2d3748;", "{title}" }
            div { style: "margin-bottom: 12px; padding: 8px; background: #f7fafc; border-radius: 4px; font-size: 0.9em; color: #4a5568;",
                "{description}"
            }
            div { style: "margin-bottom: 8px; font-weight: bold;",
                span { style: "color: {status_color};", "{status_emoji} " }
                "Status: {behavior}"
            }
            match &*user_state.read() {
                AsyncState::Loading => rsx! {
                    div { style: "color: #4a5568; font-style: italic;", "Loading user data..." }
                },
                AsyncState::Success(user) => rsx! {
                    div {
                        div { style: "margin: 4px 0;",
                            strong { "Name: " }
                            "{user.name}"
                        }
                        div { style: "margin: 4px 0; color: #6b7280; font-size: 0.9em;",
                            strong { "Last fetched: " }
                            "{user.fetch_time}"
                        }
                        div { style: "margin: 4px 0; color: #6b7280; font-size: 0.9em;",
                            strong { "Data age: " }
                            "{user.fetched_at.elapsed().as_secs()}s ago"
                        }
                        div { style: "margin: 4px 0; color: #6b7280; font-size: 0.9em;",
                            strong { "API Call #: " }
                            "{user.api_call_number}"
                        }
                    }
                },
                AsyncState::Error(error) => rsx! {
                    div { style: "color: #e53e3e;", "Error: {error.0}" }
                },
            }
        }
    }
}

/// Main demo component
#[component]
fn SimpleSWRDemo() -> Element {
    // Three different caching patterns
    let swr_user = use_provider(fetch_user_swr_only, ());
    let traditional_user = use_provider(fetch_user_traditional, ());
    let no_cache_user = use_provider(fetch_user_no_cache, ());

    // Manual refresh functions
    let refresh_swr = use_invalidate_provider(fetch_user_swr_only, ());
    let refresh_traditional = use_invalidate_provider(fetch_user_traditional, ());
    let refresh_no_cache = use_invalidate_provider(fetch_user_no_cache, ());

    // Force UI update every second to show real-time data age
    let update_trigger = use_signal(|| 0);
    use_effect(move || {
        let mut update_trigger = update_trigger;
        spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                update_trigger += 1;
            }
        });
    });

    rsx! {
        div { style: "font-family: system-ui, sans-serif; max-width: 1000px; margin: 0 auto; padding: 20px; background: #f7fafc;",
            header { style: "text-align: center; margin-bottom: 32px;",
                h1 { style: "color: #2d3748; margin-bottom: 8px;", "🎯 Simplified Cache Patterns" }
                p { style: "color: #4a5568; max-width: 600px; margin: 0 auto; line-height: 1.6;",
                    "Three clear caching patterns with distinct behaviors - no confusion!"
                }
            }

            div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 16px; margin-bottom: 24px;",
                UserCard {
                    title: "🔄 Stale-While-Revalidate".to_string(),
                    description: "Serves stale data instantly (after 5s), fetches fresh data in background. Never shows loading after first load."
                        .to_string(),
                    user_state: swr_user,
                    behavior: {
                        let _ = update_trigger.read();
                        match &*swr_user.read() {
                            AsyncState::Loading => "Initial Loading".to_string(),
                            AsyncState::Success(data) => {
                                let age = data.fetched_at.elapsed().as_secs();
                                if age >= 5 {
                                    "Serving Stale Data".to_string()
                                } else {
                                    "Fresh Data".to_string()
                                }
                            }
                            AsyncState::Error(_) => "Error State".to_string(),
                        }
                    },
                }

                UserCard {
                    title: "🗄️ Traditional Cache".to_string(),
                    description: "Shows loading state when cache expires (after 8s). Classic cache behavior."
                        .to_string(),
                    user_state: traditional_user,
                    behavior: match &*traditional_user.read() {
                        AsyncState::Loading => "Cache Expired - Loading".to_string(),
                        _ => "Cached Data".to_string(),
                    },
                }

                UserCard {
                    title: "🚫 No Caching".to_string(),
                    description: "Always fetches fresh data. Always shows loading state for every request."
                        .to_string(),
                    user_state: no_cache_user,
                    behavior: match &*no_cache_user.read() {
                        AsyncState::Loading => "Always Loading".to_string(),
                        _ => "Fresh Data".to_string(),
                    },
                }
            }

            div { style: "display: flex; flex-wrap: wrap; gap: 12px; justify-content: center; margin-bottom: 24px;",
                button {
                    style: "background: #3182ce; color: white; border: none; padding: 10px 16px; border-radius: 6px; cursor: pointer;",
                    onclick: move |_| refresh_swr(),
                    "🔄 Refresh SWR"
                }
                button {
                    style: "background: #38a169; color: white; border: none; padding: 10px 16px; border-radius: 6px; cursor: pointer;",
                    onclick: move |_| refresh_traditional(),
                    "🗄️ Refresh Traditional"
                }
                button {
                    style: "background: #e53e3e; color: white; border: none; padding: 10px 16px; border-radius: 6px; cursor: pointer;",
                    onclick: move |_| refresh_no_cache(),
                    "🚫 Refresh No Cache"
                }
            }

            div { style: "background: white; border-radius: 8px; padding: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);",
                h2 { style: "color: #2d3748; margin-bottom: 16px;", "📖 Behavior Patterns" }
                div { style: "display: grid; gap: 16px;",
                    div {
                        h3 { style: "color: #3182ce; margin-bottom: 8px;",
                            "🔄 Stale-While-Revalidate"
                        }
                        ul { style: "color: #4a5568; line-height: 1.6; margin: 0; padding-left: 20px;",
                            li { "✅ Instant responses (no loading states after first load)" }
                            li { "🔄 Background updates when data becomes stale" }
                            li { "🎯 Best user experience - never wait for data" }
                            li { "⚠️ Data might be slightly outdated temporarily" }
                        }
                    }
                    div {
                        h3 { style: "color: #38a169; margin-bottom: 8px;", "🗄️ Traditional Cache" }
                        ul { style: "color: #4a5568; line-height: 1.6; margin: 0; padding-left: 20px;",
                            li { "💾 Serves cached data while valid" }
                            li { "⏳ Shows loading when cache expires" }
                            li { "🎯 Guarantees fresh data after loading" }
                            li { "⚠️ Users experience loading delays" }
                        }
                    }
                    div {
                        h3 { style: "color: #e53e3e; margin-bottom: 8px;", "🚫 No Caching" }
                        ul { style: "color: #4a5568; line-height: 1.6; margin: 0; padding-left: 20px;",
                            li { "🔄 Always fetches fresh data" }
                            li { "⏳ Always shows loading states" }
                            li { "🎯 Guaranteed freshness" }
                            li { "⚠️ High network usage, poor UX" }
                        }
                    }
                }

                div { style: "margin-top: 20px; padding: 16px; background: #f0f9ff; border-radius: 6px;",
                    h3 { style: "color: #0369a1; margin-bottom: 8px;", "🧪 Test Scenario" }
                    ol { style: "color: #4a5568; line-height: 1.6; margin: 0; padding-left: 20px;",
                        li { "Initial load: All providers show loading states" }
                        li {
                            "Wait 5+ seconds: SWR starts serving stale data instantly + background updates"
                        }
                        li { "Wait 8+ seconds: Traditional cache expires, shows loading again" }
                        li { "Click refresh: See different behaviors for each pattern" }
                    }
                }
            }
        }
    }
}

fn main() {
    // Initialize tracing for better debug output
    tracing_subscriber::fmt::init();
    dioxus::launch(app);
}

fn app() -> Element {
    use_context_provider(dioxus_riverpod::providers::ProviderCache::new);
    use_context_provider(dioxus_riverpod::providers::RefreshRegistry::new);
    rsx! {
        SimpleSWRDemo {}
    }
}
