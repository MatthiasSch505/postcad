use postcad_registry::snapshot::ManufacturerComplianceSnapshot;

pub struct ComplianceGate;

impl ComplianceGate {
    /// Filters `manufacturer_ids` to those with an eligible compliance snapshot.
    ///
    /// - Manufacturers whose snapshot has `is_eligible == true` are kept.
    /// - Manufacturers with no snapshot are treated as not eligible and removed.
    /// - Input ordering is preserved deterministically.
    pub fn filter_compliant_manufacturers(
        manufacturer_ids: &[String],
        snapshots: &[ManufacturerComplianceSnapshot],
    ) -> Vec<String> {
        manufacturer_ids
            .iter()
            .filter(|id| {
                snapshots
                    .iter()
                    .any(|s| &s.manufacturer_id == *id && s.is_eligible)
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcad_registry::snapshot::ManufacturerComplianceSnapshot;

    fn eligible(id: &str) -> ManufacturerComplianceSnapshot {
        ManufacturerComplianceSnapshot::new(id, vec!["REF-001".to_string()], vec!["verified".to_string()], true)
    }

    fn ineligible(id: &str) -> ManufacturerComplianceSnapshot {
        ManufacturerComplianceSnapshot::new(id, vec!["REF-001".to_string()], vec!["rejected".to_string()], false)
    }

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn eligible_manufacturer_passes() {
        let snapshots = vec![eligible("mfr-01")];
        let result = ComplianceGate::filter_compliant_manufacturers(&ids(&["mfr-01"]), &snapshots);
        assert_eq!(result, ids(&["mfr-01"]));
    }

    #[test]
    fn ineligible_manufacturer_is_filtered_out() {
        let snapshots = vec![ineligible("mfr-01")];
        let result = ComplianceGate::filter_compliant_manufacturers(&ids(&["mfr-01"]), &snapshots);
        assert!(result.is_empty());
    }

    #[test]
    fn manufacturer_without_snapshot_is_filtered_out() {
        let result =
            ComplianceGate::filter_compliant_manufacturers(&ids(&["mfr-99"]), &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn multiple_manufacturers_preserve_deterministic_ordering() {
        let snapshots = vec![eligible("mfr-01"), eligible("mfr-02"), eligible("mfr-03")];
        let result = ComplianceGate::filter_compliant_manufacturers(
            &ids(&["mfr-03", "mfr-01", "mfr-02"]),
            &snapshots,
        );
        assert_eq!(result, ids(&["mfr-03", "mfr-01", "mfr-02"]));
    }

    #[test]
    fn mixed_eligibility_keeps_only_eligible() {
        let snapshots = vec![eligible("mfr-01"), ineligible("mfr-02"), eligible("mfr-03")];
        let result = ComplianceGate::filter_compliant_manufacturers(
            &ids(&["mfr-01", "mfr-02", "mfr-03"]),
            &snapshots,
        );
        assert_eq!(result, ids(&["mfr-01", "mfr-03"]));
    }
}
