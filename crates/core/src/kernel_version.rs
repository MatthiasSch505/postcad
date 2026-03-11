//! Routing kernel version constants for PostCAD.
//!
//! `ROUTING_KERNEL_SEMVER` is the semantic versioning identifier for the
//! routing selection algorithm. It is independent of the label string
//! `"postcad-routing-v1"` used inside receipt artifacts and proof objects —
//! that label is defined in `crates/routing` and must not change without a
//! full receipt schema migration.
//!
//! Bump `ROUTING_KERNEL_SEMVER` when the kernel algorithm changes in a way
//! that produces different routing outcomes for the same inputs.

/// Semantic version of the PostCAD routing kernel algorithm.
pub const ROUTING_KERNEL_SEMVER: &str = "1.0";
