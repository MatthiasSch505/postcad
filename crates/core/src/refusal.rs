use serde::Serialize;

use crate::CaseId;

/// Machine-readable explanation attached to every refused routing outcome.
///
/// All fields are stable strings — no prose, no timestamps.
/// `failed_constraint` identifies which pipeline stage blocked routing:
/// - `"case_validation"` — the case itself was invalid
/// - `"compliance_gate"` — all candidates were removed by compliance pre-filtering
/// - `"routing_policy"` — all compliant candidates were removed by the routing policy filter
/// - `"no_input_candidates"` — the candidate list was empty before any filtering
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RefusalExplanation {
    pub refusal_code: String,
    pub evaluated_candidate_ids: Vec<String>,
    pub failed_constraint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefusalReason {
    ValidationFailed,
    UnsupportedFileType,
    MissingManufacturingMetadata,
    UnsupportedJurisdiction,
    ManufacturerNotEligible,
    NoEligibleCandidate,
    ComplianceExclusion,
    Unknown,
}

impl RefusalReason {
    /// Stable machine-readable code for this refusal reason.
    ///
    /// These strings are part of the public contract and must not change once
    /// published. Add new variants rather than renaming existing codes.
    pub fn code(&self) -> &'static str {
        match self {
            RefusalReason::ValidationFailed => "invalid_input",
            RefusalReason::UnsupportedFileType => "unsupported_case",
            RefusalReason::MissingManufacturingMetadata => "invalid_input",
            RefusalReason::UnsupportedJurisdiction => "unsupported_case",
            RefusalReason::ManufacturerNotEligible => "compliance_failed",
            RefusalReason::NoEligibleCandidate => "no_eligible_candidates",
            RefusalReason::ComplianceExclusion => "compliance_failed",
            RefusalReason::Unknown => "unknown",
        }
    }

    /// Human-readable description of this refusal reason.
    pub fn message(&self) -> &'static str {
        match self {
            RefusalReason::ValidationFailed => "Case failed validation",
            RefusalReason::UnsupportedFileType => "File type is not supported",
            RefusalReason::MissingManufacturingMetadata => {
                "Required manufacturing metadata is missing"
            }
            RefusalReason::UnsupportedJurisdiction => "Jurisdiction is not supported",
            RefusalReason::ManufacturerNotEligible => "Manufacturer is not eligible",
            RefusalReason::NoEligibleCandidate => "No eligible candidate found",
            RefusalReason::ComplianceExclusion => "Compliance requirements not met",
            RefusalReason::Unknown => "Unknown refusal reason",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseRefusal {
    pub case_id: CaseId,
    pub reasons: Vec<RefusalReason>,
}

impl CaseRefusal {
    pub fn new(case_id: CaseId) -> Self {
        Self {
            case_id,
            reasons: Vec::new(),
        }
    }

    pub fn add_reason(&mut self, reason: RefusalReason) {
        self.reasons.push(reason);
    }

    pub fn is_empty(&self) -> bool {
        self.reasons.is_empty()
    }

    pub fn with_reason(case_id: CaseId, reason: RefusalReason) -> Self {
        Self {
            case_id,
            reasons: vec![reason],
        }
    }

    pub fn with_reasons(case_id: CaseId, reasons: Vec<RefusalReason>) -> Self {
        Self { case_id, reasons }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CaseId;

    #[test]
    fn new_refusal_is_empty() {
        let r = CaseRefusal::new(CaseId::new());
        assert!(r.is_empty());
        assert!(r.reasons.is_empty());
    }

    #[test]
    fn add_reason_is_no_longer_empty() {
        let mut r = CaseRefusal::new(CaseId::new());
        r.add_reason(RefusalReason::ValidationFailed);
        assert!(!r.is_empty());
    }

    #[test]
    fn add_multiple_reasons() {
        let mut r = CaseRefusal::new(CaseId::new());
        r.add_reason(RefusalReason::UnsupportedFileType);
        r.add_reason(RefusalReason::MissingManufacturingMetadata);
        r.add_reason(RefusalReason::UnsupportedJurisdiction);
        assert_eq!(r.reasons.len(), 3);
        assert!(r.reasons.contains(&RefusalReason::UnsupportedFileType));
        assert!(r
            .reasons
            .contains(&RefusalReason::MissingManufacturingMetadata));
        assert!(r.reasons.contains(&RefusalReason::UnsupportedJurisdiction));
    }

    #[test]
    fn case_id_is_preserved() {
        let id = CaseId::new();
        let r = CaseRefusal::new(id.clone());
        assert_eq!(r.case_id, id);
    }

    #[test]
    fn with_reason_creates_one_reason() {
        let id = CaseId::new();
        let r = CaseRefusal::with_reason(id.clone(), RefusalReason::ValidationFailed);
        assert_eq!(r.case_id, id);
        assert_eq!(r.reasons.len(), 1);
        assert!(r.reasons.contains(&RefusalReason::ValidationFailed));
    }

    #[test]
    fn with_reasons_preserves_order() {
        let id = CaseId::new();
        let reasons = vec![
            RefusalReason::UnsupportedFileType,
            RefusalReason::MissingManufacturingMetadata,
            RefusalReason::NoEligibleCandidate,
        ];
        let r = CaseRefusal::with_reasons(id.clone(), reasons);
        assert_eq!(r.case_id, id);
        assert_eq!(r.reasons[0], RefusalReason::UnsupportedFileType);
        assert_eq!(r.reasons[1], RefusalReason::MissingManufacturingMetadata);
        assert_eq!(r.reasons[2], RefusalReason::NoEligibleCandidate);
    }

    #[test]
    fn with_reasons_empty_vec_is_allowed() {
        let r = CaseRefusal::with_reasons(CaseId::new(), vec![]);
        assert!(r.is_empty());
    }

    #[test]
    fn no_eligible_candidate_reason() {
        let mut r = CaseRefusal::new(CaseId::new());
        r.add_reason(RefusalReason::NoEligibleCandidate);
        assert!(r.reasons.contains(&RefusalReason::NoEligibleCandidate));
    }

    #[test]
    fn compliance_exclusion_reason() {
        let mut r = CaseRefusal::new(CaseId::new());
        r.add_reason(RefusalReason::ComplianceExclusion);
        assert!(r.reasons.contains(&RefusalReason::ComplianceExclusion));
    }

    // ── code() and message() ─────────────────────────────────────────────────

    #[test]
    fn all_variants_have_stable_codes() {
        assert_eq!(RefusalReason::ValidationFailed.code(), "invalid_input");
        assert_eq!(
            RefusalReason::UnsupportedFileType.code(),
            "unsupported_case"
        );
        assert_eq!(
            RefusalReason::MissingManufacturingMetadata.code(),
            "invalid_input"
        );
        assert_eq!(
            RefusalReason::UnsupportedJurisdiction.code(),
            "unsupported_case"
        );
        assert_eq!(
            RefusalReason::ManufacturerNotEligible.code(),
            "compliance_failed"
        );
        assert_eq!(
            RefusalReason::NoEligibleCandidate.code(),
            "no_eligible_candidates"
        );
        assert_eq!(
            RefusalReason::ComplianceExclusion.code(),
            "compliance_failed"
        );
        assert_eq!(RefusalReason::Unknown.code(), "unknown");
    }

    #[test]
    fn code_is_stable_across_calls() {
        let r = RefusalReason::ComplianceExclusion;
        assert_eq!(r.code(), r.code());
    }

    #[test]
    fn all_variants_have_non_empty_messages() {
        let variants = [
            RefusalReason::ValidationFailed,
            RefusalReason::UnsupportedFileType,
            RefusalReason::MissingManufacturingMetadata,
            RefusalReason::UnsupportedJurisdiction,
            RefusalReason::ManufacturerNotEligible,
            RefusalReason::NoEligibleCandidate,
            RefusalReason::ComplianceExclusion,
            RefusalReason::Unknown,
        ];
        for v in &variants {
            assert!(!v.message().is_empty(), "empty message for {:?}", v);
        }
    }

    #[test]
    fn all_reason_variants_are_usable() {
        let mut r = CaseRefusal::new(CaseId::new());
        r.add_reason(RefusalReason::ValidationFailed);
        r.add_reason(RefusalReason::UnsupportedFileType);
        r.add_reason(RefusalReason::MissingManufacturingMetadata);
        r.add_reason(RefusalReason::UnsupportedJurisdiction);
        r.add_reason(RefusalReason::ManufacturerNotEligible);
        r.add_reason(RefusalReason::NoEligibleCandidate);
        r.add_reason(RefusalReason::ComplianceExclusion);
        r.add_reason(RefusalReason::Unknown);
        assert_eq!(r.reasons.len(), 8);
    }
}
