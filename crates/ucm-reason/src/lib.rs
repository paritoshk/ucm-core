//! Reasoning & Test Intent Engine — the "brain" of autonomous QA.
//!
//! This crate answers the core questions:
//! 1. What changed? (Change analysis)
//! 2. What is impacted? (Impact analysis with graph traversal)
//! 3. What is NOT impacted? (Equally important — explains WHY)
//! 4. What should be tested? (Test intent with confidence tiers)
//! 5. What is uncertain? (Ambiguity detection)
//! 6. WHY did the system reach these conclusions? (Explanation chains)
//!
//! Every output includes an `explanation_chain` — a sequence of
//! ReasoningStep structs, each with evidence (what data), inference
//! (what conclusion), and confidence (how sure). This is the
//! "show your work" layer that makes the system auditable.

pub mod impact;
pub mod intent;
pub mod explanation;
pub mod ambiguity;

pub use impact::*;
pub use intent::*;
pub use explanation::*;
pub use ambiguity::*;
