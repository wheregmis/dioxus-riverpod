# Dioxus-Riverpod Feature Roadmap & TODOs

## ✅ Already Implemented Features

**Dioxus-Riverpod currently has these features that are competitive with Riverpod and TanStack Query:**

### Core Functionality
- **✅ Basic Provider System**: Async provider support with automatic caching
- **✅ Family Providers**: Parameterized providers for dynamic data fetching  
- **✅ Cache Expiration**: TTL-based cache invalidation (`cache_expiration`)
- **✅ Interval Refresh**: Proactive background updates (`interval`)
- **✅ Stale-While-Revalidate**: Instant responses with background revalidation (`stale_time`)
- **✅ Auto-Dispose**: Automatic cleanup of unused providers to prevent memory leaks (`auto_dispose`)
- **✅ Manual Refresh**: Ability to manually refresh provider data via refresh registry
- **✅ Macro-based API**: Clean, declarative provider definition using `#[provider]` macro
- **✅ Error Handling**: Basic error propagation in async providers

### Current API Examples
```rust
// Stale-while-revalidate - serves stale data instantly, background refresh
#[provider(stale_time = "5s")]
async fn user_data_swr(id: u32) -> Result<User, Error> { }

// Cache expiration - data expires and requires full reload
#[provider(cache_expiration = "10s")]
async fn user_data(id: u32) -> Result<User, Error> { }

// Auto-dispose - automatically cleans up after 5 seconds of no usage
#[provider(auto_dispose = true, dispose_delay = "5s")]
async fn auto_dispose_data() -> Result<String, Error> { }

// Interval refresh - proactive background updates every 5 seconds
#[provider(interval = "5s")]
async fn live_metrics() -> Result<Metrics, Error> { }

// Combined - background updates every 30s, cache expires after 60s
#[provider(interval = "30s", cache_expiration = "1min")]
async fn server_status() -> Result<Status, Error> { }

// Family providers with SWR and auto-dispose
#[provider(stale_time = "5s", auto_dispose = true, dispose_delay = "3s")]
async fn user_posts(user_id: u32) -> Result<Vec<Post>, Error> { }
```

---

## High-Priority Features Implementation Plan

### 1. ✅ Stale-While-Revalidate (COMPLETED - Biggest UX Improvement)
**Goal**: Serve cached data immediately while fetching fresh data in the background

**STATUS**: ✅ **IMPLEMENTATION COMPLETE** - All SWR functionality working correctly

**Implementation Summary**:
- ✅ Added `stale_time` parameter to `#[provider]` macro
- ✅ Extended cache system with staleness detection and SWR-aware data access
- ✅ Implemented background revalidation that triggers UI updates when fresh data arrives
- ✅ Created comprehensive demo showcasing three distinct caching patterns
- ✅ All patterns work correctly with no overlapping behaviors or confusion

**API Examples**:
```rust
// Pure SWR - serves stale data instantly after 5s, background revalidation
#[provider(stale_time = "5s")]
async fn user_profile_swr() -> Result<User, Error> { }

// Traditional cache - shows loading when expired after 8s
#[provider(cache_expiration = "8s")]
async fn user_profile_traditional() -> Result<User, Error> { }

// No caching - always fresh, always shows loading
#[provider]
async fn user_profile_fresh() -> Result<User, Error> { }
```

**Completed Features**:
- ✅ **Core Implementation**
  - ✅ Added `stale_time` parameter to `#[provider]` macro
  - ✅ Modified cache storage to track data freshness separately from cache expiration
  - ✅ Implemented automatic background revalidation when stale data is accessed
  - ✅ Added comprehensive debug logging for SWR behavior

- ✅ **Cache State Management**
  - ✅ Extended `CacheEntry` with `is_stale()` method for staleness detection
  - ✅ Implemented `get_with_staleness()` method in `ProviderCache`
  - ✅ Added background revalidation scheduler triggered when stale data is accessed
  - ✅ Proper handling of race conditions between manual refresh and automatic revalidation

- ✅ **Integration Points**
  - ✅ Modified `use_provider` hook to return stale data immediately when accessed
  - ✅ Trigger automatic revalidation when stale (but not expired) data is accessed
  - ✅ Components re-render when revalidated fresh data arrives via refresh registry
  - ✅ Clean separation between SWR, traditional cache, and no-cache patterns

- ✅ **Testing & Examples**
  - ✅ Working demo (`simple_swr_demo.rs`) demonstrating all three caching patterns
  - ✅ Real-time UI showing distinct behaviors: instant SWR responses vs loading states
  - ✅ Background revalidation verified working with proper UI updates
  - ✅ All patterns tested and verified to work correctly without conflicts

---

### 2. ✅ Auto-dispose (COMPLETED - Prevents Memory Leaks)
**Goal**: Automatically clean up unused providers and their cached data

**STATUS**: ✅ **IMPLEMENTATION COMPLETE** - All auto-dispose functionality working correctly

**Implementation Summary**:
- ✅ Added `auto_dispose = true` and `dispose_delay` parameters to `#[provider]` macro
- ✅ Implemented reference counting system to track active provider usage
- ✅ Created disposal scheduling with configurable delays
- ✅ Integrated with component lifecycle for automatic cleanup
- ✅ Built comprehensive demo showcasing auto-dispose functionality

**API Examples**:
```rust
// Auto-dispose with custom delay
#[provider(auto_dispose = true, dispose_delay = "5s")]
async fn auto_dispose_data() -> Result<String, String> { }

// Parameterized auto-dispose provider
#[provider(auto_dispose = true, dispose_delay = "3s")]
async fn user_profile(user_id: u32) -> Result<String, String> { }

// Regular provider (no auto-dispose)
#[provider]
async fn regular_data() -> Result<String, String> { }
```

**Completed Features**:
- ✅ **Reference Counting System**
  - ✅ Added usage counter to `CacheEntry` with atomic operations
  - ✅ Track active `use_provider` hook instances via reference counting
  - ✅ Implement reference counting on provider access/release
  - ✅ Support for both parameterized and non-parameterized providers

- ✅ **API Design**
  - ✅ Added `auto_dispose = true` parameter to `#[provider]` macro
  - ✅ Added `dispose_delay` parameter with humantime duration format
  - ✅ Clean integration with existing provider API
  - ✅ Backward compatibility with existing providers

- ✅ **Disposal Logic**
  - ✅ Created `DisposalRegistry` with configurable delay scheduling
  - ✅ Implement graceful disposal checking reference counts
  - ✅ Add disposal cancellation when providers are accessed again
  - ✅ Handle disposal of dependent providers independently

- ✅ **Memory Management**
  - ✅ Automatic cleanup of unused cache entries after specified delays
  - ✅ Reference count tracking and proper cleanup on component unmount
  - ✅ Debug logging for disposal actions and reference counting
  - ✅ Prevention of disposal while providers are actively in use

- ✅ **Integration**
  - ✅ Hook into Dioxus component lifecycle via `use_drop`
  - ✅ Track component mount/unmount for reference counting
  - ✅ Ensure disposal doesn't affect active providers
  - ✅ Context-based disposal registry for provider management

- ✅ **Testing**
  - ✅ Working demo (`auto_dispose_demo.rs`) demonstrating all functionality
  - ✅ Reference counting correctness verified through console logging
  - ✅ Component lifecycle integration working properly
  - ✅ Memory leak prevention verified with actual disposal messages

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
