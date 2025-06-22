/*!
 * Global Dependency Injection System
 *
 * Provides a type-safe way to register and access shared dependencies
 * that don't fit well as provider parameters (e.g., API clients, databases).
 */

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

/// Global registry for dependency injection
static DEPENDENCY_REGISTRY: OnceLock<DependencyRegistry> = OnceLock::new();

/// Registry that holds all injected dependencies
pub struct DependencyRegistry {
    dependencies: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl DependencyRegistry {
    /// Create a new dependency registry
    fn new() -> Self {
        Self {
            dependencies: RwLock::new(HashMap::new()),
        }
    }

    /// Register a dependency of type T
    pub fn register<T: Send + Sync + 'static>(&self, dependency: T) -> Result<(), String> {
        let type_id = TypeId::of::<T>();
        let mut deps = self
            .dependencies
            .write()
            .map_err(|_| "Failed to acquire write lock on dependencies")?;

        if deps.contains_key(&type_id) {
            return Err(format!(
                "Dependency of type {} already registered",
                std::any::type_name::<T>()
            ));
        }

        deps.insert(type_id, Arc::new(dependency));
        Ok(())
    }

    /// Get a dependency of type T
    pub fn get<T: Send + Sync + 'static>(&self) -> Result<Arc<T>, String> {
        let type_id = TypeId::of::<T>();
        let deps = self
            .dependencies
            .read()
            .map_err(|_| "Failed to acquire read lock on dependencies")?;

        let dependency = deps.get(&type_id).ok_or_else(|| {
            format!(
                "Dependency of type {} not found",
                std::any::type_name::<T>()
            )
        })?;

        dependency.clone().downcast::<T>().map_err(|_| {
            format!(
                "Failed to downcast dependency of type {}",
                std::any::type_name::<T>()
            )
        })
    }

    /// Check if a dependency of type T is registered
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.dependencies
            .read()
            .map(|deps| deps.contains_key(&type_id))
            .unwrap_or(false)
    }

    /// Clear all dependencies (mainly for testing)
    pub fn clear(&self) -> Result<(), String> {
        let mut deps = self
            .dependencies
            .write()
            .map_err(|_| "Failed to acquire write lock on dependencies")?;
        deps.clear();
        Ok(())
    }

    /// Get all registered dependency type names (for debugging)
    pub fn list_types(&self) -> Result<Vec<String>, String> {
        let deps = self
            .dependencies
            .read()
            .map_err(|_| "Failed to acquire read lock on dependencies")?;

        // Note: We can't easily get type names from TypeId,
        // so this is mainly useful for debugging count
        Ok(vec![format!("{} dependencies registered", deps.len())])
    }
}

/// Initialize the global dependency registry
pub fn init_dependency_injection() {
    DEPENDENCY_REGISTRY.get_or_init(DependencyRegistry::new);
}

/// Register a global dependency
pub fn register_dependency<T: Send + Sync + 'static>(dependency: T) -> Result<(), String> {
    let registry = DEPENDENCY_REGISTRY
        .get()
        .ok_or("Dependency registry not initialized. Call init_dependency_injection() first.")?;
    registry.register(dependency)
}

/// Get a global dependency
pub fn inject<T: Send + Sync + 'static>() -> Result<Arc<T>, String> {
    let registry = DEPENDENCY_REGISTRY
        .get()
        .ok_or("Dependency registry not initialized. Call init_dependency_injection() first.")?;
    registry.get()
}

/// Check if a dependency is registered
pub fn has_dependency<T: Send + Sync + 'static>() -> bool {
    DEPENDENCY_REGISTRY
        .get()
        .map(|registry| registry.contains::<T>())
        .unwrap_or(false)
}

/// Clear all dependencies (mainly for testing)
pub fn clear_dependencies() -> Result<(), String> {
    let registry = DEPENDENCY_REGISTRY
        .get()
        .ok_or("Dependency registry not initialized")?;
    registry.clear()
}

/// Macro for easy dependency injection in providers
#[macro_export]
macro_rules! inject {
    ($type:ty) => {
        $crate::injection::inject::<$type>()
            .map_err(|e| format!("Dependency injection failed: {}", e))?
    };
}

/// Macro for registering dependencies with error handling
#[macro_export]
macro_rules! register {
    ($dependency:expr) => {
        $crate::injection::register_dependency($dependency)
            .map_err(|e| format!("Dependency registration failed: {}", e))?
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestService {
        name: String,
    }

    impl TestService {
        fn new(name: String) -> Self {
            Self { name }
        }

        fn get_name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_dependency_injection() {
        init_dependency_injection();

        // Clear any existing dependencies
        clear_dependencies().unwrap();

        // Register a dependency
        let service = TestService::new("test".to_string());
        register_dependency(service).unwrap();

        // Inject the dependency
        let injected: Arc<TestService> = inject().unwrap();
        assert_eq!(injected.get_name(), "test");

        // Check if dependency exists
        assert!(has_dependency::<TestService>());
        assert!(!has_dependency::<String>());
    }

    #[test]
    fn test_duplicate_registration() {
        init_dependency_injection();
        clear_dependencies().unwrap();

        let service1 = TestService::new("first".to_string());
        let service2 = TestService::new("second".to_string());

        // First registration should succeed
        assert!(register_dependency(service1).is_ok());

        // Second registration should fail
        assert!(register_dependency(service2).is_err());
    }

    #[test]
    fn test_missing_dependency() {
        init_dependency_injection();
        clear_dependencies().unwrap();

        // Try to inject non-existent dependency
        let result: Result<Arc<TestService>, String> = inject();
        assert!(result.is_err());
    }
}
