//! Integration tests for Chimera Builder.

use chimera_builder::agents::{AnalystAgent, PlannerAgent, BuilderAgent, CriticAgent, Agent, base::AgentContext};
use chimera_builder::core::types::Verdict;
use chimera_builder::memory::MemoryStore;
use chimera_builder::memory::compression::{extract_abilities, AbilityCategory};

#[tokio::test]
async fn test_analyst_agent() {
    let analyst = AnalystAgent::new();
    assert_eq!(analyst.name(), "analyst");

    let ctx = AgentContext::new(0);
    let result = analyst.run(serde_json::json!({}), &ctx).await;
    assert!(result.is_ok());

    let output: serde_json::Value = result.unwrap();
    assert!(output["audit_report"]["tech_stack"].is_array());
    assert!(output["audit_report"]["confidence"].as_f64().unwrap_or(0.0) > 0.0);
}

#[tokio::test]
async fn test_planner_agent() {
    let planner = PlannerAgent::new();
    assert_eq!(planner.name(), "planner");

    let ctx = AgentContext::new(0);
    let input = serde_json::json!({
        "issues": [],
        "tech_stack": ["Rust", "tokio"]
    });
    let result = planner.run(input, &ctx).await;
    assert!(result.is_ok());

    let output: serde_json::Value = result.unwrap();
    assert_eq!(output["plan"]["confidence"], 0.88);
}

#[tokio::test]
async fn test_builder_agent() {
    let builder = BuilderAgent::new();
    assert_eq!(builder.name(), "builder");

    let ctx = AgentContext::new(0);
    let input = serde_json::json!({
        "roadmap": [
            {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "title": "Test task",
                "description": "A test task",
                "priority": "P1",
                "effort": "Medium",
                "category": "Test",
                "depends_on": [],
                "file_paths": ["src/test.rs"]
            }
        ]
    });
    let result = builder.run(input, &ctx).await;
    assert!(result.is_ok());

    let output: serde_json::Value = result.unwrap();
    assert!(output["build_output"]["changes"].is_array());
}

#[tokio::test]
async fn test_critic_agent() {
    let critic = CriticAgent::new();
    assert_eq!(critic.name(), "critic");

    let ctx = AgentContext::new(0);
    let input = serde_json::json!({
        "plan": { "confidence": 0.90 },
        "build": { "confidence": 0.87 },
        "test": { "confidence": 0.84 }
    });
    let result = critic.run(input, &ctx).await;
    assert!(result.is_ok());

    let output: serde_json::Value = result.unwrap();
    assert_eq!(output["verdict"], "APPROVE");
}

#[tokio::test]
async fn test_memory_store() {
    let path = "/tmp/test-chimera-mem.db";
    let mut store = MemoryStore::new(path).expect("Failed to create memory store");

    let session_id = store.create_session("https://github.com/test/repo").await;
    assert!(session_id.is_ok());

    let id = session_id.unwrap();
    assert!(!id.is_empty());

    // Cleanup
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn test_ability_extraction() {
    let input = "tech_stack detected: Rust, tokio, serde. issues: no tests found.";
    let abilities = extract_abilities(input, AbilityCategory::RepoAnalysis);
    assert!(!abilities.is_empty());

    // The input contains "tech_stack" and "issues" which trigger RepoAnalysis abilities
    // It also contains "Rust" which triggers CodeGeneration
    let has_repo = abilities.iter().any(|a| a.category == AbilityCategory::RepoAnalysis);
    assert!(has_repo, "Expected at least one RepoAnalysis ability");
    assert!(!abilities.is_empty());
}
