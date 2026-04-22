pub mod repo_parser;
pub mod dep_mapper;
pub mod issue_detect;

pub use repo_parser::{RepoParser, RepoStructure, TechStackDetector};
pub use dep_mapper::DependencyMapper;
pub use issue_detect::IssueDetector;
