//! TRUST 5 Quality Dashboard -- Data model + scoring engine (SPEC-V3-017 MS-1/MS-3)
//!
//! This module provides the core data structures and scoring logic for the TRUST 5 framework:
//! - `Trust5Score`: 5-dimension quality score (Tested, Readable, Unified, Secured, Trackable)
//! - `ScoringEngine`: Trait for computing scores from metrics
//! - `DefaultHeuristicEngine`: Default heuristic implementation
//! - Metric snapshots: LSP, test, git, security metrics
//! - `QualityHistory`: Ring buffer of quality snapshots (MS-3)

pub mod engine;
pub mod history;
pub mod metrics;
pub mod score;

// Re-export common types
pub use engine::{DefaultHeuristicEngine, ScoringEngine};
pub use history::{QualityHistory, QualitySnapshot};
pub use metrics::{GitMetrics, LspMetrics, SecurityMetrics, TestMetrics};
pub use score::Trust5Score;
