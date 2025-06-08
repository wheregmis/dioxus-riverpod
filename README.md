# 🚀 Dioxus Riverpod

A powerful, type-safe state management library for [Dioxus](https://dioxuslabs.com/) inspired by Flutter's [Riverpod](https://riverpod.dev/). 

## ✨ Features

- **🎯 Single `#[provider]` Attribute**: Automatically detects provider type based on function parameters
- **⚡ Automatic Caching**: Prevents unnecessary re-executions with intelligent result caching
- **🔗 Provider Composition**: Seamlessly compose providers within other providers
- **⚡ Async First**: Built-in support for async operations with loading states
- **🛡️ Type Safe**: Full compile-time type checking and inference
- **🔄 Reactive**: Automatic updates when dependencies change
- **📦 Zero Boilerplate**: Clean, intuitive API that feels like native Rust

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
dioxus-riverpod = "0.1"
dioxus = "0.6"
```

## 📖 Basic Usage

### Future Providers (No Parameters)

```rust
use dioxus_riverpod::prelude::*;
use std::time::Duration;

#[provider]
async fn current_time() -> Result<String, ()> {
    tokio::time::sleep(Duration::from_millis(500)).await;
    Ok(format!("Current time: {}", chrono::Utc::now().format("%H:%M:%S")))
}

#[component]
fn TimeDisplay() -> Element {
    let time_signal = use_future_provider(current_time);
    
    rsx! {
        match &*time_signal.read() {
            AsyncState::Loading => rsx! { p { "Loading current time..." } },
            AsyncState::Success(time) => rsx! { p { "{time}" } },
            AsyncState::Error(_) => rsx! { p { "Failed to get time" } },
        }
    }
}
```

### Family Providers (With Parameters)

```rust
#[provider]
async fn user_name(id: usize) -> Result<String, String> {
    tokio::time::sleep(Duration::from_millis(500)).await;
    match id {
        1 => Ok("Alice".to_string()),
        2 => Ok("Bob".to_string()),
        _ => Err("User not found".to_string()),
    }
}

#[component]
fn UserCard(id: usize) -> Element {
    let user_signal = use_family_provider(user_name, id);
    
    rsx! {
        match &*user_signal.read() {
            AsyncState::Loading => rsx! { p { "Loading user {id}..." } },
            AsyncState::Success(name) => rsx! { h3 { "{name}" } },
            AsyncState::Error(error) => rsx! { p { "Error: {error}" } },
        }
    }
}
```

### Provider Composition

```rust
#[provider]
async fn user_details(id: usize) -> Result<(String, u8, String), String> {
    // Compose other providers!
    let name = UserNameProvider::call(id).await?;
    
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    match id {
        1 => Ok((name, 25, "Designer".to_string())),
        2 => Ok((name, 35, "Manager".to_string())),
        _ => Err("User details not found".to_string()),
    }
}
```

## 🎯 How It Works

The `#[provider]` attribute automatically detects the provider type:

- **No parameters** → **Future Provider**: Single async operation
- **Has parameters** → **Family Provider**: Parameterized async operation

This generates the necessary boilerplate code and implements the appropriate traits (`FutureProvider` or `FamilyProvider`) behind the scenes.

## 🔄 Async State Management

All providers return an `AsyncState<T, E>` that represents the current state:

```rust
pub enum AsyncState<T, E> {
    Loading,           // Operation in progress
    Success(T),        // Operation completed successfully
    Error(E),          // Operation failed
}
```

## 🛠️ Advanced Features

### Suspense Support (Experimental)

```rust
#[component]
fn UserProfile(id: usize) -> Element {
    // This will suspend the component until data is ready
    let user = use_family_provider_suspense(user_details, id)?;
    
    rsx! {
        div {
            h1 { "{user.0}" }  // name
            p { "Age: {user.1}" }  // age  
            p { "Role: {user.2}" }  // role
        }
    }
}
```

## 📚 Examples

Check out the [examples](./examples) directory for complete working examples:

- [`riverpod_style.rs`](./examples/riverpod_style.rs) - Complete demo with multiple provider types

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Inspired by [Riverpod](https://riverpod.dev/) for Flutter
- Built for the amazing [Dioxus](https://dioxuslabs.com/) framework