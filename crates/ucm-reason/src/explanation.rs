//! Explanation engine — traceable reasoning chains for every conclusion.
//!
//! Every output from the reasoning engine includes an explanation_chain:
//! a sequence of ReasoningStep structs that trace from raw evidence
//! through inferences to final conclusions. This is what makes
//! the system auditable and debuggable.
//!
//! Example chain for "processPayment is indirectly impacted":
//! Step 1: evidence="git diff shows validateToken() signature changed"
//!         inference="Return type changed from boolean to Result<Claims, AuthError>"
//!         confidence=1.0
//! Step 2: evidence="Static analysis: 3 call sites found via reverse BFS"
//!         inference="All callers must handle new Result type"
//!         confidence=0.95
//! Step 3: evidence="API logs: /checkout called 1.2M times/day via authMiddleware"
//!         inference="Payment flow is high-traffic indirect dependency"
//!         confidence=0.72

use serde::{Deserialize, Serialize};

/// A single step in a reasoning chain — the atomic unit of explanation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    /// Step number in the chain
    pub step: usize,
    /// What data / observation this step is based on
    pub evidence: String,
    /// What conclusion was drawn from the evidence
    pub inference: String,
    /// How confident we are in this inference [0.0, 1.0]
    pub confidence: f64,
}

/// A complete explanation chain — tells the full story of a conclusion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationChain {
    /// Human-readable summary of the conclusion
    pub summary: String,
    /// The ordered sequence of reasoning steps
    pub steps: Vec<ReasoningStep>,
    /// Overall confidence (product of step confidences, or custom)
    pub overall_confidence: f64,
}

impl ExplanationChain {
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            steps: Vec::new(),
            overall_confidence: 1.0,
        }
    }

    /// Add a reasoning step to the chain.
    pub fn add_step(
        &mut self,
        evidence: impl Into<String>,
        inference: impl Into<String>,
        confidence: f64,
    ) -> &mut Self {
        let step_num = self.steps.len() + 1;
        self.steps.push(ReasoningStep {
            step: step_num,
            evidence: evidence.into(),
            inference: inference.into(),
            confidence,
        });
        // Update overall confidence as product
        self.overall_confidence = self.steps.iter().map(|s| s.confidence).product();
        self
    }

    /// Convert to a human-readable narrative.
    pub fn to_narrative(&self) -> String {
        let mut narrative = format!("**{}**\n\n", self.summary);
        for step in &self.steps {
            narrative.push_str(&format!(
                "Step {}: Based on {} → concluded {} (confidence: {:.0}%)\n",
                step.step,
                step.evidence,
                step.inference,
                step.confidence * 100.0
            ));
        }
        narrative.push_str(&format!(
            "\nOverall confidence: {:.0}%",
            self.overall_confidence * 100.0
        ));
        narrative
    }
}

/// Build an explanation for why an entity IS impacted.
pub fn explain_impact(entity_name: &str, path: &[String], confidence: f64) -> ExplanationChain {
    let mut chain = ExplanationChain::new(format!("{entity_name} is impacted by this change"));

    if path.len() <= 1 {
        chain.add_step(
            "Direct reference to changed entity found in code",
            format!("{entity_name} directly references the changed code"),
            confidence,
        );
    } else {
        chain.add_step(
            format!(
                "Graph traversal found dependency path: {}",
                path.join(" → ")
            ),
            format!(
                "{entity_name} is transitively dependent via {} hops",
                path.len() - 1
            ),
            confidence,
        );

        if confidence < 0.85 {
            chain.add_step(
                format!("Transitive confidence decays over {} hops", path.len() - 1),
                "Each intermediate dependency reduces certainty",
                confidence,
            );
        }
    }

    chain
}

/// Build an explanation for why an entity is NOT impacted.
pub fn explain_not_impacted(entity_name: &str, reason: &str, confidence: f64) -> ExplanationChain {
    let mut chain = ExplanationChain::new(format!("{entity_name} is NOT impacted by this change"));

    chain.add_step(
        format!("Analyzed graph connectivity for {entity_name}"),
        reason.to_string(),
        confidence,
    );

    chain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explanation_chain() {
        let mut chain = ExplanationChain::new("authMiddleware is directly impacted");
        chain.add_step(
            "git diff shows validateToken() signature changed",
            "Return type changed from boolean to Result<Claims, AuthError>",
            1.0,
        );
        chain.add_step(
            "Static analysis: authMiddleware imports validateToken",
            "authMiddleware must handle new Result type",
            0.95,
        );

        assert_eq!(chain.steps.len(), 2);
        assert!((chain.overall_confidence - 0.95).abs() < 0.01);

        let narrative = chain.to_narrative();
        assert!(narrative.contains("Step 1"));
        assert!(narrative.contains("Step 2"));
    }

    #[test]
    fn test_explain_impact() {
        let chain = explain_impact(
            "processPayment",
            &[
                "validateToken".into(),
                "authMiddleware".into(),
                "processPayment".into(),
            ],
            0.72,
        );
        assert!(!chain.steps.is_empty());
        assert!(chain.summary.contains("processPayment"));
    }

    #[test]
    fn test_explain_not_impacted() {
        let chain = explain_not_impacted(
            "generateReport",
            "No graph path exists; uses separate admin auth flow",
            0.88,
        );
        assert!(chain.summary.contains("NOT impacted"));
    }
}
