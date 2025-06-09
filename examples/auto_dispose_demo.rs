//! # Auto-Dispose Demo
//!
//! This example demonstrates the auto-dispose functionality that automatically
//! cleans up unused providers and their cached data to prevent memory leaks.
//!
//! ## Features Demonstrated
//!
//! - Auto-dispose providers with configurable delays
//! - Reference counting for active provider usage
//! - Disposal cancellation when providers are accessed again
//! - Memory management and cleanup
//!
//! ## How It Works
//!
//! 1. Providers with `auto_dispose = true` track their usage via reference counting
//! 2. When a component unmounts, it decrements the reference count
//! 3. If no references remain, a disposal timer is scheduled
//! 4. If the provider is accessed again before the timer expires, disposal is cancelled
//! 5. Otherwise, the provider's cached data is cleaned up automatically

use dioxus::prelude::*;
use dioxus_riverpod::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

// Auto-dispose provider that will be cleaned up after 5 seconds of no usage
#[provider(auto_dispose = true, dispose_delay = "5s")]
async fn auto_dispose_data() -> Result<String, String> {
    println!("🔄 [AUTO-DISPOSE] Fetching auto-dispose data...");
    sleep(Duration::from_millis(500)).await;
    Ok(format!(
        "Auto-dispose data fetched at: {}",
        chrono::Utc::now().format("%H:%M:%S")
    ))
}

// Regular provider for comparison (no auto-dispose)
#[provider]
async fn regular_data() -> Result<String, String> {
    println!("🔄 [REGULAR] Fetching regular data...");
    sleep(Duration::from_millis(500)).await;
    Ok(format!(
        "Regular data fetched at: {}",
        chrono::Utc::now().format("%H:%M:%S")
    ))
}

// Parameterized auto-dispose provider
#[provider(auto_dispose = true, dispose_delay = "3s")]
async fn user_profile(user_id: u32) -> Result<String, String> {
    println!(
        "🔄 [AUTO-DISPOSE] Fetching user profile for ID: {}",
        user_id
    );
    sleep(Duration::from_millis(300)).await;
    Ok(format!(
        "User {} profile fetched at: {}",
        user_id,
        chrono::Utc::now().format("%H:%M:%S")
    ))
}

#[component]
fn AutoDisposeComponent() -> Element {
    let mut show_auto_dispose = use_signal(|| true);
    let mut show_user = use_signal(|| true);
    let mut user_id = use_signal(|| 1u32);

    rsx! {
        div {
            style: "padding: 20px; font-family: monospace;",

            h2 { "🗑️ Auto-Dispose Demo" }

            p {
                "This demo shows auto-dispose providers that automatically clean up "
                "their cached data when no longer in use. Watch the console for disposal messages."
            }

            div {
                style: "margin: 20px 0; padding: 15px; border: 1px solid #ccc; border-radius: 5px;",

                h3 { "Auto-Dispose Provider (5s delay)" }

                button {
                    onclick: move |_| show_auto_dispose.set(!show_auto_dispose()),
                    if show_auto_dispose() { "Hide Auto-Dispose Data" } else { "Show Auto-Dispose Data" }
                }

                if show_auto_dispose() {
                    AutoDisposeDataComponent {}
                }

                p {
                    style: "font-size: 12px; color: #666; margin-top: 10px;",
                    "Toggle this off and watch console - data will be disposed after 5 seconds"
                }
            }

            div {
                style: "margin: 20px 0; padding: 15px; border: 1px solid #ccc; border-radius: 5px;",

                h3 { "Parameterized Auto-Dispose Provider (3s delay)" }

                div { style: "margin-bottom: 10px;",
                    "User ID: "
                    input {
                        r#type: "number",
                        value: "{user_id}",
                        onchange: move |evt| {
                            if let Ok(id) = evt.value().parse::<u32>() {
                                user_id.set(id);
                            }
                        }
                    }
                }

                button {
                    onclick: move |_| show_user.set(!show_user()),
                    if show_user() { "Hide User Profile" } else { "Show User Profile" }
                }

                if show_user() {
                    UserProfileComponent { user_id: user_id() }
                }

                p {
                    style: "font-size: 12px; color: #666; margin-top: 10px;",
                    "Change user ID or toggle off - each user's data is disposed independently"
                }
            }

            div {
                style: "margin: 20px 0; padding: 15px; border: 1px solid #ccc; border-radius: 5px;",

                h3 { "Regular Provider (No Auto-Dispose)" }

                RegularDataComponent {}

                p {
                    style: "font-size: 12px; color: #666; margin-top: 10px;",
                    "This provider's data persists indefinitely (no auto-dispose)"
                }
            }

            div {
                style: "margin-top: 30px; padding: 15px; background: #f5f5f5; border-radius: 5px;",

                h4 { "💡 Instructions" }
                ul {
                    li { "Open browser console to see disposal messages" }
                    li { "Toggle providers off and wait for disposal messages" }
                    li { "Toggle back on before disposal to see cancellation" }
                    li { "Change user ID to see independent disposal per parameter" }
                    li { "Compare with regular provider that never gets disposed" }
                }
            }
        }
    }
}

#[component]
fn AutoDisposeDataComponent() -> Element {
    let data = use_provider(auto_dispose_data, ());

    rsx! {
        div {
            match data() {
                AsyncState::Loading => rsx! {
                    p { style: "color: #666;", "🔄 Loading auto-dispose data..." }
                },
                AsyncState::Success(data) => rsx! {
                    p { style: "color: #28a745;", "✅ {data}" }
                },
                AsyncState::Error(error) => rsx! {
                    p { style: "color: #dc3545;", "❌ Error: {error}" }
                },
            }
        }
    }
}

#[component]
fn UserProfileComponent(user_id: u32) -> Element {
    let profile = use_provider(user_profile, (user_id,));

    rsx! {
        div {
            match profile() {
                AsyncState::Loading => rsx! {
                    p { style: "color: #666;", "🔄 Loading user {user_id} profile..." }
                },
                AsyncState::Success(data) => rsx! {
                    p { style: "color: #28a745;", "✅ {data}" }
                },
                AsyncState::Error(error) => rsx! {
                    p { style: "color: #dc3545;", "❌ Error: {error}" }
                },
            }
        }
    }
}

#[component]
fn RegularDataComponent() -> Element {
    let data = use_provider(regular_data, ());

    rsx! {
        div {
            match data() {
                AsyncState::Loading => rsx! {
                    p { style: "color: #666;", "🔄 Loading regular data..." }
                },
                AsyncState::Success(data) => rsx! {
                    p { style: "color: #28a745;", "✅ {data}" }
                },
                AsyncState::Error(error) => rsx! {
                    p { style: "color: #dc3545;", "❌ Error: {error}" }
                },
            }
        }
    }
}

fn main() {
    // Initialize the logger to see disposal messages in console
    env_logger::init();

    dioxus::launch(|| {
        rsx! {
            // Provide the necessary contexts for dioxus-riverpod
            ProviderCacheProvider {
                RefreshRegistryProvider {
                    DisposalRegistryProvider {
                        AutoDisposeComponent {}
                    }
                }
            }
        }
    });
}

// Context providers for the auto-dispose functionality
#[component]
fn ProviderCacheProvider(children: Element) -> Element {
    use_context_provider(|| ProviderCache::new());
    rsx! { {children} }
}

#[component]
fn RefreshRegistryProvider(children: Element) -> Element {
    use_context_provider(|| RefreshRegistry::default());
    rsx! { {children} }
}

#[component]
fn DisposalRegistryProvider(children: Element) -> Element {
    let cache = use_context::<ProviderCache>();
    use_context_provider(|| DisposalRegistry::new(cache));
    rsx! { {children} }
}
