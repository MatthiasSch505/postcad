//! Tests for the `registry-export` command logic.

use postcad_cli::export_registry;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn single_active_record(id: &str) -> String {
    format!(
        r#"[{{
            "manufacturer_id": "{id}",
            "country": "germany",
            "capabilities": ["crown"],
            "materials": ["zirconia"],
            "certifications": ["verified"],
            "active": true
        }}]"#
    )
}

fn two_records_unsorted() -> &'static str {
    r#"[
        {
            "manufacturer_id": "mfr-de-002",
            "country": "germany",
            "capabilities": ["bridge"],
            "materials": ["pmma"],
            "certifications": ["verified"],
            "active": true
        },
        {
            "manufacturer_id": "mfr-de-001",
            "country": "germany",
            "capabilities": ["crown"],
            "materials": ["zirconia"],
            "certifications": ["verified"],
            "active": true
        }
    ]"#
}

fn mixed_active_inactive() -> &'static str {
    r#"[
        {
            "manufacturer_id": "mfr-active",
            "country": "germany",
            "capabilities": ["crown"],
            "materials": ["zirconia"],
            "certifications": ["verified"],
            "active": true
        },
        {
            "manufacturer_id": "mfr-inactive",
            "country": "germany",
            "capabilities": ["crown"],
            "materials": ["zirconia"],
            "certifications": ["verified"],
            "active": false
        }
    ]"#
}

// ── registry_export_deterministic ─────────────────────────────────────────────

#[test]
fn registry_export_deterministic() {
    let input = single_active_record("mfr-de-001");
    let out1 = export_registry(&input).expect("export must succeed");
    let out2 = export_registry(&input).expect("export must succeed on second call");
    assert_eq!(out1, out2, "export output must be identical across repeated calls");
}

#[test]
fn registry_export_deterministic_multi_candidate() {
    let out1 = export_registry(two_records_unsorted()).expect("first export must succeed");
    let out2 = export_registry(two_records_unsorted()).expect("second export must succeed");
    assert_eq!(out1, out2, "export output must be identical for multiple records");
}

// ── registry_export_filters_inactive ─────────────────────────────────────────

#[test]
fn registry_export_filters_inactive() {
    let output = export_registry(mixed_active_inactive()).expect("export must succeed");
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&output).expect("output must be valid JSON");
    assert_eq!(parsed.len(), 1, "only one active record should be emitted");
    assert_eq!(
        parsed[0]["manufacturer_id"].as_str().unwrap(),
        "mfr-active",
        "the emitted record must be the active one"
    );
}

#[test]
fn registry_export_all_inactive_produces_empty_snapshot() {
    let input = r#"[{
        "manufacturer_id": "mfr-gone",
        "country": "germany",
        "capabilities": ["crown"],
        "materials": ["zirconia"],
        "certifications": ["verified"],
        "active": false
    }]"#;
    let output = export_registry(input).expect("export must succeed even with no active records");
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&output).expect("output must be valid JSON");
    assert!(parsed.is_empty(), "all-inactive input must produce an empty snapshot");
}

// ── registry_export_sorted ────────────────────────────────────────────────────

#[test]
fn registry_export_sorted() {
    let output = export_registry(two_records_unsorted()).expect("export must succeed");
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&output).expect("output must be valid JSON");
    assert_eq!(parsed.len(), 2, "both active records must be emitted");
    assert_eq!(
        parsed[0]["manufacturer_id"].as_str().unwrap(),
        "mfr-de-001",
        "first record must be lexicographically smallest manufacturer_id"
    );
    assert_eq!(
        parsed[1]["manufacturer_id"].as_str().unwrap(),
        "mfr-de-002",
        "second record must follow in lexicographic order"
    );
}

#[test]
fn registry_export_sort_is_by_manufacturer_id_not_insertion_order() {
    // Records given in Z→A order; output must be A→Z.
    let input = r#"[
        {"manufacturer_id":"zzz","country":"germany","capabilities":["crown"],"materials":["zirconia"],"certifications":["verified"],"active":true},
        {"manufacturer_id":"aaa","country":"germany","capabilities":["crown"],"materials":["zirconia"],"certifications":["verified"],"active":true}
    ]"#;
    let output = export_registry(input).expect("export must succeed");
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&output).expect("output must be valid JSON");
    assert_eq!(parsed[0]["manufacturer_id"].as_str().unwrap(), "aaa");
    assert_eq!(parsed[1]["manufacturer_id"].as_str().unwrap(), "zzz");
}

// ── validation ────────────────────────────────────────────────────────────────

#[test]
fn registry_export_rejects_duplicate_manufacturer_id() {
    let input = r#"[
        {"manufacturer_id":"dup","country":"germany","capabilities":["crown"],"materials":["zirconia"],"certifications":["verified"],"active":true},
        {"manufacturer_id":"dup","country":"germany","capabilities":["bridge"],"materials":["pmma"],"certifications":["verified"],"active":true}
    ]"#;
    let err = export_registry(input).expect_err("duplicate IDs must be rejected");
    assert_eq!(err.code(), "duplicate_manufacturer_id");
}

#[test]
fn registry_export_rejects_empty_capabilities() {
    let input = r#"[{
        "manufacturer_id": "mfr-x",
        "country": "germany",
        "capabilities": [],
        "materials": ["zirconia"],
        "certifications": ["verified"],
        "active": true
    }]"#;
    let err = export_registry(input).expect_err("empty capabilities must be rejected");
    assert_eq!(err.code(), "empty_capabilities");
}

#[test]
fn registry_export_rejects_empty_materials() {
    let input = r#"[{
        "manufacturer_id": "mfr-x",
        "country": "germany",
        "capabilities": ["crown"],
        "materials": [],
        "certifications": ["verified"],
        "active": true
    }]"#;
    let err = export_registry(input).expect_err("empty materials must be rejected");
    assert_eq!(err.code(), "empty_materials");
}

// ── output schema matches ManufacturerRecord ──────────────────────────────────

#[test]
fn registry_export_output_matches_manufacturer_record_schema() {
    let output = export_registry(&single_active_record("mfr-de-001")).expect("export must succeed");
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&output).expect("output must be valid JSON");
    let rec = &parsed[0];
    // All ManufacturerRecord fields must be present.
    assert!(rec.get("manufacturer_id").is_some());
    assert!(rec.get("country").is_some());
    assert!(rec.get("capabilities").is_some());
    assert!(rec.get("materials_supported").is_some());
    assert!(rec.get("attestation_statuses").is_some());
    assert!(rec.get("is_active").is_some());
    assert!(rec.get("display_name").is_some());
    assert!(rec.get("jurisdictions_served").is_some());
    assert!(rec.get("sla_days").is_some());
    // is_active must be true (inactive records are filtered before output).
    assert_eq!(rec["is_active"].as_bool().unwrap(), true);
}

#[test]
fn registry_export_maps_field_names_correctly() {
    let output = export_registry(&single_active_record("mfr-de-001")).expect("export must succeed");
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&output).expect("output must be valid JSON");
    let rec = &parsed[0];
    // Source `certifications` must appear as `attestation_statuses`.
    assert_eq!(rec["attestation_statuses"][0].as_str().unwrap(), "verified");
    // Source `materials` must appear as `materials_supported`.
    assert_eq!(rec["materials_supported"][0].as_str().unwrap(), "zirconia");
}

#[test]
fn registry_export_default_display_name_is_manufacturer_id() {
    // No `display_name` in source → must default to manufacturer_id.
    let output = export_registry(&single_active_record("mfr-de-001")).expect("export must succeed");
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&output).expect("output must be valid JSON");
    assert_eq!(parsed[0]["display_name"].as_str().unwrap(), "mfr-de-001");
}

#[test]
fn registry_export_default_jurisdictions_served_is_country() {
    // No `jurisdictions_served` in source → must default to [country].
    let output = export_registry(&single_active_record("mfr-de-001")).expect("export must succeed");
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&output).expect("output must be valid JSON");
    assert_eq!(parsed[0]["jurisdictions_served"][0].as_str().unwrap(), "germany");
}

// ── optional field override ───────────────────────────────────────────────────

#[test]
fn registry_export_respects_explicit_display_name() {
    let input = r#"[{
        "manufacturer_id": "mfr-de-001",
        "country": "germany",
        "capabilities": ["crown"],
        "materials": ["zirconia"],
        "certifications": ["verified"],
        "active": true,
        "display_name": "Alpha Dental GmbH",
        "sla_days": 5
    }]"#;
    let output = export_registry(input).expect("export must succeed");
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&output).expect("output must be valid JSON");
    assert_eq!(parsed[0]["display_name"].as_str().unwrap(), "Alpha Dental GmbH");
    assert_eq!(parsed[0]["sla_days"].as_u64().unwrap(), 5);
}
