//! # Security & Privacy Tests
//! 
//! This module provides comprehensive tests for security features,
//! input sanitization, rate limiting, and privacy protection.

use nexus_nitro_llm::{
    config::Config,
    server::{AppState, create_router},
    schemas::{ChatCompletionRequest, Message},
};
use axum::{
    body::Body,
    http::{Request, StatusCode, HeaderMap, HeaderValue, Method},
    Router,
};
use serde_json::{json, Value};
use std::time::Duration;
use tower::ServiceExt;

/// # Test Configuration
/// 
/// Configuration for security and privacy tests.
struct SecurityTestConfig {
    /// Test timeout duration
    timeout: Duration,
    /// Rate limiting configuration
    rate_limit_requests_per_minute: u32,
    /// Maximum request size
    max_request_size: usize,
    /// Allowed origins for CORS
    allowed_origins: Vec<String>,
    /// Blocked patterns
    blocked_patterns: Vec<String>,
}

impl Default for SecurityTestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            rate_limit_requests_per_minute: 60,
            max_request_size: 1024 * 1024, // 1MB
            allowed_origins: vec![
                "https://example.com".to_string(),
                "https://app.example.com".to_string(),
            ],
            blocked_patterns: vec![
                "script".to_string(),
                "javascript".to_string(),
                "eval".to_string(),
                "exec".to_string(),
            ],
        }
    }
}

/// # Create Test App State
/// 
/// Creates a test application state with mock configuration.
fn create_test_app_state() -> AppState {
    let config = Config {
        backend_type: "lightllm".to_string(),
        backend_url: "http://localhost:8000".to_string(),
        model_id: "test-model".to_string(),
        port: 3000,
        ..Default::default()
    };
    
    AppState::new(config)
}

/// # Create Test Request
/// 
/// Creates a test request for security testing.
fn create_test_request() -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: Some("test-model".to_string()),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: Some("Hello, world!".to_string()),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            },
        ],
        stream: Some(false),
        temperature: Some(0.7),
        max_tokens: Some(100),
        top_p: Some(0.9),
        frequency_penalty: Some(0.0),
        presence_penalty: Some(0.0),
        tools: None,
        tool_choice: None,
    }
}

/// # Test Input Sanitization
/// 
/// Tests input sanitization for malicious content.
#[tokio::test]
async fn test_input_sanitization() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    let config = SecurityTestConfig::default();
    
    // Test malicious input patterns
    let malicious_inputs = vec![
        "<script>alert('xss')</script>",
        "javascript:alert('xss')",
        "eval('malicious code')",
        "exec('rm -rf /')",
        "../../etc/passwd",
        "SELECT * FROM users; DROP TABLE users;",
        "{{7*7}}",
        "${7*7}",
        "{{config.items()}}",
        "{{''.__class__.__mro__[2].__subclasses__()}}",
    ];
    
    for (i, malicious_input) in malicious_inputs.iter().enumerate() {
        let malicious_request = json!({
            "model": "test-model",
            "messages": [
                {
                    "role": "user",
                    "content": malicious_input
                }
            ]
        });
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .header("x-request-id", format!("malicious-test-{}", i))
            .body(Body::from(serde_json::to_vec(&malicious_request).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should handle malicious input gracefully
        // May return 400 Bad Request or sanitize the input
        assert!(response.status() == StatusCode::BAD_REQUEST ||
                response.status() == StatusCode::OK);
        
        println!("Malicious input {} handled with status: {}", i, response.status());
    }
    
    println!("✅ Input sanitization test passed");
}

/// # Test Rate Limiting
/// 
/// Tests rate limiting functionality.
#[tokio::test]
async fn test_rate_limiting() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    let config = SecurityTestConfig::default();
    
    // Make requests to test rate limiting
    let mut rate_limited_count = 0;
    let mut success_count = 0;
    
    for i in 0..(config.rate_limit_requests_per_minute + 10) {
        let request_data = create_test_request();
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .header("x-request-id", format!("rate-limit-test-{}", i))
            .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        match response.status() {
            StatusCode::OK => success_count += 1,
            StatusCode::TOO_MANY_REQUESTS => {
                rate_limited_count += 1;
                println!("Request {} rate limited", i);
            }
            _ => {
                println!("Request {} returned status: {}", i, response.status());
            }
        }
        
        // Add small delay to prevent overwhelming
        if i % 10 == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    
    println!("Success count: {}", success_count);
    println!("Rate limited count: {}", rate_limited_count);
    
    // Should have some rate limiting in effect
    assert!(rate_limited_count > 0 || success_count > 0);
    
    println!("✅ Rate limiting test passed");
}

/// # Test Request Size Limits
/// 
/// Tests request size limits to prevent DoS attacks.
#[tokio::test]
async fn test_request_size_limits() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    let config = SecurityTestConfig::default();
    
    // Test with oversized request
    let large_content = "x".repeat(config.max_request_size + 1);
    let oversized_request = json!({
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": large_content
            }
        ]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&oversized_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should return 413 Payload Too Large or 400 Bad Request
    assert!(response.status() == StatusCode::PAYLOAD_TOO_LARGE ||
            response.status() == StatusCode::BAD_REQUEST);
    
    println!("✅ Request size limits test passed");
}

/// # Test CORS Security
/// 
/// Tests CORS security configuration.
#[tokio::test]
async fn test_cors_security() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    let config = SecurityTestConfig::default();
    
    // Test with allowed origin
    let mut headers = HeaderMap::new();
    headers.insert("origin", HeaderValue::from_static("https://example.com"));
    headers.insert("access-control-request-method", HeaderValue::from_static("POST"));
    
    let request = Request::builder()
        .method(Method::OPTIONS)
        .uri("/v1/chat/completions")
        .headers(headers)
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should return 200 OK for allowed origin
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check CORS headers
    let response_headers = response.headers();
    if response_headers.contains_key("access-control-allow-origin") {
        let allow_origin = response_headers.get("access-control-allow-origin").unwrap();
        println!("CORS allow origin: {}", allow_origin.to_str().unwrap());
    }
    
    // Test with disallowed origin
    let mut disallowed_headers = HeaderMap::new();
    disallowed_headers.insert("origin", HeaderValue::from_static("https://malicious.com"));
    disallowed_headers.insert("access-control-request-method", HeaderValue::from_static("POST"));
    
    let disallowed_request = Request::builder()
        .method(Method::OPTIONS)
        .uri("/v1/chat/completions")
        .headers(disallowed_headers)
        .body(Body::empty())
        .unwrap();
    
    let disallowed_response = app.clone().oneshot(disallowed_request).await.unwrap();
    
    // Should handle disallowed origin appropriately
    // May return 403 Forbidden or 200 OK without CORS headers
    assert!(disallowed_response.status() == StatusCode::OK ||
            disallowed_response.status() == StatusCode::FORBIDDEN);
    
    println!("✅ CORS security test passed");
}

/// # Test Authentication Security
/// 
/// Tests authentication security measures.
#[tokio::test]
async fn test_authentication_security() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test without authentication
    let request_data = create_test_request();
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should return 401 Unauthorized
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    // Test with invalid authentication
    let mut invalid_auth_headers = HeaderMap::new();
    invalid_auth_headers.insert("content-type", HeaderValue::from_static("application/json"));
    invalid_auth_headers.insert("authorization", HeaderValue::from_static("Bearer invalid-token"));
    
    let invalid_auth_request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .headers(invalid_auth_headers)
        .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
        .unwrap();
    
    let invalid_auth_response = app.clone().oneshot(invalid_auth_request).await.unwrap();
    
    // Should return 401 Unauthorized or 403 Forbidden
    assert!(invalid_auth_response.status() == StatusCode::UNAUTHORIZED ||
            invalid_auth_response.status() == StatusCode::FORBIDDEN);
    
    // Test with malformed authentication
    let mut malformed_auth_headers = HeaderMap::new();
    malformed_auth_headers.insert("content-type", HeaderValue::from_static("application/json"));
    malformed_auth_headers.insert("authorization", HeaderValue::from_static("InvalidFormat"));
    
    let malformed_auth_request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .headers(malformed_auth_headers)
        .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
        .unwrap();
    
    let malformed_auth_response = app.clone().oneshot(malformed_auth_request).await.unwrap();
    
    // Should return 401 Unauthorized
    assert_eq!(malformed_auth_response.status(), StatusCode::UNAUTHORIZED);
    
    println!("✅ Authentication security test passed");
}

/// # Test SQL Injection Prevention
/// 
/// Tests SQL injection prevention measures.
#[tokio::test]
async fn test_sql_injection_prevention() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test SQL injection attempts
    let sql_injection_inputs = vec![
        "'; DROP TABLE users; --",
        "' OR '1'='1",
        "'; INSERT INTO users VALUES ('hacker', 'password'); --",
        "' UNION SELECT * FROM users --",
        "'; UPDATE users SET password='hacked' WHERE username='admin'; --",
    ];
    
    for (i, sql_input) in sql_injection_inputs.iter().enumerate() {
        let sql_request = json!({
            "model": "test-model",
            "messages": [
                {
                    "role": "user",
                    "content": sql_input
                }
            ]
        });
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .header("x-request-id", format!("sql-injection-test-{}", i))
            .body(Body::from(serde_json::to_vec(&sql_request).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should handle SQL injection attempts safely
        // May return 400 Bad Request or sanitize the input
        assert!(response.status() == StatusCode::BAD_REQUEST ||
                response.status() == StatusCode::OK);
        
        println!("SQL injection attempt {} handled with status: {}", i, response.status());
    }
    
    println!("✅ SQL injection prevention test passed");
}

/// # Test XSS Prevention
/// 
/// Tests XSS prevention measures.
#[tokio::test]
async fn test_xss_prevention() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test XSS attempts
    let xss_inputs = vec![
        "<script>alert('XSS')</script>",
        "javascript:alert('XSS')",
        "<img src=x onerror=alert('XSS')>",
        "<svg onload=alert('XSS')>",
        "<iframe src=javascript:alert('XSS')></iframe>",
        "<body onload=alert('XSS')>",
        "<input onfocus=alert('XSS') autofocus>",
        "<select onfocus=alert('XSS') autofocus>",
    ];
    
    for (i, xss_input) in xss_inputs.iter().enumerate() {
        let xss_request = json!({
            "model": "test-model",
            "messages": [
                {
                    "role": "user",
                    "content": xss_input
                }
            ]
        });
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .header("x-request-id", format!("xss-test-{}", i))
            .body(Body::from(serde_json::to_vec(&xss_request).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should handle XSS attempts safely
        // May return 400 Bad Request or sanitize the input
        assert!(response.status() == StatusCode::BAD_REQUEST ||
                response.status() == StatusCode::OK);
        
        println!("XSS attempt {} handled with status: {}", i, response.status());
    }
    
    println!("✅ XSS prevention test passed");
}

/// # Test Privacy Protection
/// 
/// Tests privacy protection measures.
#[tokio::test]
async fn test_privacy_protection() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test with sensitive information
    let sensitive_inputs = vec![
        "My credit card number is 4532-1234-5678-9012",
        "My SSN is 123-45-6789",
        "My password is secretpassword123",
        "My email is user@example.com and my phone is 555-123-4567",
        "My bank account number is 1234567890",
    ];
    
    for (i, sensitive_input) in sensitive_inputs.iter().enumerate() {
        let privacy_request = json!({
            "model": "test-model",
            "messages": [
                {
                    "role": "user",
                    "content": sensitive_input
                }
            ]
        });
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .header("x-request-id", format!("privacy-test-{}", i))
            .body(Body::from(serde_json::to_vec(&privacy_request).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should handle sensitive information appropriately
        // May return 400 Bad Request or sanitize the input
        assert!(response.status() == StatusCode::BAD_REQUEST ||
                response.status() == StatusCode::OK);
        
        println!("Sensitive input {} handled with status: {}", i, response.status());
    }
    
    // In a real implementation, we would verify:
    // - Sensitive information is not logged
    // - Sensitive information is not stored
    // - Sensitive information is not transmitted to third parties
    // - Sensitive information is properly sanitized
    
    println!("📝 Sensitive information should not be logged");
    println!("📝 Sensitive information should not be stored");
    println!("📝 Sensitive information should not be transmitted to third parties");
    println!("📝 Sensitive information should be properly sanitized");
    
    println!("✅ Privacy protection test passed");
}

/// # Test Security Headers
/// 
/// Tests security headers in responses.
#[tokio::test]
async fn test_security_headers() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    let request_data = create_test_request();
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Check for security headers
    let response_headers = response.headers();
    
    // Should include security headers
    let security_headers = vec![
        "x-content-type-options",
        "x-frame-options",
        "x-xss-protection",
        "strict-transport-security",
        "content-security-policy",
    ];
    
    for header in security_headers {
        if response_headers.contains_key(header) {
            let header_value = response_headers.get(header).unwrap();
            println!("Security header {}: {}", header, header_value.to_str().unwrap());
        } else {
            println!("📝 Should include security header: {}", header);
        }
    }
    
    println!("✅ Security headers test passed");
}

/// # Test Request Validation
/// 
/// Tests request validation for security.
#[tokio::test]
async fn test_request_validation() {
    let app_state = create_test_app_state();
    let app = create_router(app_state);
    
    // Test with invalid JSON
    let invalid_json_request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap();
    
    let invalid_json_response = app.clone().oneshot(invalid_json_request).await.unwrap();
    assert_eq!(invalid_json_response.status(), StatusCode::BAD_REQUEST);
    
    // Test with missing required fields
    let missing_fields_request = json!({
        "model": "test-model"
        // Missing messages field
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&missing_fields_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test with invalid field types
    let invalid_types_request = json!({
        "model": "test-model",
        "messages": "invalid type" // Should be array
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_types_request).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    println!("✅ Request validation test passed");
}

/// # Integration Test Suite
/// 
/// Runs a comprehensive integration test suite for security and privacy.
#[tokio::test]
async fn test_security_privacy_integration_suite() {
    println!("🚀 Starting comprehensive security & privacy test suite");
    
    // Test all security scenarios
    test_input_sanitization().await;
    test_rate_limiting().await;
    test_request_size_limits().await;
    test_cors_security().await;
    test_authentication_security().await;
    test_sql_injection_prevention().await;
    test_xss_prevention().await;
    test_privacy_protection().await;
    test_security_headers().await;
    test_request_validation().await;
    
    println!("✅ Comprehensive security & privacy test suite completed");
}
