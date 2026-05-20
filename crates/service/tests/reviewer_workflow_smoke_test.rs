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
    assert!(html.contains("phase-upload"),               "phase-upload id must be present");
    assert!(html.contains("upload-zone"),                "upload-zone id must be present");
    assert!(html.contains("file-input"),                 "file-input id must be present");
    assert!(html.contains("STL-Datens\u{00e4}tze lokal laden"), "upload title DE must say STL-Datensätze lokal laden");
    assert!(html.contains("Load STL datasets locally"),        "upload title EN must say Load STL datasets locally");
}

/// Upload area must allow one or two STL files with correct labelling.
#[test]
fn reviewer_multi_stl_upload_text() {
    assert!(REVIEWER_HTML.contains("STL-Datens\u{00e4}tze lokal laden"),  "DE upload title must say STL-Datensätze lokal laden");
    assert!(REVIEWER_HTML.contains("Load STL datasets locally"),           "EN upload title must say Load STL datasets locally");
    assert!(REVIEWER_HTML.contains("Oberkiefer und Unterkiefer"),          "DE upload sub must mention Oberkiefer und Unterkiefer");
    assert!(REVIEWER_HTML.contains("upper and lower jaw"),                 "EN upload sub must mention upper and lower jaw");
    assert!(REVIEWER_HTML.contains("Die Dateien bleiben lokal"),           "local-only wording must be present in upload sub");
    assert!(REVIEWER_HTML.contains("Files stay local"),                    "EN local-only wording must be present");
    assert!(REVIEWER_HTML.contains("multiple"),                            "file-input must have multiple attribute");
}

/// Multi-STL controls section with show/hide toggles must be present.
#[test]
fn reviewer_multi_stl_controls_present() {
    assert!(REVIEWER_HTML.contains("stl-datasets-section"),      "stl-datasets-section id must be present");
    assert!(REVIEWER_HTML.contains("stl1-toggle"),               "stl1-toggle checkbox id must be present");
    assert!(REVIEWER_HTML.contains("stl2-toggle"),               "stl2-toggle checkbox id must be present");
    assert!(REVIEWER_HTML.contains("STL 1 anzeigen"),            "STL 1 toggle label DE must be present");
    assert!(REVIEWER_HTML.contains("STL 2 anzeigen"),            "STL 2 toggle label DE must be present");
    assert!(REVIEWER_HTML.contains("toggleStl1"),                "toggleStl1 function must be callable");
    assert!(REVIEWER_HTML.contains("toggleStl2"),                "toggleStl2 function must be callable");
    assert!(REVIEWER_HTML.contains("function toggleStl1"),       "toggleStl1 function must be defined");
    assert!(REVIEWER_HTML.contains("function toggleStl2"),       "toggleStl2 function must be defined");
    assert!(REVIEWER_HTML.contains("t-stl-datasets-badge"),      "t-stl-datasets-badge id must be present");
    assert!(REVIEWER_HTML.contains("t-stl-datasets-hint"),       "t-stl-datasets-hint id must be present");
    assert!(REVIEWER_HTML.contains("STL-Datens\u{00e4}tze anzeigen"), "DE badge must say STL-Datensätze anzeigen");
    assert!(REVIEWER_HTML.contains("Show STL datasets"),         "EN badge must say Show STL datasets");
}

/// Simplified multi-STL UI must have show/hide checkboxes and a hint stating
/// no automatic alignment and no occlusion check.
#[test]
fn reviewer_multi_stl_simple_show_hide_controls() {
    assert!(REVIEWER_HTML.contains("STL 1 anzeigen"),                 "STL 1 toggle label DE must be present");
    assert!(REVIEWER_HTML.contains("STL 2 anzeigen"),                 "STL 2 toggle label DE must be present");
    assert!(REVIEWER_HTML.contains("Keine automatische Ausrichtung"), "DE hint must state no automatic alignment");
    assert!(REVIEWER_HTML.contains("keine Okklusionspr\u{00fc}fung"),"DE hint must state no occlusion inspection");
}

/// Filename labels STL 1 / STL 2 must be shown in the datasets section.
#[test]
fn reviewer_multi_stl_filename_labels() {
    assert!(REVIEWER_HTML.contains("stl-filename-1"),    "stl-filename-1 span id must be present");
    assert!(REVIEWER_HTML.contains("stl-filename-2"),    "stl-filename-2 span id must be present");
    assert!(REVIEWER_HTML.contains("stl-filename-labels"), "stl-filename-labels container must be present");
}

/// Two-dataset note must reflect the simplified wording: datasets shown as exported,
/// no automatic alignment, no occlusion check.
#[test]
fn reviewer_multi_stl_two_dataset_note() {
    assert!(REVIEWER_HTML.contains("stl-multi-note"),                        "stl-multi-note element id must be present");
    assert!(REVIEWER_HTML.contains("2 STL-Datens\u{00e4}tze geladen"),      "DE note must say 2 STL-Datensätze geladen");
    assert!(REVIEWER_HTML.contains("wie sie exportiert wurden"),             "hint must say datasets shown as exported");
    assert!(REVIEWER_HTML.contains("Keine automatische Ausrichtung"),        "DE hint must state no automatic alignment");
    assert!(REVIEWER_HTML.contains("keine Okklusionspr\u{00fc}fung"),       "DE hint must state no occlusion inspection");
    assert!(REVIEWER_HTML.contains("stlMultiNote"),                          "stlMultiNote T key must be referenced");
}

/// Hint text must clearly state no automatic alignment and no occlusion inspection.
#[test]
fn reviewer_multi_stl_no_auto_alignment() {
    assert!(REVIEWER_HTML.contains("Keine automatische Ausrichtung"),  "DE multi-STL hint must state no automatic alignment");
    assert!(REVIEWER_HTML.contains("No automatic alignment"),          "EN multi-STL hint must state no automatic alignment");
    assert!(REVIEWER_HTML.contains("keine Okklusionspr\u{00fc}fung"), "DE hint must state no occlusion inspection");
    assert!(REVIEWER_HTML.contains("no occlusion inspection"),         "EN hint must state no occlusion inspection");
}

/// showVisualStep must not reference the removed display-mode-together element.
/// If it did, getElementById returns null and .checked throws, hanging the processing screen.
#[test]
fn reviewer_show_visual_step_no_stale_display_mode_reference() {
    let pos = REVIEWER_HTML
        .find("function showVisualStep(")
        .expect("showVisualStep must be defined");
    let after = &REVIEWER_HTML[pos..];
    let fn_end = after
        .find("function proceedToDecision")
        .unwrap_or(after.len());
    let body = &after[..fn_end];
    assert!(
        !body.contains("display-mode-together"),
        "showVisualStep must not reference display-mode-together (element was removed; null.checked throws)"
    );
}

/// Two-file upload must use a _done counter so _enableStartReview is called exactly
/// once after both FileReaders complete, regardless of order.
#[test]
fn reviewer_two_file_upload_synchronizes_readers() {
    let pos = REVIEWER_HTML
        .find("function onFileInput(")
        .expect("onFileInput must be defined");
    let after = &REVIEWER_HTML[pos..pos + 1200];
    assert!(after.contains("_done++"),               "two-file branch must increment a _done counter");
    assert!(after.contains("_done === 2"),           "two-file branch must gate _onBoth on _done reaching 2");
    assert!(after.contains("_enableStartReview()"),  "two-file _onBoth must call _enableStartReview");
}

/// When the second FileReader fails, _pendingFilename2 must be cleared so stale
/// filename state does not bleed into the visual step.
#[test]
fn reviewer_two_file_second_read_failure_clears_pending_filename() {
    let pos = REVIEWER_HTML
        .find("function onFileInput(")
        .expect("onFileInput must be defined");
    let after = &REVIEWER_HTML[pos..pos + 1200];
    assert!(
        after.contains("_pendingFilename2 = null"),
        "_r2.onerror must clear _pendingFilename2 on second-file read failure"
    );
}

/// Single-file upload path must stage _pendingBuffer via FileReader.onload and call
/// _enableStartReview — not startProcessing directly. startReview() is the deliberate gate.
#[test]
fn reviewer_single_file_upload_path_preserved() {
    let pos = REVIEWER_HTML
        .find("function onFileInput(")
        .expect("onFileInput must be defined");
    let after = &REVIEWER_HTML[pos..pos + 1200];
    assert!(
        after.contains("_pendingBuffer = e.target.result; _enableStartReview()"),
        "single-file onload must set _pendingBuffer and call _enableStartReview"
    );
    assert!(
        after.contains("_pendingBuffer2 = null"),
        "single-file branch must clear _pendingBuffer2"
    );
}

/// Upload phase must show a staged-files section and a "Fall prüfen" button that the
/// user must click before the viewer opens — files are staged on selection, not immediately
/// processed.
#[test]
fn reviewer_staged_upload_ui_present() {
    assert!(REVIEWER_HTML.contains("staged-files-section"),  "staged-files-section id must be present");
    assert!(REVIEWER_HTML.contains("staged-filename-1"),     "staged-filename-1 span id must be present");
    assert!(REVIEWER_HTML.contains("staged-filename-2-row"), "staged-filename-2-row id must be present");
    assert!(REVIEWER_HTML.contains("staged-filename-2"),     "staged-filename-2 span id must be present");
    assert!(REVIEWER_HTML.contains("start-review-btn"),      "start-review-btn id must be present");
    assert!(REVIEWER_HTML.contains("t-start-review-btn"),    "t-start-review-btn span id must be present");
    assert!(REVIEWER_HTML.contains("startReview()"),         "start-review-btn must call startReview()");
}

/// "Fall prüfen" button must start disabled so the user cannot click it before a file
/// is staged and the FileReader has finished reading the buffer.
#[test]
fn reviewer_start_review_btn_initially_disabled() {
    let pos = REVIEWER_HTML
        .find("id=\"start-review-btn\"")
        .expect("start-review-btn must be present");
    let snippet = &REVIEWER_HTML[pos..pos + 120];
    assert!(
        snippet.contains("disabled"),
        "start-review-btn must be disabled in the initial HTML"
    );
}

/// _showStagedFiles, _enableStartReview, and startReview must all be defined.
#[test]
fn reviewer_staged_upload_helper_functions_present() {
    assert!(REVIEWER_HTML.contains("function _showStagedFiles("),  "_showStagedFiles function must be defined");
    assert!(REVIEWER_HTML.contains("function _enableStartReview("),"_enableStartReview function must be defined");
    assert!(REVIEWER_HTML.contains("function startReview()"),       "startReview function must be defined");
}

/// startReview must call startProcessing with the staged primary filename.
#[test]
fn reviewer_start_review_calls_start_processing() {
    let pos = REVIEWER_HTML
        .find("function startReview()")
        .expect("startReview must be defined");
    let snippet = &REVIEWER_HTML[pos..pos + 200];
    assert!(
        snippet.contains("startProcessing(_stagedPrimaryFilename)"),
        "startReview must call startProcessing with _stagedPrimaryFilename"
    );
}

/// Upload local note must be present in both DE and EN, separate from the old
/// privacy notice, and must use the t-upload-local-note element id.
#[test]
fn reviewer_upload_local_note_present() {
    assert!(
        REVIEWER_HTML.contains("t-upload-local-note"),
        "t-upload-local-note element id must be present"
    );
    assert!(
        REVIEWER_HTML.contains("bleiben lokal im Browser und werden erst nach Klick"),
        "DE upload local note must state files stay local until the button is clicked"
    );
    assert!(
        REVIEWER_HTML.contains("Files stay local in your browser and are only opened after clicking"),
        "EN upload local note must be present"
    );
}

/// resetDemo must clear staged state so a fresh upload starts from scratch.
#[test]
fn reviewer_reset_demo_clears_staged_state() {
    let pos = REVIEWER_HTML.find("function resetDemo()").expect("resetDemo must be defined");
    // resetDemo is a long function; use a 6 000-char window to cover all reset lines
    let body = &REVIEWER_HTML[pos..pos + 6000];
    assert!(
        body.contains("_stagedPrimaryFilename = null"),
        "resetDemo must clear _stagedPrimaryFilename"
    );
    assert!(
        body.contains("staged-files-section"),
        "resetDemo must hide staged-files-section"
    );
    assert!(
        body.contains("start-review-btn"),
        "resetDemo must reset start-review-btn"
    );
}

/// Nachweis must include an STL-Datensätze row.
#[test]
fn reviewer_nachweis_includes_stl_datensaetze_row() {
    assert!(REVIEWER_HTML.contains("nachweis-stl-datensaetze-row"),   "nachweis-stl-datensaetze-row id must be present");
    assert!(REVIEWER_HTML.contains("nachweis-stl-datensaetze"),       "nachweis-stl-datensaetze value span must be present");
    assert!(REVIEWER_HTML.contains("t-nachweis-stl-datensaetze-lbl"), "t-nachweis-stl-datensaetze-lbl id must be present");
    assert!(
        REVIEWER_HTML.contains("STL-Datens\u{00e4}tze"),
        "STL-Datensätze label must be present in nachweis"
    );
}

/// Praxis-Nachricht must include a note when two STL datasets are loaded.
#[test]
fn reviewer_multi_stl_praxis_nachricht_note() {
    let pos = REVIEWER_HTML.find("function buildPraxisNachrichtText()").expect("buildPraxisNachrichtText must be defined");
    let snippet = &REVIEWER_HTML[pos..pos + 2200];
    assert!(snippet.contains("_pendingBuffer2"),    "buildPraxisNachrichtText must check _pendingBuffer2 for two-STL note");
    assert!(snippet.contains("stlMultiPraxisNote"), "buildPraxisNachrichtText must include stlMultiPraxisNote when two files loaded");
    assert!(
        REVIEWER_HTML.contains("Es wurden zwei STL-Datens\u{00e4}tze"),
        "DE stlMultiPraxisNote text must be present in T strings"
    );
    assert!(
        REVIEWER_HTML.contains("Two STL datasets were loaded"),
        "EN stlMultiPraxisNote text must be present in T strings"
    );
    assert!(
        REVIEWER_HTML.contains("keine automatische Ausrichtung oder Okklusionspr\u{00fc}fung"),
        "DE praxis note must state no alignment and no occlusion inspection"
    );
    assert!(
        REVIEWER_HTML.contains("no automatic alignment or occlusion inspection"),
        "EN praxis note must state no alignment and no occlusion inspection"
    );
}

/// Nachweis STL-Datensätze row must represent 1 or 2 files loaded locally.
#[test]
fn reviewer_nachweis_stl_datensaetze_values() {
    assert!(REVIEWER_HTML.contains("nachweis-stl-datensaetze-row"),  "nachweis-stl-datensaetze-row id must be present");
    assert!(REVIEWER_HTML.contains("nachweis-stl-datensaetze"),      "nachweis-stl-datensaetze value span must be present");
    assert!(REVIEWER_HTML.contains("1 Datei lokal geladen"),         "DE 1-file nachweis value must be present");
    assert!(REVIEWER_HTML.contains("2 Dateien lokal geladen"),       "DE 2-file nachweis value must be present");
    assert!(REVIEWER_HTML.contains("stlMultiNachweis1"),             "stlMultiNachweis1 T key must be referenced");
    assert!(REVIEWER_HTML.contains("stlMultiNachweis2"),             "stlMultiNachweis2 T key must be referenced");
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
    assert!(html.contains("Entscheidung sauber dokumentieren"), "DE intro line must mention Entscheidung sauber dokumentieren");
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

/// onFileInput must stage the buffer inside reader.onload, then call _enableStartReview —
/// not call startProcessing directly, so the user must click "Fall prüfen" to proceed.
#[test]
fn stl_file_input_waits_for_filereader() {
    assert!(
        REVIEWER_HTML.contains("reader.onload = function(e) { _pendingBuffer = e.target.result; _enableStartReview(); }"),
        "onFileInput must stage buffer via reader.onload and call _enableStartReview, not startProcessing directly"
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
    assert!(html.contains("LABORAKTION FESTHALTEN"),         "gate badge DE must be present");
    assert!(html.contains("RECORD LAB ACTION"),              "gate badge EN must be present");
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
    assert!(REVIEWER_HTML.contains("Laboraktion festgehalten"),                  "DE event 5 label must be present");
    assert!(REVIEWER_HTML.contains("Entscheidungsnachweis erstellt"),            "DE event 6 label must be present");
}

/// All 6 EN event labels must be present in the translation table.
#[test]
fn reviewer_verlauf_en_event_labels_present() {
    assert!(REVIEWER_HTML.contains("STL file loaded locally"),                   "EN event 1 label must be present");
    assert!(REVIEWER_HTML.contains("Lab case reviewed visually"),                "EN event 2 label must be present");
    assert!(REVIEWER_HTML.contains("Practice query prepared"),                   "EN event 3 label must be present");
    assert!(REVIEWER_HTML.contains("Practice response documented"),              "EN event 4 label must be present");
    assert!(REVIEWER_HTML.contains("Lab action recorded"),                       "EN event 5 label must be present");
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
    assert!(REVIEWER_HTML.contains("Das Labor startet"),          "decision hints must explicitly name Das Labor");
    assert!(REVIEWER_HTML.contains("The lab starts manufacturing"), "EN decision hints must explicitly name The lab");
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

/// Praxis-Nachricht kopieren button must be present in the Erklärungspaket section.
/// The legacy copyPracticeRequest function is kept for compatibility.
#[test]
fn reviewer_copy_practice_request_button_present() {
    assert!(REVIEWER_HTML.contains("copyPracticeRequest()"),         "copyPracticeRequest function must be callable");
    assert!(REVIEWER_HTML.contains("t-copy-practice-request-btn"),   "t-copy-practice-request-btn span id must be present");
    assert!(REVIEWER_HTML.contains("praxis-request-copy-confirm"),   "praxis-request-copy-confirm confirm element must be present");
    assert!(REVIEWER_HTML.contains("Praxis-Anfrage kopieren"),       "DE copy practice request label must be present in T strings");
    assert!(REVIEWER_HTML.contains("Copy practice query"),           "EN copy practice request label must be present in T strings");
    // New Erklärungspaket copy button
    assert!(REVIEWER_HTML.contains("copyPraxisNachricht()"),         "copyPraxisNachricht must be callable");
    assert!(REVIEWER_HTML.contains("t-praxis-nachricht-copy-btn"),   "t-praxis-nachricht-copy-btn must be present");
    assert!(REVIEWER_HTML.contains("praxis-nachricht-copy-confirm"), "praxis-nachricht-copy-confirm must be present");
    assert!(REVIEWER_HTML.contains("Praxis-Nachricht kopieren"),     "DE Praxis-Nachricht copy button label must be present");
    assert!(REVIEWER_HTML.contains("Copy practice message"),         "EN practice message copy button label must be present");
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

/// Step 4 laboraktion wording must be present.
#[test]
fn reviewer_step4_decision_wording_present() {
    assert!(REVIEWER_HTML.contains("Schritt 4"),                           "decision section must carry Schritt 4 badge");
    assert!(REVIEWER_HTML.contains("Step 4"),                              "EN decision section must carry Step 4 badge");
    assert!(REVIEWER_HTML.contains("Laboraktion festhalten"),              "DE step 4 title must be Laboraktion festhalten");
    assert!(REVIEWER_HTML.contains("Record lab action"),                   "EN step 4 title must be Record lab action");
}

/// Step 4 option labels must use Herstellung-oriented wording.
#[test]
fn reviewer_laboraktion_option_labels() {
    assert!(REVIEWER_HTML.contains("Herstellung starten"),                  "DE option proceed must be Herstellung starten");
    assert!(REVIEWER_HTML.contains("Herstellung mit Hinweis starten"),     "DE option risk must be Herstellung mit Hinweis starten");
    assert!(REVIEWER_HTML.contains("Herstellung nicht starten"),           "DE option block must be Herstellung nicht starten");
    assert!(REVIEWER_HTML.contains("Laboraktion best&#228;tigen")
        || REVIEWER_HTML.contains("Laboraktion best\u{00e4}tigen"),        "DE confirm button must be Laboraktion bestätigen");
    assert!(REVIEWER_HTML.contains("Start manufacturing"),                  "EN option proceed must be Start manufacturing");
    assert!(REVIEWER_HTML.contains("Start manufacturing with note"),        "EN option risk must be present");
    assert!(REVIEWER_HTML.contains("Do not start manufacturing"),           "EN option block must be present");
    assert!(REVIEWER_HTML.contains("Confirm lab action"),                   "EN confirm button must be Confirm lab action");
}

/// showDecisionGate must include auto-suggest logic based on Praxis response status.
#[test]
fn reviewer_laboraktion_auto_suggest() {
    assert!(REVIEWER_HTML.contains("autoMap"),                              "auto-suggest map must be present in showDecisionGate");
    assert!(REVIEWER_HTML.contains("confirm: 'proceed'"),                   "autoMap must map confirm to proceed");
    assert!(REVIEWER_HTML.contains("correction: 'request_correction'"),    "autoMap must map correction to request_correction");
    assert!(REVIEWER_HTML.contains("praxisAntwortStatus && autoMap"),      "auto-suggest must guard on praxisAntwortStatus");
}

/// Nachweis labels must use Laboraktion terminology.
#[test]
fn reviewer_laboraktion_nachweis_labels() {
    assert!(REVIEWER_HTML.contains("Lab action documented"),                "EN audit label must be Lab action documented");
    assert!(REVIEWER_HTML.contains("Lab action"),                           "EN nachweis decision label must be Lab action");
}

/// Verlauf event 5 must use Laboraktion terminology.
#[test]
fn reviewer_verlauf_laboraktion_event() {
    assert!(REVIEWER_HTML.contains("Laboraktion festgehalten"),             "DE verlauf event 5 must be Laboraktion festgehalten");
    assert!(REVIEWER_HTML.contains("Lab action recorded"),                  "EN verlauf event 5 must be Lab action recorded");
    assert!(REVIEWER_HTML.contains("ob die Herstellung startet"),          "DE verlauf event 5 desc must mention Herstellung");
    assert!(REVIEWER_HTML.contains("whether manufacturing starts"),         "EN verlauf event 5 desc must mention manufacturing");
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

/// Erklärung aufnehmen section must be present with badge, subtext, and buttons.
#[test]
fn reviewer_erklarclip_section_present() {
    assert!(REVIEWER_HTML.contains("erklarclip-section"),              "erklarclip-section id must be present");
    assert!(REVIEWER_HTML.contains("t-erklarclip-badge"),              "t-erklarclip-badge id must be present");
    assert!(REVIEWER_HTML.contains("t-erklarclip-sub"),                "t-erklarclip-sub id must be present");
    assert!(REVIEWER_HTML.contains("Erkl\u{00e4}rung aufnehmen"),      "DE badge text must be present");
    assert!(REVIEWER_HTML.contains("Record explanation"),              "EN badge text must be present");
    assert!(REVIEWER_HTML.contains("bleibt lokal"),                    "DE subtext must mention file stays local");
}

/// Record, stop, and download buttons must all be present.
#[test]
fn reviewer_erklarclip_buttons_present() {
    assert!(REVIEWER_HTML.contains("clip-record-btn"),              "clip-record-btn id must be present");
    assert!(REVIEWER_HTML.contains("clip-stop-btn"),                "clip-stop-btn id must be present");
    assert!(REVIEWER_HTML.contains("clip-download-btn"),            "clip-download-btn id must be present");
    assert!(REVIEWER_HTML.contains("t-clip-record-btn"),            "t-clip-record-btn span id must be present");
    assert!(REVIEWER_HTML.contains("t-clip-stop-btn"),              "t-clip-stop-btn span id must be present");
    assert!(REVIEWER_HTML.contains("t-clip-download-btn"),          "t-clip-download-btn span id must be present");
    assert!(REVIEWER_HTML.contains("Erkl\u{00e4}rclip aufnehmen"), "DE record button label must be present");
    assert!(REVIEWER_HTML.contains("Aufnahme stoppen"),             "DE stop button label must be present");
    assert!(REVIEWER_HTML.contains("Clip herunterladen"),           "DE download button label must be present");
}

/// MediaRecorder and composite canvas captureStream logic must be present in startClipRecording.
#[test]
fn reviewer_erklarclip_recording_logic_present() {
    assert!(REVIEWER_HTML.contains("function startClipRecording()"),  "startClipRecording function must be defined");
    assert!(REVIEWER_HTML.contains("function stopClipRecording()"),   "stopClipRecording function must be defined");
    assert!(REVIEWER_HTML.contains("function downloadClip()"),        "downloadClip function must be defined");
    assert!(REVIEWER_HTML.contains("MediaRecorder"),                  "MediaRecorder must be referenced");
    assert!(REVIEWER_HTML.contains("_compositeCanvas.captureStream"), "composite canvas captureStream must be used");
    assert!(REVIEWER_HTML.contains("getUserMedia"),                   "getUserMedia must be used for microphone");
    assert!(REVIEWER_HTML.contains("postcad-erklaerclip-"),           "generated filename must contain postcad-erklaerclip-");
    assert!(REVIEWER_HTML.contains(".webm"),                          "recording must target webm format");
}

/// Privacy wording must clearly state the clip is not uploaded or stored.
#[test]
fn reviewer_erklarclip_privacy_wording_present() {
    assert!(REVIEWER_HTML.contains("t-clip-privacy"),                           "t-clip-privacy id must be present");
    assert!(REVIEWER_HTML.contains("nicht hochgeladen oder gespeichert"),        "DE privacy text must be present");
    assert!(REVIEWER_HTML.contains("not uploaded or stored"),                    "EN privacy text must be present");
}

/// copyPracticeRequest must include the recorded clip note when a clip exists.
#[test]
fn reviewer_copy_practice_request_includes_clip_note() {
    let pos = REVIEWER_HTML.find("function copyPracticeRequest()").expect("copyPracticeRequest must be defined");
    let snippet = &REVIEWER_HTML[pos..pos + 2400];
    assert!(snippet.contains("_clipBlobUrl"),                      "copyPracticeRequest must check _clipBlobUrl");
    assert!(snippet.contains("clipRecordedNote"),                   "copyPracticeRequest must include t.clipRecordedNote");
    assert!(
        REVIEWER_HTML.contains("lokal erstellt und als Datei beizuf"),
        "DE clipRecordedNote text must be present in T strings"
    );
}

/// Internal nachweis must include an Erklärclip row.
#[test]
fn reviewer_nachweis_includes_erklarclip_row() {
    assert!(REVIEWER_HTML.contains("nachweis-erklarclip-row"),     "nachweis-erklarclip-row id must be present");
    assert!(REVIEWER_HTML.contains("nachweis-erklarclip"),         "nachweis-erklarclip value span id must be present");
    assert!(REVIEWER_HTML.contains("t-nachweis-erklarclip-lbl"),   "t-nachweis-erklarclip-lbl id must be present");
    assert!(REVIEWER_HTML.contains("Erkl\u{00e4}rclip"),          "Erklärclip label must appear in nachweis section");
}

/// Erklärungspaket section must be present with badge, sub, and three checklist steps.
#[test]
fn reviewer_erklarungspaket_section_present() {
    assert!(REVIEWER_HTML.contains("erklarungspaket-section"),              "erklarungspaket-section id must be present");
    assert!(REVIEWER_HTML.contains("t-erklarungspaket-badge"),              "t-erklarungspaket-badge id must be present");
    assert!(REVIEWER_HTML.contains("t-erklarungspaket-sub"),                "t-erklarungspaket-sub id must be present");
    assert!(REVIEWER_HTML.contains("Erkl\u{00e4}rungspaket f\u{00fc}r die Praxis"), "DE badge text must be present");
    assert!(REVIEWER_HTML.contains("Explanation package for the practice"), "EN badge text must be present");
    assert!(REVIEWER_HTML.contains("Paket an die Praxis senden"),           "DE sub text must be present");
}

/// Erklärungspaket checklist steps must all be present.
#[test]
fn reviewer_erklarungspaket_checklist_present() {
    assert!(REVIEWER_HTML.contains("t-erklarungspaket-step1"), "step 1 id must be present");
    assert!(REVIEWER_HTML.contains("t-erklarungspaket-step2"), "step 2 id must be present");
    assert!(REVIEWER_HTML.contains("t-erklarungspaket-step3"), "step 3 id must be present");
    assert!(REVIEWER_HTML.contains("erklarungspaket-steps"),   "erklarungspaket-steps list must be present");
    assert!(REVIEWER_HTML.contains("erklarungspaket-step-num"), "erklarungspaket-step-num class must be present");
    assert!(REVIEWER_HTML.contains("als Datei beif"),           "DE step 1 clip download text must be present");
    assert!(REVIEWER_HTML.contains("Praxis-Nachricht unten kopieren"), "DE step 2 text must be present");
    assert!(REVIEWER_HTML.contains("bestehendem Kanal an die Praxis senden"), "DE step 3 text must be present");
}

/// Praxis-Nachricht textarea and copy button must be present in Erklärungspaket.
#[test]
fn reviewer_praxis_nachricht_textarea_present() {
    assert!(REVIEWER_HTML.contains("praxis-nachricht-textarea"),     "praxis-nachricht-textarea id must be present");
    assert!(REVIEWER_HTML.contains("t-praxis-nachricht-label"),      "t-praxis-nachricht-label id must be present");
    assert!(REVIEWER_HTML.contains("Praxis-Nachricht\u{003c}"),      "DE label text must be present");
}

/// copyPraxisNachricht and buildPraxisNachrichtText functions must be defined.
#[test]
fn reviewer_praxis_nachricht_functions_present() {
    assert!(REVIEWER_HTML.contains("function copyPraxisNachricht()"),        "copyPraxisNachricht function must be defined");
    assert!(REVIEWER_HTML.contains("function buildPraxisNachrichtText()"),   "buildPraxisNachrichtText function must be defined");
    assert!(REVIEWER_HTML.contains("function updatePraxisNachricht()"),      "updatePraxisNachricht function must be defined");
    let pos = REVIEWER_HTML.find("function copyPraxisNachricht()").expect("copyPraxisNachricht must be defined");
    let snippet = &REVIEWER_HTML[pos..pos + 1000];
    assert!(snippet.contains("navigator.clipboard"),     "copyPraxisNachricht must attempt clipboard API");
    assert!(snippet.contains("praxisNachrichtCopied"),   "copyPraxisNachricht must show copied confirmation");
    assert!(
        REVIEWER_HTML.contains("Nachricht kopiert"),
        "DE praxisNachrichtCopied text must be present in T strings"
    );
    assert!(
        REVIEWER_HTML.contains("Message copied"),
        "EN praxisNachrichtCopied text must be present in T strings"
    );
}

/// buildPraxisNachrichtText must include safety wording and clip note.
#[test]
fn reviewer_praxis_nachricht_includes_safety_and_clip() {
    let pos = REVIEWER_HTML.find("function buildPraxisNachrichtText()").expect("buildPraxisNachrichtText must be defined");
    let snippet = &REVIEWER_HTML[pos..pos + 2000];
    assert!(snippet.contains("copyPracticeRequestSafety"),  "buildPraxisNachrichtText must include safety wording");
    assert!(snippet.contains("copyPracticeRequestPrompt"),  "buildPraxisNachrichtText must include confirmation prompt");
    assert!(snippet.contains("_clipBlobUrl"),               "buildPraxisNachrichtText must check _clipBlobUrl");
    assert!(snippet.contains("clipRecordedNote"),           "buildPraxisNachrichtText must include clip note");
}

/// Pointer overlay and toggle button must be present.
#[test]
fn reviewer_pointer_overlay_present() {
    assert!(REVIEWER_HTML.contains("pointer-overlay"),               "pointer-overlay canvas id must be present");
    assert!(REVIEWER_HTML.contains("viewer-pointer-overlay"),        "viewer-pointer-overlay CSS class must be present");
    assert!(REVIEWER_HTML.contains("pointer-toggle-btn"),            "pointer-toggle-btn id must be present");
    assert!(REVIEWER_HTML.contains("t-pointer-toggle-btn"),          "t-pointer-toggle-btn span id must be present");
    assert!(REVIEWER_HTML.contains("togglePointer()"),               "togglePointer function must be callable");
    assert!(REVIEWER_HTML.contains("function togglePointer()"),      "togglePointer function must be defined");
    assert!(REVIEWER_HTML.contains("onViewerPointerMove"),           "onViewerPointerMove must be present");
    assert!(REVIEWER_HTML.contains("onViewerPointerLeave"),          "onViewerPointerLeave must be present");
    assert!(REVIEWER_HTML.contains("_pointerActive"),                "_pointerActive state variable must be present");
    assert!(REVIEWER_HTML.contains("Zeiger einblenden"),             "DE pointer show label must be present");
    assert!(REVIEWER_HTML.contains("Show pointer"),                  "EN pointer show label must be present");
}

/// updatePraxisNachricht oninput must be wired to key form fields.
#[test]
fn reviewer_praxis_nachricht_oninput_wired() {
    assert!(REVIEWER_HTML.contains("meta-bezeichnung\" type=\"text\" placeholder") &&
            REVIEWER_HTML.contains("oninput=\"updatePraxisNachricht()\""),
            "meta-bezeichnung must have oninput handler");
    assert!(REVIEWER_HTML.contains("lab-comment\" placeholder") &&
            REVIEWER_HTML.contains("oninput=\"updatePraxisNachricht()\""),
            "lab-comment must have oninput handler");
    assert!(REVIEWER_HTML.contains("praxis-rueckmeldung\" placeholder") &&
            REVIEWER_HTML.contains("oninput=\"updatePraxisNachricht()\""),
            "praxis-rueckmeldung must have oninput handler");
}

/// Fallback wording must be present for unsupported browsers.
#[test]
fn reviewer_erklarclip_fallback_present() {
    assert!(REVIEWER_HTML.contains("clip-fallback"),                               "clip-fallback element id must be present");
    assert!(REVIEWER_HTML.contains("function showClipFallback()"),                  "showClipFallback function must be defined");
    assert!(REVIEWER_HTML.contains("im Browser nicht verf"),                        "DE fallback wording must be present");
    assert!(REVIEWER_HTML.contains("Recording not available in this browser"),       "EN fallback wording must be present");
}

/// Erklärclip controls must appear near the viewer (before Step 2 / praxiserklaerung-section).
#[test]
fn reviewer_erklarclip_near_viewer() {
    let clip_pos = REVIEWER_HTML.find("id=\"erklarclip-section\"")
        .expect("erklarclip-section must be present");
    let step2_pos = REVIEWER_HTML.find("id=\"praxiserklaerung-section\"")
        .expect("praxiserklaerung-section must be present");
    assert!(
        clip_pos < step2_pos,
        "erklarclip-section must appear before praxiserklaerung-section (near viewer, not inside Step 2)"
    );
    // Also verify it appears after the STL viewer wrapper
    let viewer_pos = REVIEWER_HTML.find("id=\"stl-viewer-wrap\"")
        .expect("stl-viewer-wrap must be present");
    assert!(
        viewer_pos < clip_pos,
        "erklarclip-section must appear after stl-viewer-wrap"
    );
}

/// Composite canvas must be used for recording so pointer is captured in the clip.
#[test]
fn reviewer_composite_canvas_recording() {
    assert!(REVIEWER_HTML.contains("function _drawCompositeFrame()"),  "_drawCompositeFrame function must be defined");
    assert!(REVIEWER_HTML.contains("_compositeCanvas"),                "_compositeCanvas state var must be present");
    assert!(REVIEWER_HTML.contains("_compositeCanvas.captureStream"),  "recording must call captureStream on composite canvas");
    assert!(REVIEWER_HTML.contains("_pointerNX"),                      "normalized pointer X must be tracked for composite drawing");
    assert!(REVIEWER_HTML.contains("_pointerNY"),                      "normalized pointer Y must be tracked for composite drawing");
    let pos = REVIEWER_HTML.find("function _drawCompositeFrame()").expect("_drawCompositeFrame must be defined");
    let snippet = &REVIEWER_HTML[pos..pos + 1200];
    assert!(snippet.contains("drawImage"),         "composite frame must draw STL canvas content with drawImage");
    assert!(snippet.contains("_pointerActive"),    "composite frame must check _pointerActive before drawing pointer");
    assert!(snippet.contains("_pointerNX"),        "composite frame must use _pointerNX coordinate");
    assert!(snippet.contains("requestAnimationFrame"), "composite frame must use requestAnimationFrame loop");
}

/// Pointer fallback message must be present for browsers where composite capture fails.
#[test]
fn reviewer_clip_pointer_fallback_present() {
    assert!(REVIEWER_HTML.contains("clipPointerFallback"),                                    "clipPointerFallback key must be present");
    assert!(REVIEWER_HTML.contains("Zeiger kann in diesem Browser nicht im Clip"),            "DE clipPointerFallback text must be present");
    assert!(REVIEWER_HTML.contains("Pointer cannot be captured in the clip in this browser"), "EN clipPointerFallback text must be present");
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
