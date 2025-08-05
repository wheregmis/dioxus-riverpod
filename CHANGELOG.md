# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.7](https://github.com/wheregmis/dioxus-provider/compare/dioxus-provider-v0.0.6...dioxus-provider-v0.0.7) - 2025-08-05

### <!-- 3 -->Other

- small refactor
- bump to alpha 3
- Refactor todo app filter button props and input signal
- Update todo_app.rs
- Tracking mutation state and showing it
- Refactor filter signal variable in FilterButton
- Add comprehensive todo app example with persistence
- Refactor string formatting to use inline syntax
- Improve param_utils.rs documentation for IntoProviderParam
- Remove ProviderState from cache.rs and update re-exports
- more cllippy fixes
- some clippy fixes
- Replace tokio sleep with platform sleep in example
- Revamp examples section in README
- Add combinator methods to ProviderState and example
- Add detailed Rust doc comments to cache module
- Add module-level docs for cache management
- Refactor provider trait bounds into reusable traits
- Move IntoProviderParam trait to param_utils module
- Extract ProviderState to separate module and re-export
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `ProviderState` now supports combinator methods: `map`, `map_err`, and `and_then` for ergonomic state transformations in provider logic and UI code.
- Expanded documentation for all cache and provider state APIs.

## [0.0.6](https://github.com/wheregmis/dioxus-provider/compare/dioxus-provider-v0.0.5...dioxus-provider-v0.0.6) - 2025-07-12

### <!-- 3 -->Other

- Rename AsyncState to ProviderState throughout codebase
- Support for Suspense
### Breaking Change
- `AsyncState` has been renamed to `ProviderState` for clarity and consistency with the library's naming. The loading state is now `ProviderState::Loading { task: Task }`. All pattern matches must use `ProviderState::Loading { .. }` instead of `AsyncState::Loading`. This affects all provider and consumer code that matches on loading state.

## [0.0.5](https://github.com/wheregmis/dioxus-provider/compare/dioxus-provider-v0.0.4...dioxus-provider-v0.0.5) - 2025-07-08

### <!-- 3 -->Other

- Optimize cache set to avoid unnecessary updates
- Make CacheEntry cached_at field thread-safe
- Reapply "Avoid redundant cache updates for unchanged values"

## [0.0.4](https://github.com/wheregmis/dioxus-provider/compare/dioxus-provider-v0.0.3...dioxus-provider-v0.0.4) - 2025-06-30

### <!-- 3 -->Other

- Add Dependabot config and enhance release-plz settings
- Create FUNDING.yml
- Prefix composed provider result variables for uniqueness
- Update README with new features and examples
- Remove macro-based dependency injection support
- Refactor composable provider demo for CSS and structure
- Switch from tokio to futures join and platform-specific sleep
- Add composable provider support and demo example
- Update dependencies and refactor error handling in dependency injection
- Unify provider parameter handling with IntoProviderParam

## [0.0.3](https://github.com/wheregmis/dioxus-provider/compare/dioxus-provider-v0.0.2...dioxus-provider-v0.0.3) - 2025-06-24

### Other

- Re-invalidate optimistic cache keys on mutation rollback
- Update README.md
- Update README.md
- Mutation support
- initial mutation support

## [0.1.1](https://github.com/wheregmis/dioxus-provider/compare/dioxus-provider-macros-v0.1.0...dioxus-provider-macros-v0.1.1) - 2025-06-24

### Other

- Mutation support
- initial mutation support

## [0.0.2](https://github.com/wheregmis/dioxus-provider/compare/dioxus-provider-v0.0.1...dioxus-provider-v0.0.2) - 2025-06-22

### Other

- Update README with development warning and clean up example

## [0.0.1] - 2024-12-19

### Added
- **Global Provider System**: Manage application-wide data without nesting context providers
- **Declarative `#[provider]` Macro**: Define data sources with a simple attribute macro
- **Intelligent Caching Strategies**:
  - **Stale-While-Revalidate (SWR)**: Serve stale data instantly while fetching fresh data in the background
  - **Time-to-Live (TTL) Cache Expiration**: Automatically evict cached data after a configured duration
- **Automatic Refresh**: Keep data fresh with interval-based background refetching
- **Parameterized Queries**: Create providers that depend on dynamic arguments (e.g., fetching user data by ID)
- **Manual Cache Control**: Hooks to manually invalidate cached data or clear the entire cache
- **Cross-Platform Support**: Works seamlessly on both Desktop and Web (WASM)
- **Minimal Boilerplate**: Get started in minutes with intuitive hooks and macros

### Features
- `use_provider` hook for consuming provider data
- `use_invalidate_provider` hook for manual cache invalidation
- `use_clear_provider_cache` hook for clearing entire cache
- `