//! Agent trait and base types.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

pub struct AgentContext {
    pub iteration: u32,
    pub abilities: Vec<String>,
    pub repo_path: Option<String>,
    pub llm_client: Option<crate::llm::LlmClient>,
}

impl AgentContext {
    pub fn new(iteration: u32) -> Self {
        Self {
            iteration,
            abilities: Vec::new(),
            repo_path: None,
            llm_client: None,
        }
    }

    pub fn with_repo_path(mut self, path: String) -> Self {
        self.repo_path = Some(path);
        self
    }

    pub fn with_llm_client(mut self, client: crate::llm::LlmClient) -> Self {
        self.llm_client = Some(client);
        self
    }
}

#[async_trait]
pub trait Agent: Send + Sync {
    async fn run(&self, input: Value, ctx: &AgentContext) -> Result<Value>;
    fn name(&self) -> &str;
    fn prompt_template(&self) -> &'static str;
    fn mcp_tools(&self) -> Vec<String>;
    fn min_confidence(&self) -> f64;
}
