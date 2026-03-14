//! End-to-end smoke test for the pilot reviewer workflow.
//!
//! Exercises the full operator path in a single test:
//!
//!   1. POST /pilot/route-normalized → receipt + derived_policy
//!   2. GET  /reviewer               → reviewer shell HTML is accessible
//!   3. POST /dispatch/create        → draft dispatch bound to the receipt
//!   4. POST /dispatch/{id}/approve  → dispatch transitions to approved
//!   5. GET  /dispatch/{id}/export   → approved dispatch exported with all fields
//!
//! State persists across steps via a shared `TempDir` (all stores point to the
//! same directory on disk). Each call creates a fresh router from `make_app`
//! because `tower::ServiceExt::oneshot` consumes the router.
//!
//! No port binding. No schema changes. No new fixtures.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::util::ServiceExt;

// ── Pilot fixtures ────────────────────────────────────────────────────────────

/// Full CaseInput — used as the `case` field for dispatch/create + verify.
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

// ── UX state tests ────────────────────────────────────────────────────────────

/// Reviewer shell HTML must contain all required UX state elements for the
/// normalized pilot submission flow: idle → submitting → success / failure.
///
/// These are static assertions on the served HTML — no network calls needed.
#[tokio::test]
async fn reviewer_shell_norm_submit_states_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // Inline state element must exist
    assert!(
        html.contains("route-norm-inline"),
        "inline state element id must be present"
    );
    // Button id must be present and the disable-while-in-flight label
    assert!(
        html.contains("btn-route-norm"),
        "normalized-route button id must be present"
    );
    assert!(
        html.contains("Running kernel"),
        "in-flight button label must be present"
    );
    // Explicit submitting / success / failure state labels
    assert!(html.contains("Submitting"), "submitting state label must be present");
    assert!(
        html.contains("Routing complete"),
        "success state label must be present"
    );
    assert!(
        html.contains("Network failure"),
        "network failure error label must be present"
    );
    assert!(
        html.contains("Invalid JSON response"),
        "parse error label must be present"
    );
    // CSS classes for all three states must be referenced
    assert!(html.contains("loading-note"), "loading-note class must be present");
    assert!(html.contains("success-note"), "success-note class must be present");
    assert!(html.contains("error-note"), "error-note class must be present");
}

/// Reviewer shell HTML must expose all normalized-form controls and success-preview
/// controls added since the initial UX hardening.
///
/// Covers:
///   - form actions: submit, clear, load sample
///   - success preview: receipt summary container, copy-hash, download, JSON toggle
///   - validation surface: inline error element, invalid-field CSS marker
#[tokio::test]
async fn reviewer_shell_norm_ux_surface() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // ── form controls ─────────────────────────────────────────────────────────
    assert!(html.contains("routeNormalized"),  "submit function must be present");
    assert!(html.contains("clearNormForm"),    "clear action must be present");
    assert!(html.contains("Clear form"),       "clear button label must be present");
    assert!(html.contains("loadNormSample"),   "load-sample action must be present");
    assert!(html.contains("Load sample"),      "load-sample button label must be present");

    // ── success preview ───────────────────────────────────────────────────────
    assert!(html.contains("route-norm-preview"),  "success preview container id must be present");
    assert!(html.contains("norm-preview"),         "receipt summary CSS class must be present");
    assert!(html.contains("copyReceiptHash"),      "copy-receipt-hash function must be present");
    assert!(html.contains("downloadReceiptJson"),  "download function must be present");
    assert!(html.contains("Download receipt.json"),"download button label must be present");
    assert!(html.contains("toggleNormReceiptJson"),"receipt JSON toggle function must be present");
    assert!(html.contains("Show receipt JSON"),    "receipt JSON toggle label must be present");
    assert!(html.contains("btn-toggle-receipt"),   "receipt JSON toggle button id must be present");

    // ── validation / UI markers ───────────────────────────────────────────────
    assert!(html.contains("route-norm-inline"),       "inline validation element id must be present");
    assert!(html.contains("Required fields missing"), "validation error message must be present");
    assert!(html.contains("validateNormInput"),        "validation function must be present");
    assert!(html.contains("norm-field-invalid"),       "invalid-field CSS marker must be present");
}

/// Reviewer shell must expose individual `<input>` fields for all four normalized-case
/// fields so operators can edit values, trigger per-field validation, and use
/// keyboard shortcuts without touching raw JSON.
///
/// Covers:
///   - input element IDs for each of the four required fields
///   - field-wrapper CSS structure (norm-field-wrap, norm-field-label, norm-req)
///   - JS helpers that read/validate/mark those inputs (readNormInputs, markNormInvalid, clearNormInvalid)
#[tokio::test]
async fn reviewer_shell_norm_form_inputs_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // ── input element IDs ─────────────────────────────────────────────────────
    assert!(html.contains("norm-case-id"),          "case_id input id must be present");
    assert!(html.contains("norm-restoration-type"), "restoration_type input id must be present");
    assert!(html.contains("norm-material"),         "material input id must be present");
    assert!(html.contains("norm-jurisdiction"),     "jurisdiction input id must be present");

    // ── field-wrapper structure ───────────────────────────────────────────────
    assert!(html.contains("norm-field-wrap"),  "field wrapper class must be present");
    assert!(html.contains("norm-field-label"), "field label class must be present");
    assert!(html.contains("norm-input"),       "norm-input class must be used on inputs");
    assert!(html.contains("norm-req"),         "required marker must be present");

    // ── JS helpers for input reading / validation ─────────────────────────────
    assert!(html.contains("readNormInputs"),   "readNormInputs helper must be present");
    assert!(html.contains("markNormInvalid"),  "markNormInvalid helper must be present");
    assert!(html.contains("clearNormInvalid"), "clearNormInvalid helper must be present");
    assert!(html.contains("NORM_INPUT_IDS"),   "NORM_INPUT_IDS field-id map must be present");
}

/// Reviewer shell must expose explicit step framing for the 4-step operator flow
/// so a non-technical operator can navigate the submission pipeline without guidance.
///
/// Steps checked:
///   1. Enter normalized pilot input — static HTML label
///   2. Submit for review — static HTML label
///   3. Inspect receipt summary — JS-rendered label (in success preview builder)
///   4. Copy or download receipt — JS-rendered label (in success preview builder)
#[tokio::test]
async fn reviewer_shell_step_framing_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // CSS classes for step framing must be defined
    assert!(html.contains("norm-step"),     "norm-step CSS class must be referenced");
    assert!(html.contains("norm-step-num"), "norm-step-num CSS class must be referenced");
    assert!(html.contains("norm-step-lbl"), "norm-step-lbl CSS class must be referenced");

    // Static HTML step labels (steps 1 and 2)
    assert!(
        html.contains("Enter normalized pilot input"),
        "step 1 label must be present in HTML"
    );
    assert!(
        html.contains("Submit for review"),
        "step 2 label must be present in HTML"
    );

    // JS-rendered step labels (steps 3 and 4, emitted by the success preview builder)
    assert!(
        html.contains("Inspect receipt summary"),
        "step 3 label must be present in JS success preview"
    );
    assert!(
        html.contains("Copy or download receipt"),
        "step 4 label must be present in JS success preview"
    );

    // Keyboard shortcut hint must be present
    assert!(
        html.contains("Ctrl+Enter"),
        "keyboard shortcut hint must be present"
    );
}

/// Reviewer shell must render actionable error guidance when routing fails so that
/// a non-technical operator sees a next step rather than a raw error code.
///
/// Checks:
///   - Error panel CSS classes (norm-error-panel, norm-error-hint) are defined
///   - errorHint JS function is present
///   - Specific guidance phrases appear in the JS source for validation and routing errors
#[tokio::test]
async fn reviewer_shell_error_guidance_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // Error panel CSS classes must be defined
    assert!(
        html.contains("norm-error-panel"),
        "norm-error-panel CSS class must be defined"
    );
    assert!(
        html.contains("norm-error-hint"),
        "norm-error-hint CSS class must be defined"
    );

    // JS errorHint function must be present
    assert!(
        html.contains("errorHint"),
        "errorHint function must be defined in JS"
    );

    // Operator-readable guidance phrases must appear in the JS source
    assert!(
        html.contains("Check that all fields contain valid values"),
        "field-check hint must be present in errorHint"
    );
    assert!(
        html.contains("No manufacturer matched"),
        "routing-refused hint must be present in errorHint"
    );
}

// ── Export/handoff surface tests ──────────────────────────────────────────────

/// Reviewer shell HTML must expose download and copy actions for the export packet
/// so an operator can hand off the dispatch artifact without manual copy-paste.
#[tokio::test]
async fn reviewer_shell_export_packet_handoff_actions_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // Action container
    assert!(
        html.contains("dispatch-export-actions"),
        "dispatch-export-actions container id must be present"
    );

    // Download button
    assert!(
        html.contains("downloadExportPacket"),
        "downloadExportPacket JS function must be present"
    );
    assert!(
        html.contains("export_packet"),
        "export_packet filename hint must appear in download function"
    );

    // Copy JSON button
    assert!(
        html.contains("copyExportJson"),
        "copyExportJson JS function must be present"
    );

    // Export section carries handoff label
    assert!(
        html.contains("ready for handoff"),
        "ready for handoff label must appear in export packet section"
    );

    // Completion message
    assert!(
        html.contains("dispatch packet ready for handoff"),
        "completion message must say dispatch packet ready for handoff"
    );
}

/// Reviewer shell HTML must expose a copy button for the dispatch ID
/// so an operator can reference it during handoff without manual text selection.
#[tokio::test]
async fn reviewer_shell_dispatch_id_copy_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("art-dispatch-id-copy"),
        "art-dispatch-id-copy button id must be present"
    );
    assert!(
        html.contains("copyDispatchId"),
        "copyDispatchId JS function must be present"
    );
}

// ── Copyable artifact panel tests ─────────────────────────────────────────────

/// Reviewer shell HTML must expose a "Copy artifact" button for the receipt JSON
/// panel so an operator can copy the full artifact to the clipboard during a pilot run.
#[tokio::test]
async fn reviewer_shell_receipt_json_copy_button_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("receipt-json-actions"),
        "receipt-json-actions container id must be present"
    );
    assert!(
        html.contains("copyReceiptJson"),
        "copyReceiptJson JS function must be present"
    );
}

/// Reviewer shell HTML must expose a "Copy artifact" button for the verification
/// result panel so an operator can copy the verification JSON during review.
#[tokio::test]
async fn reviewer_shell_verify_json_copy_button_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("verify-json-actions"),
        "verify-json-actions container id must be present"
    );
    assert!(
        html.contains("copyVerifyJson"),
        "copyVerifyJson JS function must be present"
    );
}

/// Reviewer shell HTML must expose a "Copy artifact" button for the route error
/// panel so an operator can copy the error JSON for triage.
#[tokio::test]
async fn reviewer_shell_route_error_copy_button_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("route-error-json-actions"),
        "route-error-json-actions container id must be present"
    );
    assert!(
        html.contains("copyRouteErrorJson"),
        "copyRouteErrorJson JS function must be present"
    );
}

/// Reviewer shell HTML must contain "Copy artifact" button text so the action
/// is consistently labelled across all artifact panels.
#[tokio::test]
async fn reviewer_shell_copy_artifact_label_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Copy artifact"),
        "Copy artifact button text must appear in the reviewer shell"
    );
}

/// Reviewer shell HTML must contain the verify-artifact-note element with the
/// expected empty-state text for the verification artifact.
#[tokio::test]
async fn reviewer_shell_verify_artifact_note_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("verify-artifact-note"),
        "verify-artifact-note element id must be present"
    );
    assert!(
        html.contains("Artifact not yet generated"),
        "Artifact not yet generated empty-state text must be present"
    );
}

// ── Artifact export/discovery tests ──────────────────────────────────────────

/// Reviewer shell HTML must contain the artifact guide panel with all four
/// artifact entries and their purpose labels, including source-of-truth and
/// inspect-before-dispatch markers.
#[tokio::test]
async fn reviewer_shell_artifact_guide_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // Panel anchor and summary toggle
    assert!(
        html.contains("artifact-guide"),
        "artifact-guide element id must be present"
    );
    assert!(
        html.contains("Artifacts in this flow"),
        "artifact guide summary text must be present"
    );

    // All four artifact labels
    assert!(html.contains("Receipt Hash"),     "Receipt Hash label must be present in artifact guide");
    assert!(html.contains("Verification"),     "Verification label must be present in artifact guide");
    assert!(html.contains("Dispatch packet"),  "Dispatch packet label must be present in artifact guide");

    // Key guidance phrases
    assert!(
        html.contains("inspect this first"),
        "inspect-this-first guidance must be present in artifact guide"
    );
    assert!(
        html.contains("Verification source of truth"),
        "Verification source of truth label must be present"
    );
    assert!(
        html.contains("Required before dispatch"),
        "required-before-dispatch guidance must be present"
    );
}

/// Reviewer shell HTML must show the source-of-truth badge on the Receipt Hash
/// artifact row so an operator can immediately identify which field drives verification.
#[tokio::test]
async fn reviewer_shell_receipt_hash_source_of_truth_badge() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("sot-badge"),
        "sot-badge CSS class must be present on the Receipt Hash row"
    );
    assert!(
        html.contains("source of truth"),
        "source of truth text must appear in the receipt hash row"
    );
    assert!(
        html.contains("art-hash-copy"),
        "art-hash-copy button id must be present for one-click hash copy"
    );
    assert!(
        html.contains("copyArtHashVal"),
        "copyArtHashVal JS function must be present"
    );
}

/// Reviewer shell section titles must clearly identify each artifact stage so an
/// operator can tell receipt, verification, and dispatch results apart at a glance.
#[tokio::test]
async fn reviewer_shell_artifact_section_titles_clear() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // Receipt JSON section subtitle
    assert!(
        html.contains("inspect before dispatch"),
        "receipt JSON section must carry inspect-before-dispatch subtitle"
    );

    // Verification section subtitle
    assert!(
        html.contains("confirms receipt hash is authentic"),
        "verification section must carry receipt-hash-confirmation subtitle"
    );

    // Dispatch export section title
    assert!(
        html.contains("Export packet"),
        "dispatch export section title must be present"
    );
}

// ── Golden-path guidance + CLI reference tests ────────────────────────────────

/// Reviewer shell HTML must contain the CLI quick-reference panel with the
/// two helper script names and the golden-path sequence so a new operator
/// can move from the reviewer UI to the CLI and back without reading external docs.
#[tokio::test]
async fn reviewer_shell_cli_quickref_panel_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // Panel anchor
    assert!(
        html.contains("cli-quickref"),
        "cli-quickref element id must be present"
    );

    // Section heading
    assert!(
        html.contains("CLI helper commands"),
        "CLI helper commands heading must be present"
    );

    // Golden-path label
    assert!(
        html.contains("Golden path"),
        "Golden path label must be present in quickref"
    );

    // Both CLI companion script names must appear
    assert!(
        html.contains("run_pilot.sh"),
        "run_pilot.sh script name must be referenced"
    );
    assert!(
        html.contains("verify.sh"),
        "verify.sh script name must be referenced"
    );

    // The HTTP-vs-CLI orientation note
    assert!(
        html.contains("examples/pilot/"),
        "examples/pilot/ fixture path must be referenced"
    );
}

/// Reviewer shell HTML must carry the full golden-path sequence wording
/// consistently in both the hero flow steps and the quick-reference panel.
#[tokio::test]
async fn reviewer_shell_golden_path_wording_consistent() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // Hero 5-step labels
    assert!(html.contains("Open reviewer"),  "hero step 1 label must be present");
    assert!(html.contains("Run route"),       "hero step 2 label must be present");
    assert!(html.contains("Inspect receipt"), "hero step 3 label must be present");
    assert!(html.contains("Verify replay"),   "hero step 4 label must be present");
    assert!(html.contains("Dispatch"),        "hero step 5 label must be present");

    // Quick-ref golden path summary also contains the sequence
    assert!(
        html.contains("Verify replay"),
        "Verify replay must appear in quick-reference golden path"
    );
}

// ── Operator state block tests ────────────────────────────────────────────────

/// Reviewer shell HTML must contain a visible workflow status block with all four
/// stage indicators: Routing, Receipt, Verification, Dispatch.
///
/// These are static HTML assertions — the block is always rendered in the page
/// so an operator can see current stage at a glance without running any action.
#[tokio::test]
async fn reviewer_shell_operator_state_block_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // State block container
    assert!(
        html.contains("op-state-block"),
        "op-state-block CSS class must be present"
    );
    assert!(
        html.contains("op-state-grid"),
        "op-state-grid CSS class must be present"
    );

    // All four stage label strings must appear
    assert!(
        html.contains("Routing"),
        "Routing status label must be present"
    );
    assert!(
        html.contains("Receipt"),
        "Receipt status label must be present"
    );
    assert!(
        html.contains("Verification"),
        "Verification status label must be present"
    );
    assert!(
        html.contains("Dispatch"),
        "Dispatch status label must be present"
    );

    // Default state values must be initialised to not-run
    assert!(
        html.contains("not-run"),
        "initial not-run state value must be present"
    );
}

/// Reviewer shell HTML must expose the four operator state element IDs so that
/// the JS state machine can update them on every routing / verification / dispatch
/// transition without a page reload.
#[tokio::test]
async fn reviewer_shell_operator_state_ids_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("ops-routing"),  "ops-routing element id must be present");
    assert!(html.contains("ops-receipt"),  "ops-receipt element id must be present");
    assert!(html.contains("ops-verify"),   "ops-verify element id must be present");
    assert!(html.contains("ops-dispatch"), "ops-dispatch element id must be present");

    // CSS state classes for all five possible state values
    assert!(html.contains("op-not-run"),   "op-not-run CSS class must be defined");
    assert!(html.contains("op-available"), "op-available CSS class must be defined");
    assert!(html.contains("op-verified"),  "op-verified CSS class must be defined");
    assert!(html.contains("op-failed"),    "op-failed CSS class must be defined");
    assert!(html.contains("op-missing"),   "op-missing CSS class must be defined");

    // updateOpState JS function must be present to drive the block
    assert!(
        html.contains("updateOpState"),
        "updateOpState JS function must be present"
    );
}

/// Reviewer shell HTML must include operator guidance notes for the two key
/// blocking states: verification pending and dispatch blocked.
#[tokio::test]
async fn reviewer_shell_operator_guidance_notes_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("verify-pending-note"),
        "verify-pending-note element id must be present"
    );
    assert!(
        html.contains("Verification pending"),
        "verification pending guidance text must be present"
    );
    assert!(
        html.contains("Run verify before dispatch"),
        "run-verify guidance phrase must be present"
    );

    assert!(
        html.contains("dispatch-blocked-note"),
        "dispatch-blocked-note element id must be present"
    );
    assert!(
        html.contains("Dispatch blocked until verification succeeds"),
        "dispatch blocked guidance text must be present"
    );
}

// ── Smoke test ────────────────────────────────────────────────────────────────

/// Full pilot reviewer workflow smoke test.
///
/// Mirrors the sequence a reviewer executes in the reviewer shell:
/// normalized route → reviewer accessible → dispatch create → approve → export.
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
    let (reviewer_status, reviewer_html) =
        get_html(make_app(&tmp), "/reviewer").await;

    assert_eq!(reviewer_status, StatusCode::OK, "step 2 reviewer shell unreachable");
    assert!(
        reviewer_html.contains("/pilot/route-normalized"),
        "reviewer HTML must reference the normalized pilot endpoint"
    );
    assert!(
        reviewer_html.contains("/dispatch/create"),
        "reviewer HTML must reference the dispatch create endpoint"
    );

    // ── Step 3: create dispatch from the normalized receipt ───────────────────
    //
    // The reviewer shell sends: receipt (from step 1) + full CaseInput
    // (examples/pilot/case.json) + derived_policy (from step 1).
    // The normalized pilot case and the full CaseInput share the same
    // case_fingerprint — verified by the kernel before dispatch is created.
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
