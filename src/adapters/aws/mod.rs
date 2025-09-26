//! # AWS Bedrock Adapter Module
//!
//! This module provides the AWS Bedrock adapter implementation
//! with AWS-specific authentication and API format handling.

use crate::{
    adapters::base::{AdapterTrait, AdapterUtils},
    error::ProxyError,
    schemas::{ChatCompletionRequest, ChatCompletionResponse},
};
#[cfg(feature = "adapter-aws")]
use crate::schemas::{Message, Choice, Usage};
#[cfg(feature = "server")]
use axum::response::Response;
use reqwest::Client;
use serde_json::Value;
#[cfg(feature = "adapter-aws")]
use serde_json::json;
#[cfg(feature = "adapter-aws")]
use chrono::Utc;
#[cfg(feature = "adapter-aws")]
use sha2::{Sha256, Digest};
#[cfg(feature = "adapter-aws")]
use hmac::{Hmac, Mac};
#[cfg(feature = "adapter-aws")]
type HmacSha256 = Hmac<Sha256>;

/// # AWS Bedrock Adapter
///
/// Adapter for Amazon Web Services Bedrock with AWS-specific
/// authentication and API format conversion.
#[derive(Clone, Debug)]
pub struct AWSBedrockAdapter {
    /// Base URL for AWS Bedrock
    base: String,
    /// Model identifier
    model_id: String,
    /// AWS access key ID
    access_key_id: Option<String>,
    /// AWS secret access key
    secret_access_key: Option<String>,
    /// AWS region
    #[allow(dead_code)]
    region: String,
    /// HTTP client with connection pooling
    #[allow(dead_code)]
    client: Client,
}

impl AWSBedrockAdapter {
    /// Create a new AWS Bedrock adapter instance
    pub fn new(base: String, model_id: String, access_key: Option<String>, client: Client) -> Self {
        // Parse access_key as "access_key_id:secret_access_key" format
        let (access_key_id, secret_access_key) = if let Some(key) = access_key {
            if let Some((access, secret)) = key.split_once(':') {
                (Some(access.to_string()), Some(secret.to_string()))
            } else {
                (Some(key), None)
            }
        } else {
            (None, None)
        };

        // Extract region from base URL or default to us-east-1
        let region = if base.contains("us-west-2") {
            "us-west-2".to_string()
        } else if base.contains("eu-west-1") {
            "eu-west-1".to_string()
        } else {
            "us-east-1".to_string()
        };

        Self {
            base,
            model_id,
            access_key_id,
            secret_access_key,
            region,
            client,
        }
    }

    /// Convert OpenAI chat completion format to AWS Bedrock format
    #[cfg(feature = "adapter-aws")]
    fn convert_to_bedrock_format(&self, req: &ChatCompletionRequest) -> Result<Value, ProxyError> {
        // Extract the conversation from OpenAI messages
        let mut prompt = String::new();

        for message in &req.messages {
            match message.role.as_str() {
                "system" => {
                    if let Some(content) = &message.content {
                        prompt.push_str(&format!("System: {}\n", content));
                    }
                }
                "user" => {
                    if let Some(content) = &message.content {
                        prompt.push_str(&format!("Human: {}\n", content));
                    }
                }
                "assistant" => {
                    if let Some(content) = &message.content {
                        prompt.push_str(&format!("Assistant: {}\n", content));
                    }
                }
                _ => {} // Skip unknown roles
            }
        }

        // Add assistant prompt to get the model to respond
        prompt.push_str("Assistant:");

        // Create Bedrock request format (Claude-specific)
        let bedrock_request = json!({
            "prompt": prompt,
            "max_tokens_to_sample": req.max_tokens.unwrap_or(1000),
            "temperature": req.temperature.unwrap_or(0.7),
            "top_p": req.top_p.unwrap_or(1.0),
            "stop_sequences": ["\nHuman:"],
        });

        Ok(bedrock_request)
    }

    /// Convert AWS Bedrock response format to OpenAI format
    #[cfg(feature = "adapter-aws")]
    fn convert_from_bedrock_format(&self, aws_response: Value, original_req: &ChatCompletionRequest) -> Result<ChatCompletionResponse, ProxyError> {
        // Extract completion from AWS response
        let completion = aws_response.get("completion")
            .and_then(|c| c.as_str())
            .unwrap_or("");

        // Get token usage from AWS response
        let prompt_tokens = aws_response.get("prompt_tokens")
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as i32;
        let completion_tokens = aws_response.get("completion_tokens")
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as i32;

        // Create OpenAI format response
        let response = ChatCompletionResponse {
            id: format!("chatcmpl-aws-{}", chrono::Utc::now().timestamp()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: AdapterUtils::extract_model(original_req, &self.model_id),
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: "assistant".to_string(),
                    content: Some(completion.trim().to_string()),
                    name: None,
                    function_call: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: "stop".to_string(),
                logprobs: None,
            }],
            usage: Some(Usage {
                prompt_tokens: prompt_tokens.max(0) as u32,
                completion_tokens: completion_tokens.max(0) as u32,
                total_tokens: (prompt_tokens + completion_tokens).max(0) as u32,
            }),
        };

        Ok(response)
    }

    /// Create AWS Signature V4 headers for authentication
    #[cfg(feature = "adapter-aws")]
    async fn create_aws_headers(&self, payload: &Value, _endpoint: &str) -> Result<reqwest::header::HeaderMap, ProxyError> {
        let mut headers = reqwest::header::HeaderMap::new();

        let access_key_id = self.access_key_id.as_ref()
            .ok_or_else(|| ProxyError::Internal("AWS access key ID not set".to_string()))?;
        let secret_access_key = self.secret_access_key.as_ref()
            .ok_or_else(|| ProxyError::Internal("AWS secret access key not set".to_string()))?;

        // Basic headers
        headers.insert("content-type", "application/json".parse().unwrap());
        headers.insert("host", format!("bedrock-runtime.{}.amazonaws.com", self.region).parse().unwrap());

        // AWS date format
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();

        headers.insert("x-amz-date", amz_date.parse().unwrap());

        // Payload hash
        let payload_str = serde_json::to_string(payload)
            .map_err(|e| ProxyError::Internal(format!("Failed to serialize payload: {}", e)))?;
        let payload_hash = format!("{:x}", Sha256::digest(payload_str.as_bytes()));

        // Create canonical request
        let canonical_uri = "/";
        let canonical_querystring = "";
        let canonical_headers = format!(
            "content-type:application/json\nhost:bedrock-runtime.{}.amazonaws.com\nx-amz-date:{}\n",
            self.region, amz_date
        );
        let signed_headers = "content-type;host;x-amz-date";

        let canonical_request = format!(
            "POST\n{}\n{}\n{}\n{}\n{}",
            canonical_uri, canonical_querystring, canonical_headers, signed_headers, payload_hash
        );

        // Create string to sign
        let algorithm = "AWS4-HMAC-SHA256";
        let credential_scope = format!("{}/{}/bedrock/aws4_request", date_stamp, self.region);
        let string_to_sign = format!(
            "{}\n{}\n{}\n{:x}",
            algorithm, amz_date, credential_scope, Sha256::digest(canonical_request.as_bytes())
        );

        // Calculate signature
        let signature = self.calculate_signature(secret_access_key, &date_stamp, &string_to_sign)?;

        // Create authorization header
        let authorization = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm, access_key_id, credential_scope, signed_headers, signature
        );

        headers.insert("authorization", authorization.parse().unwrap());

        Ok(headers)
    }

    /// Calculate AWS Signature V4 signature
    #[cfg(feature = "adapter-aws")]
    fn calculate_signature(&self, secret_key: &str, date_stamp: &str, string_to_sign: &str) -> Result<String, ProxyError> {
        let key = format!("AWS4{}", secret_key);

        let mut mac = HmacSha256::new_from_slice(key.as_bytes())
            .map_err(|e| ProxyError::Internal(format!("HMAC key error: {}", e)))?;
        mac.update(date_stamp.as_bytes());
        let k_date = mac.finalize().into_bytes();

        let mut mac = HmacSha256::new_from_slice(&k_date)
            .map_err(|e| ProxyError::Internal(format!("HMAC key error: {}", e)))?;
        mac.update(self.region.as_bytes());
        let k_region = mac.finalize().into_bytes();

        let mut mac = HmacSha256::new_from_slice(&k_region)
            .map_err(|e| ProxyError::Internal(format!("HMAC key error: {}", e)))?;
        mac.update(b"bedrock");
        let k_service = mac.finalize().into_bytes();

        let mut mac = HmacSha256::new_from_slice(&k_service)
            .map_err(|e| ProxyError::Internal(format!("HMAC key error: {}", e)))?;
        mac.update(b"aws4_request");
        let k_signing = mac.finalize().into_bytes();

        let mut mac = HmacSha256::new_from_slice(&k_signing)
            .map_err(|e| ProxyError::Internal(format!("HMAC key error: {}", e)))?;
        mac.update(string_to_sign.as_bytes());
        let signature = mac.finalize().into_bytes();

        let sig_hex = signature.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        Ok(sig_hex)
    }

    /// Fallback implementations when AWS feature is not enabled
    #[cfg(not(feature = "adapter-aws"))]
    #[allow(dead_code)]
    fn convert_to_bedrock_format(&self, _req: &ChatCompletionRequest) -> Result<Value, ProxyError> {
        Err(ProxyError::BadRequest("AWS Bedrock adapter requires 'adapter-aws' feature".to_string()))
    }

    #[cfg(not(feature = "adapter-aws"))]
    #[allow(dead_code)]
    fn convert_from_bedrock_format(&self, _aws_response: Value, _original_req: &ChatCompletionRequest) -> Result<ChatCompletionResponse, ProxyError> {
        Err(ProxyError::BadRequest("AWS Bedrock adapter requires 'adapter-aws' feature".to_string()))
    }

    #[cfg(not(feature = "adapter-aws"))]
    #[allow(dead_code)]
    async fn create_aws_headers(&self, _payload: &Value, _endpoint: &str) -> Result<reqwest::header::HeaderMap, ProxyError> {
        Err(ProxyError::BadRequest("AWS Bedrock adapter requires 'adapter-aws' feature".to_string()))
    }

    #[cfg(not(feature = "adapter-aws"))]
    #[allow(dead_code)]
    fn calculate_signature(&self, _secret_key: &str, _date_stamp: &str, _string_to_sign: &str) -> Result<String, ProxyError> {
        Err(ProxyError::BadRequest("AWS Bedrock adapter requires 'adapter-aws' feature".to_string()))
    }

    /// Process chat completion requests with AWS Bedrock-specific handling
    #[cfg(feature = "server")]
    pub async fn chat_completions_http(&self, req: ChatCompletionRequest) -> Result<Response, ProxyError> {
        AdapterUtils::log_request("aws", &AdapterUtils::extract_model(&req, &self.model_id), req.messages.len());

        #[cfg(feature = "adapter-aws")]
        let start_time = std::time::Instant::now();

        #[cfg(not(feature = "adapter-aws"))]
        {
            return Err(ProxyError::BadRequest(
                "AWS Bedrock adapter requires 'adapter-aws' feature to be enabled".to_string()
            ));
        }

        #[cfg(feature = "adapter-aws")]
        {
            // Check if we have proper AWS credentials
            if !self.has_auth() {
                return Err(ProxyError::BadRequest(
                    "AWS credentials (access_key_id:secret_access_key) required".to_string()
                ));
            }
        }

        #[cfg(feature = "adapter-aws")]
        {
        // Convert OpenAI format to AWS Bedrock format
        let bedrock_request = self.convert_to_bedrock_format(&req)?;

        // Build AWS Bedrock endpoint URL
        let model = AdapterUtils::extract_model(&req, &self.model_id);
        let endpoint = format!(
            "https://bedrock-runtime.{}.amazonaws.com/model/{}/invoke",
            self.region, model
        );

        // Create AWS Signature V4 headers
        let headers = self.create_aws_headers(&bedrock_request, &endpoint).await?;

        // Make the request to AWS Bedrock
        let response = self.client
            .post(&endpoint)
            .headers(headers)
            .json(&bedrock_request)
            .send()
            .await
            .map_err(|e| ProxyError::Upstream(format!("AWS Bedrock request failed: {}", e)))?;

        let response_time = start_time.elapsed().as_millis() as u64;
        let success = response.status().is_success();
        AdapterUtils::log_response("aws", &model, success, response_time);

        if !success {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProxyError::Upstream(format!(
                "AWS Bedrock error {}: {}", status, error_text
            )));
        }

        // Parse AWS response and convert to OpenAI format
        let aws_response: Value = response.json().await
            .map_err(|e| ProxyError::Internal(format!("Failed to parse AWS response: {}", e)))?;

        let openai_response = self.convert_from_bedrock_format(aws_response, &req)?;

        // Convert to HTTP response
        let json_response = serde_json::to_string(&openai_response)
            .map_err(|e| ProxyError::Internal(format!("Failed to serialize response: {}", e)))?;

        Ok(Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(axum::body::Body::from(json_response))
            .map_err(|e| ProxyError::Internal(format!("Failed to build response: {}", e)))?)
        }
    }
}

#[async_trait::async_trait]
impl AdapterTrait for AWSBedrockAdapter {
    fn name(&self) -> &'static str {
        "aws"
    }

    fn base_url(&self) -> &str {
        &self.base
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn has_auth(&self) -> bool {
        self.access_key_id.is_some() && self.secret_access_key.is_some()
    }

    #[cfg(feature = "server")]
    async fn chat_completions(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse, ProxyError> {
        let http_response = self.chat_completions_http(request).await?;

        // Extract the response body
        let body_bytes = axum::body::to_bytes(http_response.into_body(), usize::MAX)
            .await
            .map_err(|e| ProxyError::Internal(format!("Failed to read response body: {}", e)))?;

        // Parse the JSON response into ChatCompletionResponse
        let response: ChatCompletionResponse = serde_json::from_slice(&body_bytes)
            .map_err(|e| ProxyError::Internal(format!("Failed to parse response JSON: {}", e)))?;

        Ok(response)
        
    }

    #[cfg(not(feature = "server"))]
    async fn chat_completions(&self, _request: ChatCompletionRequest) -> Result<ChatCompletionResponse, ProxyError> {
        Err(ProxyError::Internal("Server feature not enabled".to_string()))
    }
}