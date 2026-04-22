//! Ollama local server provider.
//!
//! POST http://localhost:11434/api/chat
//! No authentication required (local only)

use crate::llm::config::ProviderType;
use crate::llm::LlmProvider;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaError {
    error: String,
}

#[derive(Clone)]
pub struct OllamaProvider {
    client: reqwest::Client,
    config: crate::llm::config::ProviderConfig,
}

impl OllamaProvider {
    pub fn new(config: crate::llm::config::ProviderConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    async fn do_generate(&self, system: &str, prompt: &str) -> Result<String> {
        let body = OllamaRequest {
            model: self.config.model.model.clone(),
            messages: vec![
                OllamaMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                },
                OllamaMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            stream: false,
        };

        let resp = self.client
            .post(format!("{}/api/chat", self.config.base_url))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await?;
            return Err(anyhow::anyhow!("Ollama API error ({}): {}", status, err_text));
        }

        let resp_body: OllamaResponse = resp.json().await?;

        if resp_body.message.content.is_empty() {
            return Err(anyhow::anyhow!("Ollama returned empty content"));
        }

        Ok(resp_body.message.content)
    }
}

#[async_trait::async_trait]
impl LlmProvider for OllamaProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Ollama
    }

    async fn generate(&self, system: &str, prompt: &str) -> Result<String> {
        self.do_generate(system, prompt).await
    }
}
