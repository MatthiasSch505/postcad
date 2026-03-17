//! PostCAD Constitution surface tests.
//!
//! Verifies that the frozen governance artifact `ops/POSTCAD_CONSTITUTION.md`
//! exists, contains required anchor phrases, and that `ops/README.md`
//! references it. These tests act as a lint on the governance layer and must
//! never be deleted or weakened without a core-truth campaign.

const CONSTITUTION: &str = include_str!("../../../ops/POSTCAD_CONSTITUTION.md");
const OPS_README: &str = include_str!("../../../ops/README.md");

// ── file exists ────────────────────────────────────────────────────────────────

#[test]
fn constitution_file_exists() {
    assert!(
        !CONSTITUTION.is_empty(),
        "ops/POSTCAD_CONSTITUTION.md must exist and must not be empty"
    );
}

// ── mission anchor ─────────────────────────────────────────────────────────────

#[test]
fn constitution_contains_mission_phrase() {
    assert!(
        CONSTITUTION.contains("deterministic post-CAD routing and audit infrastructure layer"),
        "constitution must contain mission phrase: \
         'deterministic post-CAD routing and audit infrastructure layer'"
    );
}

// ── frozen kernel invariants ───────────────────────────────────────────────────

#[test]
fn constitution_contains_routing_determinism_invariant() {
    assert!(
        CONSTITUTION.contains("routing semantics are deterministic and rule-driven"),
        "constitution must contain invariant: \
         'routing semantics are deterministic and rule-driven'"
    );
}

#[test]
fn constitution_contains_refusal_invariant() {
    assert!(
        CONSTITUTION.contains("refusal semantics must remain explicit"),
        "constitution must contain invariant: \
         'refusal semantics must remain explicit'"
    );
}

// ── forbidden expansions ───────────────────────────────────────────────────────

#[test]
fn constitution_forbids_hidden_ai_heuristics() {
    assert!(
        CONSTITUTION.contains("no hidden AI heuristics in core routing"),
        "constitution must forbid: 'no hidden AI heuristics in core routing'"
    );
}

// ── campaign rule ──────────────────────────────────────────────────────────────

#[test]
fn constitution_contains_campaign_rule_phrase() {
    assert!(
        CONSTITUTION.contains("every campaign must declare allowed files"),
        "constitution must contain campaign rule: \
         'every campaign must declare allowed files'"
    );
}

// ── ops README references constitution ────────────────────────────────────────

#[test]
fn ops_readme_references_constitution() {
    assert!(
        OPS_README.contains("POSTCAD_CONSTITUTION.md"),
        "ops/README.md must reference POSTCAD_CONSTITUTION.md"
    );
}
