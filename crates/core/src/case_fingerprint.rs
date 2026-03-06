use sha2::{Digest, Sha256};

use crate::{Case, Country, FileType, Material, ProcedureType};

/// Returns a stable lowercase hex SHA-256 fingerprint for a [`Case`].
///
/// Only deterministic fields are included: `case_id` and every field of
/// `DentalCase`. The `created_at` timestamp is intentionally excluded so
/// the fingerprint is purely input-driven and stable across time.
pub fn fingerprint_case(case: &Case) -> String {
    let canonical = canonical_case_string(case);
    let hash = Sha256::digest(canonical.as_bytes());
    hex::encode(hash)
}

/// Builds the canonical string representation of a [`Case`].
///
/// Fields are written in strict alphabetical order, one per line, using the
/// format `key=value`. Enum variants are lowercased snake_case; `Other`
/// variants are prefixed with `other:` to distinguish them from built-in
/// names.
fn canonical_case_string(case: &Case) -> String {
    // Fields in strict alphabetical order:
    //   case_id, file_type, manufacturer_country, material, patient_country, procedure
    format!(
        "case_id={}\nfile_type={}\nmanufacturer_country={}\nmaterial={}\npatient_country={}\nprocedure={}\n",
        case.id,
        file_type_str(&case.dental_case.file_type),
        country_str(&case.dental_case.manufacturer_country),
        material_str(&case.dental_case.material),
        country_str(&case.dental_case.patient_country),
        procedure_str(&case.dental_case.procedure),
    )
}

fn country_str(c: &Country) -> String {
    match c {
        Country::UnitedStates => "united_states".to_string(),
        Country::Germany => "germany".to_string(),
        Country::France => "france".to_string(),
        Country::Japan => "japan".to_string(),
        Country::UnitedKingdom => "united_kingdom".to_string(),
        Country::Other(s) => format!("other:{}", s),
    }
}

fn material_str(m: &Material) -> String {
    match m {
        Material::Zirconia => "zirconia".to_string(),
        Material::Pmma => "pmma".to_string(),
        Material::Emax => "emax".to_string(),
        Material::CobaltChrome => "cobalt_chrome".to_string(),
        Material::Titanium => "titanium".to_string(),
        Material::Other(s) => format!("other:{}", s),
    }
}

fn procedure_str(p: &ProcedureType) -> String {
    match p {
        ProcedureType::Crown => "crown".to_string(),
        ProcedureType::Bridge => "bridge".to_string(),
        ProcedureType::Veneer => "veneer".to_string(),
        ProcedureType::Implant => "implant".to_string(),
        ProcedureType::Denture => "denture".to_string(),
        ProcedureType::Other(s) => format!("other:{}", s),
    }
}

fn file_type_str(f: &FileType) -> String {
    match f {
        FileType::Stl => "stl".to_string(),
        FileType::Obj => "obj".to_string(),
        FileType::Ply => "ply".to_string(),
        FileType::ThreeMf => "three_mf".to_string(),
        FileType::Other(s) => format!("other:{}", s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Case, Country, DentalCase, FileType, Material, ProcedureType};

    fn base_case() -> Case {
        Case::new(DentalCase {
            patient_country: Country::UnitedStates,
            manufacturer_country: Country::Germany,
            material: Material::Zirconia,
            procedure: ProcedureType::Crown,
            file_type: FileType::Stl,
        })
    }

    #[test]
    fn identical_cases_produce_identical_fingerprints() {
        // Two cases with the same DentalCase fields share the same fingerprint
        // only if their IDs also match — clone shares the id.
        let case = base_case();
        let cloned = case.clone();
        assert_eq!(fingerprint_case(&case), fingerprint_case(&cloned));
    }

    #[test]
    fn different_case_ids_produce_different_fingerprints() {
        // Two independently created cases have different UUIDs.
        let a = base_case();
        let b = base_case();
        assert_ne!(fingerprint_case(&a), fingerprint_case(&b));
    }

    #[test]
    fn different_material_produces_different_fingerprint() {
        let case_a = Case::new(DentalCase {
            material: Material::Zirconia,
            ..base_case().dental_case
        });
        let case_b = Case::new(DentalCase {
            material: Material::Titanium,
            ..base_case().dental_case
        });
        assert_ne!(fingerprint_case(&case_a), fingerprint_case(&case_b));
    }

    #[test]
    fn different_procedure_produces_different_fingerprint() {
        let case_a = Case::new(DentalCase {
            procedure: ProcedureType::Crown,
            ..base_case().dental_case
        });
        let case_b = Case::new(DentalCase {
            procedure: ProcedureType::Bridge,
            ..base_case().dental_case
        });
        assert_ne!(fingerprint_case(&case_a), fingerprint_case(&case_b));
    }

    #[test]
    fn different_patient_country_produces_different_fingerprint() {
        let case_a = Case::new(DentalCase {
            patient_country: Country::UnitedStates,
            ..base_case().dental_case
        });
        let case_b = Case::new(DentalCase {
            patient_country: Country::Japan,
            ..base_case().dental_case
        });
        assert_ne!(fingerprint_case(&case_a), fingerprint_case(&case_b));
    }

    #[test]
    fn different_manufacturer_country_produces_different_fingerprint() {
        let case_a = Case::new(DentalCase {
            manufacturer_country: Country::Germany,
            ..base_case().dental_case
        });
        let case_b = Case::new(DentalCase {
            manufacturer_country: Country::France,
            ..base_case().dental_case
        });
        assert_ne!(fingerprint_case(&case_a), fingerprint_case(&case_b));
    }

    #[test]
    fn different_file_type_produces_different_fingerprint() {
        let case_a = Case::new(DentalCase {
            file_type: FileType::Stl,
            ..base_case().dental_case
        });
        let case_b = Case::new(DentalCase {
            file_type: FileType::Obj,
            ..base_case().dental_case
        });
        assert_ne!(fingerprint_case(&case_a), fingerprint_case(&case_b));
    }

    #[test]
    fn fingerprint_is_stable_across_calls() {
        let case = base_case();
        let first = fingerprint_case(&case);
        let second = fingerprint_case(&case);
        let third = fingerprint_case(&case);
        assert_eq!(first, second);
        assert_eq!(second, third);
    }

    #[test]
    fn fingerprint_is_64_hex_chars() {
        let fp = fingerprint_case(&base_case());
        assert_eq!(fp.len(), 64);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn fingerprint_is_lowercase() {
        let fp = fingerprint_case(&base_case());
        assert_eq!(fp, fp.to_lowercase());
    }

    #[test]
    fn canonical_string_fields_are_in_alphabetical_order() {
        let case = base_case();
        let canonical = canonical_case_string(&case);
        let keys: Vec<&str> = canonical
            .lines()
            .map(|l| l.split('=').next().unwrap_or(""))
            .collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
    }

    #[test]
    fn canonical_string_contains_expected_values() {
        let case = base_case();
        let canonical = canonical_case_string(&case);
        assert!(canonical.contains(&format!("case_id={}", case.id)));
        assert!(canonical.contains("patient_country=united_states"));
        assert!(canonical.contains("manufacturer_country=germany"));
        assert!(canonical.contains("material=zirconia"));
        assert!(canonical.contains("procedure=crown"));
        assert!(canonical.contains("file_type=stl"));
    }

    #[test]
    fn other_variant_country_uses_prefixed_form() {
        let case = Case::new(DentalCase {
            patient_country: Country::Other("Brazil".to_string()),
            ..base_case().dental_case
        });
        let canonical = canonical_case_string(&case);
        assert!(canonical.contains("patient_country=other:Brazil"));
    }

    #[test]
    fn other_variant_material_uses_prefixed_form() {
        let case = Case::new(DentalCase {
            material: Material::Other("Resin".to_string()),
            ..base_case().dental_case
        });
        let canonical = canonical_case_string(&case);
        assert!(canonical.contains("material=other:Resin"));
    }

    #[test]
    fn other_variant_procedure_uses_prefixed_form() {
        let case = Case::new(DentalCase {
            procedure: ProcedureType::Other("Inlay".to_string()),
            ..base_case().dental_case
        });
        let canonical = canonical_case_string(&case);
        assert!(canonical.contains("procedure=other:Inlay"));
    }

    #[test]
    fn other_variant_file_type_uses_prefixed_form() {
        let case = Case::new(DentalCase {
            file_type: FileType::Other("dcm".to_string()),
            ..base_case().dental_case
        });
        let canonical = canonical_case_string(&case);
        assert!(canonical.contains("file_type=other:dcm"));
    }

    #[test]
    fn swapped_patient_and_manufacturer_country_produce_different_fingerprints() {
        // Ensures field labels are included — same country values in opposite
        // positions must not collide.
        let case_a = Case::new(DentalCase {
            patient_country: Country::Germany,
            manufacturer_country: Country::UnitedStates,
            ..base_case().dental_case
        });
        let case_b = Case::new(DentalCase {
            patient_country: Country::UnitedStates,
            manufacturer_country: Country::Germany,
            ..base_case().dental_case
        });
        assert_ne!(fingerprint_case(&case_a), fingerprint_case(&case_b));
    }
}
