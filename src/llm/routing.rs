//! ModelRouter — routes requests to appropriate models per agent.

use crate::llm::client::AnyProvider;
use crate::llm::config::{RoutingConfig, ProviderType};
use serde::{Deserialize, Serialize};

/// Routes agent requests to appropriate LLM models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRouter {
    default_provider: ProviderType,
    routing: RoutingConfig,
}

impl ModelRouter {
    pub fn new(default_provider: ProviderType, routing: RoutingConfig) -> Self {
        Self {
            default_provider,
            routing,
        }
    }

    /// Create a LlmClient for a specific agent.
    pub fn create_client(&self, agent: &str) -> Option<crate::llm::LlmClient> {
        let config = self.routing.create_provider_config(agent);
        let provider = match &config.provider {
            ProviderType::Anthropic => AnyProvider::Anthropic(
                crate::llm::providers::AnthropicProvider::new(config.clone())
            ),
            ProviderType::OpenAi => AnyProvider::OpenAi(
                crate::llm::providers::OpenAiProvider::new(config.clone())
            ),
            ProviderType::Ollama => AnyProvider::Ollama(
                crate::llm::providers::OllamaProvider::new(config.clone())
            ),
            ProviderType::OpenAiCompatible => AnyProvider::OpenAiCompatible(
                crate::llm::providers::OpenAiCompatibleProvider::new(config.clone())
            ),
        };
        Some(crate::llm::LlmClient::new(provider, config))
    }
}
