//! Ollama local server provider.
//!
//! POST http://localhost:11434/api/chat
//! No authentication required (local only)

use crate::llm::client_shared::{retry_with_backoff, HttpClientConfig};
use crate::llm::config::ProviderType;
use crate::llm::providers::SharedHttpClient;
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

#[derive(Clone)]
pub struct OllamaProvider {
    shared_client: SharedHttpClient,
    config: crate::llm::config::ProviderConfig,
}

impl OllamaProvider {
    pub fn new(config: crate::llm::config::ProviderConfig, shared_client: SharedHttpClient) -> Self {
        Self { shared_client, config }
    }
}

#[async_trait::async_trait]
impl LlmProvider for OllamaProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Ollama
    }

    async fn generate(&self, system: &str, prompt: &str) -> Result<String> {
        let shared = self.shared_client.clone();
        let cfg = HttpClientConfig::default();
        let config = self.config.clone();
        retry_with_backoff(&cfg, move || {
            let shared = shared.clone();
            let config = config.clone();
            async move {
            let body = OllamaRequest {
                model: config.model.model.clone(),
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
            let resp = shared.inner()
                .post(format!("{}/api/chat", config.base_url))
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
        }).await
    }
}
