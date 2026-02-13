//! Observability layer — decision traces + replay debugger.
//!
//! Every reasoning decision is recorded as a structured trace.
//! Traces can be replayed from the event log to verify determinism.
//!
//! This is what makes the system auditble. When a developer asks
//! "why did the system recommend testing X?", the trace contains
//! the full derivation chain.

pub mod trace;
pub mod replay;

pub use trace::*;
pub use replay::*;
