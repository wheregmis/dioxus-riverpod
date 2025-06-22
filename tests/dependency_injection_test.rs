// Test different approaches for handling dependencies like API clients

use dioxus_provider::prelude::*;
use std::sync::Arc;

// Example: API client that doesn't implement PartialEq/Hash
#[derive(Clone)]
struct ApiClient {
    base_url: String,
    // HTTP client, auth tokens, etc.
}

impl ApiClient {
    fn new(base_url: String) -> Self {
        Self { base_url }
    }

    async fn fetch_user(&self, id: u32) -> Result<String, String> {
        // Simulate API call
        Ok(format!("User {} from {}", id, self.base_url))
    }
}

// Problem solved: Use global dependency injection!

// Approach 1: Global dependency injection with manual injection
#[derive(Clone, PartialEq)]
struct UserProvider;

impl Provider<u32> for UserProvider {
    type Output = String;
    type Error = String;

    async fn run(&self, user_id: u32) -> Result<Self::Output, Self::Error> {
        let client = inject::<ApiClient>()?;
        client.fetch_user(user_id).await
    }

    fn id(&self, user_id: &u32) -> String {
        format!("user_{}", user_id)
    }
}

// Approach 2: Macro-based dependency injection (future enhancement)
// This syntax would be nice but requires more macro work:
/*
#[provider(inject = [ApiClient])]
async fn fetch_user_with_macro(user_id: u32) -> Result<String, String> {
    // injected_0 (ApiClient) would be automatically available
    injected_0.fetch_user(user_id).await
}
*/

// Approach 3: Multiple dependencies
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Database {
    connection_string: String,
}

impl Database {
    fn new(connection_string: String) -> Self {
        Self { connection_string }
    }

    async fn log_access(&self, user_id: u32, resource: &str) -> Result<(), String> {
        // Simulate database write
        println!("DB: User {} accessed {}", user_id, resource);
        Ok(())
    }
}

#[derive(Clone, PartialEq)]
struct UserWithLoggingProvider;

impl Provider<u32> for UserWithLoggingProvider {
    type Output = String;
    type Error = String;

    async fn run(&self, user_id: u32) -> Result<Self::Output, Self::Error> {
        let client = inject::<ApiClient>()?;
        let db = inject::<Database>()?;

        // Log the access
        db.log_access(user_id, "user_profile").await?;

        // Fetch the user
        client.fetch_user(user_id).await
    }

    fn id(&self, user_id: &u32) -> String {
        format!("user_with_logging_{}", user_id)
    }
}

#[tokio::test]
async fn test_dependency_injection() {
    // Initialize dependency injection
    init_dependency_injection();

    // Clear any previous dependencies
    clear_dependencies().unwrap();

    // Register dependencies
    register_dependency(ApiClient::new("https://api.example.com".to_string())).unwrap();
    register_dependency(Database::new("postgresql://localhost/test".to_string())).unwrap();

    // Test provider with injected dependencies
    let provider = UserProvider;
    let result = provider.run(42).await.unwrap();
    assert!(result.contains("User 42"));
    assert!(result.contains("api.example.com"));

    // Test provider with multiple dependencies
    let provider_with_logging = UserWithLoggingProvider;
    let result = provider_with_logging.run(123).await.unwrap();
    assert!(result.contains("User 123"));
}

#[test]
fn test_dependency_registration() {
    init_dependency_injection();
    clear_dependencies().unwrap();

    // Test registration
    let client = ApiClient::new("https://test.com".to_string());
    assert!(register_dependency(client).is_ok());

    // Test duplicate registration fails
    let client2 = ApiClient::new("https://test2.com".to_string());
    assert!(register_dependency(client2).is_err());

    // Test injection works
    let injected: Arc<ApiClient> = inject().unwrap();
    assert_eq!(injected.base_url, "https://test.com");
}
