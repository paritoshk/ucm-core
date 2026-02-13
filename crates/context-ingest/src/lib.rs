//! Context ingestion layer — multi-source signal extraction.
//!
//! Ingests heterogeneous signals about an application and emits
//! typed ContextEvents. Each adapter normalizes its input format
//! into the unified event model.
//!
//! Current adapters:
//! - Code parser (mock tree-sitter): source code → functions, imports, classes
//! - Git diff parser: before/after → ChangeDetected events
//! - Jira adapter: ticket JSON → Requirement entities
//! - API log adapter: access logs → ApiEndpoint entities + traffic confidence
//!
//! In production, the code parser would use real tree-sitter bindings
//! (56+ languages). The mock parser demonstrates the same API surface
//! and event flow without the native C dependency.

pub mod code_parser;
pub mod diff_parser;
pub mod jira_adapter;
pub mod api_log_adapter;

pub use code_parser::*;
pub use diff_parser::*;
pub use jira_adapter::*;
pub use api_log_adapter::*;
