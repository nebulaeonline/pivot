//! Proposed transport-independent contract. No core runtime identifiers or hosting.
pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/pivot.v1.rs"));
}
pub mod validation;
pub mod authoring {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/pivot.authoring.v1.rs"));
    }
}
pub mod foundation;
pub mod application {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/pivot.application.v1.rs"));
    }
}
pub const DESCRIPTORS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/schema.bin"));
