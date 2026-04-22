use anyhow::Result;

/// Trait for LLM providers. Each provider implements the API call pattern.
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// The provider type (for logging/debugging).
    fn provider_type(&self) -> crate::llm::config::ProviderType;

    /// Generate a completion from the model.
    /// `system` is the system prompt, `prompt` is the user prompt.
    async fn generate(&self, system: &str, prompt: &str) -> Result<String>;
}
