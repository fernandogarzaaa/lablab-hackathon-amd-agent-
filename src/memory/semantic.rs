//! SemanticStore — ChromaDB vector store integration.

use serde::{Deserialize, Serialize};

/// ChromaDB client wrapper for vector storage.
///
/// In production, this connects to the ChromaDB sidecar container
/// via HTTP API. For now, provides a simplified in-memory implementation.
pub struct SemanticStore {
    collections: std::collections::HashMap<String, Vec<SemanticEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticEntry {
    pub id: String,
    pub collection: String,
    pub text: String,
    pub metadata: std::collections::HashMap<String, String>,
    pub score: f64,
}

impl SemanticStore {
    pub fn new() -> Self {
        Self {
            collections: std::collections::HashMap::new(),
        }
    }

    /// Add an embedding to a collection.
    pub async fn add(&mut self, collection: &str, id: &str, text: &str, metadata: std::collections::HashMap<String, String>) {
        let entry = SemanticEntry {
            id: id.to_string(),
            collection: collection.to_string(),
            text: text.to_string(),
            metadata,
            score: 0.0,
        };

        self.collections.entry(collection.to_string()).or_insert_with(Vec::new).push(entry);
    }

    /// Query a collection by semantic similarity (simulated).
    ///
    /// In production, this would compute vector similarity against ChromaDB.
    /// For now, returns entries matching metadata filters.
    pub async fn query(&self, collection: &str, query: &str, n: usize) -> Vec<SemanticEntry> {
        let entries = self.collections.get(collection).cloned().unwrap_or_default();

        // Simulate semantic scoring by checking query term overlap
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<_> = entries.into_iter().map(|e| {
            let overlap = query_terms.iter()
                .filter(|t| e.text.to_lowercase().contains(**t))
                .count();
            let score = overlap as f64 / query_terms.len().max(1) as f64;
            SemanticEntry { score, ..e }
        }).collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(n).collect()
    }

    /// List all collections.
    pub fn list_collections(&self) -> Vec<String> {
        self.collections.keys().cloned().collect()
    }
}
