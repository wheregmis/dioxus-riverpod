use dioxus::prelude::*;
use dioxus_provider::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    let _ = init_global_providers();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Hero {}
        Echo {}
        GreetingDemo {}
        GreetingMutationDemo {}
    }
}

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.6/", "📚 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}

/// Echo component that demonstrates fullstack server functions.
#[component]
fn Echo() -> Element {
    let mut response = use_signal(|| String::new());

    rsx! {
        div {
            id: "echo",
            h4 { "ServerFn Echo" }
            input {
                placeholder: "Type here to echo...",
                oninput:  move |event| async move {
                    let data = echo_server(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

/// Echo the user input on the server.
#[server(EchoServer)]
async fn echo_server(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

// --- Provider + Server Function Example ---
#[provider]
async fn greeting_provider(name: String) -> Result<String, String> {
    Ok(format!("Hello, {name}!"))
}

#[component]
fn GreetingDemo() -> Element {
    let mut name = use_signal(|| "World".to_string());
    let greeting = use_provider(greeting_provider(), name().clone());
    let mut echo_response = use_signal(|| String::new());

    rsx! {
        div {
            h3 { "Provider + Server Function Example" }
            input {
                value: "{name}",
                oninput: move |evt| name.set(evt.value()),
                placeholder: "Enter your name"
            }
            button {
                onclick: move |_| {
                    let name = name().clone();
                    async move {
                        if let Ok(resp) = echo_server(name).await {
                            echo_response.set(resp);
                        }
                    }
                },
                "Echo on Server"
            }
            if let Some(greet) = greeting().data() {
                p { "Provider says: {greet}" }
            }
            if !echo_response().is_empty() {
                p { "Server echoed: {echo_response}" }
            }
        }
    }
}

#[mutation]
async fn set_greeting(name: String) -> Result<String, String> {
    // In a real app, this would update a database or server state.
    // Here, just echo the new name as the new greeting.
    // Manually invalidate the provider cache for this name
    let cache = dioxus_provider::hooks::use_provider_cache();
    let key = dioxus_provider::mutation::provider_cache_key(greeting_provider(), name.clone());
    cache.invalidate(&key);
    Ok(format!("Hello, {name}!"))
}

#[component]
fn GreetingMutationDemo() -> Element {
    let mut name = use_signal(|| "World".to_string());
    let greeting = use_provider(greeting_provider(), name().clone());
    let (mutation_state, set_greeting_mutate) = use_mutation(set_greeting());

    rsx! {
        div {
            h3 { "Provider + Mutation Example" }
            input {
                value: "{name}",
                oninput: move |evt| name.set(evt.value()),
                placeholder: "Enter your name"
            }
            button {
                onclick: move |_| set_greeting_mutate(name().clone()),
                "Set Greeting (Mutation)"
            }
            if let Some(greet) = greeting().data() {
                p { "Provider says: {greet}" }
            }
            match &*mutation_state.read() {
                MutationState::Idle => rsx!(),
                MutationState::Loading => rsx!(p { "Updating greeting..." }),
                MutationState::Success(msg) => rsx!(p { "Mutation success: {msg}" }),
                MutationState::Error(err) => rsx!(p { "Mutation error: {err}" }),
            }
        }
    }
}
