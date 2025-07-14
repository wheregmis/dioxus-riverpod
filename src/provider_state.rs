//! ProviderState: Async state enum for dioxus-provider

use dioxus_lib::prelude::Task;

/// Represents the state of an async operation
#[derive(Clone, PartialEq, Debug)]
pub enum ProviderState<T, E> {
    /// The operation is currently loading
    Loading { task: Task },
    /// The operation completed successfully with data
    Success(T),
    /// The operation failed with an error
    Error(E),
}

impl<T, E> ProviderState<T, E> {
    /// Returns true if the state is currently loading
    pub fn is_loading(&self) -> bool {
        matches!(self, ProviderState::Loading { task: _ })
    }

    /// Returns true if the state contains successful data
    pub fn is_success(&self) -> bool {
        matches!(self, ProviderState::Success(_))
    }

    /// Returns true if the state contains an error
    pub fn is_error(&self) -> bool {
        matches!(self, ProviderState::Error(_))
    }

    /// Returns the data if successful, None otherwise
    pub fn data(&self) -> Option<&T> {
        match self {
            ProviderState::Success(data) => Some(data),
            _ => None,
        }
    }

    /// Returns the error if failed, None otherwise
    pub fn error(&self) -> Option<&E> {
        match self {
            ProviderState::Error(error) => Some(error),
            _ => None,
        }
    }

    /// Maps a ProviderState<T, E> to ProviderState<U, E> by applying a function to the contained data if successful.
    pub fn map<U, F>(self, op: F) -> ProviderState<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            ProviderState::Success(data) => ProviderState::Success(op(data)),
            ProviderState::Error(e) => ProviderState::Error(e),
            ProviderState::Loading { task } => ProviderState::Loading { task },
        }
    }

    /// Maps a ProviderState<T, E> to ProviderState<T, F> by applying a function to the contained error if failed.
    pub fn map_err<F, O>(self, op: O) -> ProviderState<T, F>
    where
        O: FnOnce(E) -> F,
    {
        match self {
            ProviderState::Success(data) => ProviderState::Success(data),
            ProviderState::Error(e) => ProviderState::Error(op(e)),
            ProviderState::Loading { task } => ProviderState::Loading { task },
        }
    }

    /// Chains a ProviderState<T, E> to ProviderState<U, E> by applying a function to the contained data if successful.
    pub fn and_then<U, F>(self, op: F) -> ProviderState<U, E>
    where
        F: FnOnce(T) -> ProviderState<U, E>,
    {
        match self {
            ProviderState::Success(data) => op(data),
            ProviderState::Error(e) => ProviderState::Error(e),
            ProviderState::Loading { task } => ProviderState::Loading { task },
        }
    }
}
