/// Configuration for a specific LLM model instance.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/// Supported LLM backends.
#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Anthropic,
    OpenAi,
    Ollama,
    Vllm,
    LlamaCpp,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::Anthropic => write!(f, "anthropic"),
            ProviderType::OpenAi => write!(f, "openai"),
            ProviderType::Ollama => write!(f, "ollama"),
            ProviderType::Vllm => write!(f, "vllm"),
            ProviderType::LlamaCpp => write!(f, "llama-cpp"),
        }
    }
}

/// Provider-level configuration (base URL, API key, etc.).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderConfig {
    pub provider: ProviderType,
    pub model: ModelConfig,
    pub base_url: String,
    pub api_key: Option<String>,
}

impl ProviderConfig {
    pub fn new(provider: ProviderType, config: ModelConfig) -> Self {
        let (base_url, api_key) = match provider {
            ProviderType::Anthropic => (
                "https://api.anthropic.com".to_string(),
                std::env::var("ANTHROPIC_API_KEY").ok(),
            ),
            ProviderType::OpenAi => (
                "https://api.openai.com".to_string(),
                std::env::var("OPENAI_API_KEY").ok(),
            ),
            ProviderType::Ollama => (
                "http://localhost:11434".to_string(),
                None,
            ),
            ProviderType::Vllm => (
                std::env::var("VLLM_API_URL")
                    .ok()
                    .unwrap_or_else(|| "http://localhost:8000".to_string()),
                std::env::var("VLLM_API_KEY").ok(),
            ),
            ProviderType::LlamaCpp => (
                std::env::var("LLAMA_CPP_API_URL")
                    .ok()
                    .unwrap_or_else(|| "http://localhost:8080".to_string()),
                None,
            ),
        };
        Self {
            provider,
            model: config,
            base_url,
            api_key,
        }
    }
}

/// Per-agent model routing config loaded from models.toml.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RoutingConfig {
    #[serde(flatten)]
    pub agents: std::collections::HashMap<String, ModelConfig>,
    #[serde(skip, default = "default_provider")]
    pub provider: ProviderType,
}

fn default_provider() -> ProviderType {
    ProviderType::Anthropic
}

impl RoutingConfig {
    pub fn with_provider(mut self, provider: ProviderType) -> Self {
        self.provider = provider;
        self
    }

    /// Get config for a specific agent, falling back to "default".
    pub fn get(&self, agent: &str) -> ModelConfig {
        self.agents.get(agent).cloned().unwrap_or_else(|| {
            self.agents.get("default").cloned().unwrap_or_default()
        })
    }

    pub fn create_provider_config(&self, agent: &str) -> ProviderConfig {
        ProviderConfig::new(self.provider, self.get(agent))
    }
}

/// Parse a models.toml file into a RoutingConfig.
pub fn load_models(path: &str, provider: ProviderType) -> std::result::Result<RoutingConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let parsed: std::collections::HashMap<String, toml::Value> = toml::from_str(&content)?;

    let mut agents = std::collections::HashMap::new();

    for (key, value) in &parsed {
        if let Some(model_value) = value.get("model") {
            let model = model_value.as_str().unwrap_or_default().to_string();
            let max_tokens = value.get("max_tokens")
                .and_then(|v| v.as_integer())
                .unwrap_or(4096) as u32;
            let temperature = value.get("temperature")
                .and_then(|v| v.as_float())
                .unwrap_or(0.1);
            agents.insert(key.clone(), ModelConfig {
                model,
                max_tokens,
                temperature,
            });
        }
    }

    Ok(RoutingConfig { agents, provider })
}
