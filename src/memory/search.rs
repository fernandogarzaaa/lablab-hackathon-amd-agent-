//! FtsSearch — full-text search via SQLite FTS5.

use anyhow::Result;
use rusqlite::{Connection, params};

/// FTS5 index for cross-session recall.
pub struct FtsSearch {
    db: Connection,
}

impl FtsSearch {
    /// Create from an existing SQLite connection.
    pub fn new(db: Connection) -> Result<Self> {
        // Ensure FTS5 table exists
        db.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS session_fts USING fts5(session_id, content, content_rowid)",
            [],
        )?;
        Ok(Self { db })
    }

    /// Index content for a session.
    pub fn index(&mut self, session_id: &str, content: &str) -> Result<()> {
        self.db.execute(
            "INSERT OR REPLACE INTO session_fts (session_id, content) VALUES (?, ?)",
            params![session_id, content],
        )?;
        Ok(())
    }

    /// Search indexed content by query terms.
    pub fn search(&self, query: &str, n: usize) -> Result<Vec<(String, String)>> {
        // Use the FTS5 match syntax for token search
        let match_query = query.replace(' ', " OR ") + " OR *";
        let sql = format!(
            "SELECT session_id, content FROM session_fts WHERE session_fts MATCH ? ORDER BY rank LIMIT ?",
        );
        let mut stmt = self.db.prepare(&sql)?;
        let rows = stmt.query_map(params![match_query, n], |row| {
            let session_id: String = row.get(0)?;
            let content: String = row.get(1)?;
            Ok((session_id, content))
        })?;

        let mut results = Vec::new();
        for row in rows {
            if let Ok(entry) = row {
                results.push(entry);
            }
        }
        Ok(results)
    }

    /// Clear the index.
    pub fn clear(&mut self) -> Result<()> {
        self.db.execute("DELETE FROM session_fts", [])?;
        Ok(())
    }
}
