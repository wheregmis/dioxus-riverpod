use dioxus::prelude::*;
use dioxus_riverpod::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

// Simple Todo data structure
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Todo {
    id: u32,
    title: String,
    completed: bool,
}

#[derive(Clone, Debug, PartialEq, Hash)]
struct CreateTodoRequest {
    title: String,
}

#[derive(Clone, Debug, PartialEq, Hash)]
struct UpdateTodoRequest {
    id: u32,
    title: Option<String>,
    completed: Option<bool>,
}

fn main() {
    launch(app);
}

// Provider to fetch all todos
#[provider]
async fn all_todos() -> Result<Vec<Todo>, String> {
    // Simulate API call delay
    sleep(Duration::from_millis(600)).await;

    // Mock API response - in a real app this would be from a database/API
    Ok(vec![
        Todo {
            id: 1,
            title: "Learn Dioxus Riverpod".to_string(),
            completed: false,
        },
        Todo {
            id: 2,
            title: "Build a todo app".to_string(),
            completed: true,
        },
        Todo {
            id: 3,
            title: "Deploy to production".to_string(),
            completed: false,
        },
    ])
}

// Provider to create a new todo
#[provider]
async fn create_todo(request: CreateTodoRequest) -> Result<Todo, String> {
    // Simulate API call delay
    sleep(Duration::from_millis(400)).await;

    // Validate input
    if request.title.trim().is_empty() {
        return Err("Title cannot be empty".to_string());
    }

    // In a real app, you'd get existing todos from the server to generate ID
    // For demo purposes, we'll use a timestamp-based ID
    let new_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32;

    let new_todo = Todo {
        id: new_id,
        title: request.title.trim().to_string(),
        completed: false,
    };

    // Simulate successful creation
    Ok(new_todo)
}

// Provider to update a todo
#[provider]
async fn update_todo(request: UpdateTodoRequest) -> Result<Todo, String> {
    // Simulate API call delay
    sleep(Duration::from_millis(300)).await;

    // In a real app, this would update the todo in the database
    // For demo purposes, we'll create a mock updated todo
    let updated_todo = Todo {
        id: request.id,
        title: request
            .title
            .unwrap_or_else(|| format!("Updated Todo {}", request.id)),
        completed: request.completed.unwrap_or(false),
    };

    Ok(updated_todo)
}

// Provider to delete a todo
#[provider]
async fn delete_todo(_todo_id: u32) -> Result<(), String> {
    // Simulate API call delay
    sleep(Duration::from_millis(300)).await;

    // In a real app, this would delete from the database
    // For demo purposes, we'll always succeed
    Ok(())
}

#[component]
fn AddTodoForm() -> Element {
    let mut title = use_signal(|| String::new());
    let mut is_creating = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);

    // Cache invalidation hook
    let invalidate_todos = use_invalidate_provider(all_todos);

    let submit_todo = use_callback(move |_| {
        let title_value = title.read().clone();

        if title_value.trim().is_empty() {
            error_message.set(Some("Please enter a todo title".to_string()));
            return;
        }

        let invalidate_todos = invalidate_todos.clone();

        spawn(async move {
            is_creating.set(true);
            error_message.set(None);

            let request = CreateTodoRequest { title: title_value };

            match CreateTodoProvider::call(request).await {
                Ok(_new_todo) => {
                    // Clear the form
                    title.set(String::new());
                    error_message.set(None);

                    // Invalidate cache to refresh the todo list
                    invalidate_todos();
                }
                Err(err) => {
                    error_message.set(Some(err));
                }
            }

            is_creating.set(false);
        });
    });

    rsx! {
        div { class: "bg-white rounded-lg p-6 shadow-sm border",
            h3 { class: "text-lg font-semibold text-gray-800 mb-4", "➕ Add New Todo" }
            if let Some(error) = error_message.read().as_ref() {
                div { class: "mb-4 p-3 bg-red-50 border border-red-200 rounded text-red-700 text-sm",
                    "{error}"
                }
            }
            div { class: "flex gap-2",
                input {
                    class: "flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                    r#type: "text",
                    placeholder: "What needs to be done?",
                    value: "{title.read()}",
                    disabled: *is_creating.read(),
                    oninput: move |evt| title.set(evt.value()),
                    onkeypress: move |evt: KeyboardEvent| {
                        if evt.key() == Key::Enter {
                            submit_todo(());
                        }
                    },
                }
                button {
                    class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors",
                    disabled: *is_creating.read(),
                    onclick: move |_| submit_todo(()),
                    if *is_creating.read() {
                        "Adding..."
                    } else {
                        "Add"
                    }
                }
            }
        }
    }
}

#[component]
fn TodoItem(todo: Todo) -> Element {
    let mut is_updating = use_signal(|| false);
    let invalidate_todos = use_invalidate_provider(all_todos);

    let toggle_completed = use_callback({
        let todo = todo.clone();
        let invalidate_todos = invalidate_todos.clone();

        move |_| {
            let todo = todo.clone();
            let invalidate_todos = invalidate_todos.clone();

            spawn(async move {
                is_updating.set(true);

                let request = UpdateTodoRequest {
                    id: todo.id,
                    title: None,
                    completed: Some(!todo.completed),
                };

                match UpdateTodoProvider::call(request).await {
                    Ok(_) => {
                        invalidate_todos();
                    }
                    Err(_) => {
                        // In a real app, show error message
                    }
                }

                is_updating.set(false);
            });
        }
    });

    let delete_todo_action = use_callback({
        let todo = todo.clone();
        let invalidate_todos = invalidate_todos.clone();

        move |_| {
            let todo = todo.clone();
            let invalidate_todos = invalidate_todos.clone();

            spawn(async move {
                is_updating.set(true);

                match DeleteTodoProvider::call(todo.id).await {
                    Ok(_) => {
                        invalidate_todos();
                    }
                    Err(_) => {
                        // In a real app, show error message
                    }
                }

                is_updating.set(false);
            });
        }
    });

    let status_class = if todo.completed {
        "line-through text-gray-500"
    } else {
        "text-gray-800"
    };

    let checkbox_class = if todo.completed {
        "text-green-600"
    } else {
        "text-gray-400"
    };

    rsx! {
        div { class: "flex items-center gap-3 p-3 bg-white border rounded-lg hover:shadow-md transition-shadow",
            button {
                class: "text-xl {checkbox_class} hover:scale-110 transition-transform disabled:opacity-50",
                disabled: *is_updating.read(),
                onclick: move |_| toggle_completed(()),
                if todo.completed {
                    "✅"
                } else {
                    "⭕"
                }
            }
            span { class: "flex-1 {status_class}", "{todo.title}" }
            button {
                class: "text-red-500 hover:text-red-700 hover:scale-110 transition-all disabled:opacity-50",
                disabled: *is_updating.read(),
                onclick: move |_| delete_todo_action(()),
                "🗑️"
            }
        }
    }
}

#[component]
fn TodoList() -> Element {
    let todos_signal = use_future_provider(all_todos);

    rsx! {
        div { class: "space-y-4",
            h2 { class: "text-2xl font-bold text-gray-800", "📋 Your Todos" }
            match &*todos_signal.read() {
                AsyncState::Loading => rsx! {
                    div { class: "text-center py-8",
                        div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4" }
                        p { class: "text-gray-600", "Loading todos..." }
                    }
                },
                AsyncState::Success(todos) => rsx! {
                    div { class: "space-y-2",
                        if todos.is_empty() {
                            div { class: "text-center py-8 text-gray-500",
                                "🎉 No todos yet! Add one above to get started."
                            }
                        } else {
                            {todos.iter().map(|todo| rsx! {
                                TodoItem { key: "{todo.id}", todo: todo.clone() }
                            })}
                        }
                    }
                },
                AsyncState::Error(error) => rsx! {
                    div { class: "text-center py-8 text-red-600",
                        p { "❌ Error loading todos: {error}" }
                        button {
                            class: "mt-4 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700",
                            onclick: move |_| {},
                            "Retry"
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn TodoStats() -> Element {
    let todos_signal = use_future_provider(all_todos);

    rsx! {
        div { class: "bg-gradient-to-r from-blue-50 to-purple-50 rounded-lg p-4",
            h3 { class: "font-semibold text-gray-800 mb-3", "📊 Stats" }
            match &*todos_signal.read() {
                AsyncState::Loading => rsx! {
                    p { class: "text-gray-600", "Calculating..." }
                },
                AsyncState::Success(todos) => {
                    let completed = todos.iter().filter(|t| t.completed).count();
                    let total = todos.len();
                    let progress = if total > 0 { (completed * 100) / total } else { 0 };
                    rsx! {
                        div { class: "space-y-3",
                            div { class: "flex justify-between text-sm",
                                span { "Completed:" }
                                span { class: "font-semibold", "{completed}/{total}" }
                            }
                            div { class: "w-full bg-gray-200 rounded-full h-2",
                                div {
                                    class: "bg-gradient-to-r from-blue-500 to-purple-500 h-2 rounded-full transition-all duration-500",
                                    style: "width: {progress}%",
                                }
                            }
                            p { class: "text-xs text-gray-600 text-center", "{progress}% complete" }
                        }
                    }
                }
                AsyncState::Error(_) => rsx! {
                    p { class: "text-red-600 text-sm", "Stats unavailable" }
                },
            }
        }
    }
}

fn app() -> Element {
    // Provide the cache and refresh registry contexts at the app level
    use_context_provider(dioxus_riverpod::providers::ProviderCache::new);
    use_context_provider(dioxus_riverpod::providers::RefreshRegistry::new);

    rsx! {
        head {
            script { src: "https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4" }
        }
        div { class: "min-h-screen bg-gray-100 py-8 px-4",
            div { class: "max-w-2xl mx-auto",
                header { class: "text-center mb-8",
                    h1 { class: "text-4xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent mb-2",
                        "Simple Todo"
                    }
                    p { class: "text-gray-600", "A clean example of Dioxus Riverpod in action" }
                }
                div { class: "space-y-6",
                    AddTodoForm {}
                    TodoList {}
                    TodoStats {}
                    div { class: "bg-white rounded-lg p-4 text-sm text-gray-600",
                        h4 { class: "font-semibold mb-2", "🚀 Riverpod Features Demo:" }
                        ul { class: "space-y-1 text-xs",
                            li { "✓ Automatic caching with cache invalidation" }
                            li { "✓ Loading states and error handling" }
                            li { "✓ Type-safe async providers" }
                            li { "✓ Reactive updates when data changes" }
                            li { "✓ Simple, clean provider composition" }
                        }
                    }
                }
            }
        }
    }
}
