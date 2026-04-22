//! DependencyMapper — maps dependency graphs.

use serde::{Deserialize, Serialize};

/// A dependency edge in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepEdge {
    pub from: String,
    pub to: String,
    pub type_: DependencyType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DependencyType {
    Import,
    Module,
    Crate,
    External,
}

/// Maps dependencies between files and crates.
pub struct DependencyMapper;

impl DependencyMapper {
    pub fn new() -> Self {
        Self
    }

    /// Build a dependency graph (simulated).
    pub fn map(&self) -> Vec<DepEdge> {
        vec![
            DepEdge { from: "main.rs".to_string(), to: "lib.rs".to_string(), type_: DependencyType::Module },
            DepEdge { from: "src/lib.rs".to_string(), to: "src/core/mod.rs".to_string(), type_: DependencyType::Module },
            DepEdge { from: "src/lib.rs".to_string(), to: "src/agents/mod.rs".to_string(), type_: DependencyType::Module },
            DepEdge { from: "src/lib.rs".to_string(), to: "src/analysis/mod.rs".to_string(), type_: DependencyType::Module },
            DepEdge { from: "src/lib.rs".to_string(), to: "src/middleware/mod.rs".to_string(), type_: DependencyType::Module },
            DepEdge { from: "src/lib.rs".to_string(), to: "src/memory/mod.rs".to_string(), type_: DependencyType::Module },
            DepEdge { from: "src/lib.rs".to_string(), to: "src/cli/mod.rs".to_string(), type_: DependencyType::Module },
            DepEdge { from: "Cargo.toml".to_string(), to: "tokio".to_string(), type_: DependencyType::Crate },
            DepEdge { from: "Cargo.toml".to_string(), to: "clap".to_string(), type_: DependencyType::Crate },
            DepEdge { from: "Cargo.toml".to_string(), to: "rusqlite".to_string(), type_: DependencyType::Crate },
            DepEdge { from: "Cargo.toml".to_string(), to: "serde".to_string(), type_: DependencyType::Crate },
            DepEdge { from: "Cargo.toml".to_string(), to: "reqwest".to_string(), type_: DependencyType::Crate },
            DepEdge { from: "Cargo.toml".to_string(), to: "git2".to_string(), type_: DependencyType::Crate },
        ]
    }
}
