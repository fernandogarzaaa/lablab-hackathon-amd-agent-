//! Anthropic Claude API provider.
//!
//! POST https://api.anthropic.com/v1/messages
//! Header: x-api-key: {api_key}
//! Header: anthropic-version: 2023-06-01

use crate::llm::config::ProviderType;
use crate::llm::LlmProvider;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    system: &'a str,
    max_tokens: u32,
    temperature: f64,
}

#[derive(Debug, Clone, Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicError {
    error: AnthropicErrorDetail,
}

#[derive(Debug, Deserialize)]
struct AnthropicErrorDetail {
    message: String,
}

#[derive(Clone)]
pub struct AnthropicProvider {
    client: reqwest::Client,
    config: crate::llm::config::ProviderConfig,
}

impl AnthropicProvider {
    pub fn new(config: crate::llm::config::ProviderConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    async fn do_generate(&self, system: &str, prompt: &str) -> Result<String> {
        let messages = vec![Message {
            role: "user",
            content: prompt,
        }];

        let body = AnthropicRequest {
            model: &self.config.model.model,
            messages,
            system,
            max_tokens: self.config.model.max_tokens,
            temperature: self.config.model.temperature,
        };

        let mut req = self.client
            .post(format!("{}/v1/messages", self.config.base_url));

        if let Some(ref key) = self.config.api_key {
            req = req.bearer_auth(key);
        }

        let resp = req
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await?;
            return Err(anyhow::anyhow!("Anthropic API error ({}): {}", status, err_text));
        }

        let resp_body: AnthropicResponse = resp.json().await?;

        resp_body
            .content
            .into_iter()
            .find_map(|c| if c.text.is_empty() { None } else { Some(c.text) })
            .ok_or_else(|| anyhow::anyhow!("Anthropic returned empty content"))
    }
}

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Anthropic
    }

    async fn generate(&self, system: &str, prompt: &str) -> Result<String> {
        self.do_generate(system, prompt).await
    }
}
