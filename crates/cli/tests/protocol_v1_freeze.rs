//! Protocol v1 freeze checkpoint.
//!
//! These tests lock the semver constants for PostCAD Protocol v1.
//! Any accidental change to these constants fails CI immediately.

use postcad_cli::{POSTCAD_PROTOCOL_VERSION, ROUTING_KERNEL_SEMVER};

/// Both semver constants must equal "1.0" — the frozen v1 values.
///
/// Changing either of these requires an intentional protocol version bump
/// and a corresponding update to this test.
#[test]
fn protocol_v1_constants_are_frozen() {
    assert_eq!(
        POSTCAD_PROTOCOL_VERSION, "1.0",
        "POSTCAD_PROTOCOL_VERSION must remain '1.0' for Protocol v1"
    );
    assert_eq!(
        ROUTING_KERNEL_SEMVER, "1.0",
        "ROUTING_KERNEL_SEMVER must remain '1.0' for Protocol v1"
    );
}
