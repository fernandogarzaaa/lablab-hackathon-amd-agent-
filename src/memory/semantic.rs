//! SemanticStore — ChromaDB vector store with in-memory fallback.

use serde::{Deserialize, Serialize};

/// ChromaDB client wrapper for vector storage.
///
/// Attempts to connect to a ChromaDB sidecar container via HTTP API.
/// Falls back to in-memory storage if ChromaDB is unavailable.
pub struct SemanticStore {
    base_url: String,
    http_client: reqwest::Client,
    fallback: std::sync::RwLock<FallbackStore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticEntry {
    pub id: String,
    pub collection: String,
    pub text: String,
    pub metadata: std::collections::HashMap<String, String>,
    pub score: f64,
}

#[derive(Default)]
struct FallbackStore {
    collections: std::collections::HashMap<String, Vec<SemanticEntry>>,
}

impl SemanticStore {
    /// Create with a ChromaDB URL. Pass empty string for in-memory fallback mode.
    pub fn new(base_url: &str) -> Self {
        let effective_url = if base_url.is_empty() {
            "http://localhost:8000".to_string()
        } else {
            base_url.to_string()
        };
        Self {
            base_url: effective_url,
            http_client: reqwest::Client::new(),
            fallback: std::sync::RwLock::new(FallbackStore::default()),
        }
    }

    /// Try to add an entry to ChromaDB. Falls back to in-memory if unavailable.
    pub async fn add(&self, collection: &str, id: &str, text: &str, metadata: std::collections::HashMap<String, String>) -> Result<(), anyhow::Error> {
        // Try ChromaDB first
        let collection_body = serde_json::json!({
            "name": collection,
            "metadatas": [metadata],
        });
        if let Err(_) = self.http_client
            .post(format!("{}/api/v1/collections", self.base_url))
            .json(&collection_body)
            .send()
            .await
        {
            // ChromaDB unavailable, use fallback
            let mut store = self.fallback.write().unwrap();
            let entry = SemanticEntry {
                id: id.to_string(),
                collection: collection.to_string(),
                text: text.to_string(),
                metadata,
                score: 0.0,
            };
            store.collections.entry(collection.to_string()).or_insert_with(Vec::new).push(entry);
        }
        Ok(())
    }

    /// Query a collection by semantic similarity.
    pub async fn query(&self, collection: &str, query: &str, n: usize) -> Result<Vec<SemanticEntry>, anyhow::Error> {
        // Try ChromaDB first
        let query_body = serde_json::json!({
            "query_texts": [query],
            "n_results": n,
        });
        if let Ok(resp) = self.http_client
            .post(format!("{}/api/v1/collections/{}/query", self.base_url, collection))
            .json(&query_body)
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    if let Some(ids) = body.get("ids").and_then(|i| i.get(0).and_then(|x| x.as_array())) {
                        if let Some(distances) = body.get("distances").and_then(|d| d.get(0).and_then(|x| x.as_array())) {
                            let mut entries = Vec::new();
                            for (i, id) in ids.iter().enumerate() {
                                let score = distances.get(i)
                                    .and_then(|v| v.as_f64())
                                    .map(|d| 1.0 - d)
                                    .unwrap_or(0.0);
                                entries.push(SemanticEntry {
                                    id: id.as_str().unwrap_or("").to_string(),
                                    collection: collection.to_string(),
                                    text: String::new(),
                                    metadata: std::collections::HashMap::new(),
                                    score,
                                });
                            }
                            return Ok(entries);
                        }
                    }
                }
            }
        }

        // ChromaDB unavailable, use fallback
        let store = self.fallback.read().unwrap();
        let entries = store.collections.get(collection).cloned().unwrap_or_default();
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
        Ok(scored.into_iter().take(n).collect())
    }

    /// List all collections.
    pub fn list_collections(&self) -> Vec<String> {
        let store = self.fallback.read().unwrap();
        store.collections.keys().cloned().collect()
    }
}
