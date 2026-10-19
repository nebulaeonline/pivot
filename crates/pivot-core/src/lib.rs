//! UI-free model definitions, snapshots, and decimal computation.
pub mod evaluation;
pub mod expression;
pub mod interface;
pub mod model;

/// Benchmark-only instrumentation; no default production path changes.
#[cfg(feature = "scaling-experiment")]
pub mod experiment_profile;
