# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- `init_global_providers()` function for application initialization
- Support for async functions with automatic error handling
- Automatic re-rendering when data changes
- Comprehensive examples demonstrating all features

### Documentation
- Complete API documentation with examples
- Comprehensive README with getting started guide
- Multiple example applications demonstrating different use cases
- Cross-platform compatibility documentation