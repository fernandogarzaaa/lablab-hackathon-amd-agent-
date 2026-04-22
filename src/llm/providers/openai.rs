//! OpenAI chat completions provider.

use crate::llm::client_shared::{retry_with_backoff, HttpClientConfig};
use crate::llm::config::ProviderType;
use crate::llm::providers::types::{OpenAiMessage, OpenAiRequest, OpenAiResponse};
use crate::llm::providers::SharedHttpClient;
use crate::llm::LlmProvider;
use anyhow::Result;

#[derive(Clone)]
pub struct OpenAiProvider {
    shared_client: SharedHttpClient,
    config: crate::llm::config::ProviderConfig,
}

impl OpenAiProvider {
    pub fn new(config: crate::llm::config::ProviderConfig, shared_client: SharedHttpClient) -> Self {
        Self { shared_client, config }
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::OpenAi
    }

    async fn generate(&self, system: &str, prompt: &str) -> Result<String> {
        let shared = self.shared_client.clone();
        let cfg = HttpClientConfig::default();
        let config = self.config.clone();
        retry_with_backoff(&cfg, move || {
            let shared = shared.clone();
            let config = config.clone();
            async move {
            let body = OpenAiRequest {
                model: config.model.model.clone(),
                messages: vec![
                    OpenAiMessage {
                        role: "system".to_string(),
                        content: system.to_string(),
                    },
                    OpenAiMessage {
                        role: "user".to_string(),
                        content: prompt.to_string(),
                    },
                ],
                max_tokens: config.model.max_tokens,
                temperature: config.model.temperature,
            };
            let mut req = shared.inner()
                .post(format!("{}/v1/chat/completions", config.base_url))
                .header("content-type", "application/json")
                .json(&body);
            if let Some(ref key) = config.api_key {
                req = req.bearer_auth(key);
            }
            let resp = req.send().await?;
            let status = resp.status();
            if !status.is_success() {
                let err_text = resp.text().await?;
                return Err(anyhow::anyhow!("OpenAI API error ({}): {}", status, err_text));
            }
            let resp_body: OpenAiResponse = resp.json().await?;
            resp_body
                .choices
                .into_iter()
                .find_map(|c| {
                    if c.message.content.is_empty() { None } else { Some(c.message.content) }
                })
                .ok_or_else(|| anyhow::anyhow!("OpenAI returned empty content"))
            }
        }).await
    }
}
