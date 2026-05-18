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
use postcad_service::REVIEWER_HTML;
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
    assert!(html.contains("STL-Datei hochladen"),  "upload title DE must be present");
    assert!(html.contains("Upload STL file"),      "upload title EN must be present");
}

/// Upload zone must include privacy notice and demo notice.
#[tokio::test]
async fn reviewer_upload_privacy_and_demo_notice_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("t-upload-privacy"),                    "t-upload-privacy id must be present");
    assert!(html.contains("nicht auf dem Server gespeichert"),    "DE upload privacy notice must be present");
    assert!(html.contains("not stored on the server"),            "EN upload privacy notice must be present");
    assert!(html.contains("t-demo-notice"),                       "t-demo-notice id must be present");
    assert!(html.contains("Nur Beispiel"),                        "DE demo notice must be present");
    assert!(html.contains("Demo only"),                           "EN demo notice must be present");
    assert!(html.contains("Demo-Fall ansehen"),                   "demo button label must be 'Demo-Fall ansehen'");
}

/// Workflow intro line must clearly identify lab as the actor.
#[tokio::test]
async fn reviewer_intro_line_assistant_friendly() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("Laboransicht"),                            "DE intro line must begin with Laboransicht");
    assert!(html.contains("Entscheidung vor Herstellung dokumentieren"), "DE intro line must mention Entscheidung vor Herstellung dokumentieren");
}

/// STL loaded banner must be present in the visual phase.
#[tokio::test]
async fn reviewer_stl_loaded_banner_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("stl-loaded-banner"),          "stl-loaded-banner id must be present");
    assert!(html.contains("stl-loaded-details"),         "stl-loaded-details id must be present");
    assert!(html.contains("bereit zur Kl"),              "DE loaded banner text must be present");
    assert!(html.contains("ready for clarification"),    "EN loaded banner text must be present");
}

/// Required field markers must be present for the four metadata fields and comment.
#[tokio::test]
async fn reviewer_required_field_markers_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    // CSS pseudo-element via .required class
    assert!(html.contains("case-meta-label required"), "metadata labels must have required class");
    assert!(html.contains("comment-label required"),   "comment label must have required class");
    // Legend in privacy notice
    assert!(html.contains("Pflichtfeld"),              "Pflichtfeld legend must be present");
    assert!(html.contains("Required field"),           "Required field legend must be present in EN");
}

/// Receipt section must have the human headline and subtitle.
#[tokio::test]
async fn reviewer_nachweis_headline_and_subtitle_present() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, html) = get_html(make_app(&tmp), "/reviewer").await;
    assert!(html.contains("Entscheidungsnachweis erstellt"),  "DE receipt headline must read 'Entscheidungsnachweis erstellt'");
    assert!(html.contains("Decision record created"),          "EN receipt headline must read 'Decision record created'");
    assert!(html.contains("t-nachweis-subtitle"),              "t-nachweis-subtitle id must be present");
    assert!(html.contains("Dokumentiert, was gepr"),           "DE nachweis subtitle must be present");
    assert!(html.contains("Documents what was reviewed"),      "EN nachweis subtitle must be present");
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

    assert!(html.contains("phase-visual"),                         "phase-visual id must be present");
    assert!(html.contains("Schritt 1"),                            "visual step must be labelled Schritt 1");
    assert!(html.contains("Laborfall"),                            "visual step title must mention Laborfall");
    assert!(html.contains("Demo-Ansicht:"),                        "visual placeholder hint must be present");
    assert!(html.contains("Keine automatische technische"),        "visual disclaimer must be present");
    assert!(html.contains("id=\"lab-comment\""),                   "lab-comment textarea id must be present");
    assert!(html.contains("Kurze Laborerkl"),                      "lab comment label must be present");
    assert!(html.contains("WhatsApp"),                             "praxis-erklaerung subtext must mention WhatsApp/existing channels");
    assert!(html.contains("proceedToDecision"),                    "proceedToDecision JS function must be present");
    assert!(html.contains("showVisualStep"),                       "showVisualStep JS function must be present");
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

/// Visual step must have a proceed button that attempts to advance to the decision gate.
/// The proceed action validates that the comment and all metadata fields are filled.
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

/// Viewer reset button must be present in the viewer bar with DE and EN labels.
#[test]
fn viewer_reset_button_present() {
    assert!(REVIEWER_HTML.contains("viewer-reset-btn"),        "viewer-reset-btn id must be present");
    assert!(REVIEWER_HTML.contains("function resetView()"),    "resetView JS function must be present");
    assert!(REVIEWER_HTML.contains("t-viewer-reset-btn"),      "t-viewer-reset-btn span id must be present");
    assert!(REVIEWER_HTML.contains("Ansicht zur"),             "DE reset view label must be present");
    assert!(REVIEWER_HTML.contains("Reset view"),              "EN reset view label must be present");
}

/// Camera auto-fit must use FOV-aware calculation so the model fills the viewport.
#[test]
fn viewer_auto_fit_uses_fov_calculation() {
    assert!(REVIEWER_HTML.contains("hFOVrad"),     "horizontal FOV calculation must be present in initViewer");
    assert!(REVIEWER_HTML.contains("fitZoom"),     "fitZoom variable must be computed for auto-fit");
    assert!(REVIEWER_HTML.contains("defaultZoom"), "defaultZoom must be stored so resetView can restore it");
    // Old naive formula must be gone
    assert!(
        !REVIEWER_HTML.contains("Math.max(size.x, size.y, size.z) * 2.5"),
        "old maxDim * 2.5 formula must be replaced by FOV-aware fit"
    );
}

/// Receipt copy button must be present with DE and EN labels.
#[test]
fn reviewer_receipt_copy_button_present() {
    assert!(REVIEWER_HTML.contains("Nachweis kopieren"), "DE copy button label must be present");
    assert!(REVIEWER_HTML.contains("Copy receipt"),      "EN copy button label must be present");
    assert!(REVIEWER_HTML.contains("t-copy-btn"),        "t-copy-btn translation ID must be present");
    assert!(REVIEWER_HTML.contains("copyReceipt()"),     "copy button must call copyReceipt()");
    assert!(REVIEWER_HTML.contains("copy-confirm"),      "copy-confirm element must be present for feedback");
}

/// copyReceipt function must use clipboard API with textarea fallback.
#[test]
fn reviewer_copy_receipt_function_present() {
    assert!(REVIEWER_HTML.contains("function copyReceipt()"), "copyReceipt JS function must be defined");
    assert!(REVIEWER_HTML.contains("navigator.clipboard"),    "must attempt clipboard API");
    assert!(REVIEWER_HTML.contains("showCopyFallback"),       "fallback function must be called on clipboard failure");
    assert!(REVIEWER_HTML.contains("copy-fallback-textarea"), "fallback textarea must be present");
    assert!(REVIEWER_HTML.contains("Nachweis kopiert."),      "DE copy-confirmed text must be in T strings");
    assert!(REVIEWER_HTML.contains("Receipt copied."),        "EN copy-confirmed text must be in T strings");
}

/// copyReceipt plain-text output must include the safety note.
#[test]
fn reviewer_copy_receipt_includes_safety_note() {
    let pos = REVIEWER_HTML.find("function copyReceipt()").expect("copyReceipt must be defined");
    let snippet = &REVIEWER_HTML[pos..pos + 2400];
    assert!(snippet.contains("safetyCopy"), "copyReceipt must append t.safetyCopy line");
    assert!(
        REVIEWER_HTML.contains("PostCAD erkennt keine medizinischen oder technischen Fehler und gibt keine Herstellung frei."),
        "DE safetyCopy text must be present in T strings"
    );
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

/// Drop handler must read the file with FileReader before starting processing.
/// The primary STL upload bug: old code called loadDemo(filename) without reading
/// the file bytes, so _pendingBuffer was always null and the demo mesh was shown.
#[test]
fn stl_drop_handler_reads_file_with_filereader() {
    assert!(
        REVIEWER_HTML.contains("reader.onload = function(ev) { _pendingBuffer = ev.target.result; startProcessing(name); }"),
        "drop handler must set _pendingBuffer via FileReader.onload before calling startProcessing"
    );
    assert!(
        !REVIEWER_HTML.contains("loadDemo(e.dataTransfer.files[0].name)"),
        "drop handler must not skip FileReader by calling loadDemo with only the filename"
    );
}

/// onFileInput must only start processing inside reader.onload — not before it.
#[test]
fn stl_file_input_waits_for_filereader() {
    assert!(
        REVIEWER_HTML.contains("reader.onload = function(e) { _pendingBuffer = e.target.result; startProcessing(name); }"),
        "onFileInput must call startProcessing inside reader.onload, not before readAsArrayBuffer"
    );
}

/// loadDemo (demo button) must clear _pendingBuffer so the viewer renders the
/// schematic mesh, not a leftover buffer from a previously dropped file.
#[test]
fn stl_load_demo_clears_pending_buffer() {
    assert!(
        REVIEWER_HTML.contains("_pendingBuffer = null;") && REVIEWER_HTML.contains("localStlActive = false;"),
        "loadDemo must reset _pendingBuffer = null and localStlActive = false before startProcessing"
    );
    // localStlActive = false must appear before startProcessing(filename) within loadDemo
    let load_demo_pos = REVIEWER_HTML.find("function loadDemo(filename)").unwrap();
    let after_load_demo = &REVIEWER_HTML[load_demo_pos..];
    let flag_pos = after_load_demo.find("localStlActive = false").unwrap();
    let start_pos = after_load_demo.find("startProcessing(filename)").unwrap();
    assert!(flag_pos < start_pos, "loadDemo must clear localStlActive before calling startProcessing");
}

/// When a user file is provided but STL parsing fails, the viewer must show an
/// explicit error — not silently fall back to the demo cylinder.
#[test]
fn stl_parse_error_shown_not_demo_mesh() {
    assert!(
        REVIEWER_HTML.contains("STL konnte lokal nicht dargestellt werden"),
        "HTML must contain German STL parse error message"
    );
    assert!(
        REVIEWER_HTML.contains("stlParseError"),
        "HTML must define stlParseError translation key"
    );
    assert!(
        REVIEWER_HTML.contains("T[lang].stlParseError"),
        "error path must surface stlParseError to the user"
    );
}

/// parseSTLAscii must be present so ASCII-format STL files render instead of
/// silently failing (parseSTLBinary alone only handles binary STL).
#[test]
fn stl_ascii_parser_present() {
    assert!(
        REVIEWER_HTML.contains("function parseSTLAscii(buffer)"),
        "HTML must include parseSTLAscii for ASCII STL support"
    );
    assert!(
        REVIEWER_HTML.contains("function parseSTL(buffer)"),
        "HTML must include parseSTL dispatcher that tries ASCII before binary"
    );
}

/// Viewer label must use a middle dot (·) not a colon for both demo and local modes.
#[test]
fn stl_viewer_label_uses_middle_dot() {
    let middle_dot = '\u{00b7}';
    assert!(
        REVIEWER_HTML.contains(&format!("Demo-Ansicht {middle_dot} schematische Darstellung")),
        "demo label must use middle dot"
    );
    assert!(
        REVIEWER_HTML.contains(&format!("Lokale STL-Datei {middle_dot} nur im Browser dargestellt")),
        "local file label must use middle dot"
    );
}

/// Local STL upload must not trigger "Nur Demo-Dateien werden unterstützt".
/// confirmDecision must check localStlActive BEFORE the FILE_CASES_API demo guard,
/// so a successfully-rendered local file always routes to confirmLocalDecision.
#[test]
fn local_stl_does_not_show_demo_only_error() {
    // localStlActive check must appear before the demoOnlyError gate in confirmDecision.
    let confirm_pos = REVIEWER_HTML.find("async function confirmDecision()").unwrap();
    let after_confirm = &REVIEWER_HTML[confirm_pos..];
    let local_check_pos = after_confirm.find("if (localStlActive)").unwrap();
    let demo_error_pos = after_confirm.find("demoOnlyError").unwrap();
    assert!(
        local_check_pos < demo_error_pos,
        "confirmDecision must check localStlActive before reaching demoOnlyError gate"
    );
    assert!(
        REVIEWER_HTML.contains("confirmLocalDecision"),
        "confirmLocalDecision function must exist for the local-upload path"
    );
}

/// confirmLocalDecision must generate a client-side receipt without server calls.
#[test]
fn local_stl_confirm_local_decision_present() {
    assert!(
        REVIEWER_HTML.contains("function confirmLocalDecision()"),
        "confirmLocalDecision function definition must be present"
    );
    assert!(
        REVIEWER_HTML.contains("nicht auf Server gespeichert"),
        "local receipt must note that file is not stored on server"
    );
    assert!(
        REVIEWER_HTML.contains("storage: 'Lokale Datei"),
        "local receipt JSON must include storage field"
    );
}

/// Local receipt must include Case-ID and filename from current reviewer state.
#[test]
fn local_stl_receipt_includes_case_id_and_filename() {
    assert!(
        REVIEWER_HTML.contains("nachweis-caseid"),
        "receipt must have nachweis-caseid element"
    );
    assert!(
        REVIEWER_HTML.contains("nachweis-datei"),
        "receipt must have nachweis-datei element"
    );
    assert!(
        REVIEWER_HTML.contains("nachweis-ereignis"),
        "receipt must have nachweis-ereignis element"
    );
    assert!(
        REVIEWER_HTML.contains("nachweis-person"),
        "receipt must have nachweis-person element"
    );
}

/// Local STL upload must prefill the metadata form with sensible defaults so the
/// receipt is never empty when the user has not typed anything yet.
#[test]
fn local_stl_upload_prefills_metadata_defaults() {
    assert!(
        REVIEWER_HTML.contains("LOCAL_STL_DEFAULTS"),
        "LOCAL_STL_DEFAULTS constant must be defined for local STL metadata prefill"
    );
    let pos = REVIEWER_HTML.find("LOCAL_STL_DEFAULTS").unwrap();
    let context = &REVIEWER_HTML[pos..pos + 200];
    assert!(context.contains("Krone Zahn 36"), "local default Fallbezeichnung must be 'Krone Zahn 36'");
    assert!(context.contains("Demo-Praxis"),   "local default Praxis / Kunde must be 'Demo-Praxis'");
    assert!(context.contains("E.max"),         "local default Material must be 'E.max'");
    // The defaults must be applied when _pendingBuffer is set (i.e. a real file was dropped)
    assert!(
        REVIEWER_HTML.contains("_pendingBuffer !== null ? LOCAL_STL_DEFAULTS"),
        "showVisualStep must apply LOCAL_STL_DEFAULTS when _pendingBuffer is set"
    );
}

/// Empty comment must block proceedToDecision and show a clear inline message.
#[test]
fn empty_comment_blocks_proceed_to_decision() {
    assert!(
        REVIEWER_HTML.contains("comment-error"),
        "comment-error element id must be present"
    );
    assert!(
        REVIEWER_HTML.contains("Bitte Laborerkl"),
        "comment-error message must mention Laborerklärung"
    );
    // proceedToDecision must guard on an empty comment
    let proc_pos = REVIEWER_HTML.find("function proceedToDecision()").unwrap();
    let after = &REVIEWER_HTML[proc_pos..proc_pos + 1600];
    assert!(
        after.contains("comment-error"),
        "proceedToDecision must reference comment-error to block empty submissions"
    );
    assert!(
        after.contains("rueckmeldung-error"),
        "proceedToDecision must reference rueckmeldung-error to block empty submissions"
    );
    assert!(
        after.contains("praxis-antwort-error"),
        "proceedToDecision must reference praxis-antwort-error to block missing response"
    );
    assert!(
        after.contains("if (hasError) return"),
        "proceedToDecision must return early when hasError is true"
    );
}

/// Empty metadata fields must block proceedToDecision and show a clear inline message.
#[test]
fn empty_metadata_blocks_proceed_to_decision() {
    assert!(
        REVIEWER_HTML.contains("meta-error"),
        "meta-error element id must be present"
    );
    assert!(
        REVIEWER_HTML.contains("Bitte Falldaten vollst"),
        "meta-error message must include 'Bitte Falldaten vollständig ausfüllen'"
    );
    let proc_pos = REVIEWER_HTML.find("function proceedToDecision()").unwrap();
    let after = &REVIEWER_HTML[proc_pos..proc_pos + 1200];
    assert!(
        after.contains("meta-error"),
        "proceedToDecision must reference meta-error to block incomplete metadata"
    );
}

/// Receipt rows must never display null — the na() helper must provide the
/// 'Nicht angegeben' fallback for any field that is somehow empty.
#[test]
fn receipt_fallback_uses_nicht_angegeben() {
    assert!(
        REVIEWER_HTML.contains("function na("),
        "na() helper function must be defined"
    );
    assert!(
        REVIEWER_HTML.contains("Nicht angegeben"),
        "na() must return 'Nicht angegeben' as the fallback string"
    );
    // Receipt fields must use na() rather than bare '—'
    assert!(
        REVIEWER_HTML.contains("na(caseId)"),
        "nachweis-caseid must use na() fallback"
    );
    assert!(
        REVIEWER_HTML.contains("na(labComment)"),
        "nachweis-kommentar must use na() fallback"
    );
    assert!(
        REVIEWER_HTML.contains("na(caseMetadata.bezeichnung)"),
        "nachweis-bezeichnung must use na() fallback"
    );
}

/// Demo path must still use FILE_CASES_API for the server-backed route/receipt.
#[test]
fn demo_path_preserved_via_file_cases_api() {
    assert!(
        REVIEWER_HTML.contains("const FILE_CASES_API"),
        "FILE_CASES_API must still be defined for demo routing"
    );
    assert!(
        REVIEWER_HTML.contains("fetch('/cases',"),
        "demo path must still POST to /cases"
    );
    assert!(
        REVIEWER_HTML.contains("fetch('/decisions',"),
        "demo path must still POST to /decisions"
    );
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

// ── Verlauf section tests ─────────────────────────────────────────────────────

/// Verlauf section must be present in the result phase with the correct label.
#[test]
fn reviewer_verlauf_section_present() {
    assert!(REVIEWER_HTML.contains("verlauf-section"),       "verlauf-section id must be present");
    assert!(REVIEWER_HTML.contains("verlauf-rows"),          "verlauf-rows id must be present");
    assert!(REVIEWER_HTML.contains("t-verlauf-label"),       "t-verlauf-label id must be present");
    assert!(REVIEWER_HTML.contains("t-verlauf-note"),        "t-verlauf-note id must be present");
    assert!(REVIEWER_HTML.contains("Verlauf"),               "DE section label must be present");
    assert!(REVIEWER_HTML.contains("History"),               "EN section label must be present");
}

/// Verlauf local-history note must appear in both DE and EN.
#[test]
fn reviewer_verlauf_local_note_present() {
    assert!(
        REVIEWER_HTML.contains("nicht serverseitig gespeichert"),
        "DE verlauf note must mention 'nicht serverseitig gespeichert'"
    );
    assert!(
        REVIEWER_HTML.contains("not stored server-side"),
        "EN verlauf note must mention 'not stored server-side'"
    );
}

/// All 6 DE event labels must be present in the translation table.
#[test]
fn reviewer_verlauf_de_event_labels_present() {
    assert!(REVIEWER_HTML.contains("STL-Datei lokal geladen"),                   "DE event 1 label must be present");
    assert!(REVIEWER_HTML.contains("Laborfall visuell gepr\u{00fc}ft"),          "DE event 2 label must be present");
    assert!(REVIEWER_HTML.contains("Praxis-Anfrage vorbereitet"),                "DE event 3 label must be present");
    assert!(REVIEWER_HTML.contains("Praxis-Antwort dokumentiert"),               "DE event 4 label must be present");
    assert!(REVIEWER_HTML.contains("Entscheidung vor Herstellung festgehalten"), "DE event 5 label must be present");
    assert!(REVIEWER_HTML.contains("Entscheidungsnachweis erstellt"),            "DE event 6 label must be present");
}

/// All 6 EN event labels must be present in the translation table.
#[test]
fn reviewer_verlauf_en_event_labels_present() {
    assert!(REVIEWER_HTML.contains("STL file loaded locally"),                   "EN event 1 label must be present");
    assert!(REVIEWER_HTML.contains("Lab case reviewed visually"),                "EN event 2 label must be present");
    assert!(REVIEWER_HTML.contains("Practice query prepared"),                   "EN event 3 label must be present");
    assert!(REVIEWER_HTML.contains("Practice response documented"),              "EN event 4 label must be present");
    assert!(REVIEWER_HTML.contains("Decision before manufacturing recorded"),    "EN event 5 label must be present");
    assert!(REVIEWER_HTML.contains("Decision record created"),                   "EN event 6 label must be present");
}

/// buildVerlauf JS function must be defined and populate verlauf-rows.
#[test]
fn reviewer_verlauf_build_function_present() {
    assert!(REVIEWER_HTML.contains("function buildVerlauf(ts)"), "buildVerlauf JS function must be defined");
    assert!(REVIEWER_HTML.contains("verlauf-row-lbl"),           "verlauf-row-lbl class must be used in buildVerlauf");
    assert!(REVIEWER_HTML.contains("verlauf-row-desc"),          "verlauf-row-desc class must be used in buildVerlauf");
    assert!(REVIEWER_HTML.contains("verlauf-row-time"),          "verlauf-row-time class must be used in buildVerlauf");
    assert!(REVIEWER_HTML.contains("verlaufEvents"),             "verlaufEvents key must be referenced in buildVerlauf");
}

/// buildVerlauf must be called from all three result paths.
#[test]
fn reviewer_verlauf_called_in_all_result_paths() {
    assert!(
        REVIEWER_HTML.contains("buildVerlauf(_ts)"),
        "confirmLocalDecision must call buildVerlauf"
    );
    assert!(
        REVIEWER_HTML.contains("buildVerlauf(_tsBlocked)"),
        "showResultBlocked must call buildVerlauf"
    );
    assert!(
        REVIEWER_HTML.contains("buildVerlauf(_tsResult)"),
        "showResult must call buildVerlauf"
    );
}

/// copyReceipt must include the Verlauf section in the copied plain text.
#[test]
fn reviewer_copy_receipt_includes_verlauf() {
    let pos = REVIEWER_HTML.find("function copyReceipt()").expect("copyReceipt must be defined");
    let snippet = &REVIEWER_HTML[pos..pos + 2400];
    assert!(snippet.contains("verlauf-rows"),  "copyReceipt must query verlauf-rows for copy text");
    assert!(snippet.contains("verlauf-row-lbl"), "copyReceipt must include verlauf-row-lbl in copy text");
    assert!(snippet.contains("verlaufLabel"),  "copyReceipt must use t.verlaufLabel as section header");
}

/// Safety note must remain unchanged after Verlauf addition.
#[test]
fn reviewer_safety_note_unchanged_after_verlauf() {
    assert!(
        REVIEWER_HTML.contains("PostCAD erkennt keine medizinischen oder technischen Fehler und gibt keine Herstellung frei."),
        "DE safety note must be unchanged"
    );
    assert!(
        REVIEWER_HTML.contains("PostCAD does not detect medical or technical errors and does not release manufacturing."),
        "EN safety note must be unchanged"
    );
}

// ── Praxis-Erklärung section tests ───────────────────────────────────────────

/// Praxis-Erklärung section must be present in the visual phase with simplified fields.
#[test]
fn reviewer_praxiserklaerung_section_present() {
    assert!(REVIEWER_HTML.contains("praxiserklaerung-section"),              "praxiserklaerung-section id must be present");
    assert!(REVIEWER_HTML.contains("t-praxiserklaerung-badge"),              "t-praxiserklaerung-badge id must be present");
    assert!(REVIEWER_HTML.contains("Schritt 2"),                              "Schritt 2 badge must be present");
    assert!(REVIEWER_HTML.contains("t-praxiserklaerung-sub"),                "t-praxiserklaerung-sub id must be present");
    assert!(REVIEWER_HTML.contains("WhatsApp"),                              "DE section subtext must mention WhatsApp/existing channels");
    assert!(REVIEWER_HTML.contains("praxis-rueckmeldung"),                   "praxis-rueckmeldung textarea id must be present");
    assert!(REVIEWER_HTML.contains("praxis-video-link"),                     "praxis-video-link input id must be present");
    assert!(REVIEWER_HTML.contains("Kurze Laborerkl"),                       "DE lab explanation label must be present");
    assert!(REVIEWER_HTML.contains("Was soll die Praxis tun"),               "DE practice action label must be present");
    assert!(REVIEWER_HTML.contains("Video-Link optional"),                   "DE video link label must be present");
    assert!(REVIEWER_HTML.contains("rueckmeldung-error"),                    "rueckmeldung-error id must be present for validation");
}

/// Reviewer must be clearly labelled as a lab tool in intro, step badges, and hints.
#[test]
fn reviewer_lab_first_wording_present() {
    // Intro
    assert!(REVIEWER_HTML.contains("Laboransicht"),               "intro must identify lab as actor");
    assert!(REVIEWER_HTML.contains("Lab view"),                   "EN intro must say Lab view");
    // Numbered steps
    assert!(REVIEWER_HTML.contains("Schritt 1"),                  "visual step must carry Schritt 1 badge");
    assert!(REVIEWER_HTML.contains("Step 1"),                     "EN visual step must carry Step 1 badge");
    assert!(REVIEWER_HTML.contains("Schritt 2"),                  "explanation step must carry Schritt 2 badge");
    assert!(REVIEWER_HTML.contains("Step 2"),                     "EN explanation step must carry Step 2 badge");
    assert!(REVIEWER_HTML.contains("Schritt 3"),                  "decision step must carry Schritt 3 badge");
    assert!(REVIEWER_HTML.contains("Step 3"),                     "EN decision step must carry Step 3 badge");
    // Decision actor wording
    assert!(REVIEWER_HTML.contains("Das Labor dokumentiert"),     "decision hints must explicitly name Das Labor");
    assert!(REVIEWER_HTML.contains("The lab documents"),          "EN decision hints must explicitly name The lab");
    // Result subtexts
    assert!(REVIEWER_HTML.contains("t-praxis-section-sub"),       "t-praxis-section-sub id must be present");
    assert!(REVIEWER_HTML.contains("bestehenden Kommunikationskanal"),    "praxis section subtext must mention existing communication channel");
    assert!(REVIEWER_HTML.contains("existing communication channel"), "EN praxis section subtext must mention existing communication channel");
    // Verlauf event 3 description
    assert!(REVIEWER_HTML.contains("Das Labor hat eine R\u{00fc}ckfrage"), "Verlauf event 3 must attribute action to Das Labor");
    assert!(REVIEWER_HTML.contains("The lab prepared"),            "EN Verlauf event 3 must attribute action to The lab");
}

/// Video link field must be a text/url input — not a file upload.
#[test]
fn reviewer_praxiserklaerung_video_link_is_link_only() {
    assert!(REVIEWER_HTML.contains("praxis-video-link"),  "praxis-video-link must be present");
    assert!(
        !REVIEWER_HTML.contains(r#"id="praxis-video-link" type="file""#),
        "praxis-video-link must not be a file input"
    );
    assert!(
        REVIEWER_HTML.contains("Optional"),
        "video link field must be marked as optional"
    );
}

/// Praxis-Erklärung kopieren button must be present with correct callback.
#[test]
fn reviewer_praxiserklaerung_copy_button_present() {
    assert!(REVIEWER_HTML.contains("copyPracticeExplanation()"),   "copy button must call copyPracticeExplanation()");
    assert!(REVIEWER_HTML.contains("t-copy-practice-btn"),         "t-copy-practice-btn translation id must be present");
    assert!(REVIEWER_HTML.contains("praxis-copy-confirm"),         "praxis-copy-confirm element must be present");
    assert!(REVIEWER_HTML.contains("Praxis-Text kopieren"), "DE copy practice button label must be present");
}

/// copyPracticeExplanation function must be defined and use clipboard API.
#[test]
fn reviewer_copy_practice_explanation_function_present() {
    assert!(REVIEWER_HTML.contains("function copyPracticeExplanation()"), "copyPracticeExplanation JS function must be defined");
    assert!(REVIEWER_HTML.contains("fillPraxisSection"),                  "fillPraxisSection helper must be defined");
    assert!(
        REVIEWER_HTML.contains("function copyPracticeExplanation()") &&
        REVIEWER_HTML.contains("navigator.clipboard"),
        "copyPracticeExplanation must attempt clipboard API"
    );
}

/// Copied practice explanation must include the safety wording for the practice.
#[test]
fn reviewer_copy_practice_explanation_includes_safety_wording() {
    let pos = REVIEWER_HTML.find("function copyPracticeExplanation()").expect("copyPracticeExplanation must be defined");
    let snippet = &REVIEWER_HTML[pos..pos + 2000];
    assert!(snippet.contains("praxisSafetyNote"),  "copyPracticeExplanation must append t.praxisSafetyNote");
    assert!(
        REVIEWER_HTML.contains("Diese Erkl\u{00e4}rung ersetzt keine medizinische"),
        "DE praxisSafetyNote text must be present in T strings"
    );
    assert!(
        REVIEWER_HTML.contains("This explanation does not replace medical"),
        "EN praxisSafetyNote text must be present in T strings"
    );
}

/// Result phase must include both practice-facing and internal nachweis sections.
#[test]
fn reviewer_receipt_includes_praxiserklaerung_rows() {
    assert!(REVIEWER_HTML.contains("praxis-section"),              "praxis-section id must be present in result");
    assert!(REVIEWER_HTML.contains("praxis-erklaerung"),           "praxis-erklaerung id must be present for practice display");
    assert!(REVIEWER_HTML.contains("praxis-aktion"),               "praxis-aktion id must be present for practice display");
    assert!(REVIEWER_HTML.contains("praxis-video-row"),            "praxis-video-row id must be present (conditional)");
    assert!(REVIEWER_HTML.contains("nachweis-praxis-rueckmeldung"),"nachweis-praxis-rueckmeldung id must be present in internal nachweis");
    assert!(REVIEWER_HTML.contains("nachweis-praxis-video-row"),   "nachweis-praxis-video-row id must be present in internal nachweis");
    assert!(REVIEWER_HTML.contains("t-intern-nachweis-badge"),     "intern nachweis badge must be present");
}

/// Praxis-Anfrage kopieren button must be present in the explanation step.
#[test]
fn reviewer_copy_practice_request_button_present() {
    assert!(REVIEWER_HTML.contains("copyPracticeRequest()"),         "copy button must call copyPracticeRequest()");
    assert!(REVIEWER_HTML.contains("t-copy-practice-request-btn"),   "t-copy-practice-request-btn span id must be present");
    assert!(REVIEWER_HTML.contains("praxis-request-copy-confirm"),   "praxis-request-copy-confirm confirm element must be present");
    assert!(REVIEWER_HTML.contains("Praxis-Anfrage kopieren"),       "DE copy practice request button label must be present");
    assert!(REVIEWER_HTML.contains("Copy practice query"),           "EN copy practice request button label must be present");
}

/// copyPracticeRequest function must be defined and include required text.
#[test]
fn reviewer_copy_practice_request_function_present() {
    assert!(REVIEWER_HTML.contains("function copyPracticeRequest()"), "copyPracticeRequest JS function must be defined");
    let pos = REVIEWER_HTML.find("function copyPracticeRequest()").expect("copyPracticeRequest must be defined");
    let snippet = &REVIEWER_HTML[pos..pos + 2000];
    assert!(snippet.contains("navigator.clipboard"),           "copyPracticeRequest must attempt clipboard API");
    assert!(snippet.contains("copyPracticeRequestPrompt"),     "copyPracticeRequest must include the confirmation request prompt");
    assert!(snippet.contains("copyPracticeRequestSafety"),     "copyPracticeRequest must include safety wording");
}

/// The copied practice request must include the safety wording.
#[test]
fn reviewer_copy_practice_request_includes_safety_wording() {
    assert!(
        REVIEWER_HTML.contains("Diese Nachricht ersetzt keine medizinische"),
        "DE copyPracticeRequestSafety text must be present in T strings"
    );
    assert!(
        REVIEWER_HTML.contains("This message does not replace medical"),
        "EN copyPracticeRequestSafety text must be present in T strings"
    );
    assert!(
        REVIEWER_HTML.contains("Bitte best\u{00e4}tigen Sie kurz"),
        "DE copyPracticeRequestPrompt confirmation request must be present"
    );
}

/// Practice response documentation section (Step 3) must be present.
#[test]
fn reviewer_practice_response_section_present() {
    assert!(REVIEWER_HTML.contains("praxis-rueckmeldung-section"),    "praxis-rueckmeldung-section id must be present");
    assert!(REVIEWER_HTML.contains("t-praxis-rueckmeldung-badge"),    "t-praxis-rueckmeldung-badge id must be present");
    assert!(REVIEWER_HTML.contains("Schritt 3"),                      "Step 3 badge must be present");
    assert!(REVIEWER_HTML.contains("Step 3"),                         "EN Step 3 badge must be present");
    assert!(REVIEWER_HTML.contains("praxis-antwort"),                 "praxis-antwort textarea id must be present");
    assert!(REVIEWER_HTML.contains("praxis-antwort-status"),          "praxis-antwort-status select id must be present");
    assert!(REVIEWER_HTML.contains("praxis-antwort-error"),           "praxis-antwort-error validation element must be present");
    assert!(REVIEWER_HTML.contains("Praxis best\u{00e4}tigt Fortsetzung"), "DE confirm status option must be present");
    assert!(REVIEWER_HTML.contains("Practice confirms continuation"), "EN confirm status option must be present");
}

/// Internal nachweis must include practice response rows.
#[test]
fn reviewer_internal_proof_includes_practice_response() {
    assert!(REVIEWER_HTML.contains("nachweis-praxis-antwort"),        "nachweis-praxis-antwort id must be present in internal nachweis");
    assert!(REVIEWER_HTML.contains("nachweis-praxis-antwort-status"), "nachweis-praxis-antwort-status id must be present");
    assert!(REVIEWER_HTML.contains("t-nachweis-praxis-antwort-lbl"),  "t-nachweis-praxis-antwort-lbl id must be present");
    assert!(REVIEWER_HTML.contains("Praxis-R\u{00fc}ckmeldung"),      "DE practice response label must be present in nachweis");
    assert!(REVIEWER_HTML.contains("Practice response"),              "EN practice response label must be present in nachweis");
}

/// Step 4 decision wording must be present.
#[test]
fn reviewer_step4_decision_wording_present() {
    assert!(REVIEWER_HTML.contains("Schritt 4"),                           "decision section must carry Schritt 4 badge");
    assert!(REVIEWER_HTML.contains("Step 4"),                              "EN decision section must carry Step 4 badge");
    assert!(REVIEWER_HTML.contains("Entscheidung vor Herstellung dokumentieren"), "decision section title must be present");
}

/// Verlauf must include Praxis-Anfrage and Praxis-Rückmeldung events.
#[test]
fn reviewer_verlauf_includes_practice_handoff_events() {
    assert!(REVIEWER_HTML.contains("Praxis-Anfrage vorbereitet"),            "DE Praxis-Anfrage event must be in verlauf");
    assert!(REVIEWER_HTML.contains("Practice query prepared"),               "EN practice query event must be in verlauf");
    assert!(REVIEWER_HTML.contains("Praxis-Antwort dokumentiert"),         "DE practice response event must be in verlauf");
    assert!(REVIEWER_HTML.contains("Practice response documented"),          "EN practice response event must be in verlauf");
}

/// Intro line must explain that the practice sends the case via existing channels and
/// that PostCAD starts at the lab clarification step.
#[test]
fn reviewer_intro_explains_existing_channels() {
    assert!(REVIEWER_HTML.contains("Die Praxis sendet den Fall wie bisher"), "DE intro must state practice sends case as usual");
    assert!(REVIEWER_HTML.contains("PostCAD beginnt"),                        "DE intro must explain when PostCAD starts");
    assert!(REVIEWER_HTML.contains("Lab view"),                               "EN intro must identify lab view");
}

/// Typical workflow note must be present near the top in both DE and EN.
#[test]
fn reviewer_workflow_note_present() {
    assert!(REVIEWER_HTML.contains("t-workflow-note"),     "t-workflow-note element must be present");
    assert!(REVIEWER_HTML.contains("Typischer Ablauf"),    "DE workflow note must be present");
    assert!(REVIEWER_HTML.contains("Typical workflow"),    "EN workflow note must be present");
}

/// Step 2 subtext must mention WhatsApp, E-Mail, and existing channel as sending options.
#[test]
fn reviewer_step2_mentions_existing_channels() {
    assert!(REVIEWER_HTML.contains("WhatsApp"),            "Step 2 subtext must mention WhatsApp");
    assert!(REVIEWER_HTML.contains("E-Mail"),              "Step 2 subtext must mention E-Mail");
    assert!(REVIEWER_HTML.contains("Praxis-/Labor-Kanal"),"Step 2 subtext must mention existing practice/lab channel");
}

/// Helper text under copy button must clarify PostCAD does not send automatically.
#[test]
fn reviewer_copy_request_helper_text_present() {
    assert!(REVIEWER_HTML.contains("t-copy-request-helper"),                     "t-copy-request-helper element must be present");
    assert!(REVIEWER_HTML.contains("PostCAD versendet noch nicht automatisch"),   "DE helper text must say PostCAD does not send automatically");
    assert!(REVIEWER_HTML.contains("PostCAD does not send automatically yet"),    "EN helper text must be present");
}

/// Praxis section subtext must reference the existing communication channel.
#[test]
fn reviewer_praxis_section_mentions_communication_channel() {
    assert!(REVIEWER_HTML.contains("bestehenden Kommunikationskanal"), "DE praxis section must mention existing communication channel");
    assert!(REVIEWER_HTML.contains("existing communication channel"),  "EN praxis section must mention existing communication channel");
}

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
