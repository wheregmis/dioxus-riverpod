# Dioxus Provider

[![Crates.io](https://img.shields.io/crates/v/dioxus-provider.svg)](https://crates.io/crates/dioxus-provider)
[![Docs.rs](https://docs.rs/dioxus-provider/badge.svg)](https://docs.rs/dioxus-provider)

> **⚠️ In Development**  
> This library is currently in active development. The API may change before the first stable release. Please check the [changelog](./CHANGELOG.md) for the latest updates and breaking changes.

**Effortless, powerful, and scalable data fetching and caching for Dioxus applications, inspired by [Riverpod for Flutter](https://riverpod.dev/).**

`dioxus-provider` provides a simple yet robust way to manage data fetching, handle asynchronous operations, and cache data with minimal boilerplate. It is designed to feel native to Dioxus, integrating seamlessly with its component model and hooks system.

## Key Features

- **Global Provider System**: Manage application-wide data without nesting context providers. Simplifies component architecture and avoids "provider hell."
- **Declarative `#[provider]` Macro**: Define data sources with a simple attribute. The macro handles all the complex boilerplate for you.
- **Intelligent Caching Strategies**:
    - **Stale-While-Revalidate (SWR)**: Serve stale data instantly while fetching fresh data in the background for a lightning-fast user experience.
    - **Time-to-Live (TTL) Cache Expiration**: Automatically evict cached data after a configured duration.
- **Automatic Refresh**: Keep data fresh with interval-based background refetching.
- **Parameterized Queries**: Create providers that depend on dynamic arguments (e.g., fetching user data by ID).
- **Manual Cache Control**: Hooks to manually invalidate cached data or clear the entire cache.
- **Cross-Platform by Default**: Works seamlessly on both Desktop and Web (WASM).
- **Minimal Boilerplate**: Get started in minutes with intuitive hooks and macros.

## Installation

Add `dioxus-provider` to your `Cargo.toml`:

```toml
[dependencies]
dioxus-provider = "0.0.1" # Replace with the latest version
```

## Getting Started

### 1. Initialize Global Providers

At the entry point of your application, call `init_global_providers()` once. This sets up the global cache that all providers will use.

```rust,no_run
use dioxus_provider::global::init_global_providers;
use dioxus::prelude::*;

fn main() {
    // This is required for all provider hooks to work
    init_global_providers();
    launch(app);
}

fn app() -> Element {
    rsx! { /* Your app content */ }
}
```

### 2. Create a Provider

A "provider" is a function that fetches or computes a piece of data. Use the `#[provider]` attribute to turn any `async` function into a data source that can be used throughout your app.

```rust,no_run
use dioxus_provider::prelude::*;
use std::time::Duration;

// This could be an API call, database query, etc.
#[provider]
async fn get_server_message() -> Result<String, String> {
    // Simulate a network request
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok("Hello from the server!".to_string())
}
```

### 3. Use the Provider in a Component

Use the `use_provider` hook to read data from a provider. Dioxus will automatically re-render your component when the data changes (e.g., when the `async` function completes).

The hook returns a `Signal<AsyncState<T, E>>`, which can be in one of three states: `Loading`, `Success(T)`, or `Error(E)`.

```rust,no_run
use dioxus::prelude::*;
use dioxus_provider::prelude::*;

#[component]
fn App() -> Element {
    // Use the provider hook to get the data
    let message = use_provider(get_server_message(), ());

    rsx! {
        div {
            h1 { "Dioxus Provider Demo" }
            // Pattern match on the state to render UI
            match &*message.read() {
                AsyncState::Loading => rsx! { div { "Loading..." } },
                AsyncState::Success(data) => rsx! { div { "Server says: {data}" } },
                AsyncState::Error(err) => rsx! { div { "Error: {err}" } },
            }
        }
    }
}
```

## Advanced Usage

### Parameterized Providers

Providers can take arguments to fetch dynamic data. For example, fetching a user by their ID. The cache is keyed by the arguments, so `fetch_user(1)` and `fetch_user(2)` are cached separately.

```rust,no_run
use dioxus_provider::prelude::*;

#[provider]
async fn fetch_user(user_id: u32) -> Result<String, String> {
    Ok(format!("User data for ID: {}", user_id))
}

#[component]
fn UserProfile(user_id: u32) -> Element {
    // Pass arguments as a tuple
    let user = use_provider(fetch_user(), (user_id,));

    match &*user.read() {
        AsyncState::Success(data) => rsx!{ div { "{data}" } },
        // ... other states
        _ => rsx!{ div { "Loading user..." } }
    }
}
```

### Caching Strategies

#### Stale-While-Revalidate (SWR)

`stale_time` serves cached (stale) data first, then re-fetches in the background. This provides a great UX by showing data immediately.

```rust,no_run
#[provider(stale_time = "10s")]
async fn get_dashboard_data() -> Result<String, String> {
    // ... fetch data
    Ok("Dashboard data".to_string())
}
```

#### Cache Expiration (TTL)

`cache_expiration` evicts data from the cache after a time-to-live (TTL). The next request will show a loading state while it re-fetches.

```rust,no_run
// This data will be removed from cache after 5 minutes of inactivity
#[provider(cache_expiration = "5m")]
async fn get_analytics() -> Result<String, String> {
    // ... perform expensive analytics query
    Ok("Analytics report".to_string())
}
```

### Manual Cache Invalidation

You can manually invalidate a provider's cache to force a re-fetch.

```rust,no_run
use dioxus::prelude::*;
use dioxus_provider::prelude::*;

#[component]
fn UserDashboard() -> Element {
    let user_data = use_provider(fetch_user(), (1,));
    let invalidate_user = use_invalidate_provider(fetch_user(), (1,));

    rsx! {
        // ... display user_data
        button {
            onclick: move |_| invalidate_user(),
            "Refresh User"
        }
    }
}
```

To clear the entire global cache for all providers:

```rust,no_run
let clear_cache = use_clear_provider_cache();
clear_cache();
```

## Running The Examples

The `examples` directory contains comprehensive demos.

- `comprehensive_demo.rs`: Showcases all features working together. **A great place to start!**
- `swr_demo.rs`: Focuses on the SWR pattern.
- `cache_expiration_demo.rs`: Demonstrates TTL-based cache expiration.

To run an example:
```bash
# Run the comprehensive demo
cargo run --example comprehensive_demo

# Or run a specific demo
cargo run --example swr_demo
```

## Ecosystem & Alternatives

### dioxus-query: For Complex, Type-Safe Data Management

For more complex applications requiring advanced type safety, sophisticated caching strategies, and enterprise-grade data management, we highly recommend **[dioxus-query](https://github.com/marc2332/dioxus-query)** by [Marc](https://github.com/marc2332).

**dioxus-query** is a mature, production-ready library that provides:

- **Advanced Type Safety**: Compile-time guarantees for complex data relationships
- **Sophisticated Caching**: Multi-level caching with intelligent invalidation strategies
- **Query Dependencies**: Automatic dependency tracking and cascading updates
- **Optimistic Updates**: Immediate UI updates with rollback on failure
- **Background Synchronization**: Advanced background sync with conflict resolution
- **Enterprise Features**: Built-in support for complex data patterns and edge cases

**When to choose dioxus-query:**
- Large-scale applications with complex data requirements
- Teams requiring maximum type safety and compile-time guarantees
- Applications with sophisticated caching and synchronization needs
- Enterprise applications where data consistency is critical

**When to choose dioxus-provider:**
- Smaller to medium applications
- Quick prototyping and development
- Teams new to Dioxus data management
- Applications where simplicity and ease of use are priorities

### dioxus-motion: For Smooth Animations and Transitions

Looking to add beautiful animations to your Dioxus application? Check out **[dioxus-motion](https://github.com/wheregmis/dioxus-motion)** - a lightweight, cross-platform animation library also built by me.

**dioxus-motion** provides:

- **Cross-Platform Animations**: Works seamlessly on web, desktop, and mobile
- **Declarative Animation API**: Write animations as data, not imperative code
- **Page Transitions**: Smooth route transitions with `AnimatedOutlet`
- **Spring Physics**: Natural, physics-based animations
- **Custom Easing**: Extensive easing function support
- **Type-Safe Animations**: Compile-time animation safety
- **Extensible**: Implement `Animatable` trait for custom types

**Perfect combination:**
- Use **dioxus-provider** for data fetching and caching
- Use **dioxus-motion** for smooth UI animations and transitions
- Both libraries work together seamlessly in the same application

```rust
// Example: Combining dioxus-provider with dioxus-motion
use dioxus_provider::prelude::*;
use dioxus_motion::prelude::*;

#[component]
fn AnimatedUserCard(user_id: u32) -> Element {
    // Data fetching with dioxus-provider
    let user_data = use_provider(fetch_user(), (user_id,));
    
    // Animation with dioxus-motion
    let scale = use_motion(1.0f32);
    
    match &*user_data.read() {
        AsyncState::Success(user) => rsx! {
            div {
                style: "transform: scale({scale.get_value()})",
                onclick: move |_| {
                    scale.animate_to(1.1, AnimationConfig::spring());
                },
                "Welcome, {user.name}!"
            }
        },
        _ => rsx! { div { "Loading..." } }
    }
}
```

### Acknowledgment

Special thanks to [Marc](https://github.com/marc2332) for creating the excellent **dioxus-query** library, which has been a significant inspiration for this project. Marc's work on dioxus-query has helped establish best practices for data management in the Dioxus ecosystem, and we encourage users to explore both libraries to find the best fit for their specific use case.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

## License

This project is licensed under the MIT License.