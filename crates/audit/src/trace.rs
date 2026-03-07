use postcad_core::{RoutingCandidate, RoutingDecision, RoutingOutcome};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecisionTrace {
    pub case_id: String,
    pub jurisdiction: String,
    pub evaluated_candidate_ids: Vec<String>,
    pub eligible_candidate_ids: Vec<String>,
    pub rejected_candidate_ids: Vec<String>,
    pub selected_manufacturer_id: Option<String>,
    pub final_status: String,
}

impl DecisionTrace {
    /// Build a trace from a routing outcome.
    ///
    /// - `candidates` — the full slice considered before policy filtering
    /// - `filtered_candidates` — the slice that survived policy filtering;
    ///   pass an empty slice if this information is not available
    pub fn from_outcome(
        outcome: &RoutingOutcome,
        jurisdiction: impl Into<String>,
        candidates: &[RoutingCandidate],
        filtered_candidates: &[RoutingCandidate],
    ) -> Self {
        let case_id = outcome.context.case_id.to_string();
        let jurisdiction = jurisdiction.into();

        let evaluated_candidate_ids: Vec<String> =
            candidates.iter().map(|c| c.id.to_string()).collect();

        let eligible_candidate_ids: Vec<String> =
            filtered_candidates.iter().map(|c| c.id.to_string()).collect();

        let eligible_set: std::collections::HashSet<&str> =
            eligible_candidate_ids.iter().map(String::as_str).collect();

        let rejected_candidate_ids: Vec<String> = evaluated_candidate_ids
            .iter()
            .filter(|id| !eligible_set.contains(id.as_str()))
            .cloned()
            .collect();

        let (selected_manufacturer_id, final_status) = match &outcome.decision {
            RoutingDecision::Selected(candidate_id) => {
                let manufacturer_id = candidates
                    .iter()
                    .find(|c| &c.id == candidate_id)
                    .map(|c| c.manufacturer_id.0.clone());
                (manufacturer_id, "selected".to_string())
            }
            RoutingDecision::Refused(_) | RoutingDecision::NoEligibleCandidate => {
                (None, "refused".to_string())
            }
        };

        Self {
            case_id,
            jurisdiction,
            evaluated_candidate_ids,
            eligible_candidate_ids,
            rejected_candidate_ids,
            selected_manufacturer_id,
            final_status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcad_core::{
        Case, Country, DecisionContext, DentalCase, FileType, ManufacturerEligibility,
        ManufacturingLocation, Material, ProcedureType, RoutingCandidate, RoutingCandidateId,
        RoutingDecision, RoutingOutcome,
    };

    fn make_case() -> Case {
        Case::new(DentalCase {
            patient_country: Country::UnitedStates,
            manufacturer_country: Country::Germany,
            material: Material::Zirconia,
            procedure: ProcedureType::Crown,
            file_type: FileType::Stl,
        })
    }

    fn make_candidate(rc_id: &str, mfr_id: &str) -> RoutingCandidate {
        RoutingCandidate::new(
            RoutingCandidateId::new(rc_id),
            mfr_id,
            ManufacturingLocation::Domestic,
            true,
            ManufacturerEligibility::Eligible,
        )
    }

    fn make_outcome(case: &Case, decision: RoutingDecision, original: usize, filtered: usize) -> RoutingOutcome {
        RoutingOutcome {
            decision,
            context: DecisionContext::new(case.id.clone(), original, filtered),
        }
    }

    #[test]
    fn successful_outcome_produces_selected_status() {
        let case = make_case();
        let candidates = vec![
            make_candidate("rc-1", "mfr-01"),
            make_candidate("rc-2", "mfr-02"),
        ];
        let filtered = vec![make_candidate("rc-1", "mfr-01")];
        let outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1")),
            candidates.len(),
            filtered.len(),
        );

        let trace = DecisionTrace::from_outcome(&outcome, "DE", &candidates, &filtered);

        assert_eq!(trace.final_status, "selected");
        assert_eq!(trace.selected_manufacturer_id, Some("mfr-01".to_string()));
        assert_eq!(trace.case_id, case.id.to_string());
        assert_eq!(trace.jurisdiction, "DE");
    }

    #[test]
    fn refusal_outcome_produces_refused_status() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-01")];
        let refusal =
            postcad_core::CaseRefusal::with_reason(case.id.clone(), postcad_core::RefusalReason::ValidationFailed);
        let outcome = make_outcome(
            &case,
            RoutingDecision::Refused(refusal),
            candidates.len(),
            0,
        );

        let trace = DecisionTrace::from_outcome(&outcome, "US", &candidates, &[]);

        assert_eq!(trace.final_status, "refused");
        assert!(trace.selected_manufacturer_id.is_none());
    }

    #[test]
    fn no_eligible_candidate_produces_refused_status() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-01")];
        let outcome = make_outcome(&case, RoutingDecision::NoEligibleCandidate, 1, 0);

        let trace = DecisionTrace::from_outcome(&outcome, "JP", &candidates, &[]);

        assert_eq!(trace.final_status, "refused");
        assert!(trace.selected_manufacturer_id.is_none());
    }

    #[test]
    fn candidate_ordering_is_preserved() {
        let case = make_case();
        let candidates = vec![
            make_candidate("rc-a", "mfr-1"),
            make_candidate("rc-b", "mfr-2"),
            make_candidate("rc-c", "mfr-3"),
        ];
        let filtered = vec![make_candidate("rc-a", "mfr-1")];
        let outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-a")),
            3,
            1,
        );

        let trace = DecisionTrace::from_outcome(&outcome, "DE", &candidates, &filtered);

        assert_eq!(trace.evaluated_candidate_ids, vec!["rc-a", "rc-b", "rc-c"]);
        assert_eq!(trace.eligible_candidate_ids, vec!["rc-a"]);
        assert_eq!(trace.rejected_candidate_ids, vec!["rc-b", "rc-c"]);
    }

    // ── canonical JSON ────────────────────────────────────────────────────────

    #[test]
    fn trace_canonical_json_is_identical_for_same_input() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let filtered = vec![make_candidate("rc-1", "mfr-de-01")];
        let outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1")),
            candidates.len(),
            filtered.len(),
        );
        let trace = DecisionTrace::from_outcome(&outcome, "DE", &candidates, &filtered);

        let json_a = crate::canonical::to_canonical_json(&trace);
        let json_b = crate::canonical::to_canonical_json(&trace);
        assert_eq!(json_a, json_b);
    }

    #[test]
    fn trace_canonical_json_is_compact() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let filtered = vec![make_candidate("rc-1", "mfr-de-01")];
        let outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1")),
            candidates.len(),
            filtered.len(),
        );
        let trace = DecisionTrace::from_outcome(&outcome, "DE", &candidates, &filtered);
        let json = crate::canonical::to_canonical_json(&trace);

        assert!(!json.ends_with('\n'));
        assert!(!json.contains("\n  "));
    }

    #[test]
    fn trace_canonical_json_contains_case_id_and_jurisdiction() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1")),
            candidates.len(),
            candidates.len(),
        );
        let trace = DecisionTrace::from_outcome(&outcome, "DE", &candidates, &candidates);
        let json = crate::canonical::to_canonical_json(&trace);

        assert!(json.contains(&case.id.to_string()));
        assert!(json.contains("\"DE\""));
    }

    #[test]
    fn trace_canonical_json_refused_uses_refused_status() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-01")];
        let outcome = make_outcome(&case, RoutingDecision::NoEligibleCandidate, 1, 0);
        let trace = DecisionTrace::from_outcome(&outcome, "DE", &candidates, &[]);
        let json = crate::canonical::to_canonical_json(&trace);

        assert!(json.contains("\"refused\""));
    }

    #[test]
    fn different_traces_produce_different_canonical_json() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-01")];
        let filtered = vec![make_candidate("rc-1", "mfr-01")];

        let selected_outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1")),
            1,
            1,
        );
        let refused_outcome = make_outcome(&case, RoutingDecision::NoEligibleCandidate, 1, 0);

        let trace_a = DecisionTrace::from_outcome(&selected_outcome, "DE", &candidates, &filtered);
        let trace_b = DecisionTrace::from_outcome(&refused_outcome, "JP", &candidates, &[]);

        assert_ne!(
            crate::canonical::to_canonical_json(&trace_a),
            crate::canonical::to_canonical_json(&trace_b)
        );
    }

    #[test]
    fn missing_eligibility_detail_falls_back_to_empty() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-01")];
        let outcome = make_outcome(&case, RoutingDecision::NoEligibleCandidate, 1, 0);

        // pass empty filtered slice — safe fallback
        let trace = DecisionTrace::from_outcome(&outcome, "DE", &candidates, &[]);

        assert_eq!(trace.eligible_candidate_ids, Vec::<String>::new());
        assert_eq!(trace.rejected_candidate_ids, vec!["rc-1"]);
        assert_eq!(trace.evaluated_candidate_ids, vec!["rc-1"]);
    }
}
