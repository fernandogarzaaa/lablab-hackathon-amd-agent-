//! Generic OpenAI-compatible chat completions provider.
//!
//! Works with any server that implements the OpenAI chat completions API:
//! vLLM, llama.cpp server, Ollama (in OpenAI mode), local proxies, etc.
//!
//! POST {base_url}/v1/chat/completions
//! Header: Authorization: Bearer {api_key}

use crate::llm::config::ProviderType;
use crate::llm::providers::types::{OpenAiRequest, OpenAiResponse};
use crate::llm::LlmProvider;
use anyhow::Result;

#[derive(Clone)]
pub struct OpenAiCompatibleProvider {
    client: reqwest::Client,
    config: crate::llm::config::ProviderConfig,
}

impl OpenAiCompatibleProvider {
    pub fn new(config: crate::llm::config::ProviderConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    async fn do_generate(&self, system: &str, prompt: &str) -> Result<String> {
        let body = OpenAiRequest {
            model: self.config.model.model.clone(),
            messages: vec![
                crate::llm::providers::types::OpenAiMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                },
                crate::llm::providers::types::OpenAiMessage {
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
            return Err(anyhow::anyhow!("OpenAI-compatible API error ({}): {}", status, err_text));
        }

        let resp_body: OpenAiResponse = resp.json().await?;
        resp_body
            .choices
            .into_iter()
            .find_map(|c| {
                if c.message.content.is_empty() { None } else { Some(c.message.content) }
            })
            .ok_or_else(|| anyhow::anyhow!("OpenAI-compatible API returned empty content"))
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::OpenAiCompatible
    }

    async fn generate(&self, system: &str, prompt: &str) -> Result<String> {
        self.do_generate(system, prompt).await
    }
}
