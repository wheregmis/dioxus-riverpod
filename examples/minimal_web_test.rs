use dioxus::prelude::*;
use dioxus_riverpod::prelude::*;
use std::time::Duration;

#[cfg(not(target_family = "wasm"))]
use tokio::time::sleep;
#[cfg(target_family = "wasm")]
use wasmtimer::tokio::sleep;

// Very simple provider that just returns a string after a delay
#[provider]
async fn fetch_simple_data() -> Result<String, String> {
    // Small delay to simulate async work
    sleep(Duration::from_millis(100)).await;

    Ok("Hello from provider!".to_string())
}

fn app() -> Element {
    // Provide the necessary contexts for dioxus-riverpod
    use_context_provider(dioxus_riverpod::providers::ProviderCache::new);
    use_context_provider(dioxus_riverpod::providers::RefreshRegistry::new);

    let data = use_provider(fetch_simple_data, ());

    rsx! {
        div {
            h1 { "Minimal Web Test" }
            match &*data.read() {
                AsyncState::Loading => rsx! {
                    p { "Loading..." }
                },
                AsyncState::Success(value) => rsx! {
                    p { "Data: {value}" }
                },
                AsyncState::Error(e) => rsx! {
                    p { "Error: {e}" }
                },
            }
        }
    }
}

fn main() {
    launch(app);
}
