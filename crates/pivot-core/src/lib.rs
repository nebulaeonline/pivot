//! UI-free model definitions, snapshots, and decimal computation.
pub mod evaluation;
pub mod expression;
pub mod interface;
pub mod model;

/// Benchmark-only instrumentation; no default production path changes.
#[cfg(any(feature = "scaling-experiment", feature = "core-integration"))]
pub mod experiment_profile;
