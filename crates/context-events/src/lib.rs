//! Event store — append-only source of truth for all context mutations.
//!
//! The event log is authoritative — the graph is always rebuildable from events.
//! This is the Datomic-inspired "database as a value" model: every fact is
//! appended, never updated. Queries run against immutable snapshots.
//!
//! Design: In-memory store with the same API surface as a RocksDB-backed store.
//! For production, swap to RocksDB with column families per stream.
//!
//! References:
//! - Datomic: https://docs.datomic.com/
//! - Event Sourcing: https://martinfowler.com/eaaDev/EventSourcing.html

pub mod store;
pub mod projection;

pub use store::*;
pub use projection::*;
