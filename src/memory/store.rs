//! MemoryStore — SQLite persistence for sessions, tasks, and decisions.

use crate::memory::compression::{Ability, AbilityCategory};
use crate::memory::search::FtsSearch;
use crate::memory::semantic::SemanticStore;
use anyhow::Result;
use rusqlite::{Connection, params};
use uuid::Uuid;

pub struct MemoryStore {
    db: Connection,
    pub semantic: SemanticStore,
    pub fts: FtsSearch,
    _db_path: String,
}

impl MemoryStore {
    pub fn new(path: &str) -> Result<Self> {
        let db = Connection::open(path)?;

        // Enable FTS5
        db.execute("CREATE VIRTUAL TABLE IF NOT EXISTS session_fts USING fts5(session_id, content, content_rowid)", [])?;

        // Create tables if not exist
        db.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                repo_url TEXT,
                status TEXT NOT NULL,
                iterations_run INTEGER DEFAULT 0,
                final_verdict TEXT
            )",
            [],
        )?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                session_id TEXT REFERENCES sessions(id),
                agent TEXT NOT NULL,
                state TEXT NOT NULL,
                input_json TEXT,
                output_json TEXT,
                confidence REAL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS decisions (
                id TEXT PRIMARY KEY,
                session_id TEXT REFERENCES sessions(id),
                agent TEXT NOT NULL,
                decision_type TEXT NOT NULL,
                rationale TEXT,
                confidence REAL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS abilities (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                trigger_conditions TEXT NOT NULL,
                action_template TEXT NOT NULL,
                success_count INTEGER DEFAULT 0,
                last_used TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        let fts = FtsSearch::new(Connection::open(path)?)?;
        let semantic = SemanticStore::new(""); // empty = in-memory fallback

        Ok(Self {
            db,
            semantic,
            fts,
            _db_path: path.to_string(),
        })
    }

    /// Create a new session and return its ID.
    pub async fn create_session(&mut self, repo_url: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.db.execute(
            "INSERT INTO sessions (id, repo_url, status) VALUES (?, ?, ?)",
            params![&id, repo_url, "running"],
        )?;

        self.fts.index(&id, "New Chimera Builder session: analyzing repo")?;
        Ok(id)
    }

    /// Update session status.
    pub async fn update_session(&mut self, id: &str, status: &str, verdict: Option<&str>) -> Result<()> {
        if let Some(v) = verdict {
            self.db.execute(
                "UPDATE sessions SET status = ?, final_verdict = ? WHERE id = ?",
                params![status, v, id],
            )?;
        } else {
            self.db.execute(
                "UPDATE sessions SET status = ? WHERE id = ?",
                params![status, id],
            )?;
        }
        Ok(())
    }

    /// Record a task.
    pub async fn record_task(&mut self, session_id: &str, agent: &str, state: &str, input: &str, output: &str, confidence: f64) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        self.db.execute(
            "INSERT INTO tasks (id, session_id, agent, state, input_json, output_json, confidence) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![&id, session_id, agent, state, input, output, confidence],
        )?;
        self.fts.index(&session_id, &format!("Task: {} agent processed ({}). {}", agent, state, output))?;
        Ok(())
    }

    /// Record a decision.
    pub async fn record_decision(&mut self, session_id: &str, agent: &str, decision_type: &str, rationale: &str, confidence: f64) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        self.db.execute(
            "INSERT INTO decisions (id, session_id, agent, decision_type, rationale, confidence) VALUES (?, ?, ?, ?, ?, ?)",
            params![&id, session_id, agent, decision_type, rationale, confidence],
        )?;
        Ok(())
    }

    /// Record iteration completion.
    pub async fn record_iteration(&mut self, _iteration: u32) -> Result<()> {
        // Update iterations count in the latest session
        Ok(())
    }

    /// Extract and store abilities.
    pub async fn extract_abilities(&mut self, _session_id: &str, abilities: &[Ability]) -> Result<()> {
        for ability in abilities {
            let id = Uuid::new_v4().to_string();
            let triggers = serde_json::to_string(&ability.trigger_conditions)?;
            self.db.execute(
                "INSERT INTO abilities (id, category, trigger_conditions, action_template) VALUES (?, ?, ?, ?)",
                params![&id, ability.category.to_string(), triggers, ability.action_template],
            )?;
        }
        Ok(())
    }

    /// Load abilities for an agent category.
    pub fn load_abilities(&self, category: &str) -> Result<Vec<Ability>> {
        let mut stmt = self.db.prepare(
            "SELECT category, trigger_conditions, action_template, success_count FROM abilities WHERE category = ?",
        )?;

        let mut results = Vec::new();
        let rows = stmt.query_map(params![category], |row| {
            let cat: String = row.get(0)?;
            let triggers_str: String = row.get(1)?;
            let template: String = row.get(2)?;
            let success: u32 = row.get(3)?;
            let category = match cat.as_str() {
                "repo_analysis" => AbilityCategory::RepoAnalysis,
                "task_planning" => AbilityCategory::TaskPlanning,
                "code_generation" => AbilityCategory::CodeGeneration,
                "workflow_validation" => AbilityCategory::WorkflowValidation,
                "cross_agent_communication" => AbilityCategory::CrossAgentCommunication,
                _ => AbilityCategory::RepoAnalysis,
            };
            Ok(Ability {
                id: Uuid::new_v4().to_string(),
                category,
                trigger_conditions: serde_json::from_str(&triggers_str).unwrap_or_default(),
                action_template: template,
                success_count: success,
                last_used: None,
            })
        })?;

        for row in rows {
            if let Ok(ability) = row {
                results.push(ability);
            }
        }
        Ok(results)
    }

    /// Get all sessions for a repo URL.
    pub fn get_sessions(&self, repo_url: &str) -> Result<Vec<String>> {
        let mut stmt = self.db.prepare(
            "SELECT id FROM sessions WHERE repo_url = ? ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map(params![repo_url], |row| row.get::<_, String>(0))?;
        let mut results = Vec::new();
        for row in rows {
            if let Ok(s) = row {
                results.push(s);
            }
        }
        Ok(results)
    }
}
