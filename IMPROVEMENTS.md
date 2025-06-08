# Dioxus Riverpod - Clean and Unified Provider System

## 🎉 Major Improvements Completed

### ✅ Unified `#[provider]` Attribute
- **Single attribute** for all provider types
- **Automatic detection** of provider type based on function parameters
- **No more separate** `#[future_provider]` and `#[family_provider]` attributes needed

### ✅ Simplified Library Structure
- **Consolidated** all provider functionality into a single `providers.rs` module
- **Removed** old macro-based approach and separate modules
- **Cleaner exports** with only the essential API surface

### ✅ Better Developer Experience
- **Natural Rust syntax** - providers look like regular async functions
- **Seamless composition** - providers can call other providers directly
- **Better IDE support** - full syntax highlighting and error detection
- **Type-safe** - same compile-time guarantees with cleaner syntax

## API Examples

### Future Provider (No Parameters)
```rust
use dioxus_riverpod::prelude::*;

#[provider]
async fn current_time() -> Result<String, ()> {
    tokio::time::sleep(Duration::from_millis(500)).await;
    Ok(format!("Current time: {}", chrono::Utc::now().format("%H:%M:%S")))
}

// Usage in component
#[component]
fn TimeDisplay() -> Element {
    let time_signal = use_future_provider(current_time);
    
    rsx! {
        match &*time_signal.read() {
            AsyncState::Loading => rsx! { p { "Loading..." } },
            AsyncState::Success(time) => rsx! { p { "{time}" } },
            AsyncState::Error(_) => rsx! { p { "Error loading time" } },
        }
    }
}
```

### Family Provider (With Parameters)
```rust
#[provider]
async fn user_details(id: usize) -> Result<UserProfile, String> {
    // Can compose other providers!
    let name = UserNameProvider::call(id).await?;
    
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    Ok(UserProfile {
        id,
        name,
        age: 30,
        role: "Developer".to_string(),
    })
}

// Usage in component
#[component]
fn UserCard(id: usize) -> Element {
    let user_signal = use_family_provider(user_details, id);
    
    rsx! {
        match &*user_signal.read() {
            AsyncState::Loading => rsx! { p { "Loading user {id}..." } },
            AsyncState::Success(user) => rsx! {
                div {
                    h3 { "{user.name}" }
                    p { "Age: {user.age}" }
                    p { "Role: {user.role}" }
                }
            },
            AsyncState::Error(error) => rsx! { p { "Error: {error}" } },
        }
    }
}
```

## Library Structure

```
dioxus-riverpod/
├── src/
│   ├── lib.rs              # Main exports
│   └── providers.rs        # Unified provider system
├── dioxus-riverpod-macros/ # Procedural macros
│   └── src/
│       └── lib.rs          # #[provider] attribute macro
└── examples/
    └── riverpod_style.rs   # Demo application
```

## Key Benefits

1. **🚀 Reduced Boilerplate**: Single `#[provider]` attribute does everything
2. **🔧 Better Tooling**: Full IDE support with syntax highlighting and error detection  
3. **🔗 Easy Composition**: Providers can call other providers naturally
4. **📦 Cleaner API**: Unified interface with automatic type detection
5. **⚡ Same Performance**: Zero overhead - same underlying implementation
6. **🛡️ Type Safety**: Full compile-time guarantees maintained

## Migration from Old API

### Before (Multiple Attributes)
```rust
#[future_provider]
fn get_time() -> Result<String, ()> { ... }

#[family_provider] 
fn get_user(id: usize) -> Result<User, String> { ... }
```

### After (Unified Attribute)
```rust
#[provider]  // Automatically detects it's a future provider
async fn get_time() -> Result<String, ()> { ... }

#[provider]  // Automatically detects it's a family provider  
async fn get_user(id: usize) -> Result<User, String> { ... }
```

## ⚡ Performance Optimizations

### Automatic Provider Caching
The library now includes **automatic caching** for all providers to prevent unnecessary re-executions:

- **Global Cache**: Results are cached globally across all components using a type-erased cache
- **Smart Cache Keys**: Each provider generates unique cache keys based on provider type and parameters  
- **Automatic Cache Hits**: Subsequent calls to the same provider with same parameters return cached results instantly
- **Memory Efficient**: Uses `Arc<Mutex<HashMap>>` with type-erased `CacheEntry` for optimal performance

### Before vs After Performance

**Before** (multiple executions):
```
Fetching details for user 1
Fetching name for user 1  
Fetching details for user 1  // Duplicate execution!
Fetching name for user 1     // Duplicate execution!
```

**After** (single execution per unique key):
```
Fetching details for user 1
Fetching name for user 1
// All subsequent calls use cached results instantly
```

### Cache Management API

You can manually control the cache when needed:

```rust
use dioxus_riverpod::prelude::*;

#[component]
fn MyComponent() -> Element {
    // Get access to the cache
    let cache = use_provider_cache();
    
    // Invalidate specific provider
    use_invalidate_provider(my_provider);
    
    // Invalidate family provider with specific param
    use_invalidate_family_provider(user_provider, user_id);
    
    // Clear entire cache
    use_clear_provider_cache();
    
    // ...rest of component
}
```

### Technical Implementation

The caching system uses:
- **`use_hook`**: Ensures provider logic runs only once per component instance
- **Type-erased cache**: `HashMap<String, CacheEntry>` where `CacheEntry` wraps `Arc<dyn Any>`
- **Provider IDs**: Each provider generates unique cache keys (e.g., `"UserDetailsProvider(123)"`)
- **Lazy loading**: Cache checks happen first, provider execution only on cache miss

The library is now much cleaner, more intuitive, and follows Rust best practices while maintaining all the power of the Riverpod-style provider system!
