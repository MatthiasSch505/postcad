//! Pilot case input normalization.
//!
//! Defines a minimal [`PilotCaseInput`] shape with only the fields a lab
//! operator needs to supply, and provides [`normalize_pilot_case_json`] to
//! convert it into a [`CaseInput`]-compatible JSON string for use with
//! [`route_case_from_registry_json`].
//!
//! This is a pure input adapter — it does not touch the routing kernel,
//! receipt schema, dispatch schema, or audit logic.
//!
//! # Mapping
//!
//! | Pilot field        | CaseInput field          | Notes                              |
//! |--------------------|-------------------------|------------------------------------|
//! | `case_id`          | `case_id`               | Optional; kernel generates one if absent |
//! | `restoration_type` | `procedure`             | Trimmed and lowercased             |
//! | `material`         | `material`              | Trimmed and lowercased             |
//! | `jurisdiction`     | `jurisdiction`          | ISO-like code: `DE`, `US`, `JP`, `FR`, `GB` |
//! | *(derived)*        | `patient_country`       | Derived from `jurisdiction`        |
//! | *(derived)*        | `manufacturer_country`  | Derived from `jurisdiction`        |
//! | *(default)*        | `routing_policy`        | `"allow_domestic_and_cross_border"` |
//! | *(default)*        | `file_type`             | `"stl"`                            |

use serde::Deserialize;

use crate::CliError;

// ── Public types ──────────────────────────────────────────────────────────────

/// Minimal pilot case input for lab operators.
///
/// Only four fields are required. All routing internals are derived or
/// defaulted. The struct is intentionally small — adding fields here must
/// be justified by an operator need, not a kernel convenience.
#[derive(Debug, Deserialize)]
pub struct PilotCaseInput {
    /// Stable case identifier. Optional; a fresh UUID is generated if absent.
    pub case_id: Option<String>,
    /// Restoration type requested: `crown`, `bridge`, `veneer`, `implant`, `denture`.
    /// Case-insensitive; whitespace is stripped.
    pub restoration_type: String,
    /// Material requested: `zirconia`, `pmma`, `emax`, `cobalt_chrome`, `titanium`.
    /// Case-insensitive; whitespace is stripped.
    pub material: String,
    /// Jurisdiction code: `DE`, `US`, `JP`, `FR`, `GB`.
    /// Determines `patient_country`, `manufacturer_country`, and compliance rules applied.
    pub jurisdiction: String,
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Parses a [`PilotCaseInput`] JSON string and converts it to a
/// [`CaseInput`]-compatible JSON string.
///
/// The returned string can be passed directly to [`route_case_from_registry_json`]
/// as the `case_json` argument.
///
/// # Errors
///
/// - [`CliError::ParseError`] — `pilot_json` is not valid JSON or is missing
///   a required field (`restoration_type`, `material`, `jurisdiction`).
/// - [`CliError::InvalidField`] — `jurisdiction` is not a recognized code.
pub fn normalize_pilot_case_json(pilot_json: &str) -> Result<String, CliError> {
    let input: PilotCaseInput = serde_json::from_str(pilot_json)?;

    let procedure = input.restoration_type.trim().to_lowercase();
    let material = input.material.trim().to_lowercase();
    let jurisdiction = input.jurisdiction.trim().to_uppercase();
    let country = jurisdiction_to_country(&jurisdiction)?;

    let case = serde_json::json!({
        "case_id":              input.case_id,
        "jurisdiction":         jurisdiction,
        "routing_policy":       "allow_domestic_and_cross_border",
        "patient_country":      country,
        "manufacturer_country": country,
        "material":             material,
        "procedure":            procedure,
        "file_type":            "stl"
    });

    Ok(serde_json::to_string(&case)?)
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Maps a jurisdiction code to the country string expected by the routing kernel.
///
/// Recognized codes: `DE`, `US`, `JP`, `FR`, `GB`.
fn jurisdiction_to_country(code: &str) -> Result<&'static str, CliError> {
    match code {
        "DE" => Ok("germany"),
        "US" => Ok("united_states"),
        "JP" => Ok("japan"),
        "FR" => Ok("france"),
        "GB" => Ok("united_kingdom"),
        other => Err(CliError::InvalidField(format!(
            "unknown jurisdiction code: {other:?}; \
             recognized codes are DE, US, JP, FR, GB"
        ))),
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_INPUT: &str = r#"{
        "case_id": "f1000001-0000-0000-0000-000000000001",
        "restoration_type": "crown",
        "material": "zirconia",
        "jurisdiction": "DE"
    }"#;

    #[test]
    fn valid_input_produces_case_json() {
        let json = normalize_pilot_case_json(VALID_INPUT).expect("must normalize");
        let v: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");
        assert_eq!(v["procedure"], "crown");
        assert_eq!(v["material"], "zirconia");
        assert_eq!(v["jurisdiction"], "DE");
        assert_eq!(v["patient_country"], "germany");
        assert_eq!(v["manufacturer_country"], "germany");
        assert_eq!(v["routing_policy"], "allow_domestic_and_cross_border");
        assert_eq!(v["file_type"], "stl");
        assert_eq!(
            v["case_id"],
            "f1000001-0000-0000-0000-000000000001"
        );
    }

    #[test]
    fn restoration_type_is_lowercased_and_trimmed() {
        let json = normalize_pilot_case_json(
            r#"{"restoration_type": "  Crown  ", "material": "zirconia", "jurisdiction": "DE"}"#,
        )
        .expect("must normalize");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["procedure"], "crown");
    }

    #[test]
    fn material_is_lowercased_and_trimmed() {
        let json = normalize_pilot_case_json(
            r#"{"restoration_type": "crown", "material": "  Zirconia  ", "jurisdiction": "DE"}"#,
        )
        .expect("must normalize");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["material"], "zirconia");
    }

    #[test]
    fn jurisdiction_de_maps_to_germany() {
        assert_eq!(jurisdiction_to_country("DE").unwrap(), "germany");
    }

    #[test]
    fn jurisdiction_us_maps_to_united_states() {
        assert_eq!(jurisdiction_to_country("US").unwrap(), "united_states");
    }

    #[test]
    fn jurisdiction_jp_maps_to_japan() {
        assert_eq!(jurisdiction_to_country("JP").unwrap(), "japan");
    }

    #[test]
    fn unknown_jurisdiction_returns_invalid_field_error() {
        let err = normalize_pilot_case_json(
            r#"{"restoration_type": "crown", "material": "zirconia", "jurisdiction": "XX"}"#,
        )
        .unwrap_err();
        assert!(
            matches!(err, CliError::InvalidField(_)),
            "expected InvalidField, got {err:?}"
        );
        assert!(err.to_string().contains("XX"));
    }

    #[test]
    fn missing_required_field_returns_parse_error() {
        // Missing `restoration_type`
        let err = normalize_pilot_case_json(
            r#"{"material": "zirconia", "jurisdiction": "DE"}"#,
        )
        .unwrap_err();
        assert!(matches!(err, CliError::ParseError(_)));
    }

    #[test]
    fn missing_jurisdiction_returns_parse_error() {
        let err = normalize_pilot_case_json(
            r#"{"restoration_type": "crown", "material": "zirconia"}"#,
        )
        .unwrap_err();
        assert!(matches!(err, CliError::ParseError(_)));
    }

    #[test]
    fn absent_case_id_becomes_json_null() {
        let json = normalize_pilot_case_json(
            r#"{"restoration_type": "crown", "material": "zirconia", "jurisdiction": "DE"}"#,
        )
        .expect("must normalize");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v["case_id"].is_null());
    }
}
