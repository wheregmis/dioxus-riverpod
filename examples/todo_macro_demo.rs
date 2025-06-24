use dioxus::prelude::*;
use dioxus_provider::prelude::*;
#[cfg(target_family = "wasm")]
use std::cell::RefCell;
#[cfg(not(target_family = "wasm"))]
use std::sync::Mutex;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Todo {
    pub id: u32,
    pub title: String,
    pub completed: bool,
}

#[cfg(not(target_family = "wasm"))]
static TODOS: Mutex<Vec<Todo>> = Mutex::new(Vec::new());
#[cfg(target_family = "wasm")]
thread_local! {
    static TODOS: RefCell<Vec<Todo>> = RefCell::new(Vec::new());
}

fn get_todos() -> Vec<Todo> {
    #[cfg(not(target_family = "wasm"))]
    {
        TODOS.lock().unwrap().clone()
    }
    #[cfg(target_family = "wasm")]
    {
        TODOS.with(|todos| todos.borrow().clone())
    }
}

fn set_todos(todos: Vec<Todo>) {
    #[cfg(not(target_family = "wasm"))]
    {
        *TODOS.lock().unwrap() = todos;
    }
    #[cfg(target_family = "wasm")]
    {
        TODOS.with(|cell| *cell.borrow_mut() = todos);
    }
}

// Provider for fetching todos
#[provider]
async fn fetch_todos() -> Result<Vec<Todo>, String> {
    // Simulate network delay
    #[cfg(not(target_family = "wasm"))]
    tokio::time::sleep(Duration::from_millis(200)).await;
    #[cfg(target_family = "wasm")]
    wasmtimer::tokio::sleep(Duration::from_millis(200)).await;
    Ok(get_todos())
}

// Add todo mutation
#[mutation(invalidates = [fetch_todos])]
async fn add_todo(title: String) -> Result<Todo, String> {
    #[cfg(not(target_family = "wasm"))]
    tokio::time::sleep(Duration::from_millis(100)).await;
    #[cfg(target_family = "wasm")]
    wasmtimer::tokio::sleep(Duration::from_millis(100)).await;
    let mut todos = get_todos();
    let id = todos.last().map(|t| t.id + 1).unwrap_or(1);
    let todo = Todo {
        id,
        title,
        completed: false,
    };
    todos.push(todo.clone());
    set_todos(todos);
    Ok(todo)
}

// Toggle todo mutation (optimistic)
#[mutation(invalidates = [fetch_todos])]
async fn toggle_todo(id: u32) -> Result<Todo, String> {
    #[cfg(not(target_family = "wasm"))]
    tokio::time::sleep(Duration::from_millis(100)).await;
    #[cfg(target_family = "wasm")]
    wasmtimer::tokio::sleep(Duration::from_millis(100)).await;
    let mut todos = get_todos();
    if let Some(idx) = todos.iter().position(|t| t.id == id) {
        todos[idx].completed = !todos[idx].completed;
        let todo = todos[idx].clone();
        set_todos(todos);
        Ok(todo)
    } else {
        Err("Todo not found".to_string())
    }
}

// Remove todo mutation
#[mutation(invalidates = [fetch_todos])]
async fn remove_todo(id: u32) -> Result<(), String> {
    #[cfg(not(target_family = "wasm"))]
    tokio::time::sleep(Duration::from_millis(100)).await;
    #[cfg(target_family = "wasm")]
    wasmtimer::tokio::sleep(Duration::from_millis(100)).await;
    let mut todos = get_todos();
    let len_before = todos.len();
    todos.retain(|t| t.id != id);
    set_todos(todos);
    if len_before > get_todos().len() {
        Ok(())
    } else {
        Err("Todo not found".to_string())
    }
}

#[component]
fn TodoApp() -> Element {
    let todos = use_provider(fetch_todos(), ());
    let (add_state, add_fn) = use_mutation(add_todo());
    let (toggle_state, toggle_fn) = use_optimistic_mutation(toggle_todo());
    let (remove_state, remove_fn) = use_mutation(remove_todo());
    let mut new_title = use_signal(|| String::new());

    rsx! {
        div {
            style: "max-width: 400px; margin: 40px auto; font-family: system-ui;",
            h1 { style: "text-align: center;", "📝 Todo App" }
            form {
                onsubmit: move |evt| {
                    evt.prevent_default();
                    let title = new_title.read().trim().to_string();
                    if !title.is_empty() {
                        add_fn(title.clone());
                        new_title.set(String::new());
                    }
                },
                input {
                    r#type: "text",
                    value: "{new_title.read()}",
                    oninput: move |evt| new_title.set(evt.value()),
                    placeholder: "Add a new todo...",
                    style: "width: 70%; padding: 8px; margin-right: 8px;"
                }
                button {
                    r#type: "submit",
                    disabled: add_state.read().is_loading(),
                    style: "padding: 8px 16px;",
                    "Add"
                }
            }
            match &*todos.read() {
                AsyncState::Loading => rsx! { p { "Loading..." } },
                AsyncState::Error(err) => rsx! { p { style: "color: red;", "Error: {err}" } },
                AsyncState::Success(list) => {
                    let todos_vec = list.clone();
                    rsx! {
                        ul {
                            style: "list-style: none; padding: 0; margin-top: 24px;",
                            for todo in &todos_vec {
                                li {
                                    key: "{todo.id}",
                                    style: "display: flex; align-items: center; margin-bottom: 8px;",
                                    input {
                                        r#type: "checkbox",
                                        checked: todo.completed,
                                        onclick: {
                                            let toggle_fn = toggle_fn.clone();
                                            let id = todo.id;
                                            move |_| toggle_fn(id)
                                        }
                                    }
                                    span {
                                        style: if todo.completed { "text-decoration: line-through; margin-left: 8px; flex: 1;" } else { "margin-left: 8px; flex: 1;" },
                                        "{todo.title}"
                                    }
                                    button {
                                        style: "margin-left: 8px;",
                                        onclick: {
                                            let remove_fn = remove_fn.clone();
                                            let id = todo.id;
                                            move |_| remove_fn(id)
                                        },
                                        "❌"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if add_state.read().is_loading() {
                p { style: "color: #888;", "Adding todo..." }
            }
            if let MutationState::Error(err) = &*add_state.read() {
                p { style: "color: red;", "Add error: {err}" }
            }
            if let MutationState::Error(err) = &*toggle_state.read() {
                p { style: "color: red;", "Toggle error: {err}" }
            }
            if let MutationState::Error(err) = &*remove_state.read() {
                p { style: "color: red;", "Remove error: {err}" }
            }
        }
    }
}

fn main() {
    // Initialize the in-memory todos
    set_todos(vec![
        Todo {
            id: 1,
            title: "Learn Rust".to_string(),
            completed: false,
        },
        Todo {
            id: 2,
            title: "Build a Dioxus app".to_string(),
            completed: false,
        },
    ]);
    init_global_providers();
    dioxus::launch(TodoApp);
}
