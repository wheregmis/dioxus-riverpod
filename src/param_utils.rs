//! Parameter normalization utilities for dioxus-provider

use std::fmt::Debug;
use std::hash::Hash;

/// Trait for normalizing different parameter formats to work with providers
///
/// This trait allows the `use_provider` hook to accept parameters in different formats:
/// - `()` for no parameters
/// - `(param,)` for single parameter in tuple (e.g., `(42,)`)
/// - Common primitive types directly (e.g., `42`, `"foo".to_string()`)
///
/// # Usage and Ambiguity
///
/// - If your provider expects a single parameter, you can pass it directly (e.g., `use_provider(my_provider(), 42)`) or as a single-element tuple (e.g., `use_provider(my_provider(), (42,))`).
/// - **Note:** If you pass a single-element tuple containing a primitive (e.g., `(42,)`), the tuple implementation will be used, not the direct primitive implementation. This is usually what you want, but be aware of the distinction.
/// - For string parameters, both `String` and `&str` are supported directly.
///
/// # Examples
///
/// ```rust
/// use dioxus_provider::prelude::*;
///
/// #[provider]
/// async fn fetch_user(user_id: u32) -> Result<User, String> { ... }
///
/// // All of these are valid:
/// let user = use_provider(fetch_user(), 42);      // direct primitive
/// let user = use_provider(fetch_user(), (42,));   // single-element tuple
/// let user = use_provider(fetch_user(), "foo".to_string()); // String
/// let user = use_provider(fetch_user(), ("foo".to_string(),)); // tuple with String
/// ```
pub trait IntoProviderParam {
    /// The target parameter type after conversion
    type Param: Clone + PartialEq + Hash + Debug + Send + Sync + 'static;

    /// Convert the input into the parameter format expected by the provider
    fn into_param(self) -> Self::Param;
}

// Implementation for no parameters: () -> ()
impl IntoProviderParam for () {
    type Param = ();

    fn into_param(self) -> Self::Param {
        // nothing needed
    }
}

// Implementation for tuple parameters: (Param,) -> Param
impl<T> IntoProviderParam for (T,)
where
    T: Clone + PartialEq + Hash + Debug + Send + Sync + 'static,
{
    type Param = T;

    fn into_param(self) -> Self::Param {
        self.0
    }
}

// Common direct parameter implementations to avoid conflicts
impl IntoProviderParam for u32 {
    type Param = u32;

    fn into_param(self) -> Self::Param {
        self
    }
}

impl IntoProviderParam for i32 {
    type Param = i32;

    fn into_param(self) -> Self::Param {
        self
    }
}

impl IntoProviderParam for String {
    type Param = String;

    fn into_param(self) -> Self::Param {
        self
    }
}

impl IntoProviderParam for &str {
    type Param = String;

    fn into_param(self) -> Self::Param {
        self.to_string()
    }
}
