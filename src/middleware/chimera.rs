//! ConfidenceMiddleware — the global ChimeraLang integration layer.
//!
//! Every agent output flows through this pipeline before being passed to the next agent:
//! detect → confident → constrain → audit

use crate::core::types::ConfidenceLevel;
use crate::middleware::gate::{audit_id, gate_candidates, GateStrategy};
use crate::middleware::types::*;
use anyhow::Result;
use sha2::{Digest, Sha256};
use tracing::{info, warn};

pub struct ConfidenceMiddleware {
    active: bool,
    audit_log: Vec<AuditEntry>,
}

impl ConfidenceMiddleware {
    pub fn new() -> Self {
        Self {
            active: true,
            audit_log: Vec::new(),
        }
    }

    /// Process an agent output through the full ChimeraLang pipeline.
    pub async fn process<T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone>(
        &mut self,
        data: T,
        min_confidence: f64,
    ) -> Result<ProcessedOutput<T>> {
        if !self.active {
            return Ok(ProcessedOutput {
                data,
                confidence: ConfidenceLevel::Uncertain(1.0),
                detected: DetectionResult {
                    strategy: HallucinationStrategy::Semantic,
                    has_hallucination: false,
                    confidence: 1.0,
                    details: "Middleware disabled".to_string(),
                },
                gated: true,
                audit_id: String::new(),
            });
        }

        let input_str = serde_json::to_string(&data)?;
        let input_hash = Self::hash(&input_str);

        // Step 1: Hallucination detection (semantic)
        let detection = self.detect_hallucination(&input_str).await?;
        if detection.has_hallucination {
            warn!("Hallucination detected (confidence: {:.2})", detection.confidence);
        }

        // Step 2: Confidence gating
        let confidence = self.confidence_gate(&input_str, min_confidence).await?;
        if confidence < min_confidence {
            return Err(anyhow::anyhow!(
                "Output failed confidence gate at {:.2} (threshold: {:.2})",
                min_confidence,
                min_confidence
            ));
        }

        // Step 3: Audit logging
        let output_hash = input_hash.clone(); // In real impl, hash output after processing
        let audit_id = audit_id(&input_str);
        self.audit_log.push(AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            confidence,
            action: format!("processed (min_conf: {:.2})", min_confidence),
            input_hash: input_hash.clone(),
            output_hash,
        });

        info!("[MIDDLEWARE] Output validated (confidence: {:.2}, audit: {})", confidence, audit_id);

        Ok(ProcessedOutput {
            data,
            confidence: ConfidenceLevel::from(confidence),
            detected: detection,
            gated: true,
            audit_id,
        })
    }

    /// Gate multiple candidate outputs via consensus.
    pub fn gate<T: std::hash::Hash + PartialEq + Eq + Clone + serde::Serialize>(
        &self,
        candidates: Vec<(T, f64)>,
        strategy: GateStrategy,
        threshold: f64,
    ) -> Result<Option<(T, ConfidenceLevel)>> {
        Ok(gate_candidates(candidates, strategy, threshold))
    }

    /// Run a session-level audit.
    pub fn audit(&self, data: &serde_json::Value, confidence: f64) -> Vec<AuditEntry> {
        let input_str = serde_json::to_string(data).unwrap_or_default();
        let input_hash = Self::hash(&input_str);
        let _audit_id = audit_id(&input_str);

        vec![AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            confidence,
            action: "session_audit".to_string(),
            input_hash: input_hash.clone(),
            output_hash: input_hash,
        }]
    }

    async fn detect_hallucination(&self, input: &str) -> Result<DetectionResult> {
        // Simulate semantic detection: check for absolute certainty markers
        let markers = ["definitely", "absolutely", "undoubtedly", "certainly", "100%", "100% sure"];
        let has_hallucination = markers.iter().any(|m| input.to_lowercase().contains(m));

        let confidence = if has_hallucination { 0.65 } else { 0.92 };

        Ok(DetectionResult {
            strategy: HallucinationStrategy::Semantic,
            has_hallucination,
            confidence,
            details: if has_hallucination {
                "Potential absolute certainty markers found in output".to_string()
            } else {
                "No hallucination signals detected".to_string()
            },
        })
    }

    async fn confidence_gate(&self, input: &str, threshold: f64) -> Result<f64> {
        // Simulate confidence scoring
        // In production, this would call the ChimeraLang MCP server
        let score = Self::compute_confidence(input);
        if score < threshold {
            warn!("Confidence {:.2} below threshold {:.2}", score, threshold);
        }
        Ok(score)
    }

    fn compute_confidence(input: &str) -> f64 {
        // Deterministic confidence based on content length and structure
        let len_factor = (input.len() as f64 / 1000.0).min(1.0);
        let structure_factor = input.lines().count() as f64 / 50.0;
        len_factor.min(structure_factor).min(0.95).max(0.50)
    }

    fn hash(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }
}
