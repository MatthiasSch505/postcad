/// Stable identifier for the routing kernel algorithm used to produce receipts.
///
/// Committed into every [`RoutingReceipt`] and validated during `verify-receipt`.
/// Increment this constant when the routing algorithm changes in a way that
/// would produce a different selected candidate for the same inputs, so that
/// receipts produced by the old kernel cannot be verified against the new one.
pub const ROUTING_KERNEL_VERSION: &str = "postcad-routing-v1";

pub fn ping() -> bool {
    true
}
