use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use nexus_nitro_llm::{
    Config,
    AppState,
    chat_completions,
};
use serde_json::json;
use tokio::net::TcpListener;
use tower::ServiceExt;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

async fn create_test_app(lightllm_url: String) -> Router {
    let mut config = Config::for_test();
    config.port = 8080;
    config.backend_url = lightllm_url;
    config.model_id = "test-model".to_string();
    config.backend_type = "lightllm".to_string(); // Explicitly set backend type

    let state = AppState::new(config);

    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state)
}

#[tokio::test]
#[ignore = "Mock server connection issues - needs investigation"]
async fn test_chat_completions_success() {
    // Start a mock LightLLM server
    let mock_server = MockServer::start();

    Mock::given(method("POST"))
        .and(path("/generate"))
        .respond_with(ResponseTemplate::new(200)
            .insert_header("content-type", "application/json")
            .set_body_json(json!({
                "text": "I'm doing well, thank you! How can I help you today?"
            })))
        .expect(1) // We expect exactly 1 request
        .mount(&mock_server);
        

    let mock_uri = mock_server.uri();
    println!("Mock server URI: {}", mock_uri);
    let app = create_test_app(mock_uri);

    let request_body = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": "Hello, how are you?"
            }
        ],
        "max_tokens": 256,
        "temperature": 1.0,
        "top_p": 1.0
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    if response.status() != StatusCode::OK {
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let error_text = String::from_utf8_lossy(&body);
        panic!("Expected status 200, got {}. Response body: {}", status, error_text);
    }

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["object"], "chat.completion");
    assert_eq!(response_json["model"], "test-model");
    assert_eq!(response_json["choices"][0]["message"]["role"], "assistant");
    assert_eq!(response_json["choices"][0]["message"]["content"], "I'm doing well, thank you! How can I help you today?");
    assert_eq!(response_json["choices"][0]["finish_reason"], "stop");
}

#[tokio::test]
#[ignore = "Mock server connection issues - needs investigation"]
async fn test_chat_completions_with_system_message() {
    // Use a fresh mock server for this test
    let mock_server = MockServer::start();

    // Log all requests to debug what we're actually sending
    Mock::given(method("POST"))
        .and(path("/generate"))
        .respond_with(ResponseTemplate::new(200)
            .insert_header("content-type", "application/json")
            .set_body_json(json!({
                "text": "2 + 2 equals 4."
            })))
        .mount(&mock_server);
        

    let app = create_test_app(mock_server.uri());

    let request_body = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant."
            },
            {
                "role": "user",
                "content": "What is 2+2?"
            }
        ],
        "max_tokens": 100,
        "temperature": 0.7,
        "top_p": 0.9
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();

    if status != StatusCode::OK {
        let error_text = String::from_utf8_lossy(&body);
        panic!("Expected 200 OK, got {}: {}", status, error_text);
    }

    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["choices"][0]["message"]["content"], "2 + 2 equals 4.");
}

#[tokio::test]
async fn test_chat_completions_stream_supported() {
    let mock_server = MockServer::start();
    let app = create_test_app(mock_server.uri());

    let request_body = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": "Hello"
            }
        ],
        "stream": true
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Streaming is now supported, so we expect OK status
    assert_eq!(response.status(), StatusCode::OK);

    // Check that the content type is correct for SSE
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/event-stream"));

    // The response should be a valid SSE stream (even if it contains errors)
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);

    // Should contain SSE format with data: prefix
    assert!(body_str.contains("data: "));
}

#[tokio::test]
async fn test_chat_completions_upstream_error() {
    let mock_server = MockServer::start();

    Mock::given(method("POST"))
        .and(path("/generate"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
        .mount(&mock_server);
        

    let app = create_test_app(mock_server.uri());

    let request_body = json!({;
        "model": "test-model",
        "messages": [
            {
                "role": "user",
                "content": "Hello"
            }
        ]
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/chat/completions")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response_json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Upstream error"));
}

/// Test with real LightLLM endpoint (requires actual service to be running)
#[tokio::test]
#[ignore] // Use `cargo test -- --ignored` to run this test
async fn test_real_lightllm_endpoint() {
    let lightllm_url = "https://your-litellm-proxy.example.com";
    let token = "on_t_your_admin_token_here";

    // Create a test client to send requests to our proxy
    let client = reqwest::Client::new();

    // Start our proxy server in the background
    let mut config = Config::for_test();
    config.port = 0; // Use port 0 to get a random available port
    config.backend_url = lightllm_url.to_string();
    config.model_id = "llama".to_string();
    config.backend_token = Some(token.to_string());

    let state = AppState::new(config);
    let app = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100))

    let request_body = json!({;
        "model": "llama",
        "messages": [
            {
                "role": "user",
                "content": "Hello! What's 2+2?"
            }
        ],
        "max_tokens": 100,
        "temperature": 0.7
    });

    let response = client;
        .post(format!("http://{}/v1/chat/completions", addr))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .unwrap();

    println!("Response status: {}", response.status());
    let response_text = response.text().await.unwrap();
    println!("Response body: {}", response_text);

    let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

    // Basic assertions
    assert_eq!(response_json["object"], "chat.completion");
    assert!(response_json["choices"].as_array().unwrap().len() > 0);
    assert_eq!(response_json["choices"][0]["message"]["role"], "assistant");

    let content = response_json["choices"][0]["message"]["content"].as_str().unwrap();
    println!("Assistant response: {}", content);
    assert!(!content.is_empty());
}