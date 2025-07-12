//! Parameter normalization utilities for dioxus-provider

use std::fmt::Debug;
use std::hash::Hash;

/// Trait for normalizing different parameter formats to work with providers
///
/// This trait allows the `use_provider` hook to accept parameters in different formats:
/// - `()` for no parameters
/// - `(param,)` for single parameter in tuple
/// - Common primitive types directly
///
/// This eliminates the need for multiple `UseProvider` implementations.
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
