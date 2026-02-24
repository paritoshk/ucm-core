//! Entity model using SCIP-style human-readable, globally-unique symbol strings.
//!
//! Identity format: `scip:<manager>/<package>/<version>/<path>#<symbol>`
//! Example: `scip:npm/my-app/1.0/src/auth/service.ts/AuthService#validateToken`
//!
//! This approach is drawn from Sourcegraph's SCIP protocol — because each document
//! is self-contained with symbol strings rather than graph-local IDs, individual
//! files can be re-indexed independently without global coordination.
//! Reference: https://github.com/sourcegraph/scip

use serde::{Deserialize, Serialize};

/// SCIP-style globally unique identifier for any code entity.
///
/// Format: `scip:<manager>/<package>/<version>/<path>#<symbol>`
/// The string encoding ensures files can be re-indexed independently.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub String);

impl EntityId {
    /// Create a new SCIP-style entity ID.
    ///
    /// # Arguments
    /// - `manager` - Package manager (e.g., "npm", "pip", "cargo", "local")
    /// - `package` - Package name (e.g., "my-app")
    /// - `version` - Version string (e.g., "1.0.0")
    /// - `path` - File path relative to package root
    /// - `symbol` - Symbol name within the file (function, class, etc.)
    pub fn new(manager: &str, package: &str, version: &str, path: &str, symbol: &str) -> Self {
        Self(format!(
            "scip:{manager}/{package}/{version}/{path}#{symbol}"
        ))
    }

    /// Create a simple local entity ID (for single-project analysis).
    pub fn local(path: &str, symbol: &str) -> Self {
        Self(format!("scip:local/project/0.0.0/{path}#{symbol}"))
    }

    /// Extract the file path component from the SCIP ID.
    pub fn file_path(&self) -> Option<&str> {
        let after_version = self.0.split('/').skip(3).collect::<Vec<_>>().join("/");
        let path = after_version.split('#').next()?;
        // Return from the original string to avoid allocation
        let start = self.0.find(path)?;
        let end = self.0.find('#').unwrap_or(self.0.len());
        Some(&self.0[start..end])
    }

    /// Extract the symbol name from the SCIP ID.
    pub fn symbol_name(&self) -> Option<&str> {
        self.0.split('#').nth(1)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The kind of code entity in the context graph.
///
/// This union-type approach (inspired by Glean's `code.Entity` sum type)
/// provides a unified view across languages while carrying language-specific
/// metadata in each variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityKind {
    /// A function or method definition
    Function {
        is_async: bool,
        parameter_count: usize,
        return_type: Option<String>,
    },
    /// An API endpoint (REST, GraphQL, gRPC)
    ApiEndpoint {
        method: String,  // GET, POST, etc.
        route: String,   // /api/v1/users
        handler: String, // function that handles the route
    },
    /// A data model / struct / class / table
    DataModel { fields: Vec<String> },
    /// A feature or capability (extracted from tickets/docs)
    Feature {
        description: String,
        source: String, // "jira", "docs", etc.
    },
    /// A test case
    TestCase {
        test_type: TestType,
        targets: Vec<EntityId>, // what entities this test covers
    },
    /// A requirement (from Jira, docs, specs)
    Requirement {
        ticket_id: Option<String>,
        acceptance_criteria: Vec<String>,
    },
    /// A module or file-level entity
    Module {
        language: String,
        exports: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TestType {
    Unit,
    Integration,
    E2E,
    Property,
}

/// A node in the context graph — an entity with its identity and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UcmEntity {
    /// SCIP-style globally unique identifier
    pub id: EntityId,
    /// What kind of entity this is
    pub kind: EntityKind,
    /// Human-readable name
    pub name: String,
    /// Source file path (relative to project root)
    pub file_path: String,
    /// Line range in source file [start, end]
    pub line_range: Option<(usize, usize)>,
    /// Programming language
    pub language: String,
    /// When this entity was first discovered
    pub discovered_at: chrono::DateTime<chrono::Utc>,
    /// Which ingestion source discovered it
    pub discovery_source: DiscoverySource,
}

/// How an entity was discovered — determines base confidence and decay rate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiscoverySource {
    /// Extracted from source code via tree-sitter AST parsing
    StaticAnalysis,
    /// Inferred from git diff comparison
    GitDiff,
    /// Extracted from Jira/ticket system
    TicketSystem,
    /// Observed in API traffic logs
    ApiTraffic,
    /// Loaded from historical context snapshot
    HistoricalContext,
    /// Manually specified
    Manual,
}

impl UcmEntity {
    pub fn new(
        id: EntityId,
        kind: EntityKind,
        name: impl Into<String>,
        file_path: impl Into<String>,
        language: impl Into<String>,
        source: DiscoverySource,
    ) -> Self {
        Self {
            id,
            kind,
            name: name.into(),
            file_path: file_path.into(),
            line_range: None,
            language: language.into(),
            discovered_at: chrono::Utc::now(),
            discovery_source: source,
        }
    }

    pub fn with_line_range(mut self, start: usize, end: usize) -> Self {
        self.line_range = Some((start, end));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scip_id_construction() {
        let id = EntityId::new(
            "npm",
            "my-app",
            "1.0.0",
            "src/auth/service.ts",
            "validateToken",
        );
        assert_eq!(
            id.as_str(),
            "scip:npm/my-app/1.0.0/src/auth/service.ts#validateToken"
        );
    }

    #[test]
    fn test_scip_id_local() {
        let id = EntityId::local("src/main.rs", "main");
        assert!(id.as_str().contains("local/project"));
    }

    #[test]
    fn test_entity_symbol_name() {
        let id = EntityId::new(
            "npm",
            "my-app",
            "1.0.0",
            "src/auth/service.ts",
            "validateToken",
        );
        assert_eq!(id.symbol_name(), Some("validateToken"));
    }
}
