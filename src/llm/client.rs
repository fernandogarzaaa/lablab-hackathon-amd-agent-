//! LlmClient — LLM client wrapper for agent communication.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Configuration for an LLM model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 4096,
            temperature: 0.1,
        }
    }
}

/// LLM client that wraps model API calls.
///
/// In production, this would use the Anthropic API via a Rust SDK.
/// For now, provides a simulated client that demonstrates the interface.
pub struct LlmClient {
    config: ModelConfig,
}

impl LlmClient {
    pub fn new(config: ModelConfig) -> Self {
        Self { config }
    }

    /// Generate a response from the LLM.
    pub async fn generate(&self, prompt: &str, _system: &str) -> Result<String> {
        info!("[LLM] Using model: {} (tokens: {}, temp: {})",
            self.config.model, self.config.max_tokens, self.config.temperature);

        // Simulate LLM generation
        // In production: call Anthropic API with prompt + system message
        Ok(format!(
            "[LLM Response (model: {}, tokens: {})]\nBased on the analysis:\n{}\n\n[End Response]",
            self.config.model, self.config.max_tokens, prompt.chars().take(200).collect::<String>()
        ))
    }

    /// Get the current model config.
    pub fn config(&self) -> &ModelConfig {
        &self.config
    }
}
