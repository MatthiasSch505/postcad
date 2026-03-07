use postcad_cli::route_case_from_json;
use serde_json::Value;

fn fixtures_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn read_fixture(name: &str) -> String {
    let path = fixtures_dir().join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read fixture {}: {}", path.display(), e))
}

fn as_json_value(json_str: &str) -> Value {
    serde_json::from_str(json_str).expect("expected valid JSON")
}

#[test]
fn golden_routed_output_matches_expected() {
    let output = route_case_from_json(
        &read_fixture("case.json"),
        &read_fixture("candidates.json"),
        &read_fixture("snapshot.json"),
    )
    .expect("routing should succeed");

    let actual: Value = serde_json::to_value(&output).unwrap();
    let expected: Value = as_json_value(&read_fixture("expected_routed.json"));

    assert_eq!(actual, expected);
}

#[test]
fn golden_refused_output_matches_expected() {
    let output = route_case_from_json(
        &read_fixture("case.json"),
        &read_fixture("candidates.json"),
        &read_fixture("snapshot_refusal.json"),
    )
    .expect("parse should succeed");

    let actual: Value = serde_json::to_value(&output).unwrap();
    let expected: Value = as_json_value(&read_fixture("expected_refused.json"));

    assert_eq!(actual, expected);
}
