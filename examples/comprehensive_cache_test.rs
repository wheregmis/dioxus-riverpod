use dioxus::prelude::*;
use dioxus_riverpod::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

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

    if include_details {
        Ok(format!("User {} with full details", user_id))
    } else {
        Ok(format!("User {} basic info", user_id))
    }
}

// Family provider with single parameter
#[provider]
async fn fetch_user_name(user_id: u32) -> Result<String, String> {
    println!("📡 [PROVIDER] Fetching user name for user_id={}", user_id);
    sleep(Duration::from_millis(300)).await;
    Ok(format!("User {}", user_id))
}

// Future provider for time testing
#[provider]
async fn fetch_current_time() -> Result<String, String> {
    println!("📡 [PROVIDER] Fetching current time");
    sleep(Duration::from_millis(200)).await;
    Ok(format!(
        "Current time: {}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ))
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
