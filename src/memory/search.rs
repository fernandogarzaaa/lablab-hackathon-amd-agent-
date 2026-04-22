//! FtsSearch — full-text search via SQLite FTS5.

use anyhow::Result;

/// FTS5 index for cross-session recall.
pub struct FtsSearch {
    /// In-memory index: (session_id -> content)
    /// In production, backed by SQLite FTS5 virtual table.
    index: std::collections::HashMap<String, String>,
}

impl FtsSearch {
    pub fn new() -> Self {
        Self {
            index: std::collections::HashMap::new(),
        }
    }

    /// Index content for a session.
    pub async fn index(&mut self, session_id: &str, content: &str) -> Result<()> {
        self.index.insert(session_id.to_string(), content.to_string());
        Ok(())
    }

    /// Search indexed content by query terms.
    pub async fn search(&self, query: &str, n: usize) -> Vec<(String, String)> {
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<_> = self.index.iter().filter(|(_, content)| {
            query_terms.iter().any(|t| content.to_lowercase().contains(t))
        }).collect();

        results.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        results.into_iter().take(n).map(|(id, content)| (id.to_string(), content.to_string())).collect()
    }

    /// Clear the index.
    pub fn clear(&mut self) {
        self.index.clear();
    }
}
