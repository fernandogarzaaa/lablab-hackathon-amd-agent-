//! Shared types for OpenAI-compatible API (chat completions).
//!
//! Used by both OpenAiProvider and OpenAiCompatibleProvider.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    pub max_tokens: u32,
    pub temperature: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiResponse {
    pub choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiChoice {
    pub message: OpenAiChoiceMessage,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiChoiceMessage {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiResponseError {
    pub error: OpenAiErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiErrorDetail {
    pub message: String,
}
