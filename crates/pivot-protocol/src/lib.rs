//! Proposed transport-independent contract. No core runtime identifiers or hosting.
pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/pivot.v1.rs"));
}
pub mod validation;
