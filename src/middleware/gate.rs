//! Confidence gate logic.

use crate::core::types::ConfidenceLevel;
pub use crate::middleware::types::GateStrategy;
use std::hash::Hash;

/// Collapse multiple candidate outputs via consensus.
pub fn gate_candidates<T: PartialEq + Eq + Hash + Clone>(
    candidates: Vec<(T, f64)>,
    strategy: GateStrategy,
    threshold: f64,
) -> Option<(T, ConfidenceLevel)> {
    if candidates.is_empty() {
        return None;
    }

    let result = match strategy {
        GateStrategy::Majority => majority_collapse(candidates, threshold),
        GateStrategy::WeightedVote => weighted_vote_collapse(candidates, threshold),
        GateStrategy::HighestConfidence => highest_confidence(candidates),
    };

    result.map(|(data, score)| {
        (data, ConfidenceLevel::from(score))
    })
}

fn majority_collapse<T: PartialEq + Eq + Hash + Clone>(
    candidates: Vec<(T, f64)>,
    threshold: f64,
) -> Option<(T, f64)> {
    let mut votes: std::collections::HashMap<T, u32> = std::collections::HashMap::new();

    for (data, _) in &candidates {
        *votes.entry(data.clone()).or_insert(0) += 1;
    }

    let total = candidates.len() as f64;
    let (best_data, votes) = votes.into_iter().max_by_key(|(_, v)| *v)?;
    let score = votes as f64 / total;

    if score >= threshold {
        Some((best_data, score))
    } else {
        None
    }
}

fn weighted_vote_collapse<T: PartialEq + Eq + Hash + Clone>(
    candidates: Vec<(T, f64)>,
    threshold: f64,
) -> Option<(T, f64)> {
    let mut scores: std::collections::HashMap<T, f64> = std::collections::HashMap::new();

    for (data, weight) in &candidates {
        *scores.entry(data.clone()).or_insert(0.0) += weight;
    }

    let best = scores.into_iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))?;
    let score = best.1 / candidates.len() as f64;

    if score >= threshold {
        Some((best.0, score))
    } else {
        None
    }
}

fn highest_confidence<T: PartialEq + Eq + Hash + Clone>(
    candidates: Vec<(T, f64)>,
) -> Option<(T, f64)> {
    candidates.into_iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
}

/// Generate a deterministic audit ID from input content.
pub fn audit_id(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
