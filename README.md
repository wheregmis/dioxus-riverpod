# Dioxus Riverpod

A state management library for Dioxus applications, inspired by Riverpod for Flutter.

## Features

- Provider-based state management
- Automatic cache management with expiration
- SWR (Stale-While-Revalidate) pattern support
- Auto-disposal of unused providers
- Background refresh intervals
- Humantime duration parsing

## Usage

See the examples directory for comprehensive usage examples:

- `simple_swr_demo.rs` - Basic SWR pattern demonstration
- `cache_expiration_demo.rs` - Cache expiration testing
- `auto_dispose_demo.rs` - Auto-disposal functionality
- `comprehensive_cache_test.rs` - Full feature showcase

## Documentation

This library provides macros and utilities for managing application state in Dioxus applications with automatic caching, refresh intervals, and cleanup.