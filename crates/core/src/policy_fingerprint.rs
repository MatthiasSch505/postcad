use sha2::{Digest, Sha256};

use crate::policy::{RoutingPolicy, RoutingPolicyConfig};

/// Returns a stable lowercase hex SHA-256 fingerprint for a `RoutingPolicyConfig`.
///
/// The fingerprint is derived from a deterministic canonical string that
/// serialises every field in a fixed alphabetical order with no timestamps,
/// random values, or serde-derived ordering.
pub fn fingerprint_policy(policy: &RoutingPolicyConfig) -> String {
    let canonical = canonical_policy_string(policy);
    let hash = Sha256::digest(canonical.as_bytes());
    hex::encode(hash)
}

/// Builds the canonical string representation of a `RoutingPolicyConfig`.
///
/// Fields are written in alphabetical order, one per line, using the format
/// `key=value`. Optional fields use an empty value when absent.
fn canonical_policy_string(policy: &RoutingPolicyConfig) -> String {
    let compliance_profile = policy
        .compliance_profile_name
        .as_deref()
        .unwrap_or("");

    let routing_policy = match policy.routing_policy {
        RoutingPolicy::AllowDomesticOnly => "allow_domestic_only",
        RoutingPolicy::AllowDomesticAndCrossBorder => "allow_domestic_and_cross_border",
    };

    // Fields in strict alphabetical order.
    format!(
        "compliance_profile={}\nrouting_policy={}\n",
        compliance_profile, routing_policy
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{RoutingPolicy, RoutingPolicyConfig};

    fn domestic_only() -> RoutingPolicyConfig {
        RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
    }

    fn domestic_only_with_profile(name: &str) -> RoutingPolicyConfig {
        RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
            .with_compliance_profile(name)
    }

    fn cross_border() -> RoutingPolicyConfig {
        RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticAndCrossBorder)
    }

    #[test]
    fn identical_policies_produce_identical_fingerprints() {
        let a = domestic_only();
        let b = domestic_only();
        assert_eq!(fingerprint_policy(&a), fingerprint_policy(&b));
    }

    #[test]
    fn different_routing_policies_produce_different_fingerprints() {
        let a = domestic_only();
        let b = cross_border();
        assert_ne!(fingerprint_policy(&a), fingerprint_policy(&b));
    }

    #[test]
    fn different_compliance_profiles_produce_different_fingerprints() {
        let a = domestic_only_with_profile("iso_only_v1");
        let b = domestic_only_with_profile("eu_mdr_v2");
        assert_ne!(fingerprint_policy(&a), fingerprint_policy(&b));
    }

    #[test]
    fn absent_profile_and_empty_string_profile_are_identical_in_canonical_form() {
        // Both map to compliance_profile= (empty value).
        let no_profile = domestic_only();
        let canonical_no = canonical_policy_string(&no_profile);
        assert!(canonical_no.contains("compliance_profile=\n"));
    }

    #[test]
    fn fingerprint_is_64_hex_chars() {
        let fp = fingerprint_policy(&domestic_only());
        assert_eq!(fp.len(), 64);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn fingerprint_is_lowercase() {
        let fp = fingerprint_policy(&domestic_only_with_profile("ISO_CERT_V1"));
        assert_eq!(fp, fp.to_lowercase());
    }

    #[test]
    fn fingerprint_is_stable_across_calls() {
        let policy = domestic_only_with_profile("lab_mdr");
        let first = fingerprint_policy(&policy);
        let second = fingerprint_policy(&policy);
        let third = fingerprint_policy(&policy);
        assert_eq!(first, second);
        assert_eq!(second, third);
    }

    #[test]
    fn canonical_string_fields_are_in_alphabetical_order() {
        let policy = domestic_only_with_profile("iso_only_v1");
        let canonical = canonical_policy_string(&policy);
        let lines: Vec<&str> = canonical.lines().collect();
        // compliance_profile comes before routing_policy alphabetically.
        assert!(lines[0].starts_with("compliance_profile="));
        assert!(lines[1].starts_with("routing_policy="));
    }

    #[test]
    fn canonical_string_contains_expected_values() {
        let policy = domestic_only_with_profile("iso_only_v1");
        let canonical = canonical_policy_string(&policy);
        assert!(canonical.contains("compliance_profile=iso_only_v1"));
        assert!(canonical.contains("routing_policy=allow_domestic_only"));
    }

    #[test]
    fn cross_border_policy_canonical_string_contains_correct_variant() {
        let policy = cross_border();
        let canonical = canonical_policy_string(&policy);
        assert!(canonical.contains("routing_policy=allow_domestic_and_cross_border"));
    }

    #[test]
    fn adding_compliance_profile_changes_fingerprint() {
        let without = domestic_only();
        let with_profile = domestic_only_with_profile("iso_only_v1");
        assert_ne!(fingerprint_policy(&without), fingerprint_policy(&with_profile));
    }
}
