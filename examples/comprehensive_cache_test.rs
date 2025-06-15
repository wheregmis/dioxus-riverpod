//! # Comprehensive Cache Test
//!
//! This example demonstrates all caching features of dioxus-riverpod in one comprehensive demo.
//! It showcases how different caching strategies work together and can be combined for
//! optimal data management.
//!
//! ## Features Demonstrated:
//! - Interval-based auto-refresh
//! - Stale-while-revalidate (SWR)
//! - Cache expiration with TTL
//! - Auto-dispose for memory management
//! - Manual cache invalidation
//! - Parameterized providers
//! - Error handling and recovery
//! - Performance monitoring

use dioxus::prelude::*;
use dioxus_riverpod::prelude::*;
use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

// Cross-platform time imports
#[cfg(not(target_family = "wasm"))]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_family = "wasm")]
use web_time::{SystemTime, UNIX_EPOCH};

// Cross-platform sleep function
#[cfg(not(target_family = "wasm"))]
use tokio::time::sleep;
#[cfg(target_family = "wasm")]
use wasmtimer::tokio::sleep;

/// Global counters for tracking provider behavior
static API_CALL_COUNTER: AtomicU32 = AtomicU32::new(0);
static ERROR_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Live data with interval refresh - updates every 4 seconds
#[provider(interval = "4s")]
async fn fetch_live_metrics() -> Result<LiveMetrics, String> {
    sleep(Duration::from_millis(800)).await;

    let call_number = API_CALL_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(LiveMetrics {
        cpu_usage: (call_number * 7) % 100,
        memory_usage: (call_number * 11) % 100,
        active_connections: (call_number * 3) % 50 + 10,
        timestamp,
        refresh_count: call_number,
    })
}

/// SWR data - serves stale data while revalidating after 6 seconds
#[provider(stale_time = "6s")]
async fn fetch_user_dashboard(user_id: u32) -> Result<UserDashboard, String> {
    sleep(Duration::from_millis(1200)).await;

    let call_number = API_CALL_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Simulate occasional errors
    if call_number % 8 == 0 {
        ERROR_COUNTER.fetch_add(1, Ordering::SeqCst);
        return Err("Dashboard service temporarily unavailable".to_string());
    }

    Ok(UserDashboard {
        user_id,
        notifications: generate_notifications(user_id, call_number),
        recent_activity: generate_activity(user_id, call_number),
        preferences: UserPrefs {
            theme: if call_number % 2 == 0 {
                "dark"
            } else {
                "light"
            }
            .to_string(),
            auto_refresh: call_number % 3 == 0,
            notifications_enabled: call_number % 4 != 0,
        },
        last_updated: timestamp,
        fetch_count: call_number,
    })
}

/// Cache with expiration - data expires after 10 seconds
#[provider(cache_expiration = "10s")]
async fn fetch_analytics_report() -> Result<AnalyticsReport, String> {
    sleep(Duration::from_millis(2000)).await; // Expensive operation

    let call_number = API_CALL_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(AnalyticsReport {
        page_views: (call_number * 1000) + 50000,
        unique_visitors: (call_number * 200) + 10000,
        bounce_rate: (call_number * 3) % 50 + 20,
        avg_session_duration: (call_number * 30) % 600 + 120,
        top_pages: generate_top_pages(call_number),
        generated_at: timestamp,
        processing_time_ms: 2000,
        report_id: call_number,
    })
}

/// Auto-dispose provider - cleans up after 7 seconds of no use
#[provider(auto_dispose = true, dispose_delay = "7s")]
async fn fetch_temporary_data(session_id: String) -> Result<TempSessionData, String> {
    sleep(Duration::from_millis(600)).await;

    let call_number = API_CALL_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(TempSessionData {
        session_id: session_id.clone(),
        temp_files: generate_temp_files(&session_id, call_number),
        memory_usage_mb: (call_number * 50) % 500 + 100,
        created_at: timestamp,
        instance_id: call_number,
    })
}

/// Combined strategy - SWR + Auto-dispose for optimal UX and memory management
#[provider(stale_time = "5s", auto_dispose = true, dispose_delay = "12s")]
async fn fetch_chat_messages(chat_id: u32) -> Result<ChatData, String> {
    sleep(Duration::from_millis(400)).await;

    let call_number = API_CALL_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(ChatData {
        chat_id,
        messages: generate_messages(chat_id, call_number),
        online_users: (call_number % 10) + 1,
        last_message_time: timestamp - (call_number as u64 % 300),
        fetch_timestamp: timestamp,
        message_count: call_number * 3 + 15,
    })
}

/// Data structures
#[derive(Debug, Clone, PartialEq)]
struct LiveMetrics {
    cpu_usage: u32,
    memory_usage: u32,
    active_connections: u32,
    timestamp: u64,
    refresh_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct UserDashboard {
    user_id: u32,
    notifications: Vec<Notification>,
    recent_activity: Vec<Activity>,
    preferences: UserPrefs,
    last_updated: u64,
    fetch_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct Notification {
    id: u32,
    message: String,
    priority: String,
}

#[derive(Debug, Clone, PartialEq)]
struct Activity {
    action: String,
    timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
struct UserPrefs {
    theme: String,
    auto_refresh: bool,
    notifications_enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct AnalyticsReport {
    page_views: u32,
    unique_visitors: u32,
    bounce_rate: u32,
    avg_session_duration: u32,
    top_pages: Vec<PageStats>,
    generated_at: u64,
    processing_time_ms: u32,
    report_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct PageStats {
    path: String,
    views: u32,
    unique_views: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct TempSessionData {
    session_id: String,
    temp_files: Vec<TempFile>,
    memory_usage_mb: u32,
    created_at: u64,
    instance_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct TempFile {
    name: String,
    size_kb: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct ChatData {
    chat_id: u32,
    messages: Vec<Message>,
    online_users: u32,
    last_message_time: u64,
    fetch_timestamp: u64,
    message_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct Message {
    id: u32,
    user: String,
    content: String,
    timestamp: u64,
}

/// Helper functions for generating demo data
fn generate_notifications(user_id: u32, call_number: u32) -> Vec<Notification> {
    (1..=3)
        .map(|i| Notification {
            id: call_number * 10 + i,
            message: format!("Notification {} for user {}", i, user_id),
            priority: if i % 2 == 0 { "high" } else { "normal" }.to_string(),
        })
        .collect()
}

fn generate_activity(_user_id: u32, call_number: u32) -> Vec<Activity> {
    let actions = ["login", "view_page", "edit_profile", "send_message"];
    (1..=4)
        .map(|i| Activity {
            action: actions[(call_number as usize + i) % actions.len()].to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - (i as u64 * 300),
        })
        .collect()
}

fn generate_top_pages(call_number: u32) -> Vec<PageStats> {
    let pages = ["/home", "/dashboard", "/profile", "/settings", "/help"];
    pages
        .iter()
        .enumerate()
        .map(|(i, page)| PageStats {
            path: page.to_string(),
            views: (call_number * (i as u32 + 1) * 100) % 10000 + 1000,
            unique_views: (call_number * (i as u32 + 1) * 50) % 5000 + 500,
        })
        .collect()
}

fn generate_temp_files(session_id: &str, call_number: u32) -> Vec<TempFile> {
    (1..=3)
        .map(|i| TempFile {
            name: format!("{}_{}_temp_{}.tmp", session_id, call_number, i),
            size_kb: (call_number * i * 10) % 1000 + 10,
        })
        .collect()
}

fn generate_messages(chat_id: u32, call_number: u32) -> Vec<Message> {
    let users = ["Alice", "Bob", "Charlie", "Diana"];
    (1..=5)
        .map(|i| Message {
            id: call_number * 100 + i,
            user: users[(call_number as usize + i as usize) % users.len()].to_string(),
            content: format!("Message {} in chat {}", i, chat_id),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - (i as u64 * 60),
        })
        .collect()
}

/// Main comprehensive demo component
#[component]
fn ComprehensiveCacheTest() -> Element {
    let mut selected_user_id = use_signal(|| 1u32);
    let mut selected_chat_id = use_signal(|| 1u32);
    let mut session_id = use_signal(|| "session_001".to_string());
    let mut show_temp_data = use_signal(|| true);
    let mut show_chat = use_signal(|| true);

    // Initialize all providers
    let live_metrics = use_provider(fetch_live_metrics, ());
    let user_dashboard = use_provider(fetch_user_dashboard, (*selected_user_id.read(),));
    let analytics = use_provider(fetch_analytics_report, ());
    let temp_data = use_provider(fetch_temporary_data, (session_id.read().clone(),));
    let chat_data = use_provider(fetch_chat_messages, (*selected_chat_id.read(),));

    // Manual refresh functions
    let refresh_metrics = use_invalidate_provider(fetch_live_metrics, ());
    let refresh_dashboard = use_invalidate_provider(fetch_user_dashboard, *selected_user_id.read());
    let refresh_analytics = use_invalidate_provider(fetch_analytics_report, ());
    let refresh_temp = use_invalidate_provider(fetch_temporary_data, session_id.read().clone());
    let refresh_chat = use_invalidate_provider(fetch_chat_messages, *selected_chat_id.read());

    rsx! {
        div { class: "comprehensive-demo",
            header { class: "demo-header",
                h1 { "🚀 Comprehensive Cache Feature Test" }
                p { class: "demo-description",
                    "Complete demonstration of all dioxus-riverpod caching strategies working together."
                }
                div { class: "stats-summary",
                    div { class: "stat-item",
                        span { class: "stat-label", "Total API Calls:" }
                        span { class: "stat-value", "{API_CALL_COUNTER.load(Ordering::SeqCst)}" }
                    }
                    div { class: "stat-item",
                        span { class: "stat-label", "Error Count:" }
                        span { class: "stat-value error", "{ERROR_COUNTER.load(Ordering::SeqCst)}" }
                    }
                }
            }

            div { class: "demo-controls",
                h3 { "Test Controls" }
                div { class: "control-sections",
                    div { class: "control-section",
                        h4 { "Manual Refresh" }
                        div { class: "button-group",
                            button {
                                class: "control-btn metrics",
                                onclick: move |_| refresh_metrics(),
                                "🔄 Refresh Metrics"
                            }
                            button {
                                class: "control-btn dashboard",
                                onclick: move |_| refresh_dashboard(),
                                "🔄 Refresh Dashboard"
                            }
                            button {
                                class: "control-btn analytics",
                                onclick: move |_| refresh_analytics(),
                                "🔄 Refresh Analytics"
                            }
                        }
                    }

                    div { class: "control-section",
                        h4 { "Parameters" }
                        div { class: "param-controls",
                            div { class: "param-group",
                                label { "User ID:" }
                                input {
                                    r#type: "number",
                                    value: "{selected_user_id}",
                                    min: "1",
                                    max: "10",
                                    oninput: move |e| {
                                        if let Ok(id) = e.value().parse::<u32>() {
                                            selected_user_id.set(id);
                                        }
                                    }
                                }
                            }
                            div { class: "param-group",
                                label { "Chat ID:" }
                                input {
                                    r#type: "number",
                                    value: "{selected_chat_id}",
                                    min: "1",
                                    max: "5",
                                    oninput: move |e| {
                                        if let Ok(id) = e.value().parse::<u32>() {
                                            selected_chat_id.set(id);
                                        }
                                    }
                                }
                            }
                            div { class: "param-group",
                                label { "Session ID:" }
                                input {
                                    r#type: "text",
                                    value: "{session_id}",
                                    oninput: move |e| session_id.set(e.value())
                                }
                            }
                        }
                    }

                    div { class: "control-section",
                        h4 { "Toggle Components" }
                        div { class: "toggle-controls",
                            button {
                                class: format!("toggle-btn {}", if *show_temp_data.read() { "active" } else { "inactive" }),
                                onclick: move |_| {
                                    let current = *show_temp_data.read();
                                    show_temp_data.set(!current);
                                },
                                if *show_temp_data.read() { "🟢 Temp Data" } else { "🔴 Temp Data" }
                            }
                            button {
                                class: format!("toggle-btn {}", if *show_chat.read() { "active" } else { "inactive" }),
                                onclick: move |_| {
                                    let current = *show_chat.read();
                                    show_chat.set(!current);
                                },
                                if *show_chat.read() { "🟢 Chat Data" } else { "🔴 Chat Data" }
                            }
                        }
                    }
                }
            }

            div { class: "features-grid",
                // Live Metrics (Interval Refresh)
                LiveMetricsCard { data: live_metrics }

                // User Dashboard (SWR)
                UserDashboardCard {
                    data: user_dashboard,
                    user_id: *selected_user_id.read(),
                }

                // Analytics Report (Cache Expiration)
                AnalyticsCard { data: analytics }

                // Temporary Data (Auto-dispose)
                if *show_temp_data.read() {
                    TempDataCard { data: temp_data }
                }

                // Chat Data (SWR + Auto-dispose)
                if *show_chat.read() {
                    ChatCard {
                        data: chat_data,
                        chat_id: *selected_chat_id.read(),
                    }
                }
            }

            div { class: "strategy-comparison",
                h3 { "🔍 Caching Strategy Comparison" }
                div { class: "comparison-table",
                    div { class: "comparison-header",
                        div { "Strategy" }
                        div { "Refresh Trigger" }
                        div { "Data Availability" }
                        div { "Memory Management" }
                        div { "Best Use Case" }
                    }
                    div { class: "comparison-row",
                        div { class: "strategy-name interval", "Interval" }
                        div { "Time-based (4s)" }
                        div { "Always fresh" }
                        div { "Manual cleanup" }
                        div { "Live monitoring" }
                    }
                    div { class: "comparison-row",
                        div { class: "strategy-name swr", "SWR" }
                        div { "On stale (6s)" }
                        div { "Instant (stale ok)" }
                        div { "Manual cleanup" }
                        div { "User dashboards" }
                    }
                    div { class: "comparison-row",
                        div { class: "strategy-name cache", "Cache TTL" }
                        div { "On expiration (10s)" }
                        div { "Loading on miss" }
                        div { "Auto cleanup" }
                        div { "Heavy computations" }
                    }
                    div { class: "comparison-row",
                        div { class: "strategy-name dispose", "Auto-dispose" }
                        div { "Manual refresh" }
                        div { "Loading on miss" }
                        div { "Auto cleanup (7s)" }
                        div { "Session data" }
                    }
                    div { class: "comparison-row",
                        div { class: "strategy-name combined", "SWR + Dispose" }
                        div { "On stale (5s)" }
                        div { "Instant (stale ok)" }
                        div { "Auto cleanup (12s)" }
                        div { "Chat/messaging" }
                    }
                }
            }

            footer { class: "demo-footer",
                p { class: "instructions",
                    "💡 Experiment with different parameters and watch how each caching strategy behaves!"
                }
            }
        }

        style { {include_str!("./assets/comprehensive_cache_styles.css")} }
    }
}

/// Specific card components for each feature
#[component]
fn LiveMetricsCard(data: Signal<AsyncState<LiveMetrics, String>>) -> Element {
    let status_class = match &*data.read() {
        AsyncState::Loading => "loading",
        AsyncState::Success(_) => "success",
        AsyncState::Error(_) => "error",
    };

    rsx! {
        div { class: "feature-card metrics",
            div { class: "card-header",
                div { class: "card-title-section",
                    h3 { class: "card-title", "📡 Live Metrics (Interval: 4s)" }
                    p { class: "card-strategy", "Auto-refresh every 4 seconds" }
                }
                div { class: format!("status-indicator {}", status_class),
                    match status_class {
                        "loading" => "🔄",
                        "success" => "✅",
                        "error" => "❌",
                        _ => "❓",
                    }
                }
            }
            div { class: "card-content",
                match &*data.read() {
                    AsyncState::Loading => rsx! {
                        div { class: "loading-state",
                            div { class: "loading-spinner" }
                            span { "Loading data..." }
                        }
                    },
                    AsyncState::Error(e) => rsx! {
                        div { class: "error-state",
                            span { class: "error-icon", "❌" }
                            span { class: "error-message", "Error: {e}" }
                        }
                    },
                    AsyncState::Success(metrics) => rsx! {
                        div { class: "metrics-content",
                            div { class: "metric-item",
                                span { class: "metric-label", "CPU Usage:" }
                                div { class: "metric-bar",
                                    div {
                                        class: "metric-fill cpu",
                                        style: "width: {metrics.cpu_usage}%"
                                    }
                                    span { class: "metric-value", "{metrics.cpu_usage}%" }
                                }
                            }
                            div { class: "metric-item",
                                span { class: "metric-label", "Memory Usage:" }
                                div { class: "metric-bar",
                                    div {
                                        class: "metric-fill memory",
                                        style: "width: {metrics.memory_usage}%"
                                    }
                                    span { class: "metric-value", "{metrics.memory_usage}%" }
                                }
                            }
                            div { class: "metric-item",
                                span { class: "metric-label", "Connections:" }
                                span { class: "metric-value", "{metrics.active_connections}" }
                            }
                            div { class: "refresh-info",
                                "Refresh #{metrics.refresh_count} at {format_timestamp(metrics.timestamp)}"
                            }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn UserDashboardCard(data: Signal<AsyncState<UserDashboard, String>>, user_id: u32) -> Element {
    let status_class = match &*data.read() {
        AsyncState::Loading => "loading",
        AsyncState::Success(_) => "success",
        AsyncState::Error(_) => "error",
    };

    rsx! {
        div { class: "feature-card dashboard",
            div { class: "card-header",
                div { class: "card-title-section",
                    h3 { class: "card-title", "👤 User Dashboard (SWR: 6s) - User {user_id}" }
                    p { class: "card-strategy", "Stale-while-revalidate after 6 seconds" }
                }
                div { class: format!("status-indicator {}", status_class),
                    match status_class {
                        "loading" => "🔄",
                        "success" => "✅",
                        "error" => "❌",
                        _ => "❓",
                    }
                }
            }
            div { class: "card-content",
                match &*data.read() {
                    AsyncState::Loading => rsx! {
                        div { class: "loading-state",
                            div { class: "loading-spinner" }
                            span { "Loading data..." }
                        }
                    },
                    AsyncState::Error(e) => rsx! {
                        div { class: "error-state",
                            span { class: "error-icon", "❌" }
                            span { class: "error-message", "Error: {e}" }
                        }
                    },
                    AsyncState::Success(dashboard) => rsx! {
                        div { class: "dashboard-content",
                            div { class: "preferences",
                                h5 { "Preferences:" }
                                p { "🎨 Theme: {dashboard.preferences.theme}" }
                                p { "🔄 Auto-refresh: {dashboard.preferences.auto_refresh}" }
                                p { "🔔 Notifications: {dashboard.preferences.notifications_enabled}" }
                            }
                            div { class: "notifications",
                                h5 { "Recent Notifications:" }
                                for notification in &dashboard.notifications {
                                    div { class: format!("notification {}", notification.priority),
                                        "{notification.message}"
                                    }
                                }
                            }
                            div { class: "fetch-info",
                                "Fetch #{dashboard.fetch_count} at {format_timestamp(dashboard.last_updated)}"
                            }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn AnalyticsCard(data: Signal<AsyncState<AnalyticsReport, String>>) -> Element {
    let status_class = match &*data.read() {
        AsyncState::Loading => "loading",
        AsyncState::Success(_) => "success",
        AsyncState::Error(_) => "error",
    };

    rsx! {
        div { class: "feature-card analytics",
            div { class: "card-header",
                div { class: "card-title-section",
                    h3 { class: "card-title", "📊 Analytics Report (TTL: 10s)" }
                    p { class: "card-strategy", "Traditional cache expiration after 10 seconds" }
                }
                div { class: format!("status-indicator {}", status_class),
                    match status_class {
                        "loading" => "🔄",
                        "success" => "✅",
                        "error" => "❌",
                        _ => "❓",
                    }
                }
            }
            div { class: "card-content",
                match &*data.read() {
                    AsyncState::Loading => rsx! {
                        div { class: "loading-state",
                            div { class: "loading-spinner" }
                            span { "Loading data..." }
                        }
                    },
                    AsyncState::Error(e) => rsx! {
                        div { class: "error-state",
                            span { class: "error-icon", "❌" }
                            span { class: "error-message", "Error: {e}" }
                        }
                    },
                    AsyncState::Success(report) => rsx! {
                        div { class: "analytics-content",
                            div { class: "analytics-summary",
                                div { class: "summary-item",
                                    span { class: "summary-label", "Page Views:" }
                                    span { class: "summary-value", "{report.page_views}" }
                                }
                                div { class: "summary-item",
                                    span { class: "summary-label", "Unique Visitors:" }
                                    span { class: "summary-value", "{report.unique_visitors}" }
                                }
                                div { class: "summary-item",
                                    span { class: "summary-label", "Bounce Rate:" }
                                    span { class: "summary-value", "{report.bounce_rate}%" }
                                }
                            }
                            div { class: "top-pages",
                                h5 { "Top Pages:" }
                                for page in report.top_pages.iter().take(3) {
                                    div { class: "page-stat",
                                        span { class: "page-path", "{page.path}" }
                                        span { class: "page-views", "{page.views} views" }
                                    }
                                }
                            }
                            div { class: "report-info",
                                "Report #{report.report_id} - Processing: {report.processing_time_ms}ms"
                            }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn TempDataCard(data: Signal<AsyncState<TempSessionData, String>>) -> Element {
    let status_class = match &*data.read() {
        AsyncState::Loading => "loading",
        AsyncState::Success(_) => "success",
        AsyncState::Error(_) => "error",
    };

    rsx! {
        div { class: "feature-card temp",
            div { class: "card-header",
                div { class: "card-title-section",
                    h3 { class: "card-title", "🗑️ Temp Session Data (Auto-dispose: 7s)" }
                    p { class: "card-strategy", "Auto-dispose after 7 seconds of no use" }
                }
                div { class: format!("status-indicator {}", status_class),
                    match status_class {
                        "loading" => "🔄",
                        "success" => "✅",
                        "error" => "❌",
                        _ => "❓",
                    }
                }
            }
            div { class: "card-content",
                match &*data.read() {
                    AsyncState::Loading => rsx! {
                        div { class: "loading-state",
                            div { class: "loading-spinner" }
                            span { "Loading data..." }
                        }
                    },
                    AsyncState::Error(e) => rsx! {
                        div { class: "error-state",
                            span { class: "error-icon", "❌" }
                            span { class: "error-message", "Error: {e}" }
                        }
                    },
                    AsyncState::Success(temp) => rsx! {
                        div { class: "temp-content",
                            p { class: "session-id", "Session: {temp.session_id}" }
                            p { class: "memory-usage", "Memory: {temp.memory_usage_mb} MB" }
                            div { class: "temp-files",
                                h5 { "Temporary Files:" }
                                for file in &temp.temp_files {
                                    div { class: "temp-file",
                                        span { class: "file-name", "{file.name}" }
                                        span { class: "file-size", "{file.size_kb} KB" }
                                    }
                                }
                            }
                            div { class: "instance-info",
                                "Instance #{temp.instance_id} created at {format_timestamp(temp.created_at)}"
                            }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn ChatCard(data: Signal<AsyncState<ChatData, String>>, chat_id: u32) -> Element {
    let status_class = match &*data.read() {
        AsyncState::Loading => "loading",
        AsyncState::Success(_) => "success",
        AsyncState::Error(_) => "error",
    };

    rsx! {
        div { class: "feature-card chat",
            div { class: "card-header",
                div { class: "card-title-section",
                    h3 { class: "card-title", "💬 Chat {chat_id} (SWR: 5s + Auto-dispose: 12s)" }
                    p { class: "card-strategy", "Combined SWR and auto-dispose strategy" }
                }
                div { class: format!("status-indicator {}", status_class),
                    match status_class {
                        "loading" => "🔄",
                        "success" => "✅",
                        "error" => "❌",
                        _ => "❓",
                    }
                }
            }
            div { class: "card-content",
                match &*data.read() {
                    AsyncState::Loading => rsx! {
                        div { class: "loading-state",
                            div { class: "loading-spinner" }
                            span { "Loading data..." }
                        }
                    },
                    AsyncState::Error(e) => rsx! {
                        div { class: "error-state",
                            span { class: "error-icon", "❌" }
                            span { class: "error-message", "Error: {e}" }
                        }
                    },
                    AsyncState::Success(chat) => rsx! {
                        div { class: "chat-content",
                            div { class: "chat-header",
                                span { class: "online-users", "👥 {chat.online_users} online" }
                                span { class: "message-count", "💬 {chat.message_count} messages" }
                            }
                            div { class: "recent-messages",
                                h5 { "Recent Messages:" }
                                for message in chat.messages.iter().take(3) {
                                    div { class: "message",
                                        span { class: "message-user", "{message.user}:" }
                                        span { class: "message-content", "{message.content}" }
                                        span { class: "message-time", "{format_timestamp(message.timestamp)}" }
                                    }
                                }
                            }
                            div { class: "fetch-info",
                                "Fetched at {format_timestamp(chat.fetch_timestamp)}"
                            }
                        }
                    },
                }
            }
        }
    }
}

/// Helper function to format timestamps
fn format_timestamp(timestamp: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let diff = now.saturating_sub(timestamp);

    if diff < 60 {
        format!("{}s ago", diff)
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else {
        format!("{}h ago", diff / 3600)
    }
}

/// Application root with context setup
fn app() -> Element {
    rsx! {
        RiverpodProvider {
            ComprehensiveCacheTest {}
        }
    }
}

/// Provider component that sets up all required dioxus-riverpod contexts
#[component]
fn RiverpodProvider(children: Element) -> Element {
    use_context_provider(dioxus_riverpod::cache::ProviderCache::new);
    use_context_provider(dioxus_riverpod::refresh::RefreshRegistry::new);

    let cache = use_context::<dioxus_riverpod::cache::ProviderCache>();
    use_context_provider(move || dioxus_riverpod::disposal::DisposalRegistry::new(cache.clone()));

    rsx! { {children} }
}

fn main() {
    launch(app);
}
