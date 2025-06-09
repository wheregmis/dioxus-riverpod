# Dioxus-Riverpod Feature Roadmap & TODOs

## ✅ Already Implemented Features

**Dioxus-Riverpod currently has these features that are competitive with Riverpod and TanStack Query:**

### Core Functionality
- **✅ Basic Provider System**: Async provider support with automatic caching
- **✅ Family Providers**: Parameterized providers for dynamic data fetching  
- **✅ Cache Expiration**: TTL-based cache invalidation (`cache_expiration_secs/millis`)
- **✅ Interval Refresh**: Proactive background updates (`interval_secs/millis`)
- **✅ Manual Refresh**: Ability to manually refresh provider data via refresh registry
- **✅ Macro-based API**: Clean, declarative provider definition using `#[provider]` macro
- **✅ Error Handling**: Basic error propagation in async providers

### Current API Examples
```rust
// Cache expiration - data expires and requires full reload
#[provider(cache_expiration_secs = 10)]
async fn user_data(id: u32) -> Result<User, Error> { }

// Interval refresh - proactive background updates every 5 seconds
#[provider(interval_secs = 5)]
async fn live_metrics() -> Result<Metrics, Error> { }

// Combined - background updates every 30s, cache expires after 60s
#[provider(interval_secs = 30, cache_expiration_secs = 60)]
async fn server_status() -> Result<Status, Error> { }

// Family providers with expiration
#[provider(cache_expiration_secs = 5)]
async fn user_posts(user_id: u32) -> Result<Vec<Post>, Error> { }
```

---

## High-Priority Features Implementation Plan

### 1. 🔄 Stale-While-Revalidate (Biggest UX Improvement)
**Goal**: Serve cached data immediately while fetching fresh data in the background

**NOTE**: We already have `cache_expiration_secs/millis` and `interval_secs/millis`, but these work differently:
- **Cache expiration** = Hard expiration (cache miss, full reload required)  
- **Interval refresh** = Proactive background updates (regardless of access)
- **Stale-while-revalidate** = Serve stale data immediately + trigger background refresh on access

#### TODOs:
- [ ] **Core Implementation**
  - [ ] Add `stale_time` parameter to `#[provider]` macro (defines when to trigger revalidation)
  - [ ] Modify cache storage to track data freshness separately from cache expiration
  - [ ] Implement automatic background revalidation when stale data is accessed
  - [ ] Add cache hit/miss/stale/revalidating metrics for debugging

- [ ] **API Design**
  ```rust
  #[provider(stale_time = 5_000, cache_time = 30_000)]
  async fn user_profile(id: u32) -> Result<User, Error> {
      // Fresh for 5s, triggers revalidation after 5s, cache expires after 30s
      // 5-30s: serves stale data + background refresh
      // 30s+: cache miss, full reload
  }
  ```

- [ ] **Cache State Management**
  - [ ] Extend `CacheEntry` to include `fresh_until` timestamp (separate from `expires_at`)
  - [ ] Implement `is_fresh()`, `is_stale()`, and `is_expired()` methods on cache entries
  - [ ] Add background revalidation scheduler triggered when stale data is accessed
  - [ ] Handle race conditions between manual refresh and automatic revalidation

- [ ] **Integration Points**
  - [ ] Modify `use_provider` hook to return stale data immediately when accessed
  - [ ] Trigger automatic revalidation when stale (but not expired) data is accessed
  - [ ] Ensure components re-render when revalidated fresh data arrives
  - [ ] Add configuration for revalidation behavior (immediate, debounced, etc.)

- [ ] **Testing**
  - [ ] Unit tests for freshness and staleness time calculations
  - [ ] Integration tests for automatic revalidation behavior
  - [ ] Performance tests to ensure revalidation doesn't block UI
  - [ ] Example demonstrating stale-while-revalidate behavior

---

### 2. 🗑️ Auto-dispose (Prevents Memory Leaks)
**Goal**: Automatically clean up unused providers and their cached data

#### TODOs:
- [ ] **Reference Counting System**
  - [ ] Add usage counter to `CacheEntry`
  - [ ] Track active `use_provider` hook instances
  - [ ] Implement reference counting on provider access/release
  - [ ] Add weak reference system for family providers

- [ ] **API Design**
  ```rust
  #[provider(auto_dispose = true, dispose_delay = 10_000)]
  async fn user_posts(user_id: u32) -> Vec<Post> {
      // Auto-disposed 10s after last usage
  }
  ```

- [ ] **Disposal Logic**
  - [ ] Create disposal scheduler with configurable delay
  - [ ] Implement graceful disposal (wait for ongoing requests)
  - [ ] Add disposal hooks for cleanup callbacks
  - [ ] Handle disposal of dependent providers

- [ ] **Memory Management**
  - [ ] Periodic garbage collection of unused entries
  - [ ] Memory usage tracking and reporting
  - [ ] Configurable memory limits with LRU eviction
  - [ ] Debug utilities to inspect memory usage

- [ ] **Integration**
  - [ ] Hook into Dioxus component lifecycle
  - [ ] Track component mount/unmount for reference counting
  - [ ] Ensure disposal doesn't affect active providers
  - [ ] Add manual disposal API for advanced use cases

- [ ] **Testing**
  - [ ] Memory leak detection tests
  - [ ] Reference counting correctness tests
  - [ ] Component lifecycle integration tests
  - [ ] Performance impact measurement

---

### 3. 🔁 Query Retries (Improves Reliability)
**Goal**: Automatically retry failed requests with configurable strategies

#### TODOs:
- [ ] **Retry Configuration**
  - [ ] Add retry parameters to `#[provider]` macro
  - [ ] Support different retry strategies (fixed, exponential, custom)
  - [ ] Configurable retry conditions (error types, status codes)
  - [ ] Maximum retry count and timeout limits

- [ ] **API Design**
  ```rust
  #[provider(
      retry_count = 3,
      retry_strategy = "exponential",
      retry_delay = 1000,
      retry_on = "network_error"
  )]
  async fn api_call() -> Result<Data, Error> {
      // Auto-retry on failure
  }
  ```

- [ ] **Retry Logic Implementation**
  - [ ] Exponential backoff with jitter
  - [ ] Linear backoff strategy
  - [ ] Custom retry delay functions
  - [ ] Circuit breaker pattern for failing services
  - [ ] Retry state tracking and logging

- [ ] **Error Handling**
  - [ ] Differentiate between retryable and non-retryable errors
  - [ ] Preserve original error information through retries
  - [ ] Add retry attempt metadata to error responses
  - [ ] Implement timeout handling for long-running retries

- [ ] **Integration**
  - [ ] Integrate with existing async provider system
  - [ ] Ensure retries don't block other providers
  - [ ] Add retry status to loading states
  - [ ] Provide retry progress feedback to UI

- [ ] **Testing**
  - [ ] Flaky network simulation tests
  - [ ] Retry strategy correctness tests
  - [ ] Performance tests for retry overhead
  - [ ] End-to-end reliability improvement measurement

---

### 4. ⏳ Loading States (Better Developer Experience)
**Goal**: Rich loading state information with progress tracking

#### TODOs:
- [ ] **Enhanced AsyncState**
  - [ ] Extend `AsyncState` with loading metadata
  - [ ] Add progress tracking for long-running operations
  - [ ] Include retry attempt information
  - [ ] Add timestamp information (started, last updated)

- [ ] **API Design**
  ```rust
  let user = use_provider!(user_profile(id));
  // Access: user.is_loading(), user.is_stale(), user.data(), user.error()
  // Progress: user.progress(), user.retry_count(), user.started_at()
  ```

- [ ] **Loading State Types**
  - [ ] Initial loading (first fetch)
  - [ ] Background loading (stale-while-revalidate)
  - [ ] Retry loading (with attempt count)
  - [ ] Mutation loading (for updates)
  - [ ] Dependent loading (waiting for dependencies)

- [ ] **Progress Tracking**
  - [ ] Optional progress callback in provider functions
  - [ ] Stream-based progress updates
  - [ ] Percentage completion tracking
  - [ ] Custom progress metadata support

- [ ] **Developer Tools**
  - [ ] Loading state inspector utilities
  - [ ] Debug hooks for loading state changes
  - [ ] Performance metrics (load times, cache hit rates)
  - [ ] Loading state visualization helpers

- [ ] **Integration**
  - [ ] Seamless integration with existing `use_provider` hook
  - [ ] Loading state propagation to dependent providers
  - [ ] Integration with Dioxus suspense patterns
  - [ ] Optimistic loading state updates

- [ ] **Testing**
  - [ ] Loading state transition tests
  - [ ] Progress tracking accuracy tests
  - [ ] Performance impact assessment
  - [ ] UI integration examples

---

### 5. 🔗 Dependent Queries (Enables Complex Data Flows)
**Goal**: Declarative dependency management between providers

#### TODOs:
- [ ] **Dependency Declaration**
  - [ ] Add `depends_on` parameter to `#[provider]` macro
  - [ ] Support multiple dependencies
  - [ ] Conditional dependencies based on data
  - [ ] Circular dependency detection and prevention

- [ ] **API Design**
  ```rust
  #[provider(depends_on = [user_profile])]
  async fn user_posts(user_id: u32) -> Vec<Post> {
      let user = get_dependency!(user_profile(user_id))?;
      // Only execute if user_profile succeeds
  }
  ```

- [ ] **Dependency Resolution**
  - [ ] Dependency graph construction and validation
  - [ ] Topological sorting for execution order
  - [ ] Parallel execution of independent dependencies
  - [ ] Dependency result passing to dependent providers

- [ ] **Error Propagation**
  - [ ] Failure handling in dependency chains
  - [ ] Partial success scenarios
  - [ ] Dependency retry coordination
  - [ ] Error context preservation through dependency chain

- [ ] **Caching with Dependencies**
  - [ ] Invalidate dependents when dependencies change
  - [ ] Smart cache invalidation based on dependency data
  - [ ] Dependency-aware cache keys
  - [ ] Optimize cache usage across dependency chains

- [ ] **Advanced Features**
  - [ ] Dynamic dependencies based on runtime data
  - [ ] Conditional execution based on dependency results
  - [ ] Dependency mocking for testing
  - [ ] Dependency visualization tools

- [ ] **Testing**
  - [ ] Dependency resolution correctness tests
  - [ ] Circular dependency detection tests
  - [ ] Error propagation tests
  - [ ] Performance tests for complex dependency graphs

---

## Implementation Timeline

### Phase 1 (Weeks 1-2): Foundation
- [ ] Stale-while-revalidate core implementation
- [ ] Enhanced cache entry structure
- [ ] Basic background refresh mechanism

### Phase 2 (Weeks 3-4): Memory Management
- [ ] Auto-dispose implementation
- [ ] Reference counting system
- [ ] Memory usage tracking

### Phase 3 (Weeks 5-6): Reliability
- [ ] Query retry implementation
- [ ] Error handling improvements
- [ ] Circuit breaker patterns

### Phase 4 (Weeks 7-8): Developer Experience
- [ ] Enhanced loading states
- [ ] Progress tracking
- [ ] Developer tools and debugging

### Phase 5 (Weeks 9-10): Advanced Features
- [ ] Dependent queries implementation
- [ ] Dependency graph management
- [ ] Integration testing and optimization

### Phase 6 (Weeks 11-12): Polish & Documentation
- [ ] Comprehensive testing
- [ ] Performance optimization
- [ ] Documentation and examples
- [ ] Migration guides

---

## Technical Considerations

### Architecture Changes Needed
- [ ] **Cache Layer Redesign**
  - Extend `CacheEntry` with metadata (staleness, usage count, dependencies)
  - Implement cache invalidation strategies
  - Add background task management

- [ ] **Macro System Enhancement**
  - Extend `#[provider]` macro with new parameters
  - Generate dependency resolution code
  - Add compile-time dependency validation

- [ ] **Hook System Integration**
  - Enhance `use_provider` with loading state access
  - Add dependency injection mechanisms
  - Implement reference counting hooks

### Performance Considerations
- [ ] Minimize overhead of new features on existing code
- [ ] Efficient dependency graph traversal
- [ ] Background task resource management
- [ ] Memory usage optimization

### Breaking Changes
- [ ] Document all breaking changes
- [ ] Provide migration paths
- [ ] Version compatibility strategy
- [ ] Deprecation timeline for old APIs

---

## Success Metrics

### Performance
- [ ] No more than 5% overhead on simple providers
- [ ] Background refresh completes within stale_time window
- [ ] Memory usage reduction with auto-dispose

### Developer Experience
- [ ] Reduced boilerplate for common patterns
- [ ] Better error messages and debugging
- [ ] Comprehensive examples and documentation

### Reliability
- [ ] Automatic recovery from transient failures
- [ ] Graceful handling of network issues
- [ ] Consistent behavior across dependency chains

---

## Documentation & Examples Needed

### API Documentation
- [ ] Updated macro parameter reference
- [ ] Loading state API documentation
- [ ] Dependency management guide
- [ ] Migration guide from current version

### Examples
- [ ] Stale-while-revalidate demo
- [ ] Auto-dispose memory management example
- [ ] Retry strategy showcase
- [ ] Complex dependency chain example
- [ ] Real-world application patterns

### Testing Documentation
- [ ] Testing strategies for async providers
- [ ] Mocking dependencies in tests
- [ ] Performance testing guidelines
- [ ] Integration testing best practices
