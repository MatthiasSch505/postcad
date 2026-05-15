//! Smoke tests for the lab-side reviewer demo.
//!
//! Tests verify the current reviewer HTML structure (Klärung vor Herstellung)
//! and the end-to-end API workflow used by the reviewer backend.
//!
//! Phases covered:
//!   1. Upload   — file selection / demo button
//!   2. Processing — animated step indicators
//!   3. Visuelle Klärung — visual placeholder + lab comment
//!   4. Entscheidung — decision gate with 3 choices
//!   5. Ergebnis — decision receipt including visual/comment nachweis rows

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::util::ServiceExt;

// ── Pilot fixtures ────────────────────────────────────────────────────────────

const PILOT_CASE_JSON: &str = include_str!("../../../examples/pilot/case.json");
const REGISTRY_JSON: &str = include_str!("../../../examples/pilot/registry_snapshot.json");
const CONFIG_JSON: &str = include_str!("../../../examples/pilot/config.json");

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_app(tmp: &tempfile::TempDir) -> axum::Router {
    postcad_service::app_with_all_stores(
        Arc::new(postcad_service::CaseStore::new(tmp.path().join("cases"))),
        Arc::new(postcad_service::ReceiptStore::new(
            tmp.path().join("receipts"),
        )),
        Arc::new(postcad_service::DispatchStore::new(
            tmp.path().join("dispatch"),
        )),
        Arc::new(postcad_service::PolicyStore::new(
            tmp.path().join("policies"),
        )),
        Arc::new(postcad_service::VerificationStore::new(
            tmp.path().join("verification"),
        )),
        Arc::new(postcad_service::DispatchCommitmentStore::new(
            tmp.path().join("commitments"),
        )),
        Arc::new(postcad_service::DecisionStore::new(
            tmp.path().join("decisions"),
        )),
    )
}

async fn post_json(app: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (status, json)
}

async fn get_json(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (status, json)
}

async fn get_html(app: axum::Router, uri: &str) -> (StatusCode, String) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

// ── Basic page tests ──────────────────────────────────────────────────────────

/// Reviewer endpoint must return 200 with HTML content.
#[tokio::test]
async fn reviewer_page_is_accessible() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("<!DOCTYPE html>"), "must be an HTML document");
    assert!(html.contains("PostCAD"), "must carry the PostCAD brand");
}

/// Reviewer page must declare German as the primary language.
#[tokio::test]
async fn reviewer_page_language_german() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains(r#"lang="de""#), "html lang must be de");
    assert!(html.contains("Klärung vor Herstellung"), "DE tagline must be present");
}

/// Reviewer must expose a DE/EN language toggle.
#[tokio::test]
async fn reviewer_page_language_toggle_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("btn-de"), "DE toggle button must be present");
    assert!(html.contains("btn-en"), "EN toggle button must be present");
    assert!(html.contains("setLang"), "setLang JS function must be present");
}

// ── Upload phase ──────────────────────────────────────────────────────────────

/// Upload phase must have the drop zone and demo file button.
#[tokio::test]
async fn reviewer_upload_phase_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("phase-upload"),         "phase-upload id must be present");
    assert!(html.contains("upload-zone"),          "upload-zone id must be present");
    assert!(html.contains("file-input"),           "file-input id must be present");
    assert!(html.contains("Digitalen Laborfall"),  "upload title DE must be present");
    assert!(html.contains("Open digital lab case"), "upload title EN must be present");
}

/// Demo file button must use Zahn 36 filename (not old 3-6 range notation).
#[tokio::test]
async fn reviewer_demo_button_uses_zahn36() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("Krone_Zahn_36_DE.stl"), "demo button must use Krone_Zahn_36_DE.stl filename");
    assert!(html.contains("loadDemo"),              "loadDemo JS function must be present");
}

// ── Processing phase ──────────────────────────────────────────────────────────

/// Processing phase must have filename display and 4 step indicators.
#[tokio::test]
async fn reviewer_processing_phase_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("phase-processing"), "phase-processing id must be present");
    assert!(html.contains("proc-filename"),    "proc-filename id must be present");
    assert!(html.contains("pstep-0"),          "pstep-0 id must be present");
    assert!(html.contains("pstep-3"),          "pstep-3 id must be present");
    assert!(html.contains("startProcessing"),  "startProcessing JS function must be present");
}

// ── Visuelle Klärung step tests ───────────────────────────────────────────────

/// Reviewer shell must contain the visual clarification phase so the lab can
/// document a visual note before proceeding to the decision gate.
#[tokio::test]
async fn reviewer_shell_visual_step_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("phase-visual"),                     "phase-visual id must be present");
    assert!(html.contains("Visuelle Klärung vor Herstellung"), "visual step title must be present");
    assert!(html.contains("Demo-Ansicht:"),                "visual placeholder hint must be present");
    assert!(html.contains("Keine automatische technische"), "visual disclaimer must be present");
    assert!(html.contains("id=\"lab-comment\""),               "lab-comment textarea id must be present");
    assert!(html.contains("Was soll der Praxis vor Herstellung"), "lab comment label must be present");
    assert!(html.contains("proceedToDecision"),                "proceedToDecision JS function must be present");
    assert!(html.contains("showVisualStep"),                   "showVisualStep JS function must be present");
}

/// Visual step must show a Zahn 36 placeholder so the demo always represents
/// the correct tooth rather than the old 3–6 range notation.
#[tokio::test]
async fn reviewer_shell_visual_step_zahn36_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("Zahn 36"),              "Zahn 36 must appear in the visual step placeholder");
    assert!(html.contains("Krone_Zahn_36_DE.stl"), "demo button must use Krone_Zahn_36_DE.stl filename");
}

/// Decision receipt must include visual clarification summary and lab comment
/// rows so the documented decision captures the full pre-manufacturing context.
#[tokio::test]
async fn reviewer_shell_receipt_includes_visual_and_comment() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("nachweis-visual"),    "nachweis-visual id must be present in receipt");
    assert!(html.contains("nachweis-kommentar"), "nachweis-kommentar id must be present in receipt");
    assert!(html.contains("Visuelle Klärung"),   "Visuelle Klärung label must appear in receipt section");
    assert!(html.contains("Laborkommentar"),     "Laborkommentar label must appear in receipt section");
}

/// Visual step must have a proceed button that advances to the decision gate
/// without requiring any mandatory input so the comment is optional.
#[tokio::test]
async fn reviewer_shell_visual_step_proceed_button_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("visual-next-btn"),         "visual-next-btn id must be present");
    assert!(html.contains("Weiter zur Entscheidung"), "proceed button label must be present in DE");
    assert!(html.contains("Proceed to decision"),     "proceed button label must be present in EN");
}

/// Visual step translations must be complete in both DE and EN.
#[tokio::test]
async fn reviewer_shell_visual_step_translations_complete() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("VISUELLE KLÄRUNG"),    "DE visual badge must be in T");
    assert!(html.contains("VISUAL CLARIFICATION"),"EN visual badge must be in T");
    assert!(html.contains("visualClarificationSummary"), "summary key must be in T");
    assert!(html.contains("nachweisVisualLbl"),   "nachweisVisualLbl key must be in T");
    assert!(html.contains("nachweisKommentarLbl"),"nachweisKommentarLbl key must be in T");
}

/// STL viewer canvas and controls must be present for the interactive 3D view.
#[tokio::test]
async fn reviewer_shell_stl_viewer_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("stl-canvas"),          "stl-canvas id must be present");
    assert!(html.contains("stl-viewer-wrap"),      "stl-viewer-wrap id must be present");
    assert!(html.contains("stl-viewer-fallback"),  "stl-viewer-fallback id must be present for WebGL fallback");
    assert!(html.contains("initViewer"),           "initViewer JS function must be present");
    assert!(html.contains("disposeViewer"),        "disposeViewer JS function must be present");
    assert!(html.contains("three@0.158.0"),        "Three.js CDN script must be present");
}

/// Case metadata form must be present with all four fields and privacy notice.
#[tokio::test]
async fn reviewer_shell_case_metadata_form_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("meta-bezeichnung"),           "meta-bezeichnung input id must be present");
    assert!(html.contains("meta-zahn"),                  "meta-zahn input id must be present");
    assert!(html.contains("meta-material"),              "meta-material input id must be present");
    assert!(html.contains("meta-praxis"),                "meta-praxis input id must be present");
    assert!(html.contains("Bitte keine Patientennamen"), "privacy notice DE must be present");
    assert!(html.contains("Please do not enter patient names"), "privacy notice EN must be present");
}

/// Audit receipt must include the four case-metadata rows added in Phase 3.
#[tokio::test]
async fn reviewer_shell_nachweis_metadata_rows_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("nachweis-bezeichnung"), "nachweis-bezeichnung id must be present");
    assert!(html.contains("nachweis-zahn"),        "nachweis-zahn id must be present");
    assert!(html.contains("nachweis-material"),    "nachweis-material id must be present");
    assert!(html.contains("nachweis-praxis"),      "nachweis-praxis id must be present");
}

// ── Decision gate tests ───────────────────────────────────────────────────────

/// Decision gate must have all three choice buttons and confirm logic.
#[tokio::test]
async fn reviewer_decision_gate_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("phase-decision"),                "phase-decision id must be present");
    assert!(html.contains("ENTSCHEIDUNG VOR HERSTELLUNG"),  "gate badge DE must be present");
    assert!(html.contains("DECISION BEFORE MANUFACTURING"), "gate badge EN must be present");
    assert!(html.contains("choice-proceed"),                "choice-proceed id must be present");
    assert!(html.contains("choice-proceed_with_risk"),      "choice-proceed_with_risk id must be present");
    assert!(html.contains("choice-request_correction"),     "choice-request_correction id must be present");
    assert!(html.contains("confirm-btn"),                   "confirm-btn id must be present");
    assert!(html.contains("selectDecision"),                "selectDecision JS function must be present");
    assert!(html.contains("confirmDecision"),               "confirmDecision JS function must be present");
}

/// Reason dropdown must be present for risk and correction decisions.
#[tokio::test]
async fn reviewer_decision_gate_reason_dropdown_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("reason-row"),               "reason-row id must be present");
    assert!(html.contains("reason-code"),              "reason-code select id must be present");
    assert!(html.contains("Unvollständiger Scan"),     "incomplete scan option must be present");
    assert!(html.contains("Unklare Präp.-Grenze"),    "unclear margin option must be present");
    assert!(html.contains("updateConfirmState"),        "updateConfirmState JS function must be present");
}

// ── Result phase tests ────────────────────────────────────────────────────────

/// Result phase must have verdict display and all audit nachweis rows.
#[tokio::test]
async fn reviewer_result_phase_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("phase-result"),          "phase-result id must be present");
    assert!(html.contains("result-verdict"),        "result-verdict id must be present");
    assert!(html.contains("result-sub"),            "result-sub id must be present");
    assert!(html.contains("result-explanation"),    "result-explanation id must be present");
    assert!(html.contains("Entscheidungsnachweis"), "audit section label must be present");
}

/// Audit nachweis section must have all required rows including case, decision,
/// Grundlage, visual clarification, lab comment, time, audit ID, and status.
#[tokio::test]
async fn reviewer_audit_nachweis_rows_complete() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("nachweis-fall"),       "nachweis-fall id must be present");
    assert!(html.contains("nachweis-decision"),   "nachweis-decision id must be present");
    assert!(html.contains("nachweis-grundlage"),  "nachweis-grundlage id must be present");
    assert!(html.contains("nachweis-visual"),     "nachweis-visual id must be present");
    assert!(html.contains("nachweis-kommentar"),  "nachweis-kommentar id must be present");
    assert!(html.contains("audit-time"),          "audit-time id must be present");
    assert!(html.contains("audit-id"),            "audit-id id must be present");
    assert!(html.contains("audit-status"),        "audit-status id must be present");
}

/// Case data section in result must have proc, material, land and indication.
#[tokio::test]
async fn reviewer_result_case_data_section_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("res-proc"),        "res-proc id must be present");
    assert!(html.contains("res-material"),    "res-material id must be present");
    assert!(html.contains("res-land"),        "res-land id must be present");
    assert!(html.contains("res-indication"),  "res-indication id must be present");
    assert!(html.contains("Fall erkannt"),    "case section label DE must be present");
}

/// Check rows for material, jurisdiction and manufacturing must be present.
#[tokio::test]
async fn reviewer_result_check_rows_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("chk-material"),     "chk-material id must be present");
    assert!(html.contains("chk-jurisdiction"), "chk-jurisdiction id must be present");
    assert!(html.contains("chk-manufacturing"),"chk-manufacturing id must be present");
    assert!(html.contains("setCheck"),         "setCheck JS function must be present");
}

// ── JS flow tests ─────────────────────────────────────────────────────────────

/// Key JS state variables must all be declared.
#[tokio::test]
async fn reviewer_js_state_variables_declared() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("let lang"),             "lang state var must be declared");
    assert!(html.contains("let lastResultOk"),     "lastResultOk state var must be declared");
    assert!(html.contains("let currentFilename"),  "currentFilename state var must be declared");
    assert!(html.contains("let selectedDecision"), "selectedDecision state var must be declared");
    assert!(html.contains("let labComment"),       "labComment state var must be declared");
}

/// Key JS flow functions must be present.
#[tokio::test]
async fn reviewer_js_flow_functions_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("showVisualStep"),      "showVisualStep must be present");
    assert!(html.contains("proceedToDecision"),   "proceedToDecision must be present");
    assert!(html.contains("showDecisionGate"),    "showDecisionGate must be present");
    assert!(html.contains("showResult"),          "showResult must be present");
    assert!(html.contains("showResultBlocked"),   "showResultBlocked must be present");
    assert!(html.contains("resetDemo"),           "resetDemo must be present");
    assert!(html.contains("backToDecision"),      "backToDecision must be present");
}

/// Processing must lead to visual step, not directly to decision gate.
#[tokio::test]
async fn reviewer_processing_leads_to_visual_step() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(
        html.contains("showVisualStep(filename)"),
        "startProcessing must call showVisualStep, not showDecisionGate"
    );
}

/// AHA line must be present to clarify PostCAD's scope.
#[tokio::test]
async fn reviewer_aha_line_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("aha-line"), "aha-line element id must be present");
    assert!(
        html.contains("erkennt keine medizinischen"),
        "DE aha line must be present"
    );
    assert!(
        html.contains("does not detect medical"),
        "EN aha line must be present"
    );
}

/// Optional technical proof section must be present but hidden by default.
#[tokio::test]
async fn reviewer_proof_section_optional() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("proof-section"),       "proof-section id must be present");
    assert!(html.contains("proof-receipt-json"),  "proof-receipt-json pre id must be present");
    assert!(html.contains("Technischer Nachweis"), "proof section label DE must be present");
}

/// Legacy element div must be present (keeps legacy IDs for compatibility).
#[tokio::test]
async fn reviewer_legacy_div_preserved() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("id=\"_legacy\""),  "_legacy div must be preserved");
    assert!(html.contains("btn-route-norm"),  "btn-route-norm must be in legacy div");
}

// ── End-to-end API workflow test ──────────────────────────────────────────────

/// Full pilot workflow: route → dispatch → approve → export.
///
/// Exercises the API backend that the reviewer demo calls.
/// Reviewer shell accessibility is verified as step 2.
#[tokio::test]
async fn reviewer_workflow_normalized_route_to_export() {
    let tmp = tempfile::TempDir::new().unwrap();

    // ── Step 1: submit normalized pilot input ─────────────────────────────────
    let route_body = json!({
        "pilot_case": {
            "case_id": "f1000001-0000-0000-0000-000000000001",
            "restoration_type": "crown",
            "material": "zirconia",
            "jurisdiction": "DE"
        },
        "registry_snapshot": serde_json::from_str::<Value>(REGISTRY_JSON).unwrap(),
        "routing_config":    serde_json::from_str::<Value>(CONFIG_JSON).unwrap(),
    });
    let (route_status, route_resp) =
        post_json(make_app(&tmp), "/pilot/route-normalized", route_body).await;

    assert_eq!(route_status, StatusCode::OK, "step 1 route failed: {route_resp}");
    assert_eq!(route_resp["receipt"]["outcome"], "routed");
    assert_eq!(
        route_resp["receipt"]["selected_candidate_id"],
        "pilot-de-001",
        "canonical pilot case must route to pilot-de-001"
    );
    assert!(
        route_resp["receipt"]["receipt_hash"].is_string(),
        "receipt_hash must be present"
    );
    assert!(
        route_resp["derived_policy"].is_object(),
        "derived_policy must be present for dispatch binding"
    );

    let receipt = route_resp["receipt"].clone();
    let derived_policy = route_resp["derived_policy"].clone();
    let receipt_hash = receipt["receipt_hash"].as_str().unwrap().to_string();

    // ── Step 2: reviewer shell is reachable ───────────────────────────────────
    let (reviewer_status, _) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(reviewer_status, StatusCode::OK, "step 2 reviewer shell unreachable");

    // ── Step 3: create dispatch from the normalized receipt ───────────────────
    let create_body = json!({
        "receipt": receipt,
        "case":    serde_json::from_str::<Value>(PILOT_CASE_JSON).unwrap(),
        "policy":  derived_policy,
    });
    let (create_status, create_resp) =
        post_json(make_app(&tmp), "/dispatch/create", create_body).await;

    assert_eq!(create_status, StatusCode::OK, "step 3 dispatch create failed: {create_resp}");
    assert_eq!(create_resp["status"], "draft");
    assert_eq!(create_resp["verification_passed"], true);
    assert_eq!(create_resp["receipt_hash"], receipt_hash);
    assert!(
        create_resp["dispatch_id"].is_string(),
        "dispatch_id must be present"
    );

    let dispatch_id = create_resp["dispatch_id"].as_str().unwrap().to_string();

    // ── Step 4: approve dispatch ──────────────────────────────────────────────
    let (approve_status, approve_resp) = post_json(
        make_app(&tmp),
        &format!("/dispatch/{dispatch_id}/approve"),
        json!({"approved_by": "reviewer"}),
    )
    .await;

    assert_eq!(approve_status, StatusCode::OK, "step 4 approve failed: {approve_resp}");
    assert_eq!(approve_resp["status"], "approved");
    assert_eq!(approve_resp["approved_by"], "reviewer");
    assert_eq!(approve_resp["dispatch_id"], dispatch_id);
    assert!(
        approve_resp["approved_at"].is_string(),
        "approved_at must be set after approval"
    );

    // ── Step 5: export dispatch ───────────────────────────────────────────────
    let (export_status, export_resp) =
        get_json(make_app(&tmp), &format!("/dispatch/{dispatch_id}/export")).await;

    assert_eq!(export_status, StatusCode::OK, "step 5 export failed: {export_resp}");
    assert_eq!(export_resp["status"], "exported");
    assert_eq!(export_resp["verification_passed"], true);
    assert_eq!(export_resp["dispatch_id"], dispatch_id);
    assert_eq!(export_resp["receipt_hash"], receipt_hash);
    assert_eq!(
        export_resp["selected_candidate_id"],
        "pilot-de-001",
        "export must carry the manufacturer selected by the kernel"
    );
    assert!(
        export_resp["approved_at"].is_string(),
        "approved_at must be present in export"
    );
}
