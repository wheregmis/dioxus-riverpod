use dioxus::prelude::*;
use dioxus_provider::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

#[cfg(not(target_arch = "wasm32"))]
use tokio::time::sleep;
#[cfg(target_arch = "wasm32")]
use wasmtimer::tokio::sleep;

/// Demo showcasing composable providers - combining multiple providers into one
///
/// This example demonstrates:
/// - Parallel execution of composed providers
/// - Error handling and aggregation
/// - Real-world use cases like user profiles with permissions
/// - How composed results are available in the main provider

// Basic data structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub avatar_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPermissions {
    pub user_id: u32,
    pub permissions: Vec<String>,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSettings {
    pub user_id: u32,
    pub theme: String,
    pub language: String,
    pub notifications_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FullUserProfile {
    pub user: User,
    pub permissions: UserPermissions,
    pub settings: UserSettings,
    pub loading_time_ms: u64,
}

// Individual providers that can be composed

/// Fetch basic user information
#[provider(cache_expiration = "2min")]
async fn fetch_user(user_id: u32) -> Result<User, ProviderError> {
    // Simulate API delay
    sleep(Duration::from_millis(100)).await;

    match user_id {
        0 => Err(ProviderError::InvalidInput(
            "User ID cannot be zero".to_string(),
        )),
        1 => Ok(User {
            id: 1,
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            avatar_url: "https://via.placeholder.com/100".to_string(),
        }),
        2 => Ok(User {
            id: 2,
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            avatar_url: "https://via.placeholder.com/100".to_string(),
        }),
        _ => Err(ProviderError::Generic("User not found".to_string())),
    }
}

/// Fetch user permissions
#[provider(cache_expiration = "5min")]
async fn fetch_user_permissions(user_id: u32) -> Result<UserPermissions, ProviderError> {
    // Simulate API delay
    sleep(Duration::from_millis(150)).await;

    match user_id {
        0 => Err(ProviderError::InvalidInput(
            "User ID cannot be zero".to_string(),
        )),
        1 => Ok(UserPermissions {
            user_id: 1,
            permissions: vec!["read".to_string(), "write".to_string(), "admin".to_string()],
            role: "Administrator".to_string(),
        }),
        2 => Ok(UserPermissions {
            user_id: 2,
            permissions: vec!["read".to_string()],
            role: "User".to_string(),
        }),
        _ => Err(ProviderError::Generic("Permissions not found".to_string())),
    }
}

/// Fetch user settings
#[provider(cache_expiration = "1min")]
async fn fetch_user_settings(user_id: u32) -> Result<UserSettings, ProviderError> {
    // Simulate API delay
    sleep(Duration::from_millis(80)).await;

    match user_id {
        0 => Err(ProviderError::InvalidInput(
            "User ID cannot be zero".to_string(),
        )),
        1 => Ok(UserSettings {
            user_id: 1,
            theme: "dark".to_string(),
            language: "en".to_string(),
            notifications_enabled: true,
        }),
        2 => Ok(UserSettings {
            user_id: 2,
            theme: "light".to_string(),
            language: "es".to_string(),
            notifications_enabled: false,
        }),
        _ => Err(ProviderError::Generic("Settings not found".to_string())),
    }
}

// Composable provider that combines all user data

/// Fetch complete user profile by composing multiple providers
/// This runs all three providers in parallel and combines their results
#[provider(compose = [fetch_user, fetch_user_permissions, fetch_user_settings], cache_expiration = "3min")]
async fn fetch_full_user_profile(user_id: u32) -> Result<FullUserProfile, ProviderError> {
    let start_time = Instant::now();

    // The composed results are automatically available as variables:
    // - fetch_user_result: Result<User, ProviderError>
    // - fetch_user_permissions_result: Result<UserPermissions, ProviderError>
    // - fetch_user_settings_result: Result<UserSettings, ProviderError>

    // All three providers have been called in parallel with user_id
    // We can now combine their results

    let user = fetch_user_result?;
    let permissions = fetch_user_permissions_result?;
    let settings = fetch_user_settings_result?;

    let loading_time = start_time.elapsed();

    Ok(FullUserProfile {
        user,
        permissions,
        settings,
        loading_time_ms: loading_time.as_millis() as u64,
    })
}

// Additional example: Compose just two providers

/// Fetch user with permissions (subset of full profile)
#[provider(compose = [fetch_user, fetch_user_permissions])]
async fn fetch_user_with_permissions(
    user_id: u32,
) -> Result<(User, UserPermissions), ProviderError> {
    // The composed results are available as:
    // - fetch_user_result: Result<User, ProviderError>
    // - fetch_user_permissions_result: Result<UserPermissions, ProviderError>

    let user = fetch_user_result?;
    let permissions = fetch_user_permissions_result?;

    Ok((user, permissions))
}

// UI Components

#[component]
fn App() -> Element {
    // Initialize global providers for dependency injection
    init_global_providers();

    // State for demonstration
    let mut user_id = use_signal(|| 1u32);

    rsx! {
        div { style: "font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 1200px; margin: 0 auto; padding: 20px;",

            // Header
            div { style: "text-align: center; margin-bottom: 30px;",
                h1 { style: "color: #2c3e50; margin-bottom: 10px;", "🚀 Composable Provider Demo" }
                p { style: "color: #666; font-size: 18px;",
                    "Demonstrates the power of composable providers in parallel execution"
                }

                // User selection buttons
                div { style: "margin: 20px 0;",
                    button {
                        style: "margin: 0 10px; padding: 10px 20px; background: #007acc; color: white; border: none; border-radius: 6px; cursor: pointer;",
                        onclick: move |_| { *user_id.write() = 1; },
                        "Load User 1"
                    }
                    button {
                        style: "margin: 0 10px; padding: 10px 20px; background: #28a745; color: white; border: none; border-radius: 6px; cursor: pointer;",
                        onclick: move |_| { *user_id.write() = 2; },
                        "Load User 2"
                    }
                    button {
                        style: "margin: 0 10px; padding: 10px 20px; background: #dc3545; color: white; border: none; border-radius: 6px; cursor: pointer;",
                        onclick: move |_| { *user_id.write() = 999; },
                        "Load Non-existent User (Error)"
                    }
                }

                p { style: "color: #666;", "Current User ID: " { user_id.read().to_string() } }
            }

            // Performance comparison section
            div { style: "background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 30px;",
                h2 { style: "color: #495057; margin-top: 0;", "⚡ Performance Comparison" }
                p { style: "color: #666; margin-bottom: 20px;",
                    "Compare sequential loading (individual providers) vs parallel loading (composable providers)"
                }
            }

            // Individual providers (sequential loading simulation)
            div { style: "margin-bottom: 40px;",
                div { style: "background: linear-gradient(90deg, #ffeaa7, #fab1a0); padding: 15px; border-radius: 8px; margin-bottom: 15px;",
                    h2 { style: "margin: 0; color: #2d3436;", "🐌 Individual Providers (Sequential Loading Simulation)" }
                    p { style: "margin: 5px 0 0 0; color: #636e72;", "Each provider loads independently, taking longer total time" }
                }
                IndividualProvidersDemo { user_id: *user_id.read() }
            }

            // Composable provider (parallel loading)
            div { style: "margin-bottom: 40px;",
                div { style: "background: linear-gradient(90deg, #a8e6cf, #81c784); padding: 15px; border-radius: 8px; margin-bottom: 15px;",
                    h2 { style: "margin: 0; color: #1b5e20;", "⚡ Composable Provider (Parallel Loading)" }
                    p { style: "margin: 5px 0 0 0; color: #2e7d32;", "All providers execute in parallel, significantly faster!" }
                }
                ComposableProviderDemo { user_id: *user_id.read() }
            }

            // Partial composition demo
            div { style: "margin-bottom: 40px;",
                div { style: "background: linear-gradient(90deg, #dda0dd, #ba68c8); padding: 15px; border-radius: 8px; margin-bottom: 15px;",
                    h2 { style: "margin: 0; color: #4a148c;", "🔗 Partial Composition" }
                    p { style: "margin: 5px 0 0 0; color: #6a1b9a;", "Compose only specific providers as needed" }
                }
                PartialCompositionDemo { user_id: *user_id.read() }
            }

            // Information section
            div { style: "background: #e3f2fd; padding: 20px; border-radius: 8px; border-left: 5px solid #2196f3;",
                h3 { style: "color: #1976d2; margin-top: 0;", "💡 Key Benefits of Composable Providers" }
                ul { style: "color: #424242;",
                    li { "⚡ " strong { "Parallel Execution: " } "All composed providers run simultaneously" }
                    li { "🎯 " strong { "Type Safety: " } "Full compile-time type checking for all composed results" }
                    li { "🔄 " strong { "Automatic Caching: " } "Each provider maintains its own cache and settings" }
                    li { "🛠 " strong { "Flexible Composition: " } "Mix and match providers as needed" }
                    li { "📊 " strong { "Error Handling: " } "Individual provider errors don't break the whole composition" }
                    li { "🚀 " strong { "Performance: " } "Significantly faster than sequential loading" }
                }
            }
        }
    }
}

#[component]
fn IndividualProvidersDemo(user_id: u32) -> Element {
    let user_data = use_provider(fetch_user(), user_id);
    let permissions_data = use_provider(fetch_user_permissions(), user_id);
    let settings_data = use_provider(fetch_user_settings(), user_id);

    rsx! {
        div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin: 20px 0;",

            // User card
            div { style: "border: 1px solid #ddd; border-radius: 8px; padding: 15px; background: #fff;",
                h4 { "👤 User Data" }
                match &*user_data.read() {
                    AsyncState::Loading => rsx! { p { style: "color: #666;", "Loading user..." } },
                    AsyncState::Success(user) => rsx! {
                        div {
                            p { strong { "Name: " } {user.name.clone()} }
                            p { strong { "Email: " } {user.email.clone()} }
                            p { strong { "ID: " } {user.id.to_string()} }
                        }
                    },
                    AsyncState::Error(err) => rsx! { p { style: "color: #dc3545;", "Error: {err}" } }
                }
            }

            // Permissions card
            div { style: "border: 1px solid #ddd; border-radius: 8px; padding: 15px; background: #fff;",
                h4 { "🔐 Permissions" }
                match &*permissions_data.read() {
                    AsyncState::Loading => rsx! { p { style: "color: #666;", "Loading permissions..." } },
                    AsyncState::Success(perms) => rsx! {
                        div {
                            p { strong { "Role: " } {perms.role.clone()} }
                            p { strong { "Permissions: " } {perms.permissions.join(", ")} }
                        }
                    },
                    AsyncState::Error(err) => rsx! { p { style: "color: #dc3545;", "Error: {err}" } }
                }
            }

            // Settings card
            div { style: "border: 1px solid #ddd; border-radius: 8px; padding: 15px; background: #fff;",
                h4 { "⚙️ Settings" }
                match &*settings_data.read() {
                    AsyncState::Loading => rsx! { p { style: "color: #666;", "Loading settings..." } },
                    AsyncState::Success(settings) => rsx! {
                        div {
                            p { strong { "Theme: " } {settings.theme.clone()} }
                            p { strong { "Language: " } {settings.language.clone()} }
                            p { strong { "Notifications: " } {if settings.notifications_enabled { "Enabled" } else { "Disabled" }} }
                        }
                    },
                    AsyncState::Error(err) => rsx! { p { style: "color: #dc3545;", "Error: {err}" } }
                }
            }
        }
    }
}

#[component]
fn ComposableProviderDemo(user_id: u32) -> Element {
    let full_profile_data = use_provider(fetch_full_user_profile(), user_id);

    rsx! {
        div { style: "margin: 20px 0;",
            match &*full_profile_data.read() {
                AsyncState::Loading => rsx! {
                    div { style: "text-align: center; padding: 40px; background: #f8f9fa; border-radius: 8px;",
                        p { style: "color: #666; font-size: 18px;", "⚡ Loading full profile in parallel..." }
                    }
                },
                AsyncState::Success(profile) => rsx! {
                    div { style: "border: 2px solid #28a745; border-radius: 8px; padding: 20px; background: #f8fff8;",
                        div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;",
                            h3 { style: "color: #28a745; margin: 0;", "✅ Full Profile Loaded" }
                            span { style: "background: #28a745; color: white; padding: 4px 8px; border-radius: 4px; font-size: 12px;",
                                "⚡ {profile.loading_time_ms}ms"
                            }
                        }

                        div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 15px;",

                            // User section
                            div { style: "background: white; padding: 15px; border-radius: 6px; border-left: 4px solid #007acc;",
                                h4 { style: "margin-top: 0; color: #007acc;", "👤 User Info" }
                                p { strong { "Name: " } {profile.user.name.clone()} }
                                p { strong { "Email: " } {profile.user.email.clone()} }
                                p { strong { "ID: " } {profile.user.id.to_string()} }
                            }

                            // Permissions section
                            div { style: "background: white; padding: 15px; border-radius: 6px; border-left: 4px solid #ffc107;",
                                h4 { style: "margin-top: 0; color: #ffc107;", "🔐 Permissions" }
                                p { strong { "Role: " } {profile.permissions.role.clone()} }
                                p { strong { "Permissions: " } {profile.permissions.permissions.join(", ")} }
                            }

                            // Settings section
                            div { style: "background: white; padding: 15px; border-radius: 6px; border-left: 4px solid #17a2b8;",
                                h4 { style: "margin-top: 0; color: #17a2b8;", "⚙️ Settings" }
                                p { strong { "Theme: " } {profile.settings.theme.clone()} }
                                p { strong { "Language: " } {profile.settings.language.clone()} }
                                p { strong { "Notifications: " } {if profile.settings.notifications_enabled { "Enabled" } else { "Disabled" }} }
                            }
                        }
                    }
                },
                AsyncState::Error(err) => rsx! {
                    div { style: "border: 2px solid #dc3545; border-radius: 8px; padding: 20px; background: #fff8f8;",
                        h3 { style: "color: #dc3545; margin-top: 0;", "❌ Failed to Load Profile" }
                        p { style: "color: #dc3545;", "Error: {err}" }
                        p { style: "color: #666; font-size: 14px;",
                            "This happens when any of the composed providers fails. The entire composition fails fast."
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PartialCompositionDemo(user_id: u32) -> Element {
    let user_with_perms = use_provider(fetch_user_with_permissions(), user_id);

    rsx! {
        div { style: "margin: 20px 0;",
            match &*user_with_perms.read() {
                AsyncState::Loading => rsx! {
                    p { style: "color: #666;", "Loading user with permissions..." }
                },
                AsyncState::Success((user, permissions)) => rsx! {
                    div { style: "border: 1px solid #6f42c1; border-radius: 8px; padding: 20px; background: #f8f6ff;",
                        h3 { style: "color: #6f42c1; margin-top: 0;", "👤🔐 User + Permissions (Partial Composition)" }

                        div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                            div {
                                h4 { "User Info:" }
                                p { strong { "Name: " } {user.name.clone()} }
                                p { strong { "Email: " } {user.email.clone()} }
                            }
                            div {
                                h4 { "Permissions:" }
                                p { strong { "Role: " } {permissions.role.clone()} }
                                p { strong { "Access: " } {permissions.permissions.join(", ")} }
                            }
                        }
                    }
                },
                AsyncState::Error(err) => rsx! {
                    p { style: "color: #dc3545;", "Error: {err}" }
                }
            }
        }
    }
}

fn main() {
    // Launch the app
    dioxus::launch(App);
}
