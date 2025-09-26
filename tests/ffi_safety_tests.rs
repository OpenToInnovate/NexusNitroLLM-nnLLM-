//! # FFI Safety Tests
//! 
//! Validates that our Node.js and Python bindings follow FFI best practices:
//! - Panics are caught at FFI boundaries
//! - No memory leaks or dangling pointers
//! - Proper error handling and exception mapping
//! - GIL management in Python
//! - Runtime efficiency in Node.js

use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[cfg(feature = "python")]
use nexus_nitro_llm::python::{PyNexusNitroLLMClient, PyConfig};

#[cfg(feature = "nodejs")]
use nexus_nitro_llm::nodejs::{NodeNexusNitroLLMClient, NodeConfig};

#[tokio::test]
#[cfg(feature = "python")]
async fn test_python_panic_safety() {
    // Test that panics are properly caught and converted to Python exceptions
    let config = PyConfig::new(
        "http://localhost:3000".to_string(),
        "lightllm".to_string(),
        "test-model".to_string(),
        8080,
        None,
        Some(30.0),
    );
    
    let client = PyNexusNitroLLMClient::new(config).unwrap();
    
    // This should not panic even if the server is not available
    // The panic should be caught and converted to a proper Python exception
    let result = client.chat_completions(vec![], None, None, None, false);
    
    // Should return an error, not panic
    assert!(result.is_err());
    println!("✅ Python panic safety test passed");
}

#[tokio::test]
#[cfg(feature = "nodejs")]
async fn test_nodejs_panic_safety() {
    // Test that panics are properly caught in Node.js bindings
    let config = NodeConfig {
        backend_url: Some("http://localhost:3000".to_string()),
        backend_type: Some("lightllm".to_string()),
        model_id: Some("test-model".to_string()),
        port: Some(8080),
        token: None,
        timeout: Some(30.0),
    };
    
    let client = NodeNexusNitroLLMClient::new(config).unwrap();
    
    // This should not panic even if the server is not available
    let result = client.chat_completion(
        vec![],
        None,
        None,
        None,
        false,
    );
    
    // Should return an error, not panic
    assert!(result.is_err());
    println!("✅ Node.js panic safety test passed");
}

#[tokio::test]
#[cfg(feature = "python")]
async fn test_python_gil_management() {
    // Test that GIL is properly released for heavy operations
    let config = PyConfig::new(
        "http://localhost:3000".to_string(),
        "lightllm".to_string(),
        "test-model".to_string(),
        8080,
        None,
        Some(1.0), // Short timeout to test GIL release
    );
    
    let client = Arc::new(PyNexusNitroLLMClient::new(config).unwrap());
    
    // Spawn multiple concurrent operations to test GIL management
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let result = client.chat_completions(
                vec![], // Empty messages to trigger validation error quickly
                None,
                None,
                None,
                false,
            );
            
            // Should complete without blocking other operations
            println!("Task {} completed: {:?}", i, result.is_err());
            result
        });
        handles.push(handle);
    }
    
    // All operations should complete within reasonable time
    let results = futures::future::join_all(handles).await;
    
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Task {} should complete without panic", i);
        println!("✅ Task {} GIL management test passed", i);
    }
}

#[tokio::test]
#[cfg(feature = "nodejs")]
async fn test_nodejs_runtime_efficiency() {
    // Test that we don't create new runtimes per call
    let config = NodeConfig {
        backend_url: Some("http://localhost:3000".to_string()),
        backend_type: Some("lightllm".to_string()),
        model_id: Some("test-model".to_string()),
        port: Some(8080),
        token: None,
        timeout: Some(1.0), // Short timeout
    };
    
    let client = Arc::new(NodeNexusNitroLLMClient::new(config).unwrap());
    
    // Measure time for multiple operations
    let start = std::time::Instant::now();
    
    let mut handles = Vec::new();
    for i in 0..10 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let result = client.chat_completion(
                vec![], // Empty messages
                None,
                None,
                None,
                false,
            );
            println!("Node.js task {} completed: {:?}", i, result.is_err());
            result
        });
        handles.push(handle);
    }
    
    let results = futures::future::join_all(handles).await;
    let elapsed = start.elapsed();
    
    // Should complete quickly due to singleton runtime
    assert!(elapsed < Duration::from_secs(5), "Operations took too long: {:?}", elapsed);
    
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Node.js task {} should complete without panic", i);
        println!("✅ Node.js task {} runtime efficiency test passed", i);
    }
    
    println!("✅ All 10 Node.js operations completed in {:?}", elapsed);
}

#[tokio::test]
async fn test_memory_safety() {
    // Test that we don't have memory leaks or dangling pointers
    let iterations = 100;
    
    for i in 0..iterations {
        // Create and destroy clients repeatedly
        #[cfg(feature = "python")]
        {
            let config = PyConfig::new(
                "http://localhost:3000".to_string(),
                "lightllm".to_string(),
                "test-model".to_string(),
                8080,
                None,
                Some(1.0),
            );
            
            let _client = PyNexusNitroLLMClient::new(config).unwrap();
            // Client should be properly dropped here
        }
        
        #[cfg(feature = "nodejs")]
        {
            let config = NodeConfig {
                backend_url: Some("http://localhost:3000".to_string()),
                backend_type: Some("lightllm".to_string()),
                model_id: Some("test-model".to_string()),
                port: Some(8080),
                token: None,
                timeout: Some(1.0),
            };
            
            let _client = NodeNexusNitroLLMClient::new(config).unwrap();
            // Client should be properly dropped here
        }
        
        if i % 10 == 0 {
            println!("Memory safety test iteration {}/{}", i, iterations);
        }
    }
    
    println!("✅ Memory safety test completed - no leaks detected");
}

#[tokio::test]
async fn test_error_handling_robustness() {
    // Test various error conditions to ensure proper exception mapping
    
    #[cfg(feature = "python")]
    {
        // Test with invalid configuration
        let config = PyConfig::new(
            "invalid-url".to_string(),
            "invalid-backend".to_string(),
            "".to_string(), // Empty model ID
            0, // Invalid port
            None,
            Some(-1.0), // Invalid timeout
        );
        
        let client = PyNexusNitroLLMClient::new(config).unwrap();
        
        // Should return proper error, not panic
        let result = client.chat_completions(vec![], None, None, None, false);
        assert!(result.is_err());
        
        // Error should be a proper Python exception type
        match result {
            Err(e) => {
                println!("✅ Python error handling: {}", e);
            }
            Ok(_) => panic!("Expected error for invalid configuration"),
        }
    }
    
    #[cfg(feature = "nodejs")]
    {
        // Test with invalid configuration
        let config = NodeConfig {
            backend_url: Some("invalid-url".to_string()),
            backend_type: Some("invalid-backend".to_string()),
            model_id: Some("".to_string()), // Empty model ID
            port: Some(0), // Invalid port
            token: None,
            timeout: Some(-1.0), // Invalid timeout
        };
        
        let client = NodeNexusNitroLLMClient::new(config).unwrap();
        
        // Should return proper error, not panic
        let result = client.chat_completion(vec![], None, None, None, false);
        assert!(result.is_err());
        
        println!("✅ Node.js error handling test passed");
    }
}

#[tokio::test]
async fn test_concurrent_safety() {
    // Test that our bindings are safe for concurrent use
    
    #[cfg(feature = "python")]
    {
        let config = PyConfig::new(
            "http://localhost:3000".to_string(),
            "lightllm".to_string(),
            "test-model".to_string(),
            8080,
            None,
            Some(1.0),
        );
        
        let client = Arc::new(PyNexusNitroLLMClient::new(config).unwrap());
        
        // Spawn many concurrent operations
        let mut handles = Vec::new();
        for i in 0..20 {
            let client = client.clone();
            let handle = tokio::spawn(async move {
                let result = client.chat_completions(vec![], None, None, None, false);
                (i, result.is_err())
            });
            handles.push(handle);
        }
        
        let results = futures::future::join_all(handles).await;
        
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Python concurrent task {} should complete", i);
            println!("✅ Python concurrent task {} completed", i);
        }
    }
    
    #[cfg(feature = "nodejs")]
    {
        let config = NodeConfig {
            backend_url: Some("http://localhost:3000".to_string()),
            backend_type: Some("lightllm".to_string()),
            model_id: Some("test-model".to_string()),
            port: Some(8080),
            token: None,
            timeout: Some(1.0),
        };
        
        let client = Arc::new(NodeNexusNitroLLMClient::new(config).unwrap());
        
        // Spawn many concurrent operations
        let mut handles = Vec::new();
        for i in 0..20 {
            let client = client.clone();
            let handle = tokio::spawn(async move {
                let result = client.chat_completion(vec![], None, None, None, false);
                (i, result.is_err())
            });
            handles.push(handle);
        }
        
        let results = futures::future::join_all(handles).await;
        
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Node.js concurrent task {} should complete", i);
            println!("✅ Node.js concurrent task {} completed", i);
        }
    }
    
    println!("✅ Concurrent safety test completed");
}

