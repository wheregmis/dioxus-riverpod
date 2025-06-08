use dioxus::prelude::*;
use dioxus_riverpod::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

// Helper function to format timestamps
fn format_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

// Future provider for simple cache hit/miss testing
#[provider]
async fn fetch_data() -> Result<String, String> {
    println!(
        "🔄 [PROVIDER] Executing fetch_data - this should only happen on first load and after invalidation"
    );
    sleep(Duration::from_millis(1000)).await;
    Ok(format!(
        "Data: {}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ))
}

// Family provider with multiple parameters
#[provider]
async fn fetch_user_data(user_id: u32, include_details: bool) -> Result<String, String> {
    println!(
        "📡 [PROVIDER] Fetching user data for user_id={}, include_details={}",
        user_id, include_details
    );
    sleep(Duration::from_millis(500)).await;

    // Generate realistic user data based on user_id
    let names = [
        "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry",
    ];
    let name = names[user_id as usize % names.len()];
    let age = 20 + (user_id % 50); // Age between 20-69

    if include_details {
        Ok(format!(
            "👤 Name: {}, 🎂 Age: {}, 🆔 ID: {}, 📧 Email: {}@example.com, 📍 Location: City {}",
            name,
            age,
            user_id,
            name.to_lowercase(),
            user_id % 10 + 1
        ))
    } else {
        Ok(format!("👤 {}, 🎂 {} years old", name, age))
    }
}

// Provider with 5-second interval refresh for live data simulation
#[provider(interval_secs = 5)]
async fn live_server_status() -> Result<String, String> {
    println!("🟢 [INTERVAL PROVIDER] Checking server status (auto-refreshes every 5 seconds)");
    sleep(Duration::from_millis(200)).await;

    let statuses = ["🟢 Online", "🟡 Maintenance", "🔴 Offline", "🟢 Healthy"];
    let status = statuses[std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        % statuses.len()];

    Ok(format!(
        "Server Status: {} | Last Check: {}",
        status,
        format_timestamp()
    ))
}

// Provider with 2-second interval for metrics
#[provider(interval_secs = 2)]
async fn live_metrics() -> Result<String, String> {
    println!("📊 [INTERVAL PROVIDER] Fetching metrics (auto-refreshes every 2 seconds)");
    sleep(Duration::from_millis(100)).await;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let cpu = 10 + (now % 80);
    let memory = 30 + (now % 40);

    Ok(format!(
        "💾 CPU: {}%, 🧠 Memory: {}% | Updated: {}",
        cpu,
        memory,
        format_timestamp()
    ))
}

// Family provider with interval for user activity monitoring
#[provider(interval_secs = 3)]
async fn user_activity(user_id: u32) -> Result<String, String> {
    println!(
        "👥 [INTERVAL FAMILY] Checking activity for user {} (auto-refreshes every 3 seconds)",
        user_id
    );
    sleep(Duration::from_millis(150)).await;

    let activities = ["🔵 Online", "🟡 Away", "⚫ Offline", "🟢 Active"];
    let activity = activities[(user_id
        + std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32) as usize
        % activities.len()];

    Ok(format!(
        "User {} Activity: {} | Last Update: {}",
        user_id,
        activity,
        format_timestamp()
    ))
}

// Family provider with single parameter
#[provider]
async fn fetch_user_name(user_id: u32) -> Result<String, String> {
    println!("📡 [PROVIDER] Fetching user name for user_id={}", user_id);
    sleep(Duration::from_millis(300)).await;

    let names = [
        "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry",
    ];
    let name = names[user_id as usize % names.len()];
    Ok(format!("👤 {}", name))
}

// Future provider for time testing
#[provider]
async fn fetch_current_time() -> Result<String, String> {
    println!("📡 [PROVIDER] Fetching current time");
    sleep(Duration::from_millis(200)).await;
    Ok(format!("Current time: {}", format_timestamp()))
}

// Real-time clock provider that updates every second
#[provider(interval_secs = 1)]
async fn live_clock() -> Result<String, String> {
    // Don't log for the clock as it would be too verbose
    Ok(format!("🕐 Current Time: {}", format_timestamp()))
}

#[component]
fn DataDisplay() -> Element {
    let data_signal = use_future_provider(fetch_data);

    rsx! {
        div { style: "padding: 15px; border: 1px solid #ddd; border-radius: 5px; margin: 10px 0;",
            match &*data_signal.read() {
                AsyncState::Loading => rsx! {
                    p { "⏳ Loading..." }
                },
                AsyncState::Success(data) => rsx! {
                    p { "✅ {data}" }
                },
                AsyncState::Error(e) => rsx! {
                    p { "❌ Error: {e}" }
                },
            }
        }
    }
}

#[component]
fn UserDataDisplay(user_id: u32, include_details: bool) -> Element {
    let user_data = use_family_provider(fetch_user_data, (user_id, include_details));

    rsx! {
        div { style: "padding: 15px; border: 1px solid #007cba; border-radius: 5px; margin: 10px 0;",
            h4 { "User Data (ID: {user_id}, Details: {include_details})" }
            match &*user_data.read() {
                AsyncState::Loading => rsx! {
                    p { "⏳ Loading user data..." }
                },
                AsyncState::Success(data) => rsx! {
                    p { "✅ {data}" }
                },
                AsyncState::Error(e) => rsx! {
                    p { "❌ Error: {e}" }
                },
            }
        }
    }
}

#[component]
fn app() -> Element {
    // Provide shared cache context
    use_context_provider(dioxus_riverpod::providers::ProviderCache::new);
    use_context_provider(dioxus_riverpod::providers::RefreshRegistry::new);

    // State for UI toggles
    let mut show_second_data = use_signal(|| false);
    let mut show_user_data = use_signal(|| false);
    let mut show_time_data = use_signal(|| false);
    let mut user_id = use_signal(|| 42u32);
    let mut include_details = use_signal(|| true);

    // Provider hooks for different provider types
    let user_name = use_family_provider(fetch_user_name, *user_id.read());
    let current_time = use_future_provider(fetch_current_time);

    // Invalidation hooks
    let invalidate_data = use_invalidate_provider(fetch_data);
    let invalidate_user_data =
        use_invalidate_family_provider(fetch_user_data, (*user_id.read(), *include_details.read()));
    let invalidate_user_name = use_invalidate_family_provider(fetch_user_name, *user_id.read());
    let invalidate_time = use_invalidate_provider(fetch_current_time);
    let clear_cache = use_clear_provider_cache();

    rsx! {
        div { style: "padding: 20px; font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto;",

            h1 { "🧪 Comprehensive Cache & Provider Test" }

            // Cache Hit/Miss Testing Section
            section { style: "margin: 30px 0; padding: 20px; background: #f8f9fa; border-radius: 8px;",
                h2 { "Cache Hit/Miss Behavior Test" }
                h3 { "Component 1 (always visible):" }
                DataDisplay {}

                div { style: "margin: 20px 0;",
                    button {
                        onclick: move |_| show_second_data.toggle(),
                        style: "padding: 8px 16px; margin-right: 10px; background: #28a745; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        if *show_second_data.read() {
                            "Hide Component 2"
                        } else {
                            "Show Component 2"
                        }
                    }
                    button {
                        onclick: move |_| {
                            println!("🔄 [USER] Invalidating fetch_data cache manually");
                            invalidate_data();
                        },
                        style: "padding: 8px 16px; background: #dc3545; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "Invalidate Basic Data Cache"
                    }
                }

                if *show_second_data.read() {
                    div {
                        h3 { "Component 2 (togglable - should show cache HIT):" }
                        DataDisplay {}
                    }
                }
            }

            // Family Provider Testing Section
            section { style: "margin: 30px 0; padding: 20px; background: #e7f3ff; border-radius: 8px;",
                h2 { "Family Provider Feature Test" }

                // User ID and Details Controls
                div { style: "margin: 20px 0; padding: 15px; border: 1px solid #ccc; border-radius: 5px;",
                    h3 { "Controls" }
                    div { style: "margin: 10px 0;",
                        label { style: "margin-right: 10px;",
                            "User ID: "
                            input {
                                r#type: "number",
                                value: "{user_id}",
                                onchange: move |e| {
                                    if let Ok(id) = e.value().parse::<u32>() {
                                        user_id.set(id);
                                    }
                                },
                                style: "padding: 4px; margin-right: 10px; width: 80px;",
                            }
                        }
                        label {
                            input {
                                r#type: "checkbox",
                                checked: *include_details.read(),
                                onchange: move |e| include_details.set(e.checked()),
                                style: "margin-right: 5px;",
                            }
                            "Include Details"
                        }
                    }
                    button {
                        onclick: move |_| show_user_data.toggle(),
                        style: "padding: 8px 16px; margin-right: 10px; background: #007cba; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        if *show_user_data.read() {
                            "Hide User Data Components"
                        } else {
                            "Show User Data Components"
                        }
                    }
                }

                if *show_user_data.read() {
                    div {
                        // Multiple Parameter Provider
                        UserDataDisplay {
                            user_id: *user_id.read(),
                            include_details: *include_details.read(),
                        }

                        // Single Parameter Provider
                        div { style: "padding: 15px; border: 1px solid #28a745; border-radius: 5px; margin: 10px 0;",
                            h4 { "User Name (ID: {user_id})" }
                            match &*user_name.read() {
                                AsyncState::Loading => rsx! {
                                    p { "⏳ Loading user name..." }
                                },
                                AsyncState::Success(name) => rsx! {
                                    p { "✅ {name}" }
                                },
                                AsyncState::Error(e) => rsx! {
                                    p { "❌ Error: {e}" }
                                },
                            }
                        }

                        // Invalidation buttons for family providers
                        div { style: "margin: 15px 0;",
                            button {
                                onclick: move |_| {
                                    println!(
                                        "🗑️ [DEBUG] Invalidating user data cache for user_id={}, include_details={}",
                                        *user_id.read(),
                                        *include_details.read(),
                                    );
                                    invalidate_user_data();
                                },
                                style: "padding: 8px 16px; margin-right: 10px; background: #007cba; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                "Invalidate User Data"
                            }
                            button {
                                onclick: move |_| {
                                    println!(
                                        "🗑️ [DEBUG] Invalidating user name cache for user_id={}",
                                        *user_id.read(),
                                    );
                                    invalidate_user_name();
                                },
                                style: "padding: 8px 16px; background: #28a745; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                "Invalidate User Name"
                            }
                        }
                    }
                }
            }

            // Time Provider Testing Section
            section { style: "margin: 30px 0; padding: 20px; background: #fff3cd; border-radius: 8px;",
                h2 { "Time Provider Test" }
                button {
                    onclick: move |_| show_time_data.toggle(),
                    style: "padding: 8px 16px; margin: 10px 0; background: #ffc107; color: black; border: none; border-radius: 4px; cursor: pointer;",
                    if *show_time_data.read() {
                        "Hide Time Data"
                    } else {
                        "Show Time Data"
                    }
                }

                if *show_time_data.read() {
                    div { style: "padding: 15px; border: 1px solid #ffc107; border-radius: 5px; margin: 10px 0;",
                        h4 { "Current Time" }
                        match &*current_time.read() {
                            AsyncState::Loading => rsx! {
                                p { "⏳ Loading current time..." }
                            },
                            AsyncState::Success(time) => rsx! {
                                p { "✅ {time}" }
                            },
                            AsyncState::Error(e) => rsx! {
                                p { "❌ Error: {e}" }
                            },
                        }
                        button {
                            onclick: move |_| {
                                println!("🗑️ [DEBUG] Invalidating current time cache");
                                invalidate_time();
                            },
                            style: "margin-top: 10px; padding: 8px 16px; background: #ffc107; color: black; border: none; border-radius: 4px; cursor: pointer;",
                            "Invalidate Time"
                        }
                    }
                }
            }

            // Interval Providers Testing Section
            section { style: "margin: 30px 0; padding: 20px; background: #d4edda; border-radius: 8px;",
                h2 { "🔄 Interval Providers (Auto-Refresh)" }
                p { "These providers automatically refresh at specified intervals in the background:" }

                // Live Clock (1-second intervals)
                div { style: "padding: 15px; border: 1px solid #17a2b8; border-radius: 5px; margin: 10px 0; background: #e7f9fc;",
                    h4 { "🕐 Live Clock (Updates every second)" }
                    match &*use_future_provider(live_clock).read() {
                        AsyncState::Loading => rsx! {
                            p { "⏳ Loading clock..." }
                        },
                        AsyncState::Success(time) => rsx! {
                            p { style: "font-weight: bold; font-size: 1.2em; color: #17a2b8;", "{time}" }
                        },
                        AsyncState::Error(e) => rsx! {
                            p { "❌ Error: {e}" }
                        },
                    }
                }

                // Live Server Status (5-second intervals)
                div { style: "padding: 15px; border: 1px solid #28a745; border-radius: 5px; margin: 10px 0;",
                    h4 { "🖥️ Live Server Status (Updates every 5 seconds)" }
                    match &*use_future_provider(live_server_status).read() {
                        AsyncState::Loading => rsx! {
                            p { "⏳ Checking server status..." }
                        },
                        AsyncState::Success(status) => rsx! {
                            p { style: "font-weight: bold;", "✅ {status}" }
                        },
                        AsyncState::Error(e) => rsx! {
                            p { "❌ Error: {e}" }
                        },
                    }
                }

                // Live Metrics (2-second intervals)
                div { style: "padding: 15px; border: 1px solid #007bff; border-radius: 5px; margin: 10px 0;",
                    h4 { "📊 Live System Metrics (Updates every 2 seconds)" }
                    match &*use_future_provider(live_metrics).read() {
                        AsyncState::Loading => rsx! {
                            p { "⏳ Loading metrics..." }
                        },
                        AsyncState::Success(metrics) => rsx! {
                            p { style: "font-weight: bold; color: #007bff;", "✅ {metrics}" }
                        },
                        AsyncState::Error(e) => rsx! {
                            p { "❌ Error: {e}" }
                        },
                    }
                }

                // User Activity with Family Provider (3-second intervals)
                div { style: "padding: 15px; border: 1px solid #6f42c1; border-radius: 5px; margin: 10px 0;",
                    h4 { "👥 User Activity Monitor (Updates every 3 seconds)" }
                    div { style: "margin: 10px 0;",
                        label { style: "margin-right: 10px;",
                            "Monitor User ID: "
                            input {
                                r#type: "number",
                                value: "{user_id}",
                                onchange: move |e| {
                                    if let Ok(id) = e.value().parse::<u32>() {
                                        user_id.set(id);
                                    }
                                },
                                style: "padding: 4px; margin-left: 5px; width: 80px;",
                            }
                        }
                    }
                    match &*use_family_provider(user_activity, *user_id.read()).read() {
                        AsyncState::Loading => rsx! {
                            p { "⏳ Checking user activity..." }
                        },
                        AsyncState::Success(activity) => rsx! {
                            p { style: "font-weight: bold; color: #6f42c1;", "✅ {activity}" }
                        },
                        AsyncState::Error(e) => rsx! {
                            p { "❌ Error: {e}" }
                        },
                    }
                }

                div { style: "margin: 15px 0; padding: 10px; background: #e9ecef; border-radius: 4px; font-size: 14px;",
                    p { "💡 **How it works**: These providers run automatically in the background using tokio intervals. You should see the times updating automatically, and console logs every few seconds as they refresh!" }
                    p { style: "margin-top: 5px;", "🎯 **Watch the timestamps**: Each provider shows when it was last updated. The clock updates every second, metrics every 2 seconds, server status every 5 seconds, and user activity every 3 seconds." }
                }
            }

            // Global Cache Management
            section { style: "margin: 30px 0; padding: 20px; background: #f8d7da; border-radius: 8px;",
                h2 { "Global Cache Management" }
                p { "Clear all cached data across all providers:" }
                button {
                    onclick: move |_| {
                        println!("🧹 [DEBUG] Clearing entire provider cache");
                        clear_cache();
                    },
                    style: "padding: 12px 24px; background: #dc3545; color: white; border: none; border-radius: 4px; cursor: pointer; font-weight: bold;",
                    "🧹 Clear All Cache"
                }
            }

            // Instructions
            section { style: "margin: 30px 0; padding: 20px; background: #d1ecf1; border-radius: 8px; font-size: 14px;",
                h3 { "Expected Behavior & Testing Guide:" }
                ul {
                    li {
                        "🔄 **Cache Hit/Miss**: First load triggers provider execution, subsequent loads use cache"
                    }
                    li {
                        "🔀 **Component Toggling**: Hiding/showing components should hit cache, not re-execute providers"
                    }
                    li {
                        "👥 **Family Providers**: Different parameter combinations create separate cache entries"
                    }
                    li {
                        "🎯 **Selective Invalidation**: Each invalidate button only affects its specific provider/parameters"
                    }
                    li {
                        "🧹 **Global Clear**: Clears all cached data, forcing all providers to re-execute"
                    }
                    li {
                        "📋 **Console Logs**: Check browser console to see provider execution and cache behavior"
                    }
                    li {
                        "🔄 **Interval Providers**: Some providers auto-refresh in the background at specified intervals"
                    }
                    li {
                        "⏱️ **Background Updates**: Watch the live data sections update automatically without user interaction"
                    }
                }
                h4 { "Test Scenarios:" }
                ol {
                    li { "Load page → observe initial provider executions" }
                    li { "Toggle components → should see cache hits (no new executions)" }
                    li { "Change user ID → should trigger new provider calls for that ID" }
                    li { "Toggle include details → should create separate cache entries" }
                    li { "Use selective invalidation → only specific providers re-execute" }
                    li { "Use global clear → all visible data reloads" }
                }
            }
        }
    }
}

fn main() {
    launch(app);
}
