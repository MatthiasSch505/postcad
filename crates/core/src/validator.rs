use crate::{Case, Country, FileType, Material, ProcedureType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// FileType::Other with an empty or blank name — no usable file type declared.
    UnsupportedFileType(String),
    /// An Other-variant field contains an empty string, making it meaningless.
    EmptyOtherField(&'static str),
}

/// Checks that a [`Case`] is structurally complete for downstream routing.
///
/// Returns `Ok(())` if all checks pass, or `Err(errors)` with every error
/// found (all checks always run so callers see the full picture).
pub fn validate_case(case: &Case) -> Result<(), Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();
    let dc = &case.dental_case;

    // File type must be a known variant or a non-empty Other.
    if let FileType::Other(ref s) = dc.file_type {
        if s.trim().is_empty() {
            errors.push(ValidationError::UnsupportedFileType(s.clone()));
        }
    }

    // Material::Other must carry a non-empty name.
    if let Material::Other(ref s) = dc.material {
        if s.trim().is_empty() {
            errors.push(ValidationError::EmptyOtherField("material"));
        }
    }

    // ProcedureType::Other must carry a non-empty name.
    if let ProcedureType::Other(ref s) = dc.procedure {
        if s.trim().is_empty() {
            errors.push(ValidationError::EmptyOtherField("procedure"));
        }
    }

    // Country::Other fields must carry a non-empty name.
    if let Country::Other(ref s) = dc.patient_country {
        if s.trim().is_empty() {
            errors.push(ValidationError::EmptyOtherField("patient_country"));
        }
    }
    if let Country::Other(ref s) = dc.manufacturer_country {
        if s.trim().is_empty() {
            errors.push(ValidationError::EmptyOtherField("manufacturer_country"));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Case, DentalCase};

    fn valid_case() -> Case {
        Case::new(DentalCase {
            patient_country: Country::UnitedStates,
            manufacturer_country: Country::Germany,
            material: Material::Zirconia,
            procedure: ProcedureType::Crown,
            file_type: FileType::Stl,
        })
    }

    #[test]
    fn valid_case_passes() {
        assert!(validate_case(&valid_case()).is_ok());
    }

    #[test]
    fn other_known_file_type_passes() {
        let mut case = valid_case();
        case.dental_case.file_type = FileType::Other("iges".to_string());
        assert!(validate_case(&case).is_ok());
    }

    #[test]
    fn empty_other_file_type_fails() {
        let mut case = valid_case();
        case.dental_case.file_type = FileType::Other(String::new());
        let errs = validate_case(&case).unwrap_err();
        assert_eq!(errs, vec![ValidationError::UnsupportedFileType(String::new())]);
    }

    #[test]
    fn whitespace_only_file_type_fails() {
        let mut case = valid_case();
        case.dental_case.file_type = FileType::Other("   ".to_string());
        let errs = validate_case(&case).unwrap_err();
        assert!(errs.iter().any(|e| matches!(e, ValidationError::UnsupportedFileType(_))));
    }

    #[test]
    fn empty_other_material_fails() {
        let mut case = valid_case();
        case.dental_case.material = Material::Other(String::new());
        let errs = validate_case(&case).unwrap_err();
        assert_eq!(errs, vec![ValidationError::EmptyOtherField("material")]);
    }

    #[test]
    fn empty_other_procedure_fails() {
        let mut case = valid_case();
        case.dental_case.procedure = ProcedureType::Other(String::new());
        let errs = validate_case(&case).unwrap_err();
        assert_eq!(errs, vec![ValidationError::EmptyOtherField("procedure")]);
    }

    #[test]
    fn empty_other_patient_country_fails() {
        let mut case = valid_case();
        case.dental_case.patient_country = Country::Other(String::new());
        let errs = validate_case(&case).unwrap_err();
        assert_eq!(errs, vec![ValidationError::EmptyOtherField("patient_country")]);
    }

    #[test]
    fn multiple_errors_collected_together() {
        let mut case = valid_case();
        case.dental_case.file_type = FileType::Other(String::new());
        case.dental_case.material = Material::Other(String::new());
        case.dental_case.procedure = ProcedureType::Other(String::new());
        let errs = validate_case(&case).unwrap_err();
        assert_eq!(errs.len(), 3);
        assert!(errs.iter().any(|e| matches!(e, ValidationError::UnsupportedFileType(_))));
        assert!(errs.iter().any(|e| e == &ValidationError::EmptyOtherField("material")));
        assert!(errs.iter().any(|e| e == &ValidationError::EmptyOtherField("procedure")));
    }

    #[test]
    fn all_other_fields_empty_collects_all_errors() {
        let case = Case::new(DentalCase {
            patient_country: Country::Other(String::new()),
            manufacturer_country: Country::Other(String::new()),
            material: Material::Other(String::new()),
            procedure: ProcedureType::Other(String::new()),
            file_type: FileType::Other(String::new()),
        });
        let errs = validate_case(&case).unwrap_err();
        assert_eq!(errs.len(), 5);
    }
}
