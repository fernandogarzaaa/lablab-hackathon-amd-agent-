//! ModelRouter — routes requests to appropriate models per agent.

use crate::llm::client::LlmClient;
use crate::llm::client::ModelConfig;
use serde::{Deserialize, Serialize};

/// Routes agent requests to appropriate LLM models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRouter {
    default: ModelConfig,
    per_agent: std::collections::HashMap<String, ModelConfig>,
}

impl ModelRouter {
    pub fn new(default: ModelConfig) -> Self {
        Self {
            default,
            per_agent: std::collections::HashMap::new(),
        }
    }

    pub fn add_agent_model(&mut self, agent: &str, config: ModelConfig) {
        self.per_agent.insert(agent.to_string(), config);
    }

    pub fn get_config(&self, agent: &str) -> ModelConfig {
        self.per_agent.get(agent).cloned().unwrap_or_else(|| self.default.clone())
    }

    pub fn create_client(&self, agent: &str) -> LlmClient {
        LlmClient::new(self.get_config(agent))
    }
}
