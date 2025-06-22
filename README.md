# Dioxus Riverpod

A state management library for Dioxus applications, inspired by Riverpod for Flutter.

## Features

- **Global Provider Management** - Application-wide state without context wrappers
- **Intelligent Cache Management** - LRU and time-based automatic cleanup
- **SWR (Stale-While-Revalidate)** - Serve stale data while revalidating in background
- **Cache Expiration** - Configurable TTL with automatic cleanup
- **Interval Refresh** - Background data updates at specified intervals
- **Dependency Injection** - Macro-based DI for complex provider dependencies
- **Cross-Platform Support** - Works on Web (WASM) and Desktop platforms
- **Humantime Duration Parsing** - Natural duration syntax ("5s", "10min", "1h")

## Examples

See the examples directory for comprehensive usage demonstrations:

### Feature-Specific Examples
- `swr_demo.rs` - Stale-While-Revalidate pattern with background revalidation
- `cache_expiration_demo.rs` - Time-based cache expiration and cleanup
- `interval_refresh_demo.rs` - Automatic background data refresh at intervals
- `dependency_injection_demo.rs` - Advanced dependency injection patterns

### Complete Demo
- `comprehensive_demo.rs` - **ALL features in one demo** - start here!

### Usage
```bash
# Run the comprehensive demo showing all features
cargo run --example comprehensive_demo

# Or run specific feature demos
cargo run --example swr_demo
cargo run --example cache_expiration_demo
cargo run --example interval_refresh_demo
```

## Quick Start

1.  **Initialize global providers** at your application's entry point:

```rust,no_run
use dioxus_riverpod::global::init_global_providers;

fn main() {
    // This is required for all provider hooks to work
    init_global_providers();
    dioxus::launch(App);
}
```

2.  **Create a provider** using the `#[provider]` macro:

```rust,no_run
use dioxus_riverpod::prelude::*;

#[provider]
async fn example_provider() -> Result<String, String> {
    // This could be an API call, database query, etc.
    Ok("Hello from the provider!".to_string())
}
```

3.  **Use the provider** in your components with the `use_provider` hook:

```rust,no_run
use dioxus::prelude::*;
use dioxus_riverpod::prelude::*;

#[component]
fn App() -> Element {
    // The provider will be fetched and cached automatically
    let data = use_provider(example_provider(), ());

    match &*data.read() {
        AsyncState::Loading => rsx! { div { "Loading..." } },
        AsyncState::Success(value) => rsx! { div { "{value}" } },
        AsyncState::Error(err) => rsx! { div { "Error: {err}" } },
    }
}
```

## Documentation

This library provides macros and utilities for managing application state in Dioxus applications with automatic caching, refresh intervals, and cleanup.