//! Confidence middleware types.

use crate::core::types::ConfidenceLevel;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfidenceError {
    #[error("Hallucination detected: {0}")]
    HallucinationDetected(String),

    #[error("Confidence below threshold: {0} < {1}")]
    BelowThreshold(f64, f64),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Chimera tool error: {0}")]
    ToolError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedOutput<T> {
    pub data: T,
    pub confidence: ConfidenceLevel,
    pub detected: DetectionResult,
    pub gated: bool,
    pub audit_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub strategy: HallucinationStrategy,
    pub has_hallucination: bool,
    pub confidence: f64,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HallucinationStrategy {
    Range,
    Dictionary,
    Semantic,
    CrossReference,
    Temporal,
}

impl Default for HallucinationStrategy {
    fn default() -> Self {
        Self::Semantic
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GateStrategy {
    Majority,
    WeightedVote,
    HighestConfidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub confidence: f64,
    pub action: String,
    pub input_hash: String,
    pub output_hash: String,
}
