//! # Smoke Test Framework
//! 
//! Deadline-driven, cancellation-aware testing that catches big problems quickly.
//! Tests the core behaviors: deadlines, cancellation, retries, streaming, and resource hygiene.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::{timeout, sleep};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

/// Error taxonomy for smoke tests
#[derive(Debug, Clone)]
pub enum SmokeTestError {
    Timeout { phase: String, elapsed_ms: u64, remaining_budget_ms: u64 },
    Canceled { phase: String, elapsed_ms: u64 },
    RateLimited { retry_after_secs: u64, elapsed_ms: u64 },
    Server5xx { status: u16, elapsed_ms: u64 },
    BadRequest { status: u16, elapsed_ms: u64 },
    ConnectionFailed { phase: String, elapsed_ms: u64 },
    Unexpected { message: String, elapsed_ms: u64 },
}

impl std::fmt::Display for SmokeTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmokeTestError::Timeout { phase, elapsed_ms, remaining_budget_ms } => {
                write!(f, "Timeout({}) elapsed={}ms remaining={}ms", phase, elapsed_ms, remaining_budget_ms)
            }
            SmokeTestError::Canceled { phase, elapsed_ms } => {
                write!(f, "Canceled({}) elapsed={}ms", phase, elapsed_ms)
            }
            SmokeTestError::RateLimited { retry_after_secs, elapsed_ms } => {
                write!(f, "RateLimited retry_after={}s elapsed={}ms", retry_after_secs, elapsed_ms)
            }
            SmokeTestError::Server5xx { status, elapsed_ms } => {
                write!(f, "Server5xx({}) elapsed={}ms", status, elapsed_ms)
            }
            SmokeTestError::BadRequest { status, elapsed_ms } => {
                write!(f, "BadRequest({}) elapsed={}ms", status, elapsed_ms)
            }
            SmokeTestError::ConnectionFailed { phase, elapsed_ms } => {
                write!(f, "ConnectionFailed({}) elapsed={}ms", phase, elapsed_ms)
            }
            SmokeTestError::Unexpected { message, elapsed_ms } => {
                write!(f, "Unexpected({}) elapsed={}ms", message, elapsed_ms)
            }
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub backoff_base: f64,
    pub max_backoff_ms: u64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_base: 2.0,
            max_backoff_ms: 5000,
            jitter: true,
        }
    }
}

/// Timeout configuration
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub connect_ms: u64,
    pub tls_ms: u64,
    pub read_ms: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect_ms: 2000,
            tls_ms: 2000,
            read_ms: 8000,
        }
    }
}

/// Smoke test request configuration
#[derive(Debug, Clone)]
pub struct SmokeTestConfig {
    pub base_url: String,
    pub model: String,
    pub deadline_ms: u64,
    pub timeouts: TimeoutConfig,
    pub retry: RetryConfig,
    pub idempotency_key: Option<String>,
}

impl Default for SmokeTestConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
            model: "test-model".to_string(),
            deadline_ms: 10000,
            timeouts: TimeoutConfig::default(),
            retry: RetryConfig::default(),
            idempotency_key: Some(Uuid::new_v4().to_string()),
        }
    }
}

/// Smoke test client
pub struct SmokeTestClient {
    client: Client,
    config: SmokeTestConfig,
}

impl SmokeTestClient {
    pub fn new(config: SmokeTestConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeouts.read_ms))
            .connect_timeout(Duration::from_millis(config.timeouts.connect_ms))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, config }
    }

    /// Make a chat completion request with deadline and cancellation support
    pub async fn chat_completion(
        &self,
        messages: Vec<serde_json::Value>,
        signal: Option<tokio_util::sync::CancellationToken>,
    ) -> Result<serde_json::Value, SmokeTestError> {
        let start_time = Instant::now();
        let deadline = start_time + Duration::from_millis(self.config.deadline_ms);
        
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < self.config.retry.max_attempts {
            attempt += 1;
            let attempt_start = Instant::now();
            
            // Check if we've exceeded the deadline
            if attempt_start > deadline {
                return Err(SmokeTestError::Timeout {
                    phase: "deadline_exceeded".to_string(),
                    elapsed_ms: start_time.elapsed().as_millis() as u64,
                    remaining_budget_ms: 0,
                });
            }

            // Calculate remaining budget for this attempt
            let remaining_budget = deadline.duration_since(attempt_start);
            let timeout_duration = std::cmp::min(remaining_budget, Duration::from_millis(self.config.timeouts.read_ms));

            // Create request body
            let body = json!({;
                "model": self.config.model,
                "messages": messages,
                "max_tokens": 50
            });

            // Make the request with timeout and cancellation
            let result = self.make_request_with_cancellation(&body, timeout_duration, signal.clone());

            match result {
                Ok(response) => {
                    // Parse response
                    match response.json::<serde_json::Value>().await {
                        Ok(data) => return Ok(data),
                        Err(e) => {
                            last_error = Some(SmokeTestError::Unexpected {
                                message: format!("JSON parse error: {}", e),
                                elapsed_ms: attempt_start.elapsed().as_millis() as u64,
                            });
                        }
                    }
                }
                Err(SmokeTestError::Canceled { .. }) => {
                    return Err(SmokeTestError::Canceled {
                        phase: format!("attempt_{}", attempt),
                        elapsed_ms: attempt_start.elapsed().as_millis() as u64,
                    });
                }
                Err(SmokeTestError::Timeout { .. }) => {
                    last_error = Some(SmokeTestError::Timeout {
                        phase: format!("attempt_{}", attempt),
                        elapsed_ms: attempt_start.elapsed().as_millis() as u64,
                        remaining_budget_ms: deadline.duration_since(attempt_start).as_millis() as u64,
                    });
                }
                Err(SmokeTestError::RateLimited { retry_after_secs, .. }) => {
                    // Check if we have enough time for retry
                    let retry_duration = Duration::from_secs(retry_after_secs);
                    if attempt_start + retry_duration > deadline {
                        return Err(SmokeTestError::RateLimited {
                            retry_after_secs,
                            elapsed_ms: attempt_start.elapsed().as_millis() as u64,
                        });
                    }
                    last_error = Some(SmokeTestError::RateLimited {
                        retry_after_secs,
                        elapsed_ms: attempt_start.elapsed().as_millis() as u64,
                    });
                }
                Err(SmokeTestError::Server5xx { .. }) => {
                    last_error = Some(result.unwrap_err());
                }
                Err(e) => {
                    // Non-retriable errors
                    return Err(e);
                }
            }

            // Calculate backoff
            if attempt < self.config.retry.max_attempts {
                let backoff_ms = self.calculate_backoff(attempt);
                let backoff_duration = Duration::from_millis(backoff_ms);
                
                // Check if backoff would exceed deadline
                if attempt_start + backoff_duration > deadline {
                    break;
                }

                sleep(backoff_duration)
            }
        }

        Err(last_error.unwrap_or_else(|| SmokeTestError::Unexpected {
            message: "Max attempts exceeded".to_string(),
            elapsed_ms: start_time.elapsed().as_millis() as u64,
        }))
    }

    /// Make request with cancellation support
    async fn make_request_with_cancellation(
        &self,
        body: &serde_json::Value,
        timeout_duration: Duration,
        signal: Option<tokio_util::sync::CancellationToken>,
    ) -> Result<reqwest::Response, SmokeTestError> {
        let url = format!("{}/v1/chat/completions", self.config.base_url);
        
        let mut request = self.client
            .post(&url)
            .json(body);

        // Add idempotency key if provided
        if let Some(key) = &self.config.idempotency_key {
            request = request.header("Idempotency-Key", key);
        }

        // Execute request with timeout and cancellation
        let request_future = request.send();

        let result = if let Some(cancel_token) = signal {;
            tokio::select! {
                response = request_future => response,
                _ = cancel_token.cancelled() => {
                    return Err(SmokeTestError::Canceled {
                        phase: "request_cancelled".to_string(),
                        elapsed_ms: 0, // Will be set by caller
                    });
                }
            }
        } else {
            request_future.await
        };

        match result {
            Ok(response) => {
                let status = response.status().as_u16();
                match status {
                    200..=299 => Ok(response),
                    400..=499 => {
                        if status == 429 {
                            let retry_after = response
                                .headers()
                                .get("retry-after")
                                .and_then(|h| h.to_str().ok())
                                .and_then(|s| s.parse::<u64>().ok())
                                .unwrap_or(1);
                            
                            Err(SmokeTestError::RateLimited { retry_after_secs: retry_after, elapsed_ms: 0 })
                        } else {
                            Err(SmokeTestError::BadRequest { status, elapsed_ms: 0 })
                        }
                    }
                    500..=599 => Err(SmokeTestError::Server5xx { status, elapsed_ms: 0 }),
                    _ => Err(SmokeTestError::Unexpected {
                        message: format!("Unexpected status: {}", status),
                        elapsed_ms: 0,
                    }),
                }
            }
            Err(e) => {
                if e.is_timeout() {
                    Err(SmokeTestError::Timeout {
                        phase: "request_timeout".to_string(),
                        elapsed_ms: timeout_duration.as_millis() as u64,
                        remaining_budget_ms: 0,
                    })
                } else if e.is_connect() {
                    Err(SmokeTestError::ConnectionFailed {
                        phase: "connection_failed".to_string(),
                        elapsed_ms: 0,
                    })
                } else {
                    Err(SmokeTestError::Unexpected {
                        message: e.to_string(),
                        elapsed_ms: 0,
                    })
                }
            }
        }
    }

    /// Calculate exponential backoff with jitter
    fn calculate_backoff(&self, attempt: u32) -> u64 {
        let base_delay = (self.config.retry.backoff_base.powi(attempt as i32 - 1) * 1000.0) as u64;
        let delay = std::cmp::min(base_delay, self.config.retry.max_backoff_ms);
        
        if self.config.retry.jitter {
            let jitter = fastrand::u64(0..delay / 2);
            delay + jitter
        } else {
            delay
        }
    }
}

/// Smoke test suite
pub struct SmokeTestSuite {
    client: SmokeTestClient,
}

impl SmokeTestSuite {
    pub fn new(config: SmokeTestConfig) -> Self {
        Self {
            client: SmokeTestClient::new(config),
        }
    }

    /// Test 1: Cancel during DNS resolution
    pub async fn test_cancel_during_dns(&self) -> Result<(), String> {
        println!("🧪 Testing cancellation during DNS...");
        
        let cancel_token = tokio_util::sync::CancellationToken::new();
        
        // Cancel immediately (simulating DNS phase)
        cancel_token.cancel();
        
        let messages = vec![json!({"role": "user", "content": "Hello"})];
        let start = Instant::now();
        
        match self.client.chat_completion(messages, Some(cancel_token)).await {
            Err(SmokeTestError::Canceled { phase, elapsed_ms }) => {
                println!("✅ Canceled during {} in {}ms", phase, elapsed_ms);
                Ok(())
            }
            Ok(_) => Err("Expected cancellation, but got success".to_string()),
            Err(e) => Err(format!("Expected cancellation, but got: {}", e)),
        }
    }

    /// Test 2: Cancel during connection
    pub async fn test_cancel_during_connect(&self) -> Result<(), String> {
        println!("🧪 Testing cancellation during connection...");
        
        let cancel_token = tokio_util::sync::CancellationToken::new();
        
        // Cancel after a short delay (simulating connection phase)
        tokio::spawn({
            let token = cancel_token.clone();
            async move {
                sleep(Duration::from_millis(100));
                token.cancel();
            }
        });
        
        let messages = vec![json!({"role": "user", "content": "Hello"})];
        let start = Instant::now();
        
        match self.client.chat_completion(messages, Some(cancel_token)).await {
            Err(SmokeTestError::Canceled { phase, elapsed_ms }) => {
                println!("✅ Canceled during {} in {}ms", phase, elapsed_ms);
                Ok(())
            }
            Ok(_) => Err("Expected cancellation, but got success".to_string()),
            Err(e) => Err(format!("Expected cancellation, but got: {}", e)),
        }
    }

    /// Test 3: Deadline exceeded
    pub async fn test_deadline_exceeded(&self) -> Result<(), String> {
        println!("🧪 Testing deadline exceeded...");
        
        let mut config = self.client.config.clone();
        config.deadline_ms = 100; // Very short deadline
        config.base_url = "http://localhost:3000".to_string(); // Assuming Mockoon with timeout endpoint
        
        let short_deadline_client = SmokeTestClient::new(config);
        let messages = vec![json!({"role": "user", "content": "Hello"})];
        
        match short_deadline_client.chat_completion(messages, None).await {
            Err(SmokeTestError::Timeout { phase, elapsed_ms, remaining_budget_ms }) => {
                println!("✅ Timeout in {} (remaining: {}ms) - {}", phase, remaining_budget_ms, elapsed_ms);
                Ok(())
            }
            Ok(_) => Err("Expected timeout, but got success".to_string()),
            Err(e) => Err(format!("Expected timeout, but got: {}", e)),
        }
    }

    /// Test 4: Rate limiting with Retry-After
    pub async fn test_rate_limit_retry_after(&self) -> Result<(), String> {
        println!("🧪 Testing rate limit with Retry-After...");
        
        let mut config = self.client.config.clone();
        config.base_url = "http://localhost:3000".to_string(); // Assuming Mockoon with rate limit endpoint
        
        let rate_limit_client = SmokeTestClient::new(config);
        let messages = vec![json!({"role": "user", "content": "Hello"})];
        
        match rate_limit_client.chat_completion(messages, None).await {
            Err(SmokeTestError::RateLimited { retry_after_secs, elapsed_ms }) => {
                println!("✅ Rate limited with Retry-After: {}s (elapsed: {}ms)", retry_after_secs, elapsed_ms);
                Ok(())
            }
            Ok(_) => Err("Expected rate limit, but got success".to_string()),
            Err(e) => Err(format!("Expected rate limit, but got: {}", e)),
        }
    }

    /// Test 5: Server 5xx error
    pub async fn test_server_5xx(&self) -> Result<(), String> {
        println!("🧪 Testing server 5xx error...");
        
        let mut config = self.client.config.clone();
        config.base_url = "http://localhost:3000".to_string(); // Assuming Mockoon with error endpoint
        
        let error_client = SmokeTestClient::new(config);
        let messages = vec![json!({"role": "user", "content": "Hello"})];
        
        match error_client.chat_completion(messages, None).await {
            Err(SmokeTestError::Server5xx { status, elapsed_ms }) => {
                println!("✅ Server 5xx error: {} (elapsed: {}ms)", status, elapsed_ms);
                Ok(())
            }
            Ok(_) => Err("Expected server error, but got success".to_string()),
            Err(e) => Err(format!("Expected server error, but got: {}", e)),
        }
    }

    /// Test 6: Successful request
    pub async fn test_successful_request(&self) -> Result<(), String> {
        println!("🧪 Testing successful request...");
        
        let messages = vec![json!({"role": "user", "content": "Hello"})];
        
        match self.client.chat_completion(messages, None).await {
            Ok(response) => {
                println!("✅ Successful request: {}", response);
                Ok(())
            }
            Err(e) => Err(format!("Expected success, but got: {}", e)),
        }
    }

    /// Run all smoke tests
    pub async fn run_all_tests(&self) -> Result<(), String> {
        println!("🚀 Running smoke test suite...");
        
        let tests = vec![;
            ("Cancel during DNS", self.test_cancel_during_dns()),
            ("Cancel during connect", self.test_cancel_during_connect()),
            ("Deadline exceeded", self.test_deadline_exceeded()),
            ("Rate limit with Retry-After", self.test_rate_limit_retry_after()),
            ("Server 5xx error", self.test_server_5xx()),
            ("Successful request", self.test_successful_request()),
        ];

        let mut failed_tests = Vec::new();

        for (test_name, test_future) in tests {
            match test_future.await {
                Ok(_) => println!("✅ {}: PASSED", test_name),
                Err(e) => {
                    println!("❌ {}: FAILED - {}", test_name, e);
                    failed_tests.push((test_name, e));
                }
            }
        }

        if failed_tests.is_empty() {
            println!("🎉 All smoke tests passed!");
            Ok(())
        } else {
            let error_msg = failed_tests
                .into_iter()
                .map(|(name, error)| format!("{}: {}", name, error))
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!("Smoke tests failed: {}", error_msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_smoke_test_suite() {
        let config = SmokeTestConfig::default();
        let suite = SmokeTestSuite::new(config);
        
        // Only run tests if Mockoon is available
        if let Ok(response) = reqwest::get("http://localhost:3000/health").await {;
            if response.status().is_success() {
                match suite.run_all_tests().await {
                    Ok(_) => println!("All smoke tests passed"),
                    Err(e) => panic!("Smoke tests failed: {}", e),
                }
            } else {
                println!("⚠️  Mockoon not responding, skipping smoke tests");
            }
        } else {
            println!("⚠️  Mockoon not available, skipping smoke tests");
        }
    }
}

