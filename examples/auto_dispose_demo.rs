//! # Intelligent Cache Management Demo
//!
//! This example demonstrates the intelligent cache management functionality in dioxus-riverpod.
//! The new system provides automatic cache cleanup based on LRU (Least Recently Used) policy
//! and time-based expiration, replacing the old auto-dispose system.
//!
//! **Updated to use Global Providers**: This example now uses the new global provider
//! system for simplified setup. No RiverpodProvider wrapper component needed!
//!
//! ## Key Features Demonstrated:
//! - Intelligent cache expiration with configurable times
//! - LRU-based cleanup when cache grows too large
//! - Access-based tracking for optimal memory usage
//! - Background cleanup tasks
//! - Resource lifecycle management
//! - Global provider management

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

/// Global counters for tracking provider lifecycle
static CREATION_COUNTER: AtomicU32 = AtomicU32::new(0);
static ACTIVE_PROVIDERS: AtomicU32 = AtomicU32::new(0);

/// Fast expiring cache provider - expires after 3 seconds
#[provider(cache_expiration = "3s")]
async fn fetch_quick_dispose_data() -> Result<QuickDisposeData, String> {
    let creation_id = CREATION_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    let active_count = ACTIVE_PROVIDERS.fetch_add(1, Ordering::SeqCst) + 1;

    // Simulate provider setup work
    sleep(Duration::from_millis(500)).await;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(QuickDisposeData {
        id: creation_id,
        message: format!("Quick dispose provider #{}", creation_id),
        created_at: timestamp,
        active_providers: active_count,
        dispose_delay: 3,
    })
}

/// Medium expiring cache provider - expires after 8 seconds
#[provider(cache_expiration = "8s")]
async fn fetch_medium_dispose_data() -> Result<MediumDisposeData, String> {
    let creation_id = CREATION_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    let active_count = ACTIVE_PROVIDERS.fetch_add(1, Ordering::SeqCst) + 1;

    // Simulate expensive setup
    sleep(Duration::from_millis(1200)).await;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(MediumDisposeData {
        id: creation_id,
        title: format!("Medium-lived provider #{}", creation_id),
        expensive_data: generate_expensive_data(creation_id),
        created_at: timestamp,
        active_providers: active_count,
        dispose_delay: 8,
    })
}

/// Parameterized cache provider - user-specific data with cache expiration
#[provider(cache_expiration = "5s")]
async fn fetch_user_session_data(user_id: u32) -> Result<UserSessionData, String> {
    let creation_id = CREATION_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    let active_count = ACTIVE_PROVIDERS.fetch_add(1, Ordering::SeqCst) + 1;

    // Simulate user data fetching
    sleep(Duration::from_millis(800)).await;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(UserSessionData {
        id: creation_id,
        user_id,
        session_token: format!("session_{}_{}", user_id, creation_id),
        permissions: generate_user_permissions(user_id),
        last_activity: timestamp,
        created_at: timestamp,
        active_providers: active_count,
        dispose_delay: 5,
    })
}

/// Heavy resource provider - simulates expensive resources with cache expiration
#[provider(cache_expiration = "6s")]
async fn fetch_heavy_resource() -> Result<HeavyResource, String> {
    let creation_id = CREATION_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    let active_count = ACTIVE_PROVIDERS.fetch_add(1, Ordering::SeqCst) + 1;

    // Simulate heavy resource allocation
    sleep(Duration::from_millis(2000)).await;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(HeavyResource {
        id: creation_id,
        name: format!("Heavy Resource #{}", creation_id),
        memory_usage_mb: (creation_id * 50) % 500 + 100,
        connections: generate_connections(creation_id),
        created_at: timestamp,
        active_providers: active_count,
        dispose_delay: 6,
    })
}

/// Data structures for our demo
#[derive(Debug, Clone, PartialEq)]
struct QuickDisposeData {
    id: u32,
    message: String,
    created_at: u64,
    active_providers: u32,
    dispose_delay: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct MediumDisposeData {
    id: u32,
    title: String,
    expensive_data: Vec<String>,
    created_at: u64,
    active_providers: u32,
    dispose_delay: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct UserSessionData {
    id: u32,
    user_id: u32,
    session_token: String,
    permissions: Vec<String>,
    last_activity: u64,
    created_at: u64,
    active_providers: u32,
    dispose_delay: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct HeavyResource {
    id: u32,
    name: String,
    memory_usage_mb: u32,
    connections: Vec<Connection>,
    created_at: u64,
    active_providers: u32,
    dispose_delay: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct Connection {
    id: String,
    status: String,
    latency_ms: u32,
}

/// Helper functions to generate demo data
fn generate_expensive_data(id: u32) -> Vec<String> {
    (1..=5)
        .map(|i| format!("Expensive computation result {}-{}", id, i))
        .collect()
}

fn generate_user_permissions(user_id: u32) -> Vec<String> {
    let base_permissions = vec!["read".to_string(), "write".to_string()];
    let mut permissions = base_permissions;

    if user_id % 3 == 0 {
        permissions.push("admin".to_string());
    }
    if user_id % 2 == 0 {
        permissions.push("delete".to_string());
    }

    permissions
}

fn generate_connections(id: u32) -> Vec<Connection> {
    (1..=3)
        .map(|i| Connection {
            id: format!("conn_{}_{}", id, i),
            status: if i % 2 == 0 { "active" } else { "idle" }.to_string(),
            latency_ms: (id * i * 10) % 200 + 10,
        })
        .collect()
}

/// Main auto-dispose demo component
#[component]
fn AutoDisposeDemo() -> Element {
    let mut show_quick = use_signal(|| true);
    let mut show_medium = use_signal(|| true);
    let mut show_heavy = use_signal(|| true);
    let mut selected_user_id = use_signal(|| 1u32);
    let mut show_user_session = use_signal(|| true);

    rsx! {
        div { class: "auto-dispose-demo",
            header { class: "demo-header",
                h1 { "🧠 Intelligent Cache Management Demo" }
                p { class: "demo-description",
                    "Intelligent cache management automatically cleans up expired and least-recently-used providers to optimize memory usage."
                }
                div { class: "dispose-times-info",
                    span { class: "dispose-badge quick", "Quick: 3s" }
                    span { class: "dispose-badge medium", "Medium: 8s" }
                    span { class: "dispose-badge user", "User: 5s" }
                    span { class: "dispose-badge heavy", "Heavy: 6s" }
                }
                div { class: "provider-stats",
                    span { class: "stat", "Total Created: {CREATION_COUNTER.load(Ordering::SeqCst)}" }
                    span { class: "stat", "Currently Active: {ACTIVE_PROVIDERS.load(Ordering::SeqCst)}" }
                }
            }

            div { class: "demo-controls",
                h3 { "Toggle Components (to test cache management):" }
                div { class: "toggle-grid",
                    ToggleControl {
                        label: "Quick Dispose Component",
                        is_shown: *show_quick.read(),
                        on_toggle: move |_| {
                            let current = *show_quick.read();
                            show_quick.set(!current);
                        },
                        dispose_time: "3s",
                        color_class: "quick",
                    }
                    ToggleControl {
                        label: "Medium Dispose Component",
                        is_shown: *show_medium.read(),
                        on_toggle: move |_| {
                            let current = *show_medium.read();
                            show_medium.set(!current);
                        },
                        dispose_time: "8s",
                        color_class: "medium",
                    }
                    ToggleControl {
                        label: "Heavy Resource Component",
                        is_shown: *show_heavy.read(),
                        on_toggle: move |_| {
                            let current = *show_heavy.read();
                            show_heavy.set(!current);
                        },
                        dispose_time: "6s",
                        color_class: "heavy",
                    }
                    ToggleControl {
                        label: "User Session Component",
                        is_shown: *show_user_session.read(),
                        on_toggle: move |_| {
                            let current = *show_user_session.read();
                            show_user_session.set(!current);
                        },
                        dispose_time: "5s",
                        color_class: "user",
                    }
                }
                div { class: "user-selector",
                    label { "User ID for session data: " }
                    input {
                        r#type: "number",
                        value: "{selected_user_id}",
                        min: "1",
                        max: "10",
                        oninput: move |e| {
                            if let Ok(id) = e.value().parse::<u32>() {
                                selected_user_id.set(id);
                            }
                        },
                    }
                }
            }

            div { class: "components-grid",
                if *show_quick.read() {
                    QuickDisposeCard {}
                }

                if *show_medium.read() {
                    MediumDisposeCard {}
                }

                if *show_heavy.read() {
                    HeavyDisposeCard {}
                }

                if *show_user_session.read() {
                    UserSessionCard { user_id: *selected_user_id.read() }
                }
            }

            div { class: "dispose-explanation",
                h3 { "🧠 Intelligent Cache Behavior" }
                div { class: "explanation-grid",
                    div { class: "explanation-card",
                        h4 { "Component Mount" }
                        p { "Provider is created and cached when component first mounts" }
                    }
                    div { class: "explanation-card",
                        h4 { "Active Usage" }
                        p { "Provider stays alive as long as components are using it" }
                    }
                    div { class: "explanation-card",
                        h4 { "Component Unmount" }
                        p { "Disposal timer starts when no components are using the provider" }
                    }
                    div { class: "explanation-card",
                        h4 { "Auto Cleanup" }
                        p { "Provider is disposed after the specified delay expires" }
                    }
                }
                div { class: "memory-benefits",
                    h4 { "🎯 Memory Management Benefits" }
                    ul {
                        li { "Prevents memory leaks in dynamic UIs" }
                        li { "Automatically cleans up expensive resources" }
                        li { "Reduces memory footprint for unused data" }
                        li { "Perfect for user-specific or temporary data" }
                    }
                }
            }

            footer { class: "demo-footer",
                p { class: "instructions",
                    "💡 Toggle components off and watch cache entries expire after their configured times!"
                }
            }
        }

        style { {include_str!("./assets/auto_dispose_styles.css")} }
    }
}

/// Reusable toggle control component
#[component]
fn ToggleControl(
    label: String,
    is_shown: bool,
    on_toggle: EventHandler<()>,
    dispose_time: &'static str,
    color_class: &'static str,
) -> Element {
    rsx! {
        div { class: "toggle-control",
            div { class: "toggle-info",
                span { class: "toggle-label", "{label}" }
                span { class: format!("dispose-time {}", color_class), "⏰ {dispose_time}" }
            }
            button {
                class: format!(
                    "toggle-btn {} {}",
                    color_class,
                    if is_shown { "active" } else { "inactive" },
                ),
                onclick: move |_| on_toggle.call(()),
                if is_shown {
                    "🟢 Mounted"
                } else {
                    "🔴 Unmounted"
                }
            }
        }
    }
}

/// Reusable auto-dispose card component
#[component]
fn QuickDisposeCard() -> Element {
    let data = use_provider(fetch_quick_dispose_data, ());

    rsx! {
        div { class: "auto-dispose-card quick",
            div { class: "card-header",
                h3 { "⚡ Quick Dispose" }
                div { class: "disposal-status",
                    match &*data.read() {
                        AsyncState::Loading => "🔄 Loading",
                        AsyncState::Success(_) => "✅ Active",
                        AsyncState::Error(_) => "❌ Error",
                    }
                }
            }
            div { class: "card-content",
                match &*data.read() {
                    AsyncState::Loading => rsx! {
                        div { class: "loading-container",
                            div { class: "loading-spinner" }
                            span { "Creating provider..." }
                        }
                    },
                    AsyncState::Error(e) => rsx! {
                        div { class: "error-container",
                            span { class: "error-icon", "❌" }
                            span { class: "error-message", "Error: {e}" }
                        }
                    },
                    AsyncState::Success(value) => rsx! {
                        div { class: "data-content",
                            h4 { "{value.message}" }
                            ProviderStats {
                                id: value.id,
                                created_at: value.created_at,
                                active_providers: value.active_providers,
                                dispose_delay: value.dispose_delay,
                            }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn MediumDisposeCard() -> Element {
    let data = use_provider(fetch_medium_dispose_data, ());

    rsx! {
        div { class: "auto-dispose-card medium",
            div { class: "card-header",
                h3 { "🔄 Medium Dispose" }
                div { class: "disposal-status",
                    match &*data.read() {
                        AsyncState::Loading => "🔄 Loading",
                        AsyncState::Success(_) => "✅ Active",
                        AsyncState::Error(_) => "❌ Error",
                    }
                }
            }
            div { class: "card-content",
                match &*data.read() {
                    AsyncState::Loading => rsx! {
                        div { class: "loading-container",
                            div { class: "loading-spinner" }
                            span { "Creating provider..." }
                        }
                    },
                    AsyncState::Error(e) => rsx! {
                        div { class: "error-container",
                            span { class: "error-icon", "❌" }
                            span { class: "error-message", "Error: {e}" }
                        }
                    },
                    AsyncState::Success(value) => rsx! {
                        div { class: "data-content",
                            h4 { "{value.title}" }
                            div { class: "expensive-data",
                                h5 { "Expensive Computations:" }
                                ul {
                                    for item in &value.expensive_data {
                                        li { "{item}" }
                                    }
                                }
                            }
                            ProviderStats {
                                id: value.id,
                                created_at: value.created_at,
                                active_providers: value.active_providers,
                                dispose_delay: value.dispose_delay,
                            }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn HeavyDisposeCard() -> Element {
    let data = use_provider(fetch_heavy_resource, ());

    rsx! {
        div { class: "auto-dispose-card heavy",
            div { class: "card-header",
                h3 { "🏋️ Heavy Resource" }
                div { class: "disposal-status",
                    match &*data.read() {
                        AsyncState::Loading => "🔄 Loading",
                        AsyncState::Success(_) => "✅ Active",
                        AsyncState::Error(_) => "❌ Error",
                    }
                }
            }
            div { class: "card-content",
                match &*data.read() {
                    AsyncState::Loading => rsx! {
                        div { class: "loading-container",
                            div { class: "loading-spinner" }
                            span { "Creating provider..." }
                        }
                    },
                    AsyncState::Error(e) => rsx! {
                        div { class: "error-container",
                            span { class: "error-icon", "❌" }
                            span { class: "error-message", "Error: {e}" }
                        }
                    },
                    AsyncState::Success(value) => rsx! {
                        div { class: "data-content",
                            h4 { "{value.name}" }
                            div { class: "resource-info",
                                p { "💾 Memory Usage: {value.memory_usage_mb} MB" }
                                div { class: "connections",
                                    h5 { "Active Connections:" }
                                    for conn in &value.connections {
                                        div { class: "connection",
                                            span { class: "conn-id", "{conn.id}" }
                                            span { class: format!("conn-status {}", conn.status), "{conn.status}" }
                                            span { class: "conn-latency", "{conn.latency_ms}ms" }
                                        }
                                    }
                                }
                            }
                            ProviderStats {
                                id: value.id,
                                created_at: value.created_at,
                                active_providers: value.active_providers,
                                dispose_delay: value.dispose_delay,
                            }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn UserSessionCard(user_id: u32) -> Element {
    let data = use_provider(fetch_user_session_data, (user_id,));

    rsx! {
        div { class: "auto-dispose-card user",
            div { class: "card-header",
                h3 { "👤 User Session (ID: {user_id})" }
                div { class: "disposal-status",
                    match &*data.read() {
                        AsyncState::Loading => "🔄 Loading",
                        AsyncState::Success(_) => "✅ Active",
                        AsyncState::Error(_) => "❌ Error",
                    }
                }
            }
            div { class: "card-content",
                match &*data.read() {
                    AsyncState::Loading => rsx! {
                        div { class: "loading-container",
                            div { class: "loading-spinner" }
                            span { "Creating provider..." }
                        }
                    },
                    AsyncState::Error(e) => rsx! {
                        div { class: "error-container",
                            span { class: "error-icon", "❌" }
                            span { class: "error-message", "Error: {e}" }
                        }
                    },
                    AsyncState::Success(value) => rsx! {
                        div { class: "data-content",
                            h4 { "User {value.user_id} Session" }
                            div { class: "session-info",
                                p { class: "session-token", "🔑 Token: {value.session_token}" }
                                div { class: "permissions",
                                    h5 { "Permissions:" }
                                    div { class: "permission-tags",
                                        for permission in &value.permissions {
                                            span { class: "permission-tag", "{permission}" }
                                        }
                                    }
                                }
                                p { class: "last-activity", "⏰ Last Activity: {format_timestamp(value.last_activity)}" }
                            }
                            ProviderStats {
                                id: value.id,
                                created_at: value.created_at,
                                active_providers: value.active_providers,
                                dispose_delay: value.dispose_delay,
                            }
                        }
                    },
                }
            }
        }
    }
}

/// Component to display provider statistics
#[component]
fn ProviderStats(id: u32, created_at: u64, active_providers: u32, dispose_delay: u32) -> Element {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let age = current_time.saturating_sub(created_at);

    rsx! {
        div { class: "provider-stats",
            p { class: "provider-id", "🆔 Provider ID: #{id}" }
            p { class: "creation-time", "⏰ Created: {format_timestamp(created_at)}" }
            p { class: "provider-age", "📅 Age: {age}s" }
            p { class: "dispose-delay", "🗑️ Dispose Delay: {dispose_delay}s" }
            p { class: "active-count", "📊 Active Providers: {active_providers}" }
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

/// Application root - now using global providers for simplified setup
fn app() -> Element {
    rsx! {
        AutoDisposeDemo {}
    }
}

fn main() {
    // Initialize global providers at application startup
    dioxus_riverpod::global::init_global_providers();

    launch(app);
}
