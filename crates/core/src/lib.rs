use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod validator;
pub use validator::{validate_case, ValidationError};

pub mod refusal;
pub use refusal::{CaseRefusal, RefusalReason};

pub mod routing;
pub use routing::{ManufacturerEligibility, ManufacturerId, ManufacturingLocation, RoutingCandidate, RoutingCandidateId};

pub mod decision;
pub use decision::{DecisionContext, RoutingDecision};

pub mod policy;
pub use policy::{filter_candidates, RoutingPolicy};

pub mod selection;
pub use selection::select_candidate;

pub mod service;
pub use service::{no_candidate_refusal, route_case, route_case_with_context, RoutingOutcome};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Country {
    UnitedStates,
    Germany,
    France,
    Japan,
    UnitedKingdom,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Material {
    Zirconia,
    Pmma,
    Emax,
    CobaltChrome,
    Titanium,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcedureType {
    Crown,
    Bridge,
    Veneer,
    Implant,
    Denture,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    Stl,
    Obj,
    Ply,
    ThreeMf,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DentalCase {
    pub patient_country: Country,
    pub manufacturer_country: Country,
    pub material: Material,
    pub procedure: ProcedureType,
    pub file_type: FileType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CaseId(pub Uuid);

impl CaseId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CaseId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Case {
    pub id: CaseId,
    pub dental_case: DentalCase,
    pub created_at: DateTime<Utc>,
}

impl Case {
    pub fn new(dental_case: DentalCase) -> Self {
        Self {
            id: CaseId::new(),
            dental_case,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_dental_case() -> DentalCase {
        DentalCase {
            patient_country: Country::UnitedStates,
            manufacturer_country: Country::Germany,
            material: Material::Zirconia,
            procedure: ProcedureType::Crown,
            file_type: FileType::Stl,
        }
    }

    #[test]
    fn dental_case_fields_are_accessible() {
        let dc = sample_dental_case();
        assert_eq!(dc.patient_country, Country::UnitedStates);
        assert_eq!(dc.manufacturer_country, Country::Germany);
        assert_eq!(dc.material, Material::Zirconia);
        assert_eq!(dc.procedure, ProcedureType::Crown);
        assert_eq!(dc.file_type, FileType::Stl);
    }

    #[test]
    fn dental_case_clone_is_equal() {
        let dc = DentalCase {
            patient_country: Country::Japan,
            manufacturer_country: Country::UnitedStates,
            material: Material::Titanium,
            procedure: ProcedureType::Implant,
            file_type: FileType::ThreeMf,
        };
        assert_eq!(dc.clone(), dc);
    }

    #[test]
    fn country_other_variant() {
        let c = Country::Other("Brazil".to_string());
        assert_eq!(c, Country::Other("Brazil".to_string()));
        assert_ne!(c, Country::Other("Argentina".to_string()));
    }

    #[test]
    fn material_other_variant() {
        let m = Material::Other("Resin".to_string());
        assert_eq!(m, Material::Other("Resin".to_string()));
    }

    #[test]
    fn procedure_other_variant() {
        let p = ProcedureType::Other("Inlay".to_string());
        assert_eq!(p, ProcedureType::Other("Inlay".to_string()));
    }

    #[test]
    fn file_type_other_variant() {
        let f = FileType::Other("dcm".to_string());
        assert_eq!(f, FileType::Other("dcm".to_string()));
        assert_ne!(f, FileType::Other("iges".to_string()));
    }

    #[test]
    fn case_id_is_unique() {
        let a = CaseId::new();
        let b = CaseId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn case_id_display() {
        let id = CaseId::new();
        let s = id.to_string();
        // UUID v4 canonical form: 8-4-4-4-12 hex chars with dashes
        assert_eq!(s.len(), 36);
        assert!(s.contains('-'));
    }

    #[test]
    fn case_new_sets_fields() {
        let dc = sample_dental_case();
        let case = Case::new(dc.clone());
        assert_eq!(case.dental_case, dc);
        // created_at should be recent (within the last minute)
        let age = Utc::now() - case.created_at;
        assert!(age.num_seconds() < 60);
    }

    #[test]
    fn case_clone_shares_id() {
        let case = Case::new(sample_dental_case());
        let cloned = case.clone();
        assert_eq!(case.id, cloned.id);
        assert_eq!(case.created_at, cloned.created_at);
    }
}
