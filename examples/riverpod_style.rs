#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use dioxus_riverpod::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

use dioxus::prelude::*;

fn main() {
    launch(app);
}

// Define a simple future provider for getting current time
#[provider]
/// Get the current timestamp
fn current_time() -> Result<String, ()> {
    Ok(format!(
        "Current time: {}",
        chrono::Utc::now().format("%H:%M:%S")
    ))
}

// Define a family provider for getting user information by ID
#[provider]
/// Get user name by ID
async fn user_name(id: usize) -> Result<String, String> {
    println!("Fetching name for user {id}");
    sleep(Duration::from_millis(650)).await;
    match id {
        0 => Ok("Marc".to_string()),
        1 => Ok("Alice".to_string()),
        2 => Ok("Bob".to_string()),
        _ => Err("User not found".to_string()),
    }
}

#[provider]
/// Get user details by ID (depends on user_name)
async fn user_details(id: usize) -> Result<(String, u8, String), String> {
    println!("Fetching details for user {id}");

    // Now we can actually compose providers here!
    let name = UserNameProvider::call(id).await?;
    sleep(Duration::from_millis(1000)).await;

    match id {
        0 => Ok((name, 30, "Developer".to_string())),
        1 => Ok((name, 25, "Designer".to_string())),
        2 => Ok((name, 35, "Manager".to_string())),
        _ => Err("User not found".to_string()),
    }
}

#[allow(non_snake_case)]
#[component]
fn UserCard(id: usize) -> Element {
    // Using family provider - much simpler than the old approach!
    let user_details_signal = use_family_provider(user_details, id);

    println!("Rendering user card for {id}");

    rsx!(
        div { style: "border: 1px solid #ccc; padding: 10px; margin: 10px; border-radius: 5px;",
            match &*user_details_signal.read() {
                AsyncState::Loading => rsx! {
                    p { "Loading user {id}..." }
                },
                AsyncState::Success((name, age, role)) => rsx! {
                    h3 { "{name}" }
                    p { "Age: {age}" }
                    p { "Role: {role}" }
                    p { "ID: {id}" }
                },
                AsyncState::Error(error) => rsx! {
                    p { style: "color: red;", "Error loading user {id}: {error}" }
                },
            }
        }
    )
}

#[allow(non_snake_case)]
#[component]
fn TimeDisplay() -> Element {
    // Using future provider - no parameters needed
    let time_signal = use_future_provider(current_time);

    rsx!(
        div { style: "background: #f0f0f0; padding: 10px; margin: 10px; border-radius: 5px;",
            match &*time_signal.read() {
                AsyncState::Loading => rsx! {
                    p { "Loading current time..." }
                },
                AsyncState::Success(time_str) => rsx! {
                    p { "{time_str}" }
                },
                AsyncState::Error(_) => rsx! {
                    p { style: "color: red;", "Failed to get time" }
                },
            }
        }
    )
}

#[allow(non_snake_case)]
#[component]
fn SuspenseExample() -> Element {
    rsx!(
        div {
            h1 { "Suspense Example" }
            h2 { "Regular User Card Example" }
            UserCard { id: 1 }
        }
    )
}

fn app() -> Element {
    // Provide the cache and refresh registry contexts at the app level
    use_context_provider(ProviderCache::new);
    use_context_provider(RefreshRegistry::new);

    rsx!(
        div { style: "font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px;",
            h1 { "🚀 Dioxus Riverpod-Style Providers Demo" }
            p {
                "This demo shows the new simplified provider system inspired by Riverpod. "
                "Compare this to the old approach in composable.rs and suspense.rs!"
            }

            // Future provider example
            TimeDisplay {}

            // Family provider examples
            h2 { "User Cards (Family Providers)" }
            div { style: "display: flex; flex-wrap: wrap;",
                UserCard { id: 0 }
                UserCard { id: 1 }
                UserCard { id: 2 }
                UserCard { id: 999 } // This will show an error
            }


            // Suspense example (simplified for now)
            SuspenseExample {}

            div { style: "margin-top: 40px; padding: 20px; background: #e8f5e8; border-radius: 8px;",
                h3 { "🎉 Benefits of the New Approach:" }
                ul {
                    li { "✅ Much less boilerplate (no structs + trait implementations)" }
                    li { "✅ Single #[provider] attribute (auto-detects future vs family)" }
                    li { "✅ Works seamlessly with Dioxus signals" }
                    li { "✅ Built-in suspense support" }
                    li { "✅ Type-safe and composable" }
                    li { "✅ Familiar patterns for React/Flutter developers" }
                    li { "✅ Clean async function syntax with attributes" }
                }
            }
        }
    )
}
