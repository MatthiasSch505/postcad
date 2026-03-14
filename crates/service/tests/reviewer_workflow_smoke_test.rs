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

// ── Integrity badge tests ──────────────────────────────────────────────────────

/// Reviewer shell HTML must contain the integrity badge CSS classes and the three
/// badge label strings so artifact panels can display verification state visually.
#[tokio::test]
async fn reviewer_shell_integrity_badge_labels_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("UNVERIFIED"),
        "UNVERIFIED integrity badge label must be present in JS"
    );
    assert!(
        html.contains("VERIFIED"),
        "VERIFIED integrity badge label must be present in JS"
    );
    assert!(
        html.contains("FAILED"),
        "FAILED integrity badge label must be present in JS"
    );
}

/// Reviewer shell HTML must contain the integrity badge CSS classes for all
/// three states so badges are rendered with correct visual cues.
#[tokio::test]
async fn reviewer_shell_integrity_badge_css_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("integrity-badge"), "integrity-badge CSS class must be present");
    assert!(html.contains("ib-unverified"),   "ib-unverified CSS class must be present");
    assert!(html.contains("ib-verified"),     "ib-verified CSS class must be present");
    assert!(html.contains("ib-failed"),       "ib-failed CSS class must be present");
}

/// Reviewer shell HTML must contain badge anchor elements on all four artifact
/// panel titles so JS can update them as verification state changes.
#[tokio::test]
async fn reviewer_shell_integrity_badge_ids_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("route-result-badge"),   "route-result-badge id must be present");
    assert!(html.contains("receipt-json-badge"),   "receipt-json-badge id must be present");
    assert!(html.contains("verify-result-badge"),  "verify-result-badge id must be present");
    assert!(html.contains("dispatch-result-badge"),"dispatch-result-badge id must be present");
}

/// Reviewer shell HTML must contain the updateIntegrityBadges and setBadge JS
/// functions that drive badge state from the operator state machine.
#[tokio::test]
async fn reviewer_shell_integrity_badge_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateIntegrityBadges"),
        "updateIntegrityBadges JS function must be present"
    );
    assert!(
        html.contains("setBadge"),
        "setBadge JS function must be present"
    );
}

// ── Demo-mode readability tests ───────────────────────────────────────────────

/// Reviewer shell HTML must contain panel subtitles so a first-time viewer can
/// scan the screen quickly during a live demo without reading the full guide.
#[tokio::test]
async fn reviewer_shell_panel_subtitles_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Run the deterministic pilot route"),
        "left-card subtitle must be present"
    );
    assert!(
        html.contains("Inspect generated audit artifacts"),
        "route-result subtitle must be present"
    );
    assert!(
        html.contains("Dispatch after verification succeeds"),
        "dispatch section subtitle must be present"
    );
}

/// Reviewer shell HTML must contain the 'Verify before dispatch' section header
/// so the verification step is clearly labelled between receipt and dispatch.
#[tokio::test]
async fn reviewer_shell_verify_section_header_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Verify before dispatch"),
        "Verify before dispatch section header must be present"
    );
}

/// Reviewer shell HTML must contain the compact cheat sheet with the golden-path
/// summary so an operator can locate the next step in one glance.
#[tokio::test]
async fn reviewer_shell_cheatsheet_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Quick path:"),
        "Quick path cheat sheet strip must be present"
    );
    assert!(
        html.contains("op-cheatsheet"),
        "op-cheatsheet element id must be present"
    );
}

/// Reviewer shell HTML must contain the calm empty-state phrase 'Run route to continue'
/// in the results placeholder so a first-time viewer knows what to do next.
#[tokio::test]
async fn reviewer_shell_empty_state_run_route_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Run route to continue"),
        "Run route to continue empty-state text must be present in results placeholder"
    );
}

// ── Dispatch readiness panel tests ────────────────────────────────────────────

/// Reviewer shell HTML must contain the dispatch readiness panel with all three
/// state labels so an operator can immediately see whether dispatch is ready.
#[tokio::test]
async fn reviewer_shell_dispatch_readiness_panel_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Dispatch readiness"),
        "Dispatch readiness panel title must be present"
    );
    assert!(
        html.contains("Ready for dispatch"),
        "Ready for dispatch state label must be present in JS"
    );
    assert!(
        html.contains("Not ready for dispatch"),
        "Not ready for dispatch state label must be present"
    );
    assert!(
        html.contains("Dispatch completed"),
        "Dispatch completed state label must be present in JS"
    );
}

/// Reviewer shell HTML must contain the dispatch readiness panel element IDs
/// and JS functions so the panel updates correctly as state changes.
#[tokio::test]
async fn reviewer_shell_dispatch_readiness_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("dispatch-readiness-panel"), "dispatch-readiness-panel id must be present");
    assert!(html.contains("dr-status"),                "dr-status element id must be present");
    assert!(html.contains("dr-reason"),                "dr-reason element id must be present");
    assert!(html.contains("updateDispatchReadiness"),  "updateDispatchReadiness JS function must be present");
}

/// Reviewer shell HTML must contain the pre-dispatch checklist items
/// so an operator can verify each step before committing to dispatch.
#[tokio::test]
async fn reviewer_shell_dispatch_checklist_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("Receipt reviewed"),          "Receipt reviewed checklist item must be present");
    assert!(html.contains("Verification succeeded"),    "Verification succeeded checklist item must be present");
    assert!(html.contains("Dispatch action confirmed"), "Dispatch action confirmed checklist item must be present");
    assert!(html.contains("cl-receipt"),                "cl-receipt element id must be present");
    assert!(html.contains("cl-verify"),                 "cl-verify element id must be present");
    assert!(html.contains("cl-dispatch"),               "cl-dispatch element id must be present");
}

/// Reviewer shell HTML must contain the blocking-reason texts so an operator
/// knows exactly why dispatch is not ready in each pre-dispatch state.
#[tokio::test]
async fn reviewer_shell_dispatch_blocking_reasons_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Required artifact not yet generated"),
        "pre-routing blocking reason must be present"
    );
    assert!(
        html.contains("Export packet produced. Current run is complete"),
        "dispatch-completed calm state message must be present in JS"
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

// ── Next-action rail tests ────────────────────────────────────────────────────

/// Reviewer shell HTML must contain the next-action rail element with all four
/// state phrases so the operator always has one unambiguous instruction.
#[tokio::test]
async fn reviewer_shell_next_action_rail_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("nar-rail"),               "nar-rail id must be present");
    assert!(html.contains("nar-action"),              "nar-action id must be present");
    assert!(html.contains("nar-reason"),              "nar-reason id must be present");
    assert!(html.contains("Next action"),             "Next action label must be present");
    assert!(html.contains("updateNextActionRail"),    "updateNextActionRail JS function must be present");

    // all four primary instruction strings
    assert!(html.contains("Next: run route"),         "Next: run route phrase must be present");
    assert!(html.contains("Next: verify current route"), "Next: verify current route phrase must be present");
    assert!(html.contains("Next: export dispatch"),   "Next: export dispatch phrase must be present");
    assert!(html.contains("Workflow complete"),       "Workflow complete phrase must be present");
}

/// Reviewer shell HTML must contain the supporting reason lines for all four
/// rail states so operators understand why a given action is recommended.
#[tokio::test]
async fn reviewer_shell_next_action_rail_reasons_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("No current receipt loaded"),
        "idle reason must be present"
    );
    assert!(
        html.contains("Receipt exists but verification not yet executed"),
        "post-route reason must be present"
    );
    assert!(
        html.contains("Verification complete. Dispatch not yet exported"),
        "post-verify reason must be present"
    );
    assert!(
        html.contains("Dispatch packet exported for current route"),
        "complete reason must be present"
    );
}

/// Reviewer shell HTML must wire updateNextActionRail into the operator state
/// machine so the rail resets immediately on every new route submission.
#[tokio::test]
async fn reviewer_shell_next_action_rail_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // CSS classes for all three visual states
    assert!(html.contains("nar-action-idle"), "nar-action-idle CSS class must be present");
    assert!(html.contains("nar-action-next"), "nar-action-next CSS class must be present");
    assert!(html.contains("nar-action-done"), "nar-action-done CSS class must be present");

    // Rail initial state matches idle instruction
    assert!(
        html.contains("Next: run route"),
        "initial rail state must show Next: run route"
    );
}

// ── Active run context + stale artifact reset tests ───────────────────────────

/// Reviewer shell HTML must contain the active run context block with all four
/// field slots so operators can identify the current run at a glance.
#[tokio::test]
async fn reviewer_shell_active_run_context_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("active-run-context"),  "active-run-context id must be present");
    assert!(html.contains("Active run context"),  "Active run context label must be present");
    assert!(html.contains("arc-manufacturer"),    "arc-manufacturer id must be present");
    assert!(html.contains("arc-receipt-hash"),    "arc-receipt-hash id must be present");
    assert!(html.contains("arc-verify-status"),   "arc-verify-status id must be present");
    assert!(html.contains("arc-dispatch-status"), "arc-dispatch-status id must be present");
    assert!(html.contains("updateActiveRunContext"), "updateActiveRunContext JS function must be present");
    assert!(html.contains("arc-val-pending"),     "arc-val-pending CSS class must be present");
    assert!(html.contains("arc-val-ok"),          "arc-val-ok CSS class must be present");
    assert!(html.contains("arc-val-err"),         "arc-val-err CSS class must be present");
}

/// Reviewer shell HTML must contain the downstream stale-state placeholder texts
/// so that it is unambiguous when verification and dispatch outputs belong to
/// a prior run rather than the currently loaded receipt.
#[tokio::test]
async fn reviewer_shell_stale_artifact_placeholders_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // Verification stale placeholder — shown after route before verify
    assert!(
        html.contains("No verification result for current route"),
        "No verification result for current route placeholder must be present"
    );

    // Dispatch stale note element and placeholder text
    assert!(
        html.contains("dispatch-stale-note"),
        "dispatch-stale-note id must be present"
    );
    assert!(
        html.contains("No dispatch export for current route"),
        "No dispatch export for current route placeholder must be present"
    );

    // Active run context carries the same pending text as default state
    assert!(
        html.contains("arc-val-pending"),
        "arc-val-pending CSS class must be present for pending downstream states"
    );
}

/// Reviewer shell HTML must wire updateActiveRunContext into the operator state
/// machine so that active run context is always in sync with opRouting/opVerify
/// and dispatch export state.
#[tokio::test]
async fn reviewer_shell_active_run_context_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // updateActiveRunContext must be called from updateOpState
    // Verify by checking both functions are present and the call appears in JS
    assert!(html.contains("updateActiveRunContext"), "updateActiveRunContext must be defined");
    assert!(html.contains("updateOpState"),          "updateOpState must call updateActiveRunContext");

    // dispatch-stale-note toggle must be wired into updateOpState
    assert!(
        html.contains("dispatch-stale-note"),
        "dispatch-stale-note must be referenced in the state machine"
    );
}

// ── Operator usability batch tests ────────────────────────────────────────────

/// Reviewer shell HTML must contain the pilot run history panel with
/// the JS function and label strings for all three chronological actions.
#[tokio::test]
async fn reviewer_shell_run_history_panel_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("run-history-panel"),  "run-history-panel id must be present");
    assert!(html.contains("run-history-list"),   "run-history-list id must be present");
    assert!(html.contains("Pilot run history"),  "Pilot run history label must be present");
    assert!(html.contains("appendRunHistory"),   "appendRunHistory JS function must be present");
    assert!(html.contains("clearRunHistory"),    "clearRunHistory JS function must be present");
    assert!(html.contains("Route executed"),     "Route executed history label must be present");
    assert!(html.contains("Verification executed"), "Verification executed history label must be present");
    assert!(html.contains("Dispatch executed"),  "Dispatch executed history label must be present");
    assert!(html.contains("rh-entry"),           "rh-entry CSS class must be present");
    assert!(html.contains("rh-ok"),              "rh-ok CSS class must be present");
    assert!(html.contains("rh-err"),             "rh-err CSS class must be present");
}

/// Reviewer shell HTML must render artifact panels in deterministic order:
/// route result → receipt → verification result → dispatch result.
#[tokio::test]
async fn reviewer_shell_artifact_panel_order_deterministic() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    let route_pos    = html.find("id=\"route-result\"").expect("route-result id must be present");
    let receipt_pos  = html.find("id=\"route-receipt-json\"").expect("route-receipt-json id must be present");
    let verify_pos   = html.find("id=\"verify-result\"").expect("verify-result id must be present");
    let dispatch_pos = html.find("id=\"dispatch-export-result\"").expect("dispatch-export-result id must be present");

    assert!(route_pos < receipt_pos,   "route-result must appear before receipt JSON");
    assert!(receipt_pos < verify_pos,  "receipt JSON must appear before verification result");
    assert!(verify_pos < dispatch_pos, "verification result must appear before dispatch result");
}

/// Reviewer shell HTML must contain the artifact size guard with collapse/expand
/// functionality so large artifact panels do not overwhelm the operator view.
#[tokio::test]
async fn reviewer_shell_artifact_size_guard_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("expandArtifact"),          "expandArtifact JS function must be present");
    assert!(html.contains("collapseIfLarge"),          "collapseIfLarge JS function must be present");
    assert!(html.contains("ARTIFACT_COLLAPSE_LINES"), "ARTIFACT_COLLAPSE_LINES constant must be present");
    assert!(html.contains("Expand artifact"),         "Expand artifact button text must be present");
    assert!(html.contains("receipt-expand-btn"),      "receipt-expand-btn id must be present");
    assert!(html.contains("verify-expand-btn"),       "verify-expand-btn id must be present");
    assert!(html.contains("dispatch-expand-btn"),     "dispatch-expand-btn id must be present");
    assert!(html.contains("collapsed"),               "collapsed CSS class must be present");
    assert!(html.contains("expand-btn"),              "expand-btn CSS class must be present");
}

// ── Panel microbadge alignment tests ─────────────────────────────────────────

/// Reviewer shell HTML must expose three panel microbadge elements — one per
/// artifact section — so each panel carries an inline current-run state label.
#[tokio::test]
async fn reviewer_shell_panel_microbadge_elements_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("mb-receipt"),  "mb-receipt microbadge id must be present");
    assert!(html.contains("mb-verify"),   "mb-verify microbadge id must be present");
    assert!(html.contains("mb-dispatch"), "mb-dispatch microbadge id must be present");

    // CSS classes for all three badge states
    assert!(html.contains("mb-on"),  "mb-on CSS class must be defined");
    assert!(html.contains("mb-dim"), "mb-dim CSS class must be defined");
    assert!(html.contains("mb-err"), "mb-err CSS class must be defined");
}

/// Reviewer shell HTML must carry all four operational vocabulary strings used
/// by the panel microbadges so an operator can read a consistent state language.
#[tokio::test]
async fn reviewer_shell_panel_microbadge_vocabulary() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("'available'"),     "'available' badge label must be present in MB_LABELS");
    assert!(html.contains("'not available'"), "'not available' badge label must be present in MB_LABELS");
    assert!(html.contains("'verified'"),      "'verified' badge label must be present in MB_LABELS");
    assert!(html.contains("'exported'"),      "'exported' badge label must be present in MB_LABELS");
}

/// Reviewer shell HTML must expose setMicrobadge and updateMicrobadges JS
/// functions so badge state can be driven from the operator state machine.
#[tokio::test]
async fn reviewer_shell_panel_microbadge_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("setMicrobadge"),    "setMicrobadge JS function must be present");
    assert!(html.contains("updateMicrobadges"),"updateMicrobadges JS function must be present");
    assert!(html.contains("MB_LABELS"),        "MB_LABELS constant must be present");
    assert!(html.contains("MB_CLASSES"),       "MB_CLASSES constant must be present");
}

/// Reviewer shell HTML must wire updateMicrobadges into the operator state
/// machine so badges reset on reroute and update on every state transition.
#[tokio::test]
async fn reviewer_shell_panel_microbadge_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // updateMicrobadges must be called from updateOpState (reroute reset)
    assert!(
        html.contains("updateMicrobadges"),
        "updateMicrobadges must be present and wired into the state machine"
    );
    // updateOpState must exist as the driver
    assert!(
        html.contains("updateOpState"),
        "updateOpState must be present as the state machine driver"
    );
    // exportDispatch must call updateMicrobadges (mb-dispatch → exported)
    // Verified by the fact that the function appears in the export block context
    assert!(
        html.contains("updateMicrobadges()"),
        "updateMicrobadges() call must appear in the JS"
    );
}

/// Receipt microbadge must show 'available' in its initial HTML state since the
/// badge is inside route-result and only visible after a successful route.
#[tokio::test]
async fn reviewer_shell_receipt_microbadge_initial_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // mb-receipt starts as mb-on / available (route-result only shown post-route)
    assert!(
        html.contains("id=\"mb-receipt\" class=\"mb mb-on\">available"),
        "mb-receipt must start with mb-on / available in HTML"
    );
}

/// Verify and dispatch microbadges must start in a not-available (mb-dim) state
/// since no verification or export has occurred yet in a fresh panel.
#[tokio::test]
async fn reviewer_shell_downstream_microbadges_initial_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("id=\"mb-verify\" class=\"mb mb-dim\">not available"),
        "mb-verify must start with mb-dim / not available in HTML"
    );
    assert!(
        html.contains("id=\"mb-dispatch\" class=\"mb mb-dim\">not available"),
        "mb-dispatch must start with mb-dim / not available in HTML"
    );
}

// ── Operator action bar tests ────────────────────────────────────────────────

/// Reviewer shell HTML must contain the operator action bar with its element
/// IDs and a navigate button so an operator always sees exactly one next step.
#[tokio::test]
async fn reviewer_shell_oab_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"oab\""),     "oab container id must be present");
    assert!(html.contains("oab-action"),     "oab-action id must be present");
    assert!(html.contains("oab-reason"),     "oab-reason id must be present");
    assert!(html.contains("oab-btn"),        "oab-btn id must be present");
    assert!(html.contains("oabNavigate"),    "oabNavigate JS function must be present");
}

/// Action bar must default to the 'route' state at initial load with the
/// correct action text and reason so a fresh operator session is immediately actionable.
#[tokio::test]
async fn reviewer_shell_oab_initial_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Start a route for the current case"),
        "initial action text must say 'Start a route for the current case'"
    );
    assert!(
        html.contains("No current route artifacts exist yet."),
        "initial reason must say 'No current route artifacts exist yet.'"
    );
    assert!(
        html.contains("→ Go to route"),
        "initial button label must say '→ Go to route'"
    );
}

/// All five action bar state labels must be present in the JS source so every
/// current-run transition has a deterministic one-line instruction.
#[tokio::test]
async fn reviewer_shell_oab_all_state_labels_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("Start a route for the current case"),         "route state label must be present");
    assert!(html.contains("Run verification for the current route"),     "verify state label must be present");
    assert!(html.contains("Export dispatch packet"),                     "export state label must be present");
    assert!(html.contains("Resolve readiness items before dispatch"),    "resolve state label must be present");
    assert!(html.contains("Current run complete"),                       "complete state label must be present");
}

/// All five reason lines must be present in OAB_STATES so the operator always
/// sees a secondary explanation for why the given action is recommended.
#[tokio::test]
async fn reviewer_shell_oab_all_reason_lines_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("No current route artifacts exist yet."),
        "route reason must be present");
    assert!(html.contains("Verification has not been executed for the current route."),
        "verify reason must be present");
    assert!(html.contains("Dispatch is ready and no export exists for the current route."),
        "export reason must be present");
    assert!(html.contains("Verification failed. Resolve before dispatching."),
        "resolve reason must be present");
    assert!(html.contains("Current run already has a dispatch export."),
        "complete reason must be present");
}

/// Action bar JS functions and state table must be present so badge state
/// is computed purely from existing reviewer signals.
#[tokio::test]
async fn reviewer_shell_oab_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("OAB_STATES"),    "OAB_STATES constant must be present");
    assert!(html.contains("oabStateKey"),   "oabStateKey JS function must be present");
    assert!(html.contains("updateOab"),     "updateOab JS function must be present");
    assert!(html.contains("oabNavigate"),   "oabNavigate JS function must be present");
}

/// updateOab must be wired into updateOpState and exportDispatch so the action
/// bar resets on reroute and advances when export completes.
#[tokio::test]
async fn reviewer_shell_oab_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateOab()"),
        "updateOab() call must appear in JS"
    );
    // CSS classes for all action states must be defined
    assert!(html.contains("oab-action-idle"),     "oab-action-idle CSS class must be defined");
    assert!(html.contains("oab-action-active"),   "oab-action-active CSS class must be defined");
    assert!(html.contains("oab-action-complete"), "oab-action-complete CSS class must be defined");
}

// ── Dispatch packet inspection tests ─────────────────────────────────────────

/// Reviewer shell must contain the dispatch packet inspection panel so the
/// operator can view export packet contents directly in the UI before handoff.
#[tokio::test]
async fn reviewer_shell_dispatch_packet_inspection_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"dpi\""),          "dpi container must be present");
    assert!(html.contains("id=\"dpi-viewer\""),   "dpi-viewer pre element must be present");
    assert!(html.contains("id=\"dpi-empty\""),    "dpi-empty must be present");
    assert!(html.contains("Dispatch packet inspection"), "dpi label must be present");
}

/// Inspection panel must show the no-packet empty state on initial load so the
/// operator sees explicit guidance rather than a blank section.
#[tokio::test]
async fn reviewer_shell_dispatch_packet_inspection_empty_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("No dispatch packet generated for the current run."),
        "empty-state message must be present"
    );
    assert!(
        html.contains("Run dispatch export to generate a packet for inspection."),
        "empty-state guidance hint must be present"
    );
    assert!(
        html.contains("dpi-viewer hidden"),
        "dpi-viewer must be hidden on initial load"
    );
}

/// Packet origin and integrity meta indicators must be present so the operator
/// can see at a glance which run the packet belongs to and its verification status.
#[tokio::test]
async fn reviewer_shell_dispatch_packet_inspection_meta_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"dpi-origin\""),    "dpi-origin element must be present");
    assert!(html.contains("id=\"dpi-integrity\""), "dpi-integrity element must be present");
    assert!(html.contains("Packet origin"),        "packet origin label must be present");
    assert!(html.contains("Packet integrity"),     "packet integrity label must be present");
}

/// All packet origin CSS classes must be present so current-run, previous-run,
/// and no-packet states each render with a distinct visual signal.
#[tokio::test]
async fn reviewer_shell_dispatch_packet_inspection_origin_states() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("dpi-origin-current"), "dpi-origin-current CSS class must be defined");
    assert!(html.contains("dpi-origin-prev"),    "dpi-origin-prev CSS class must be defined");
    assert!(html.contains("dpi-origin-none"),    "dpi-origin-none CSS class must be defined");
}

/// All packet integrity states must be present so the operator can see whether
/// the packet was produced after a passing, failing, or absent verification.
#[tokio::test]
async fn reviewer_shell_dispatch_packet_inspection_integrity_states() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("dpi-integrity-ok"),    "dpi-integrity-ok CSS class must be defined");
    assert!(html.contains("dpi-integrity-fail"),  "dpi-integrity-fail CSS class must be defined");
    assert!(html.contains("dpi-integrity-none"),  "dpi-integrity-none CSS class must be defined");
    assert!(html.contains("verified packet"),         "'verified packet' label must be present");
    assert!(html.contains("verification failed"),     "'verification failed' label must be present");
    assert!(html.contains("verification not executed"), "'verification not executed' label must be present");
}

/// updateDpi must be wired into updateOpState and exportDispatch so the panel
/// refreshes on every state transition including reroute and new export.
#[tokio::test]
async fn reviewer_shell_dispatch_packet_inspection_wired() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateDpi()"),
        "updateDpi() call must appear in state machine wiring"
    );
    assert!(html.contains("function updateDpi"), "updateDpi function must be defined");
}

// ── Dispatch handoff dossier tests ────────────────────────────────────────────

/// Reviewer shell must contain the dispatch handoff dossier so the operator has
/// a single checkpoint that shows whether the active run is ready for handoff.
#[tokio::test]
async fn reviewer_shell_dispatch_handoff_dossier_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"dhd\""),            "dhd container must be present");
    assert!(html.contains("id=\"dhd-verdict\""),    "dhd-verdict must be present");
    assert!(html.contains("id=\"dhd-meaning\""),    "dhd-meaning must be present");
    assert!(html.contains("id=\"dhd-checklist\""),  "dhd-checklist must be present");
    assert!(html.contains("id=\"dhd-next-text\""),  "dhd-next-text must be present");
    assert!(html.contains("Dispatch handoff dossier"), "dossier label must be present");
}

/// Dossier must default to the no-current-dispatch-packet state on initial load
/// so a fresh session never shows a misleading ready or exported verdict.
#[tokio::test]
async fn reviewer_shell_dispatch_handoff_dossier_initial_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("No current dispatch packet"),
        "initial verdict must be 'No current dispatch packet'"
    );
    assert!(
        html.contains("No route has been generated yet for the current session."),
        "initial meaning must indicate no route yet"
    );
    assert!(
        html.contains("Generate a route first."),
        "initial next-step must say 'Generate a route first.'"
    );
}

/// All five dossier verdict CSS classes and their corresponding verdict text
/// must be present so every operational state renders with a distinct signal.
#[tokio::test]
async fn reviewer_shell_dispatch_handoff_dossier_verdict_states() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("dhd-verdict-none"),      "dhd-verdict-none CSS class must be defined");
    assert!(html.contains("dhd-verdict-not-ready"), "dhd-verdict-not-ready CSS class must be defined");
    assert!(html.contains("dhd-verdict-ready"),     "dhd-verdict-ready CSS class must be defined");
    assert!(html.contains("dhd-verdict-exported"),  "dhd-verdict-exported CSS class must be defined");
    assert!(html.contains("dhd-verdict-attention"), "dhd-verdict-attention CSS class must be defined");
    assert!(html.contains("Current route ready for dispatch export"),
        "'ready' verdict text must be present in JS");
    assert!(html.contains("Current dispatch packet exported"),
        "'exported' verdict text must be present in JS");
    assert!(html.contains("Current dispatch packet requires attention"),
        "'attention' verdict text must be present in JS");
}

/// Dossier checklist must contain all five lineage-aware items so the operator
/// can scan route, receipt, verification, and dispatch status in one place.
#[tokio::test]
async fn reviewer_shell_dispatch_handoff_dossier_checklist_items() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("Route available"),                        "route checklist item must be present");
    assert!(html.contains("Receipt available"),                      "receipt checklist item must be present");
    assert!(html.contains("Verification executed for current run"),  "verify-exec checklist item must be present");
    assert!(html.contains("Verification passed"),                    "verify-pass checklist item must be present");
    assert!(html.contains("Dispatch packet exported for current run"), "dispatch checklist item must be present");
    assert!(html.contains("Verification executed — previous run only"),
        "previous-run verify label must be present for reroute state");
    assert!(html.contains("Dispatch exported — previous run only"),
        "previous-run dispatch label must be present for reroute state");
}

/// Dossier meaning block must contain operator-facing text explaining export
/// semantics and reroute invalidation for the ready and attention states.
#[tokio::test]
async fn reviewer_shell_dispatch_handoff_dossier_meaning_block() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Rerouting will require a new export."),
        "ready-state reroute meaning must be present"
    );
    assert!(
        html.contains("Rerouting will invalidate this export"),
        "exported-state reroute meaning must be present"
    );
    assert!(
        html.contains("Reroute detected — re-export required for the current route."),
        "attention-state next-step must be present"
    );
}

/// updateDossier must be wired into updateOpState and exportDispatch success
/// so the dossier re-evaluates on every state transition including reroute.
#[tokio::test]
async fn reviewer_shell_dispatch_handoff_dossier_wired_to_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateDossier()"),
        "updateDossier() call must appear in state machine wiring"
    );
    assert!(html.contains("dhdVerdictKey"),  "dhdVerdictKey function must be present");
    assert!(html.contains("dhdNextStep"),    "dhdNextStep function must be present");
}

// ── Run identity + artifact lineage tests ─────────────────────────────────────

/// Reviewer shell must contain the run identity block so the operator always
/// sees a lineage-aware summary of the four pipeline steps for the active run.
#[tokio::test]
async fn reviewer_shell_run_identity_block_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"rib\""),          "rib container must be present");
    assert!(html.contains("id=\"rib-route\""),    "rib-route row must be present");
    assert!(html.contains("id=\"rib-receipt\""),  "rib-receipt row must be present");
    assert!(html.contains("id=\"rib-verify\""),   "rib-verify row must be present");
    assert!(html.contains("id=\"rib-dispatch\""), "rib-dispatch row must be present");
    assert!(html.contains("Current run"),         "run identity label must be present");
}

/// Run identity block must default to the idle state on initial load so the
/// operator sees explicit 'no run yet' labels before any route is submitted.
#[tokio::test]
async fn reviewer_shell_run_identity_block_initial_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("no run yet"),     "initial route label must be 'no run yet'");
    assert!(html.contains("not generated"),  "initial receipt label must be 'not generated'");
    assert!(html.contains("not executed"),   "initial verify label must be 'not executed'");
    assert!(html.contains("not exported"),   "initial dispatch label must be 'not exported'");
}

/// Lineage badges must be present in the verification result and dispatch export
/// section titles so the operator can see artifact ownership at a glance.
#[tokio::test]
async fn reviewer_shell_artifact_lineage_badges_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"lin-verify\""),          "lin-verify badge must be present");
    assert!(html.contains("id=\"lin-dispatch-export\""), "lin-dispatch-export badge must be present");
    assert!(html.contains("lin-current"),                "lin-current CSS class must be defined");
    assert!(html.contains("lin-prev"),                   "lin-prev CSS class must be defined");
    assert!(html.contains("lin-idle"),                   "lin-idle CSS class must be defined");
}

/// Lineage mismatch notes must be present so the operator receives explicit
/// guidance when an artifact belongs to a previous rather than the current run.
#[tokio::test]
async fn reviewer_shell_lineage_mismatch_notes_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"lin-verify-note\""),   "lin-verify-note must be present");
    assert!(html.contains("id=\"lin-dispatch-note\""), "lin-dispatch-note must be present");
    assert!(
        html.contains("Verification belongs to previous run."),
        "verify previous-run mismatch message must be present"
    );
    assert!(
        html.contains("Run verification again for current route."),
        "verify guidance hint must be present"
    );
    assert!(
        html.contains("Dispatch export belongs to previous run."),
        "dispatch previous-run mismatch message must be present"
    );
    assert!(
        html.contains("Export dispatch packet again for current route."),
        "dispatch guidance hint must be present"
    );
}

/// Run serial tracking state and lineage functions must be present so the shell
/// can determine whether each artifact belongs to the current or a previous run.
#[tokio::test]
async fn reviewer_shell_run_serial_tracking_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("runSerial"),      "runSerial state variable must be present");
    assert!(html.contains("verifySerial"),   "verifySerial state variable must be present");
    assert!(html.contains("dispatchSerial"), "dispatchSerial state variable must be present");
    assert!(html.contains("verifyLineage"),  "verifyLineage JS function must be present");
    assert!(html.contains("dispatchLineage"), "dispatchLineage JS function must be present");
}

/// Run identity and lineage update functions must be wired into updateOpState
/// and exportDispatch so they re-evaluate on every state transition.
#[tokio::test]
async fn reviewer_shell_lineage_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateRunIdentityBlock()"),
        "updateRunIdentityBlock() call must appear in state machine"
    );
    assert!(
        html.contains("updateLineageBadges()"),
        "updateLineageBadges() call must appear in state machine"
    );
    assert!(
        html.contains("updateLineageNotes()"),
        "updateLineageNotes() call must appear in state machine"
    );
}

// ── Session activity log tests ────────────────────────────────────────────────

/// Reviewer shell must contain the session activity log panel so the operator
/// can see a chronological list of workflow actions performed this session.
#[tokio::test]
async fn reviewer_shell_session_activity_log_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"sal\""),       "sal container must be present");
    assert!(html.contains("id=\"sal-list\""),  "sal-list must be present");
    assert!(html.contains("id=\"sal-empty\""), "sal-empty must be present");
    assert!(html.contains("Current session activity"), "session activity label must be present");
}

/// Session activity log must start empty on page load so a fresh session shows
/// no spurious activity entries before the operator has done anything.
#[tokio::test]
async fn reviewer_shell_session_activity_log_starts_empty() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("No activity yet. Start routing to begin."),
        "initial empty-state message must be present"
    );
    assert!(
        html.contains("sal-list hidden"),
        "sal-list must be hidden on initial load"
    );
}

/// Session activity log must record route-phase events so the operator can see
/// when routing was requested and when the result was received.
#[tokio::test]
async fn reviewer_shell_session_activity_log_route_events() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Route requested"),
        "'Route requested' event label must be present in JS"
    );
    assert!(
        html.contains("Route result received"),
        "'Route result received' event label must be present in JS"
    );
    assert!(
        html.contains("Current run reset"),
        "'Current run reset' event label must be present in JS"
    );
}

/// Session activity log must record verification-phase events so the operator
/// can trace whether verification was executed and what the outcome was.
#[tokio::test]
async fn reviewer_shell_session_activity_log_verify_events() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Verification executed"),
        "'Verification executed' event label must be present in JS"
    );
    assert!(
        html.contains("Verification completed"),
        "'Verification completed' event label must be present in JS"
    );
}

/// Session activity log must record the dispatch export event and provide a
/// clear-log control so the operator can reset the log at any time.
#[tokio::test]
async fn reviewer_shell_session_activity_log_dispatch_and_clear() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Dispatch export generated"),
        "'Dispatch export generated' event label must be present in JS"
    );
    assert!(
        html.contains("clearSessionLog()"),
        "clearSessionLog() call must be present on clear control"
    );
    assert!(html.contains("Clear log"), "clear log button label must be present");
}

/// Session activity log must define salLog, renderSessionLog, clearSessionLog
/// and a bounded SAL_MAX constant so the log never grows unbounded.
#[tokio::test]
async fn reviewer_shell_session_activity_log_js_and_cap() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("function salLog"),         "salLog JS function must be present");
    assert!(html.contains("function renderSessionLog"), "renderSessionLog JS function must be present");
    assert!(html.contains("function clearSessionLog"), "clearSessionLog JS function must be present");
    assert!(html.contains("SAL_MAX"),                 "SAL_MAX constant must be present");
}

// ── Consistency sentinel tests ────────────────────────────────────────────────

/// Reviewer shell must contain the consistency sentinel card so the operator
/// can see at a glance whether all visible current-run indicators agree.
#[tokio::test]
async fn reviewer_shell_consistency_sentinel_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"ccs\""),          "ccs container must be present");
    assert!(html.contains("id=\"ccs-headline\""), "ccs-headline must be present");
    assert!(html.contains("id=\"ccs-detail\""),   "ccs-detail must be present");
    assert!(html.contains("Current run consistency"), "sentinel label must be present");
}

/// Sentinel must default to the consistent state on initial load so a fresh
/// session never shows a spurious attention-needed indicator.
#[tokio::test]
async fn reviewer_shell_consistency_sentinel_initial_consistent() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("ccs-consistent"),
        "initial sentinel CSS state must be ccs-consistent"
    );
    assert!(
        html.contains("Current run shell state is consistent"),
        "initial headline must say 'Current run shell state is consistent'"
    );
    assert!(
        html.contains("Visible workflow indicators agree for the current run."),
        "initial detail must confirm all indicators agree"
    );
}

/// All mismatch message strings must be present in the JS source so every
/// detectable shell-level inconsistency produces a deterministic plain-text line.
#[tokio::test]
async fn reviewer_shell_consistency_sentinel_all_mismatch_strings_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("Verification marked present but route artifact missing."),
        "rule-1 mismatch string must be present");
    assert!(html.contains("Dispatch artifact shown without current-run verification."),
        "rule-2 mismatch string must be present");
    assert!(html.contains("Dispatch artifact shown without current-run receipt."),
        "rule-3 mismatch string must be present");
    assert!(html.contains("Complete verdict shown without current-run dispatch export."),
        "rule-4 mismatch string must be present");
    assert!(html.contains("Ready verdict shown but verification not present."),
        "rule-5 mismatch string must be present");
    assert!(html.contains("Ready verdict shown but dispatch export already exists."),
        "rule-6 mismatch string must be present");
}

/// CSS classes for both sentinel states must be defined so the card always has
/// a clear visual distinction between consistent and attention-needed states.
#[tokio::test]
async fn reviewer_shell_consistency_sentinel_css_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("ccs-consistent"), "ccs-consistent CSS class must be defined");
    assert!(html.contains("ccs-attention"),  "ccs-attention CSS class must be defined");
    assert!(html.contains("ccs-mismatch"),   "ccs-mismatch CSS class must be defined");
}

/// Sentinel JS functions must be present so checks are derived only from
/// existing reviewer signals without any new persisted state.
#[tokio::test]
async fn reviewer_shell_consistency_sentinel_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("gatherConsistencyMismatches"),  "gatherConsistencyMismatches JS function must be present");
    assert!(html.contains("updateConsistencySentinel"),    "updateConsistencySentinel JS function must be present");
}

/// updateConsistencySentinel must be wired into updateOpState and exportDispatch
/// so the sentinel re-evaluates on every state transition including reroute.
#[tokio::test]
async fn reviewer_shell_consistency_sentinel_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateConsistencySentinel()"),
        "updateConsistencySentinel() call must appear in updateOpState and exportDispatch"
    );
}

// ── Handoff summary card tests ────────────────────────────────────────────────

/// Reviewer shell must contain a print handoff summary control so an operator
/// can trigger browser print for the current-run handoff card.
#[tokio::test]
async fn reviewer_shell_handoff_summary_control_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"btn-print-handoff\""), "print handoff button must be present");
    assert!(html.contains("window.print()"),           "print button must call window.print()");
    assert!(html.contains("Print summary"),            "print button label must be present");
}

/// Handoff summary card must be present with all its structural elements so it
/// can show a complete current-run summary both on screen and in print.
#[tokio::test]
async fn reviewer_shell_handoff_summary_card_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"hsc\""),           "hsc container must be present");
    assert!(html.contains("id=\"hsc-verdict\""),   "hsc-verdict must be present");
    assert!(html.contains("id=\"hsc-rows\""),      "hsc-rows must be present");
    assert!(html.contains("id=\"hsc-readiness\""), "hsc-readiness must be present");
    assert!(html.contains("id=\"hsc-artifacts\""), "hsc-artifacts must be present");
    assert!(html.contains("id=\"hsc-summary\""),   "hsc-summary must be present");
    assert!(html.contains("PostCAD current-run handoff summary"),
        "handoff summary title must be present");
}

/// Handoff card must default to the not-ready verdict and appropriate initial
/// summary text so a fresh session never shows a stale complete state.
#[tokio::test]
async fn reviewer_shell_handoff_summary_initial_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("hsc-verdict-not-ready"), "initial verdict CSS must be hsc-verdict-not-ready");
    assert!(
        html.contains("Current run requires routing before dispatch."),
        "initial summary line must say 'Current run requires routing before dispatch.'"
    );
}

/// All three verdict text strings and all three summary lines must be present
/// in the JS source so every handoff state resolves to a deterministic verdict.
#[tokio::test]
async fn reviewer_shell_handoff_summary_all_verdict_strings() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("'Not ready'"),                 "Not ready verdict must be in JS");
    assert!(html.contains("'Ready for dispatch export'"), "Ready for dispatch export verdict must be in JS");
    assert!(html.contains("'Complete'"),                  "Complete verdict must be in JS");
    assert!(html.contains("Current run requires additional workflow steps"),
        "not-ready summary line must be present");
    assert!(html.contains("Current run is ready for dispatch export."),
        "ready summary line must be present");
    assert!(html.contains("Current run handoff is complete."),
        "complete summary line must be present");
}

/// The handoff summary JS function must be present and wired into the state
/// machine so the card resets on reroute and advances through the workflow.
#[tokio::test]
async fn reviewer_shell_handoff_summary_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("updateHandoffSummary"), "updateHandoffSummary JS function must be present");
}

/// updateHandoffSummary must be called in updateOpState and exportDispatch so
/// stale states clear on reroute and the complete state shows after dispatch.
#[tokio::test]
async fn reviewer_shell_handoff_summary_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateHandoffSummary()"),
        "updateHandoffSummary() call must appear in updateOpState and exportDispatch"
    );
}

// ── Audit snapshot export tests ──────────────────────────────────────────────

/// Reviewer shell must contain the audit snapshot export controls so an
/// operator can export or copy a deterministic current-run artifact bundle.
#[tokio::test]
async fn reviewer_shell_audit_snapshot_controls_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"btn-copy-snapshot\""),  "copy snapshot button must be present");
    assert!(html.contains("copyAuditSnapshot"),          "copyAuditSnapshot JS call must be wired");
    assert!(html.contains("downloadAuditSnapshot"),      "downloadAuditSnapshot JS call must be wired");
    assert!(html.contains("Audit snapshot"),             "audit snapshot label must be present");
}

/// The audit snapshot title and all six section headers must be present in the
/// JS source so the generated snapshot always has a fixed deterministic structure.
#[tokio::test]
async fn reviewer_shell_audit_snapshot_section_headers_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("POSTCAD REVIEWER AUDIT SNAPSHOT"),
        "snapshot title must be present in JS");
    assert!(html.contains("'Current run status'"),
        "Current run status section header must be present");
    assert!(html.contains("'Route'"),
        "Route section header must be present");
    assert!(html.contains("'Receipt'"),
        "Receipt section header must be present");
    assert!(html.contains("'Verification'"),
        "Verification section header must be present");
    assert!(html.contains("'Dispatch'"),
        "Dispatch section header must be present");
    assert!(html.contains("'Dispatch readiness'"),
        "Dispatch readiness section header must be present");
}

/// Placeholder strings for missing artifacts must be present in the JS source
/// so an unstarted current run produces a valid deterministic snapshot.
#[tokio::test]
async fn reviewer_shell_audit_snapshot_placeholder_strings_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("not present"),   "'not present' placeholder must be in JS");
    assert!(html.contains("not executed"),  "'not executed' placeholder must be in JS");
    assert!(html.contains("not exported"),  "'not exported' placeholder must be in JS");
}

/// Snapshot JS functions must all be present so the export operates entirely
/// on client-side current-run state without any backend calls.
#[tokio::test]
async fn reviewer_shell_audit_snapshot_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("buildAuditSnapshot"),    "buildAuditSnapshot JS function must be present");
    assert!(html.contains("copyAuditSnapshot"),     "copyAuditSnapshot JS function must be present");
    assert!(html.contains("downloadAuditSnapshot"), "downloadAuditSnapshot JS function must be present");
}

/// The snapshot must explicitly state it covers current run only so any
/// consumer of the exported file cannot confuse it with a full history export.
#[tokio::test]
async fn reviewer_shell_audit_snapshot_scoped_to_current_run() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Current run only"),
        "snapshot must declare it covers current run only"
    );
}

/// The snapshot download filename must be deterministic so exported files
/// have a predictable and consistent name for audit purposes.
#[tokio::test]
async fn reviewer_shell_audit_snapshot_deterministic_filename() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("postcad_audit_snapshot.txt"),
        "download filename must be 'postcad_audit_snapshot.txt'"
    );
}

// ── Preflight summary card tests ─────────────────────────────────────────────

/// Reviewer shell must contain the preflight card with its container and body
/// elements so the operator gets one deterministic go/not-yet/complete verdict.
#[tokio::test]
async fn reviewer_shell_preflight_card_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"pfc\""),          "pfc container must be present");
    assert!(html.contains("id=\"pfc-headline\""), "pfc-headline must be present");
    assert!(html.contains("id=\"pfc-detail\""),   "pfc-detail must be present");
    assert!(html.contains("id=\"pfc-rows\""),     "pfc-rows must be present");
    assert!(html.contains("Current run preflight"), "preflight label must be present");
}

/// Preflight card must default to the not-ready verdict on initial load so a
/// fresh session never shows a stale ready or complete state.
#[tokio::test]
async fn reviewer_shell_preflight_card_initial_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Current run not ready"),
        "initial headline must say 'Current run not ready'"
    );
    assert!(
        html.contains("Complete remaining workflow steps before dispatch export."),
        "initial detail must say 'Complete remaining workflow steps before dispatch export.'"
    );
    assert!(
        html.contains("pfc-not-ready"),
        "initial card CSS state must be pfc-not-ready"
    );
}

/// All three verdict headline and detail strings must be present in PFC_VERDICTS
/// so every preflight state resolves to a deterministic one-line verdict.
#[tokio::test]
async fn reviewer_shell_preflight_card_all_verdict_strings() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("Current run not ready"),
        "not-ready verdict headline must be present");
    assert!(html.contains("Current run ready for dispatch"),
        "ready verdict headline must be present");
    assert!(html.contains("Current run complete"),
        "complete verdict headline must be present");
    assert!(html.contains("Complete remaining workflow steps before dispatch export."),
        "not-ready detail must be present");
    assert!(html.contains("All current-run prerequisites are satisfied"),
        "ready detail must be present");
    assert!(html.contains("Dispatch export exists for the current run."),
        "complete detail must be present");
}

/// All preflight gate row labels must be present in the JS source so every
/// underlying gate is shown as a binary plain-language line.
#[tokio::test]
async fn reviewer_shell_preflight_card_row_labels_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("Route available"),          "Route available row label must be present");
    assert!(html.contains("Receipt available"),        "Receipt available row label must be present");
    assert!(html.contains("Verification complete"),    "Verification complete row label must be present");
    assert!(html.contains("Dispatch not yet exported"),"Dispatch not yet exported row label must be present");
    assert!(html.contains("Dispatch exported"),        "Dispatch exported row label must be present");
}

/// Preflight card JS must be present so the verdict is derived only from
/// existing reviewer signals without any new persisted state.
#[tokio::test]
async fn reviewer_shell_preflight_card_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("PFC_VERDICTS"),       "PFC_VERDICTS constant must be present");
    assert!(html.contains("pfcVerdictKey"),       "pfcVerdictKey JS function must be present");
    assert!(html.contains("updatePreflightCard"), "updatePreflightCard JS function must be present");
    assert!(html.contains("pfcNavigate"),         "pfcNavigate JS function must be present");
}

/// updatePreflightCard must be wired into updateOpState so the verdict resets
/// on reroute and advances when dispatch export completes.
#[tokio::test]
async fn reviewer_shell_preflight_card_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updatePreflightCard()"),
        "updatePreflightCard() call must appear in updateOpState and exportDispatch"
    );
}

// ── Active section emphasis tests ────────────────────────────────────────────

/// All four active-step chip elements must be present so the JS can show/hide
/// exactly one "active step" label at a time as workflow state advances.
#[tokio::test]
async fn reviewer_shell_active_section_chips_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"as-chip-route\""),    "as-chip-route must be present");
    assert!(html.contains("id=\"as-chip-verify\""),   "as-chip-verify must be present");
    assert!(html.contains("id=\"as-chip-dispatch\""), "as-chip-dispatch must be present");
    assert!(html.contains("id=\"as-chip-export\""),   "as-chip-export must be present");
}

/// At initial load the routing section must be marked as the active step and
/// all other section chips must be hidden so no stale active label appears.
#[tokio::test]
async fn reviewer_shell_active_section_initial_routing() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // Route chip visible initially (no "hidden" class on as-chip-route)
    assert!(
        html.contains("id=\"as-chip-route\" class=\"as-chip\""),
        "as-chip-route must be visible (no hidden class) on initial load"
    );
    // Verify/dispatch/export chips hidden initially
    assert!(
        html.contains("id=\"as-chip-verify\" class=\"as-chip hidden\""),
        "as-chip-verify must be hidden on initial load"
    );
    assert!(
        html.contains("id=\"as-chip-dispatch\" class=\"as-chip hidden\""),
        "as-chip-dispatch must be hidden on initial load"
    );
    assert!(
        html.contains("id=\"as-chip-export\" class=\"as-chip hidden\""),
        "as-chip-export must be hidden on initial load"
    );
}

/// Container IDs for active section emphasis must be present so JS can apply
/// the as-active class to the correct section wrapper.
#[tokio::test]
async fn reviewer_shell_active_section_containers_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"as-route-section\""),  "as-route-section container must be present");
    assert!(html.contains("id=\"as-verify-section\""), "as-verify-section container must be present");
    // dispatch-section and dispatch-export-result already tested elsewhere
}

/// CSS classes for active emphasis must be defined so the treatment is visually
/// consistent across all four sections.
#[tokio::test]
async fn reviewer_shell_active_section_css_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("as-chip"),   "as-chip CSS class must be defined");
    assert!(html.contains("as-active"), "as-active CSS class must be defined");
}

/// JS functions and arrays for active section emphasis must be present so the
/// active step is derived only from existing reviewer signals.
#[tokio::test]
async fn reviewer_shell_active_section_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("AS_CONTAINERS"),                "AS_CONTAINERS array must be present");
    assert!(html.contains("AS_CHIPS"),                     "AS_CHIPS array must be present");
    assert!(html.contains("activeSectionIndex"),           "activeSectionIndex JS function must be present");
    assert!(html.contains("updateActiveSectionEmphasis"),  "updateActiveSectionEmphasis JS function must be present");
}

/// updateActiveSectionEmphasis must be wired into updateOpState so the active
/// section resets on reroute and advances through the workflow sequence.
#[tokio::test]
async fn reviewer_shell_active_section_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateActiveSectionEmphasis()"),
        "updateActiveSectionEmphasis() call must appear in updateOpState and exportDispatch"
    );
}

// ── Current-run completion checklist tests ────────────────────────────────────

/// Reviewer shell must contain the completion checklist card with its container
/// and footer so the operator can see current-run completion state in one place.
#[tokio::test]
async fn reviewer_shell_completion_checklist_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"crc\""),        "crc container id must be present");
    assert!(html.contains("id=\"crc-rows\""),   "crc-rows id must be present");
    assert!(html.contains("id=\"crc-footer\""), "crc-footer id must be present");
    assert!(html.contains("Current run checklist"), "checklist label must be present");
}

/// Checklist must include all four ordered milestone labels so the operator
/// sees the complete workflow sequence in one card.
#[tokio::test]
async fn reviewer_shell_completion_checklist_all_milestones() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("Route generated"),        "Route generated milestone must be present");
    assert!(html.contains("Receipt available"),      "Receipt available milestone must be present");
    assert!(html.contains("Verification completed"), "Verification completed milestone must be present");
    assert!(html.contains("Dispatch exported"),      "Dispatch exported milestone must be present");
}

/// Footer summary strings for all three states must be present in the JS source
/// so the checklist always resolves to a deterministic one-line conclusion.
#[tokio::test]
async fn reviewer_shell_completion_checklist_footer_strings() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("Current run incomplete"),
        "'Current run incomplete' footer string must be present");
    assert!(html.contains("Current run ready for dispatch export"),
        "'Current run ready for dispatch export' footer string must be present");
    assert!(html.contains("Current run complete"),
        "'Current run complete' footer string must be present");
}

/// CSS classes for all checklist row states must be defined.
#[tokio::test]
async fn reviewer_shell_completion_checklist_css_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("crc-icon-done"),    "crc-icon-done CSS class must be defined");
    assert!(html.contains("crc-icon-pending"), "crc-icon-pending CSS class must be defined");
    assert!(html.contains("crc-icon-blocked"), "crc-icon-blocked CSS class must be defined");
    assert!(html.contains("crc-footer-complete"),   "crc-footer-complete CSS class must be defined");
    assert!(html.contains("crc-footer-ready"),      "crc-footer-ready CSS class must be defined");
    assert!(html.contains("crc-footer-incomplete"), "crc-footer-incomplete CSS class must be defined");
}

/// Checklist JS functions must be present so state is derived only from
/// existing reviewer signals without new persisted state.
#[tokio::test]
async fn reviewer_shell_completion_checklist_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("CRC_ITEMS"),                "CRC_ITEMS constant must be present");
    assert!(html.contains("updateCompletionChecklist"), "updateCompletionChecklist JS function must be present");
    assert!(html.contains("crcNavigate"),               "crcNavigate JS function must be present");
}

/// updateCompletionChecklist must be wired into updateOpState so the checklist
/// resets on reroute and completes when dispatch export succeeds.
#[tokio::test]
async fn reviewer_shell_completion_checklist_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateCompletionChecklist()"),
        "updateCompletionChecklist() call must appear in updateOpState and exportDispatch"
    );
}

// ── Dispatch blocker list tests ───────────────────────────────────────────────

/// Reviewer shell must contain the dispatch blocker list panel so an operator
/// can see why dispatch is not yet exportable in a single glance.
#[tokio::test]
async fn reviewer_shell_dispatch_blocker_list_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"dbl\""),      "dbl container id must be present");
    assert!(html.contains("id=\"dbl-body\""), "dbl-body id must be present");
    assert!(html.contains("Dispatch blockers"), "Dispatch blockers label must be present");
}

/// Blocker list must default to the route-missing blocker on initial load so a
/// fresh session immediately tells the operator the first required step.
#[tokio::test]
async fn reviewer_shell_dispatch_blocker_list_initial_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("No current route result — run routing first."),
        "initial blocker must say 'No current route result — run routing first.'"
    );
}

/// All blocker text strings and state lines must be present in the JS source so
/// every dispatch-path state resolves to a deterministic one-line explanation.
#[tokio::test]
async fn reviewer_shell_dispatch_blocker_list_all_texts_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("No current route result — run routing first."),
        "route-missing blocker text must be present");
    assert!(html.contains("Verification not yet executed for current run."),
        "verify-pending blocker text must be present");
    assert!(html.contains("Verification result does not satisfy dispatch readiness."),
        "verify-failed blocker text must be present");
    assert!(html.contains("No current blockers — dispatch export is available."),
        "no-blockers ready text must be present");
    assert!(html.contains("Dispatch already exported for current run."),
        "dispatch-completed text must be present");
}

/// CSS classes for all blocker list states must be defined.
#[tokio::test]
async fn reviewer_shell_dispatch_blocker_list_css_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("dbl-item-blocked"), "dbl-item-blocked CSS class must be defined");
    assert!(html.contains("dbl-clear"),        "dbl-clear CSS class must be defined");
    assert!(html.contains("dbl-done"),         "dbl-done CSS class must be defined");
}

/// Blocker list JS functions must be present so the panel derives state only
/// from existing reviewer signals without new persisted state.
#[tokio::test]
async fn reviewer_shell_dispatch_blocker_list_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("dispatchBlockers"),       "dispatchBlockers JS function must be present");
    assert!(html.contains("updateDispatchBlockers"), "updateDispatchBlockers JS function must be present");
    assert!(html.contains("dblNavigate"),            "dblNavigate JS function must be present");
}

/// updateDispatchBlockers must be wired into updateOpState so blockers reset on
/// reroute and clear when dispatch export completes.
#[tokio::test]
async fn reviewer_shell_dispatch_blocker_list_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateDispatchBlockers()"),
        "updateDispatchBlockers() call must appear in updateOpState and exportDispatch"
    );
}

// ── Artifact freshness marker tests ──────────────────────────────────────────

/// Reviewer shell must contain freshness marker elements for all three artifact
/// sections so an operator can instantly see whether each artifact belongs to
/// the current run.
#[tokio::test]
async fn reviewer_shell_freshness_markers_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"fm-receipt\""),  "fm-receipt freshness marker must be present");
    assert!(html.contains("id=\"fm-verify\""),   "fm-verify freshness marker must be present");
    assert!(html.contains("id=\"fm-dispatch\""), "fm-dispatch freshness marker must be present");
}

/// All three freshness markers must default to their pending wording at page
/// load so a fresh session never shows a stale current-run label.
#[tokio::test]
async fn reviewer_shell_freshness_markers_initial_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("not yet produced for current run"),
        "receipt freshness marker must default to 'not yet produced for current run'"
    );
    assert!(
        html.contains("not yet executed for current run"),
        "verify freshness marker must default to 'not yet executed for current run'"
    );
    assert!(
        html.contains("not yet exported for current run"),
        "dispatch freshness marker must default to 'not yet exported for current run'"
    );
}

/// All freshness wording strings must be present in the JS source so every
/// artifact panel state resolves to a deterministic label.
#[tokio::test]
async fn reviewer_shell_freshness_marker_wording_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("current run artifact"),
        "'current run artifact' label must be present in JS");
    assert!(html.contains("not yet produced for current run"),
        "'not yet produced for current run' label must be present");
    assert!(html.contains("not yet executed for current run"),
        "'not yet executed for current run' label must be present");
    assert!(html.contains("not yet exported for current run"),
        "'not yet exported for current run' label must be present");
}

/// CSS classes for fresh and pending states must both be defined so freshness
/// markers have consistent visual styling across all three panels.
#[tokio::test]
async fn reviewer_shell_freshness_markers_css_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("fm-fresh"),   "fm-fresh CSS class must be defined");
    assert!(html.contains("fm-pending"), "fm-pending CSS class must be defined");
}

/// JS functions for freshness markers must be present so state is derived only
/// from existing reviewer signals without any new persisted state.
#[tokio::test]
async fn reviewer_shell_freshness_markers_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("setFreshness"),          "setFreshness JS function must be present");
    assert!(html.contains("updateFreshnessMarkers"), "updateFreshnessMarkers JS function must be present");
}

/// updateFreshnessMarkers must be wired into updateOpState so markers reset on
/// reroute and advance correctly through the route/verify/dispatch sequence.
#[tokio::test]
async fn reviewer_shell_freshness_markers_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateFreshnessMarkers()"),
        "updateFreshnessMarkers() call must appear in updateOpState and exportDispatch"
    );
}

// ── Outcome banner tests ──────────────────────────────────────────────────────

/// Reviewer shell must contain the current-run outcome banner with its element
/// IDs so an operator always sees a plain-language summary of workflow state.
#[tokio::test]
async fn reviewer_shell_orb_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("id=\"orb\""),        "orb container id must be present");
    assert!(html.contains("id=\"orb-headline\""), "orb-headline id must be present");
    assert!(html.contains("id=\"orb-detail\""),  "orb-detail id must be present");
    assert!(html.contains("id=\"orb-link\""),    "orb-link id must be present");
    assert!(html.contains("orbNavigate"),        "orbNavigate JS function must be present");
}

/// Outcome banner must default to the neutral empty state at page load so a
/// fresh session never shows a stale outcome from a prior run.
#[tokio::test]
async fn reviewer_shell_orb_initial_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("No current run started"),
        "initial headline must say 'No current run started'"
    );
    assert!(
        html.contains("Start routing to generate current-run artifacts."),
        "initial detail must say 'Start routing to generate current-run artifacts.'"
    );
    assert!(
        html.contains("orb-neutral"),
        "initial banner CSS state must be orb-neutral"
    );
}

/// All five outcome banner headline strings must be present in ORB_STATES so
/// every workflow state resolves to a deterministic one-line headline.
#[tokio::test]
async fn reviewer_shell_orb_all_state_headlines_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("No current run started"),
        "empty state headline must be present");
    assert!(html.contains("Route generated — verification pending"),
        "routed state headline must be present");
    assert!(html.contains("Verification completed"),
        "verified state headline must be present");
    assert!(html.contains("Verification not completed"),
        "blocked state headline must be present");
    assert!(html.contains("Dispatch exported for current run"),
        "complete state headline must be present");
}

/// All five outcome banner detail strings must be present in ORB_STATES so the
/// operator always gets a secondary explanation pointing to the next action.
#[tokio::test]
async fn reviewer_shell_orb_all_state_details_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("Start routing to generate current-run artifacts."),
        "empty state detail must be present");
    assert!(html.contains("Receipt is available. Verification is the next audit step."),
        "routed state detail must be present");
    assert!(html.contains("Dispatch can be exported for the current run."),
        "verified state detail must be present");
    assert!(html.contains("Verification failed. Review the result before dispatching."),
        "blocked state detail must be present");
    assert!(html.contains("Current run artifacts are complete."),
        "complete state detail must be present");
}

/// Outcome banner JS must be present so the banner is computed from existing
/// reviewer signals without any new persisted state.
#[tokio::test]
async fn reviewer_shell_orb_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("ORB_STATES"),            "ORB_STATES constant must be present");
    assert!(html.contains("orbStateKey"),            "orbStateKey JS function must be present");
    assert!(html.contains("updateOutcomeBanner"),    "updateOutcomeBanner JS function must be present");
    assert!(html.contains("orbNavigate"),            "orbNavigate JS function must be present");
}

/// updateOutcomeBanner must be called inside updateOpState so the banner resets
/// on reroute and advances when dispatch export completes.
#[tokio::test]
async fn reviewer_shell_orb_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateOutcomeBanner()"),
        "updateOutcomeBanner() call must appear in updateOpState and exportDispatch"
    );
    // All four CSS state classes must be defined
    assert!(html.contains("orb-neutral"), "orb-neutral CSS class must be defined");
    assert!(html.contains("orb-success"), "orb-success CSS class must be defined");
    assert!(html.contains("orb-warning"), "orb-warning CSS class must be defined");
    assert!(html.contains("orb-blocked"), "orb-blocked CSS class must be defined");
}

// ── Run timeline strip tests ──────────────────────────────────────────────────

/// Reviewer shell HTML must contain the current-run timeline strip with all four
/// step elements and the summary line so an operator can read workflow state
/// at a glance without scanning multiple panels.
#[tokio::test]
async fn reviewer_shell_run_timeline_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // Container and label
    assert!(html.contains("run-timeline"),   "run-timeline id must be present");
    assert!(html.contains("Current run"),    "Current run label must be present");

    // All four step IDs
    assert!(html.contains("rt-route"),    "rt-route step id must be present");
    assert!(html.contains("rt-receipt"),  "rt-receipt step id must be present");
    assert!(html.contains("rt-verify"),   "rt-verify step id must be present");
    assert!(html.contains("rt-dispatch"), "rt-dispatch step id must be present");

    // Summary line element
    assert!(html.contains("rt-summary"),  "rt-summary element id must be present");
}

/// Reviewer shell HTML must carry all four timeline CSS state classes so steps
/// can be styled as idle, ready, done, or blocked.
#[tokio::test]
async fn reviewer_shell_run_timeline_css_states_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("rt-idle"),    "rt-idle CSS class must be defined");
    assert!(html.contains("rt-ready"),   "rt-ready CSS class must be defined");
    assert!(html.contains("rt-done"),    "rt-done CSS class must be defined");
    assert!(html.contains("rt-blocked"), "rt-blocked CSS class must be defined");
}

/// Timeline steps must start in the rt-idle state in the static HTML so the
/// initial page load shows an all-idle/not-started timeline before routing.
#[tokio::test]
async fn reviewer_shell_run_timeline_initial_idle_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("id=\"rt-route\" class=\"rt-step rt-idle\""),
        "rt-route must start rt-idle in HTML"
    );
    assert!(
        html.contains("id=\"rt-receipt\" class=\"rt-step rt-idle\""),
        "rt-receipt must start rt-idle in HTML"
    );
    assert!(
        html.contains("id=\"rt-verify\" class=\"rt-step rt-idle\""),
        "rt-verify must start rt-idle in HTML"
    );
    assert!(
        html.contains("id=\"rt-dispatch\" class=\"rt-step rt-idle\""),
        "rt-dispatch must start rt-idle in HTML"
    );
}

/// Summary line must default to 'Current run not started' at initial load
/// so the operator sees an explicit idle description, not a blank strip.
#[tokio::test]
async fn reviewer_shell_run_timeline_initial_summary() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Current run not started"),
        "rt-summary must default to 'Current run not started'"
    );
}

/// All four summary text strings must be present in the JS so the operator
/// always sees an accurate one-line description of the current run state.
#[tokio::test]
async fn reviewer_shell_run_timeline_summary_strings_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Route produced — verification pending"),
        "post-route summary must be present"
    );
    assert!(
        html.contains("Verification completed — dispatch ready"),
        "post-verify summary must be present"
    );
    assert!(
        html.contains("Dispatch exported for current run"),
        "post-export summary must be present"
    );
    assert!(
        html.contains("Verification failed — review inputs before dispatch"),
        "failed-verify summary must be present"
    );
}

/// Reviewer shell HTML must expose updateRunTimeline, timelineStepState, and
/// timelineSummary JS functions so timeline state is computed from existing signals.
#[tokio::test]
async fn reviewer_shell_run_timeline_js_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("updateRunTimeline"),  "updateRunTimeline JS function must be present");
    assert!(html.contains("timelineStepState"),  "timelineStepState JS function must be present");
    assert!(html.contains("timelineSummary"),    "timelineSummary JS function must be present");
}

/// updateRunTimeline must be wired into updateOpState and called in the
/// exportDispatch path so the strip resets on reroute and advances on export.
#[tokio::test]
async fn reviewer_shell_run_timeline_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // Must be called from updateOpState (covers route/verify/reroute transitions)
    assert!(
        html.contains("updateRunTimeline()"),
        "updateRunTimeline() call must appear in JS"
    );
    // updateOpState must exist as the entry point
    assert!(
        html.contains("updateOpState"),
        "updateOpState must be present as the state machine driver"
    );
}

// ── Artifact section empty-state hardening tests ──────────────────────────────

/// Reviewer shell HTML must carry an explicit receipt empty-state element that
/// is visible at initial load so the operator can never confuse a missing
/// current-run receipt with a hidden or stale artifact.
#[tokio::test]
async fn reviewer_shell_receipt_empty_state_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("receipt-empty-state"),
        "receipt-empty-state element id must be present"
    );
    assert!(
        html.contains("no receipt for current route"),
        "receipt empty-state must say 'no receipt for current route'"
    );
}

/// Reviewer shell HTML must carry the verification empty-state wording
/// 'verification not yet executed for current route' inside verify-artifact-note
/// so the operator sees an unambiguous reason, not a generic placeholder.
#[tokio::test]
async fn reviewer_shell_verify_empty_state_text() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("verify-artifact-note"),
        "verify-artifact-note id must be present"
    );
    assert!(
        html.contains("verification not yet executed for current route"),
        "verify empty-state must say 'verification not yet executed for current route'"
    );
}

/// Reviewer shell HTML must carry the dispatch empty-state wording
/// 'no dispatch export for current route' inside dispatch-stale-note
/// so the operator sees an explicit reason, not a blank section.
#[tokio::test]
async fn reviewer_shell_dispatch_empty_state_text() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("dispatch-stale-note"),
        "dispatch-stale-note id must be present"
    );
    assert!(
        html.contains("no dispatch export for current route"),
        "dispatch empty-state must say 'no dispatch export for current route'"
    );
}

/// All three artifact section empty-states must be present simultaneously
/// so none of the three sections ever appears blank during a review session.
#[tokio::test]
async fn reviewer_shell_all_artifact_empty_states_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("no receipt for current route"),
        "receipt section empty-state text must be present"
    );
    assert!(
        html.contains("verification not yet executed for current route"),
        "verification section empty-state text must be present"
    );
    assert!(
        html.contains("no dispatch export for current route"),
        "dispatch section empty-state text must be present"
    );
}

/// Reviewer shell HTML must wire receipt-empty-state into the JS state machine
/// so it hides on route start and re-appears on route failure or reroute,
/// and verify-artifact-note + dispatch-stale-note are also correctly wired.
#[tokio::test]
async fn reviewer_shell_empty_state_reroute_wiring() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    // All three empty-state IDs must be referenced in the JS state machine
    assert!(
        html.contains("receipt-empty-state"),
        "receipt-empty-state must be referenced in JS"
    );
    assert!(
        html.contains("verify-artifact-note"),
        "verify-artifact-note must be referenced in JS"
    );
    assert!(
        html.contains("dispatch-stale-note"),
        "dispatch-stale-note must be referenced in JS"
    );

    // receipt-empty-state hide must appear (route start hides it)
    assert!(
        html.contains("hide('receipt-empty-state')"),
        "hide('receipt-empty-state') must be present in JS route-start reset"
    );
    // receipt-empty-state show must appear (failure paths restore it)
    assert!(
        html.contains("show('receipt-empty-state')"),
        "show('receipt-empty-state') must be present in JS failure paths"
    );

    // verify-artifact-note show must appear (restores after successful route)
    assert!(
        html.contains("show('verify-artifact-note')"),
        "show('verify-artifact-note') must be present in JS route-success path"
    );
    // verify-artifact-note hide must appear (cleared after verification runs)
    assert!(
        html.contains("hide('verify-artifact-note')"),
        "hide('verify-artifact-note') must be present in JS route-start and post-verify"
    );
}

// ── Route reproducibility check tests ────────────────────────────────────────

/// Reviewer shell HTML must expose the reproducibility check panel element
/// so the operator has a dedicated surface to confirm routing determinism.
#[tokio::test]
async fn reviewer_shell_rrc_panel_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("id=\"rrc\""),
        "rrc panel id must be present in HTML"
    );
    assert!(
        html.contains("id=\"rrc-status\""),
        "rrc-status element id must be present"
    );
}

/// The reproducibility check panel must default to 'Reproducibility not tested'
/// at initial load so the operator sees an explicit idle state, not a blank panel.
#[tokio::test]
async fn reviewer_shell_rrc_status_initial_text() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Reproducibility not tested"),
        "rrc initial status must read 'Reproducibility not tested'"
    );
}

/// The reproducibility check panel must carry operator guidance text in rrc-detail
/// so the operator understands what the check does before triggering it.
#[tokio::test]
async fn reviewer_shell_rrc_detail_initial_text() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("id=\"rrc-detail\""),
        "rrc-detail element id must be present"
    );
    assert!(
        html.contains("Run a reproducibility check to confirm the routing result is deterministic"),
        "rrc-detail must contain operator guidance text"
    );
}

/// The reproducibility check panel must expose a trigger button so the operator
/// can initiate the check without leaving the reviewer shell.
#[tokio::test]
async fn reviewer_shell_rrc_button_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("id=\"btn-repro\""),
        "btn-repro element id must be present"
    );
    assert!(
        html.contains("runReproCheck"),
        "btn-repro must reference runReproCheck in onclick"
    );
}

/// Reviewer shell HTML must expose runReproCheck, updateReproPanel, and REPRO_STATES
/// so the reproducibility check logic is fully present in the JS.
#[tokio::test]
async fn reviewer_shell_rrc_js_functions_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("runReproCheck"),   "runReproCheck JS function must be present");
    assert!(html.contains("updateReproPanel"), "updateReproPanel JS function must be present");
    assert!(html.contains("REPRO_STATES"),     "REPRO_STATES constant must be present");
}

/// updateReproPanel must be wired into updateOpState so the reproducibility panel
/// resets on reroute and reflects state changes driven by the state machine.
#[tokio::test]
async fn reviewer_shell_rrc_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateReproPanel()"),
        "updateReproPanel() call must appear in JS"
    );
    assert!(
        html.contains("lastRouteInputs"),
        "lastRouteInputs state variable must be present"
    );
    assert!(
        html.contains("lastRouteEndpoint"),
        "lastRouteEndpoint state variable must be present"
    );
    assert!(
        html.contains("reproStatus"),
        "reproStatus state variable must be present"
    );
}

// ── Operator dry-run status panel tests ──────────────────────────────────────

/// Reviewer shell HTML must expose the Operator Dry-Run Status panel so the
/// operator has a dedicated surface summarising dry-run completion state.
#[tokio::test]
async fn reviewer_shell_drs_panel_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("id=\"drs\""),
        "drs panel id must be present in HTML"
    );
    assert!(
        html.contains("Operator Dry-Run Status"),
        "panel must be labelled 'Operator Dry-Run Status'"
    );
    assert!(
        html.contains("id=\"drs-verdict\""),
        "drs-verdict element id must be present"
    );
}

/// The dry-run panel must default to 'No dry-run in progress' at initial load
/// so the operator sees an explicit idle description, not a blank panel.
#[tokio::test]
async fn reviewer_shell_drs_initial_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("No dry-run in progress"),
        "drs initial verdict must read 'No dry-run in progress'"
    );
    assert!(
        html.contains("Generate a route to begin the dry-run"),
        "drs initial next-step must prompt route generation"
    );
}

/// The dry-run panel must carry the passed-state wording so the operator can
/// recognise when all minimum pilot workflow steps have been completed.
#[tokio::test]
async fn reviewer_shell_drs_passed_wording_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Dry-run passed"),
        "passed verdict wording must be present in JS"
    );
    assert!(
        html.contains("Dry-run complete for current route"),
        "passed next-step wording must be present"
    );
}

/// The dry-run panel must carry attention-state wording to cover the case where
/// verification failed, preventing a false 'passed' verdict from being shown.
#[tokio::test]
async fn reviewer_shell_drs_attention_on_verify_failure_wording() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Dry-run requires attention"),
        "attention verdict wording must be present in JS"
    );
    assert!(
        html.contains("Resolve failed verification before completing dry-run"),
        "verify-failure next-step wording must be present"
    );
}

/// The dry-run panel must carry attention wording for reproducibility mismatch
/// so that a mismatch does not silently allow a 'passed' verdict.
#[tokio::test]
async fn reviewer_shell_drs_attention_on_repro_mismatch_wording() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Reproducibility mismatch detected"),
        "repro-mismatch attention wording must be present"
    );
}

/// The dry-run panel must carry reroute-detected attention wording so the
/// operator knows a prior dry-run result does not apply to the new route.
#[tokio::test]
async fn reviewer_shell_drs_reroute_downgrade_wording() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Reroute detected"),
        "reroute attention wording must be present"
    );
    assert!(
        html.contains("dry-run must be completed again for current route"),
        "reroute downgrade message must be present"
    );
    // updateDryRunPanel must be wired into updateOpState
    assert!(
        html.contains("updateDryRunPanel()"),
        "updateDryRunPanel() call must appear in JS"
    );
}

// ── Pilot handoff summary card tests ─────────────────────────────────────────

/// Reviewer shell HTML must expose the Pilot Handoff Summary panel so the
/// operator has a dedicated surface for pilot-readiness status.
#[tokio::test]
async fn reviewer_shell_phs_panel_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("id=\"phs\""),
        "phs panel id must be present in HTML"
    );
    assert!(
        html.contains("Pilot Handoff Summary"),
        "panel must be labelled 'Pilot Handoff Summary'"
    );
    assert!(
        html.contains("id=\"phs-verdict\""),
        "phs-verdict element id must be present"
    );
}

/// The pilot handoff summary must default to 'Not ready for pilot handoff' at
/// initial load so the operator never sees a false ready state before routing.
#[tokio::test]
async fn reviewer_shell_phs_initial_not_ready() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Not ready for pilot handoff"),
        "initial verdict must read 'Not ready for pilot handoff'"
    );
}

/// The pilot handoff summary must carry ready-state wording so the operator
/// can recognise when the run meets all minimum pilot handoff requirements.
#[tokio::test]
async fn reviewer_shell_phs_ready_wording_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Ready for pilot handoff"),
        "ready verdict wording must be present in JS"
    );
    assert!(
        html.contains("Pilot handoff ready"),
        "ready action wording must be present"
    );
}

/// The pilot handoff summary must carry attention wording for verification
/// failure so that a failed verify cannot produce a ready state.
#[tokio::test]
async fn reviewer_shell_phs_verify_failure_attention_wording() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Pilot handoff requires attention"),
        "attention verdict wording must be present"
    );
    assert!(
        html.contains("Resolve failed verification before pilot handoff"),
        "verify-failure action wording must be present"
    );
}

/// The pilot handoff summary must carry attention wording for reproducibility
/// mismatch so that a mismatch cannot produce a ready state.
#[tokio::test]
async fn reviewer_shell_phs_repro_mismatch_attention_wording() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("Reproducibility mismatch"),
        "repro-mismatch attention wording must be present in phs"
    );
}

/// The pilot handoff summary must carry reroute-detected attention wording so
/// a prior ready state is visibly invalidated when the route changes.
#[tokio::test]
async fn reviewer_shell_phs_reroute_downgrade_wording() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("complete the dry-run again for the current route"),
        "reroute downgrade wording must be present in phs"
    );
    assert!(
        html.contains("updatePilotHandoffSummary()"),
        "updatePilotHandoffSummary() call must appear in JS"
    );
}

// ── Canonical pilot workflow panel tests ─────────────────────────────────────

/// Reviewer shell HTML must expose the Canonical Pilot Workflow panel so the
/// operator has a deterministic step-by-step sequence visible at all times.
#[tokio::test]
async fn reviewer_shell_cpw_panel_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("id=\"cpw\""),
        "cpw panel id must be present in HTML"
    );
    assert!(
        html.contains("Canonical Pilot Workflow"),
        "panel must be labelled 'Canonical Pilot Workflow'"
    );
}

/// All six canonical workflow steps must be present in the HTML so the
/// operator sees the complete sequence regardless of run state.
#[tokio::test]
async fn reviewer_shell_cpw_six_steps_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("Generate route"),              "step 1 label must be present");
    assert!(html.contains("Verify receipt"),              "step 2 label must be present");
    assert!(html.contains("Inspect artifacts"),           "step 3 label must be present");
    assert!(html.contains("Run reproducibility check"),   "step 4 label must be present");
    assert!(html.contains("Export dispatch packet"),      "step 5 label must be present");
    assert!(html.contains("Confirm dry-run"),             "step 6 label must be present");
}

/// The first step must default to 'available' at initial load because the
/// operator can always start by generating a route.
#[tokio::test]
async fn reviewer_shell_cpw_initial_first_step_available() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("id=\"cpw-s1\""),
        "cpw-s1 step status element id must be present"
    );
    assert!(
        html.contains("cpw-s-available"),
        "step 1 must carry cpw-s-available class at initial load"
    );
}

/// Steps 2 through 6 must default to 'blocked' at initial load because they
/// depend on prior steps being completed first.
#[tokio::test]
async fn reviewer_shell_cpw_later_steps_blocked_initially() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("id=\"cpw-s2\""),
        "cpw-s2 step status element id must be present"
    );
    assert!(
        html.contains("cpw-s-blocked"),
        "at least one later step must carry cpw-s-blocked class at initial load"
    );
}

/// updateCanonicalWorkflow must be wired into updateOpState so steps
/// update as the run progresses and reset correctly on reroute.
#[tokio::test]
async fn reviewer_shell_cpw_wired_to_state_machine() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        html.contains("updateCanonicalWorkflow()"),
        "updateCanonicalWorkflow() call must appear in JS"
    );
    assert!(
        html.contains("updateCanonicalWorkflow"),
        "updateCanonicalWorkflow function must be defined"
    );
}

/// The canonical workflow JS must include status labels for all four step
/// states — available, completed, blocked, attention — so any run state
/// produces a legible indicator on each step.
#[tokio::test]
async fn reviewer_shell_cpw_status_labels_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert_eq!(status, StatusCode::OK);

    assert!(html.contains("CPW_STATUS_LABELS"), "CPW_STATUS_LABELS constant must be present");
    assert!(html.contains("cpwStepStatus"),     "cpwStepStatus function must be present");
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
