//! Anthropic Claude API provider.
//!
//! POST https://api.anthropic.com/v1/messages
//! Header: x-api-key: {api_key}
//! Header: anthropic-version: 2023-06-01

use crate::llm::client_shared::{retry_with_backoff, HttpClientConfig};
use crate::llm::config::ProviderType;
use crate::llm::providers::SharedHttpClient;
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

#[derive(Clone)]
pub struct AnthropicProvider {
    shared_client: SharedHttpClient,
    config: crate::llm::config::ProviderConfig,
}

impl AnthropicProvider {
    pub fn new(config: crate::llm::config::ProviderConfig, shared_client: SharedHttpClient) -> Self {
        Self { shared_client, config }
    }
}

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Anthropic
    }

    async fn generate(&self, system: &str, prompt: &str) -> Result<String> {
        let shared = self.shared_client.clone();
        let cfg = HttpClientConfig::default();
        let config = self.config.clone();
        retry_with_backoff(&cfg, move || {
            let shared = shared.clone();
            let config = config.clone();
            async move {
            let client = shared.inner();
            let messages = vec![Message {
                role: "user",
                content: prompt,
            }];
            let body = AnthropicRequest {
                model: &config.model.model,
                messages,
                system,
                max_tokens: config.model.max_tokens,
                temperature: config.model.temperature,
            };
            let mut req = client
                .post(format!("{}/v1/messages", config.base_url));
            if let Some(ref key) = config.api_key {
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
        }).await
    }
}
