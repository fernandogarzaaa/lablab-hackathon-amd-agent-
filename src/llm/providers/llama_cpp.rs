//! llama.cpp server provider (OpenAI-compatible endpoint).
//!
//! POST http://localhost:8080/v1/chat/completions

use crate::llm::config::ProviderType;
use crate::llm::LlmProvider;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
struct LlamaCppRequest {
    model: String,
    messages: Vec<LlamaCppMessage>,
    max_tokens: u32,
    temperature: f64,
}

#[derive(Debug, Clone, Serialize)]
struct LlamaCppMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct LlamaCppResponse {
    choices: Vec<LlamaCppChoice>,
}

#[derive(Debug, Deserialize)]
struct LlamaCppChoice {
    message: LlamaCppChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct LlamaCppChoiceMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct LlamaCppError {
    error: LlamaCppErrorDetail,
}

#[derive(Debug, Deserialize)]
struct LlamaCppErrorDetail {
    message: String,
}

#[derive(Clone)]
pub struct LlamaCppProvider {
    client: reqwest::Client,
    config: crate::llm::config::ProviderConfig,
}

impl LlamaCppProvider {
    pub fn new(config: crate::llm::config::ProviderConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    async fn do_generate(&self, system: &str, prompt: &str) -> Result<String> {
        let body = LlamaCppRequest {
            model: self.config.model.model.clone(),
            messages: vec![
                LlamaCppMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                },
                LlamaCppMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            max_tokens: self.config.model.max_tokens,
            temperature: self.config.model.temperature,
        };

        let mut req = self.client
            .post(format!("{}/v1/chat/completions", self.config.base_url))
            .header("content-type", "application/json")
            .json(&body);

        if let Some(ref key) = self.config.api_key {
            req = req.bearer_auth(key);
        }

        let resp = req.send().await?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await?;
            return Err(anyhow::anyhow!("llama.cpp API error ({}): {}", status, err_text));
        }

        let resp_body: LlamaCppResponse = resp.json().await?;

        resp_body
            .choices
            .into_iter()
            .find_map(|c| if c.message.content.is_empty() { None } else { Some(c.message.content) })
            .ok_or_else(|| anyhow::anyhow!("llama.cpp returned empty content"))
    }
}

#[async_trait::async_trait]
impl LlmProvider for LlamaCppProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::LlamaCpp
    }

    async fn generate(&self, system: &str, prompt: &str) -> Result<String> {
        self.do_generate(system, prompt).await
    }
}
