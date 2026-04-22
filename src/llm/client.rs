//! LlmClient — wraps a provider trait object for agent LLM calls.
//!
//! Providers are selected at runtime via CLI flag or `models.toml`.
//! In demo mode (no valid API key and no local server), falls back to stub responses.

use crate::llm::LlmProvider;
use anyhow::Result;
use tracing::info;

/// Enum of all concrete provider types for cloning support.
#[derive(Clone)]
pub enum AnyProvider {
    Anthropic(crate::llm::providers::AnthropicProvider),
    OpenAi(crate::llm::providers::OpenAiProvider),
    Ollama(crate::llm::providers::OllamaProvider),
    Vllm(crate::llm::providers::VllmProvider),
    LlamaCpp(crate::llm::providers::LlamaCppProvider),
    Demo(DemoProvider),
}

#[async_trait::async_trait]
impl LlmProvider for AnyProvider {
    fn provider_type(&self) -> crate::llm::config::ProviderType {
        match self {
            AnyProvider::Anthropic(p) => p.provider_type(),
            AnyProvider::OpenAi(p) => p.provider_type(),
            AnyProvider::Ollama(p) => p.provider_type(),
            AnyProvider::Vllm(p) => p.provider_type(),
            AnyProvider::LlamaCpp(p) => p.provider_type(),
            AnyProvider::Demo(_) => crate::llm::config::ProviderType::Anthropic,
        }
    }

    async fn generate(&self, system: &str, prompt: &str) -> Result<String> {
        match self {
            AnyProvider::Anthropic(p) => p.generate(system, prompt).await,
            AnyProvider::OpenAi(p) => p.generate(system, prompt).await,
            AnyProvider::Ollama(p) => p.generate(system, prompt).await,
            AnyProvider::Vllm(p) => p.generate(system, prompt).await,
            AnyProvider::LlamaCpp(p) => p.generate(system, prompt).await,
            AnyProvider::Demo(p) => p.generate(system, prompt).await,
        }
    }
}

pub struct LlmClient {
    provider: AnyProvider,
    config: crate::llm::config::ProviderConfig,
}

impl Clone for LlmClient {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            config: self.config.clone(),
        }
    }
}

impl LlmClient {
    pub fn new(provider: AnyProvider, config: crate::llm::config::ProviderConfig) -> Self {
        Self { provider, config }
    }

    /// Generate a response from the LLM, routing through the configured provider.
    pub async fn generate(&self, _system: &str, prompt: &str) -> Result<String> {
        info!("[LLM] Provider: {} | Model: {} (tokens: {}, temp: {})",
            self.provider.provider_type(),
            self.config.model.model,
            self.config.model.max_tokens,
            self.config.model.temperature);

        self.provider.generate(_system, prompt).await
    }

    /// Create a demo (stub) LlmClient that returns simulated responses without hitting any API.
    pub fn new_demo() -> Self {
        let config = crate::llm::config::ProviderConfig::new(
            crate::llm::config::ProviderType::Anthropic,
            crate::llm::config::ModelConfig::default(),
        );
        Self {
            provider: AnyProvider::Demo(DemoProvider),
            config,
        }
    }

    /// Check whether this client is in demo mode.
    pub fn is_demo(&self) -> bool {
        matches!(self.provider, AnyProvider::Demo(_))
    }

    /// Get the current model config.
    pub fn config(&self) -> &crate::llm::config::ProviderConfig {
        &self.config
    }
}

/// Demo provider — returns simulated responses.
#[derive(Clone)]
pub struct DemoProvider;

#[async_trait::async_trait]
impl LlmProvider for DemoProvider {
    fn provider_type(&self) -> crate::llm::config::ProviderType {
        crate::llm::config::ProviderType::Anthropic
    }

    async fn generate(&self, _system: &str, prompt: &str) -> Result<String> {
        Ok(format!(
            "[LLM Demo Response]\nBased on the analysis:\n{}\n\n[End Response]",
            prompt.chars().take(200).collect::<String>()
        ))
    }
}
