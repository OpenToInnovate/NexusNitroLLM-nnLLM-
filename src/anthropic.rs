//! # Anthropic API Compatibility Layer
//!
//! This module provides support for Anthropic's Claude API format,
//! converting between Anthropic and OpenAI formats internally.

use serde::{Deserialize, Serialize, Deserializer};
use serde::de::{self, Visitor};
use std::fmt;
use crate::schemas::{ChatCompletionRequest, ChatCompletionResponse, Message, Usage};
use crate::error::ProxyError;

/// System prompt that can be either a string or an array of content blocks
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    Text(String),
    Blocks(Vec<AnthropicContentBlock>),
}

impl SystemPrompt {
    /// Convert to a string representation
    pub fn to_string(&self) -> String {
        match self {
            SystemPrompt::Text(text) => text.clone(),
            SystemPrompt::Blocks(blocks) => {
                blocks
                    .iter()
                    .filter_map(|block| match block {
                        AnthropicContentBlock::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
}

impl<'de> Deserialize<'de> for SystemPrompt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SystemPromptVisitor;

        impl<'de> Visitor<'de> for SystemPromptVisitor {
            type Value = SystemPrompt;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or an array of content blocks")
            }

            fn visit_str<E>(self, value: &str) -> Result<SystemPrompt, E>
            where
                E: de::Error,
            {
                Ok(SystemPrompt::Text(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<SystemPrompt, E>
            where
                E: de::Error,
            {
                Ok(SystemPrompt::Text(value))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<SystemPrompt, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut blocks = Vec::new();
                while let Some(block) = seq.next_element()? {
                    blocks.push(block);
                }
                Ok(SystemPrompt::Blocks(blocks))
            }
        }

        deserializer.deserialize_any(SystemPromptVisitor)
    }
}

/// Anthropic Messages API Request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnthropicRequest {
    /// Model identifier (e.g., "claude-3-5-sonnet-20241022")
    pub model: String,
    /// List of messages in the conversation
    pub messages: Vec<AnthropicMessage>,
    /// Maximum tokens to generate (required by Anthropic)
    pub max_tokens: u32,
    /// System prompt (optional, can be string or array of content blocks)
    pub system: Option<SystemPrompt>,
    /// Sampling temperature (0.0 to 1.0)
    pub temperature: Option<f32>,
    /// Nucleus sampling parameter (0.0 to 1.0)
    pub top_p: Option<f32>,
    /// Top-k sampling parameter
    pub top_k: Option<u32>,
    /// Whether to stream the response
    pub stream: Option<bool>,
    /// Stop sequences
    pub stop_sequences: Option<Vec<String>>,
    /// Metadata for the request
    pub metadata: Option<AnthropicMetadata>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: AnthropicContent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AnthropicContent {
    Text(String),
    Array(Vec<AnthropicContentBlock>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum AnthropicContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnthropicMetadata {
    pub user_id: Option<String>,
}

/// Anthropic Messages API Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<AnthropicResponseContent>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: AnthropicUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicResponseContent {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Anthropic Streaming Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<AnthropicStreamMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_block: Option<AnthropicResponseContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<AnthropicDelta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<AnthropicUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicStreamMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub role: String,
    pub content: Vec<AnthropicResponseContent>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: AnthropicUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },
}

impl AnthropicRequest {
    /// Convert Anthropic request to OpenAI format
    pub fn to_openai_request(&self) -> ChatCompletionRequest {
        let mut openai_messages = Vec::new();

        // Add system message if present
        if let Some(system) = &self.system {
            openai_messages.push(Message {
                role: "system".to_string(),
                content: Some(system.to_string()),
                name: None,
                tool_calls: None,
                function_call: None,
                tool_call_id: None,
            });
        }

        // Convert Anthropic messages to OpenAI format
        for msg in &self.messages {
            let content = match &msg.content {
                AnthropicContent::Text(text) => Some(text.clone()),
                AnthropicContent::Array(blocks) => {
                    // For now, concatenate text blocks
                    // TODO: Handle image blocks properly
                    let text = blocks
                        .iter()
                        .filter_map(|block| match block {
                            AnthropicContentBlock::Text { text } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    Some(text)
                }
            };

            openai_messages.push(Message {
                role: msg.role.clone(),
                content,
                name: None,
                tool_calls: None,
                function_call: None,
                tool_call_id: None,
            });
        }

        ChatCompletionRequest {
            messages: openai_messages,
            model: Some(self.model.clone()),
            max_tokens: Some(self.max_tokens),
            temperature: self.temperature,
            top_p: self.top_p,
            stream: self.stream,
            stop: self.stop_sequences.clone(),
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: self.metadata.as_ref().and_then(|m| m.user_id.clone()),
            n: None,
            seed: None,
            logprobs: None,
            top_logprobs: None,
            tools: None,
            tool_choice: None,
        }
    }
}

impl AnthropicResponse {
    /// Convert OpenAI response to Anthropic format
    pub fn from_openai_response(openai_resp: ChatCompletionResponse) -> Result<Self, ProxyError> {
        let choice = openai_resp
            .choices
            .first()
            .ok_or_else(|| ProxyError::Internal("No choices in OpenAI response".to_string()))?;

        let content_text = choice
            .message
            .content
            .clone()
            .unwrap_or_default();

        let content = vec![AnthropicResponseContent::Text {
            text: content_text,
        }];

        let usage = openai_resp.usage.unwrap_or(Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        });

        Ok(AnthropicResponse {
            id: openai_resp.id,
            response_type: "message".to_string(),
            role: "assistant".to_string(),
            content,
            model: openai_resp.model,
            stop_reason: Some(choice.finish_reason.clone()),
            stop_sequence: None,
            usage: AnthropicUsage {
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
            },
        })
    }
}
