//! Agent trait and base types.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

pub struct AgentContext {
    pub iteration: u32,
    pub abilities: Vec<String>,
}

impl AgentContext {
    pub fn new(iteration: u32) -> Self {
        Self {
            iteration,
            abilities: Vec::new(),
        }
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
