//! DependencyMapper — maps dependency graphs from a real repo.

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

/// Maps dependencies between files and crates from a real repo.
pub struct DependencyMapper;

impl DependencyMapper {
    pub fn new() -> Self {
        Self
    }

    /// Build a dependency graph from a real repository directory.
    pub fn map(&self, repo_path: &str) -> Vec<DepEdge> {
        let mut edges = Vec::new();
        let base = std::path::Path::new(repo_path);

        if let Ok(content) = std::fs::read_to_string(base.join("Cargo.toml")) {
            // Extract dependencies from [[package]] sections
            for line in content.lines() {
                let line = line.trim();
                if let Some(dep) = line.strip_prefix("name = \"") {
                    if let Some(end) = dep.find('"') {
                        let name = &dep[..end];
                        edges.push(DepEdge {
                            from: "Cargo.toml".to_string(),
                            to: name.to_string(),
                            type_: DependencyType::Crate,
                        });
                    }
                }
            }
            // Also extract from [dependencies] section
            let in_deps = content.lines().any(|l| l.trim() == "[dependencies]");
            if in_deps {
                let in_section = content
                    .lines()
                    .skip_while(|l| !l.trim().starts_with("[dependencies]"))
                    .skip(1)
                    .take_while(|l| l.trim().starts_with('[') == false && !l.trim().is_empty())
                    .filter_map(|l| {
                        l.trim().split('=').next().map(|k| k.trim().to_string())
                    });
                for dep_name in in_section {
                    if dep_name != "dependencies" {
                        edges.push(DepEdge {
                            from: "Cargo.toml".to_string(),
                            to: dep_name,
                            type_: DependencyType::Crate,
                        });
                    }
                }
            }
        }

        if let Ok(content) = std::fs::read_to_string(base.join("package.json")) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(deps) = val.get("dependencies").and_then(|d| d.as_object()) {
                    for name in deps.keys() {
                        edges.push(DepEdge {
                            from: "package.json".to_string(),
                            to: name.to_string(),
                            type_: DependencyType::Crate,
                        });
                    }
                }
            }
        }

        edges
    }
}
