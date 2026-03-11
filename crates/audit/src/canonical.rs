/// Serializes `value` to compact, deterministic JSON.
///
/// Rules:
/// - Compact (no extra whitespace or newlines).
/// - Struct fields appear in declaration order — stable across identical builds.
/// - `Option::None` serializes as JSON `null`.
/// - Arrays preserve element order.
/// - No floating-point or map types are used in audit structs, so key-ordering
///   ambiguity does not arise.
///
/// Panics if serialization fails, which cannot happen for the well-formed audit
/// structs used in this crate.
pub fn to_canonical_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("audit struct serialization must not fail")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Sample {
        alpha: String,
        beta: Option<String>,
        gamma: Vec<String>,
    }

    fn sample() -> Sample {
        Sample {
            alpha: "hello".to_string(),
            beta: Some("world".to_string()),
            gamma: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        }
    }

    fn sample_none() -> Sample {
        Sample {
            alpha: "hello".to_string(),
            beta: None,
            gamma: vec![],
        }
    }

    // ── stability ─────────────────────────────────────────────────────────────

    #[test]
    fn same_value_serialized_twice_yields_identical_bytes() {
        let s = sample();
        assert_eq!(to_canonical_json(&s), to_canonical_json(&s));
    }

    #[test]
    fn logically_identical_values_produce_identical_json() {
        let a = sample();
        let b = sample();
        assert_eq!(to_canonical_json(&a), to_canonical_json(&b));
    }

    // ── format ────────────────────────────────────────────────────────────────

    #[test]
    fn output_is_compact_no_trailing_newline() {
        let json = to_canonical_json(&sample());
        assert!(!json.ends_with('\n'));
        assert!(!json.contains("\n  ")); // no pretty-print indentation
    }

    #[test]
    fn none_field_serializes_as_null() {
        let json = to_canonical_json(&sample_none());
        assert!(json.contains("\"beta\":null"));
    }

    #[test]
    fn some_field_serializes_as_value() {
        let json = to_canonical_json(&sample());
        assert!(json.contains("\"beta\":\"world\""));
    }

    // ── ordering ──────────────────────────────────────────────────────────────

    #[test]
    fn struct_fields_appear_in_declaration_order() {
        let json = to_canonical_json(&sample());
        let alpha_pos = json.find("\"alpha\"").unwrap();
        let beta_pos = json.find("\"beta\"").unwrap();
        let gamma_pos = json.find("\"gamma\"").unwrap();
        assert!(alpha_pos < beta_pos);
        assert!(beta_pos < gamma_pos);
    }

    #[test]
    fn array_elements_preserve_input_order() {
        let json = to_canonical_json(&sample());
        let a_pos = json.find("\"a\"").unwrap();
        let b_pos = json.find("\"b\"").unwrap();
        let c_pos = json.find("\"c\"").unwrap();
        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn empty_array_serializes_as_brackets() {
        let json = to_canonical_json(&sample_none());
        assert!(json.contains("\"gamma\":[]"));
    }

    // ── different values ──────────────────────────────────────────────────────

    #[test]
    fn different_values_produce_different_json() {
        assert_ne!(
            to_canonical_json(&sample()),
            to_canonical_json(&sample_none())
        );
    }
}
