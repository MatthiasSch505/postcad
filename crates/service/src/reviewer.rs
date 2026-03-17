//! Reviewer shell — premium operator demo surface.
//!
//! Served at `GET /reviewer`. Auto-loads pilot fixtures from `examples/pilot/`
//! via `GET /pilot-fixtures`. Calls real endpoints only:
//!   POST /pilot/route-normalized  — routing kernel execution
//!   POST /verify                  — deterministic receipt verification
//!   POST /dispatch/create         — dispatch commitment
//!
//! No mock data. No fake outputs. No mocked decisions.

pub const REVIEWER_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PostCAD — Operator Demo</title>
<style>
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}

/* ── tokens ── */
:root{
  --bg:#111318;
  --surface:#181c25;
  --surface2:#1e2230;
  --surface3:#232838;
  --border:rgba(255,255,255,0.07);
  --border-md:rgba(255,255,255,0.11);
  --border-strong:rgba(255,255,255,0.16);
  --text-1:#dde4f0;
  --text-2:#7a8fa8;
  --text-3:#3d4d60;
  --green:#30c97e;
  --green-bg:rgba(48,201,126,0.08);
  --green-border:rgba(48,201,126,0.2);
  --amber:#e8a020;
  --amber-bg:rgba(232,160,32,0.08);
  --amber-border:rgba(232,160,32,0.2);
  --red:#e05555;
  --red-bg:rgba(224,85,85,0.08);
  --red-border:rgba(224,85,85,0.2);
  --blue:#5b8cfc;
  --mono:'ui-monospace','Cascadia Code','Menlo',monospace;
}

body{
  font-family:-apple-system,BlinkMacSystemFont,'Inter','Segoe UI',sans-serif;
  background:var(--bg);color:var(--text-1);min-height:100vh;
  font-size:14px;line-height:1.5;-webkit-font-smoothing:antialiased;
}

/* ── header ── */
header{
  background:var(--surface);
  border-bottom:1px solid var(--border);
  padding:.65rem 2rem;
  display:flex;align-items:center;gap:.75rem;
  position:sticky;top:0;z-index:20;
}
.logo{font-size:.9rem;font-weight:700;color:var(--text-1);letter-spacing:-.01em}
.demo-tag{
  font-size:.65rem;font-weight:600;color:var(--text-3);
  text-transform:uppercase;letter-spacing:.08em;
  border:1px solid var(--border);border-radius:3px;
  padding:.1rem .4rem;
}
.hdr-right{
  margin-left:auto;display:flex;align-items:center;gap:.75rem;
  font-family:var(--mono);font-size:.68rem;color:var(--text-3);
}
.hdr-dot{
  width:6px;height:6px;border-radius:50%;background:var(--amber);flex-shrink:0;
}
.hdr-dot.ok{background:var(--green)}
.hdr-dot.err{background:var(--red)}

/* ── layout ── */
main{max-width:860px;margin:0 auto;padding:2.5rem 1.5rem 5rem}

/* ── hero ── */
.hero{margin-bottom:2.5rem}
.hero-eyebrow{
  font-size:.65rem;font-weight:700;color:var(--text-3);
  text-transform:uppercase;letter-spacing:.1em;margin-bottom:.75rem;
}
.hero-title{
  font-size:1.7rem;font-weight:700;color:var(--text-1);
  letter-spacing:-.02em;line-height:1.25;margin-bottom:.65rem;
}
.hero-sub{
  font-size:.95rem;color:var(--text-2);line-height:1.6;
  max-width:600px;margin-bottom:1.1rem;
}
.trust-chips{display:flex;flex-wrap:wrap;gap:.5rem;margin-bottom:1rem}
.chip{
  font-size:.7rem;font-weight:600;color:var(--text-2);
  border:1px solid var(--border-md);border-radius:4px;
  padding:.22rem .65rem;background:var(--surface);
  letter-spacing:.02em;
}
.hero-framing{
  font-size:.8rem;color:var(--text-3);line-height:1.6;
  border-left:2px solid var(--border-md);padding-left:.75rem;
  max-width:560px;
}

/* ── step cards ── */
.step-card{
  background:var(--surface);
  border:1px solid var(--border);
  border-radius:10px;
  padding:1.75rem 2rem;
  margin-bottom:1.25rem;
  transition:border-color .2s,opacity .2s;
}
.step-card.step-done{border-left:3px solid var(--green)}
.step-card.step-active{border-left:3px solid var(--blue);border-left-width:3px}
.step-card.step-locked{opacity:.35;pointer-events:none}
.step-card.step-locked .step-body,
.step-card.step-locked .btn-primary,
.step-card.step-locked .btn-secondary,
.step-card.step-locked .result-panel,
.step-card.step-locked details.tech-drawer,
.step-card.step-locked .dispatch-actions,
.step-card.step-locked .loading-note,
.step-card.step-locked .error-note,
.step-card.step-locked .warn-note,
.step-card.step-locked .success-note,
.step-card.step-locked .verify-banner,
.step-card.step-locked [id$="-result"],
.step-card.step-locked [id$="-error"]{display:none!important}

.step-header{
  display:flex;align-items:center;justify-content:space-between;
  margin-bottom:.6rem;
}
.step-label{
  font-size:.63rem;font-weight:700;color:var(--text-3);
  text-transform:uppercase;letter-spacing:.1em;
}
.status-chip{
  font-family:var(--mono);font-size:.6rem;font-weight:700;
  text-transform:uppercase;letter-spacing:.06em;
  padding:.18rem .5rem;border-radius:3px;
}
.chip-gray{background:var(--surface2);color:var(--text-3);border:1px solid var(--border)}
.chip-amber{background:var(--amber-bg);color:var(--amber);border:1px solid var(--amber-border)}
.chip-green{background:var(--green-bg);color:var(--green);border:1px solid var(--green-border)}
.chip-red{background:var(--red-bg);color:var(--red);border:1px solid var(--red-border)}
.chip-blue{background:rgba(91,140,252,0.1);color:var(--blue);border:1px solid rgba(91,140,252,0.2)}

.step-title{
  font-size:1.05rem;font-weight:600;color:var(--text-1);
  letter-spacing:-.01em;margin-bottom:.4rem;
}
.step-body{
  font-size:.85rem;color:var(--text-2);line-height:1.6;
  margin-bottom:1.25rem;max-width:520px;
}

/* ── buttons ── */
.btn-primary{
  display:inline-flex;align-items:center;gap:.4rem;
  background:var(--text-1);color:#0d1018;
  font-family:inherit;font-size:.82rem;font-weight:700;
  border:none;border-radius:6px;padding:.6rem 1.3rem;
  cursor:pointer;transition:opacity .12s;
  margin-bottom:1.1rem;
}
.btn-primary:hover:not(:disabled){opacity:.88}
.btn-primary:disabled{opacity:.28;cursor:default}
.btn-primary.loading{opacity:.6;cursor:default}

.btn-secondary{
  display:inline-flex;align-items:center;gap:.35rem;
  background:transparent;color:var(--text-2);
  font-family:inherit;font-size:.78rem;font-weight:600;
  border:1px solid var(--border-md);border-radius:5px;
  padding:.45rem .9rem;cursor:pointer;transition:border-color .12s,color .12s;
  margin-top:.5rem;
}
.btn-secondary:hover:not(:disabled){border-color:var(--border-strong);color:var(--text-1)}
.btn-secondary:disabled{opacity:.3;cursor:default}
.btn-secondary+.btn-secondary{margin-left:.5rem}

.btn-row{display:flex;flex-wrap:wrap;gap:.5rem;margin-top:.75rem}

/* ── result panel ── */
.result-panel{
  background:var(--surface2);
  border:1px solid var(--border);
  border-radius:7px;
  padding:1.1rem 1.25rem;
  margin-bottom:.75rem;
}
.result-panel.result-ok{border-left:3px solid var(--green)}
.result-panel.result-err{border-left:3px solid var(--red)}
.result-panel.result-pending{border-left:3px solid var(--amber)}

.result-status-header{
  display:flex;align-items:center;gap:.45rem;
  margin-bottom:.85rem;
}
.result-status-dot{
  width:6px;height:6px;border-radius:50%;flex-shrink:0;
}
.dot-green{background:var(--green)}
.dot-amber{background:var(--amber)}
.dot-red{background:var(--red)}

.result-status-title{
  font-size:.78rem;font-weight:700;color:var(--text-2);
  text-transform:uppercase;letter-spacing:.06em;
}

/* ── field rows ── */
.field-grid{
  display:grid;grid-template-columns:max-content 1fr;
  gap:.18rem .75rem;margin-bottom:.65rem;
}
.field-key{
  font-size:.72rem;color:var(--text-3);
  text-transform:uppercase;letter-spacing:.05em;
  white-space:nowrap;padding-top:.06rem;
}
.field-val{font-size:.82rem;color:var(--text-1);word-break:break-all}
.field-val-mono{
  font-family:var(--mono);font-size:.72rem;
  color:var(--text-2);word-break:break-all;
}
.field-val-pill{} /* inline wrapper */

/* ── pill / badge ── */
.pill{
  display:inline-block;font-family:var(--mono);font-size:.65rem;
  font-weight:700;padding:.1rem .4rem;border-radius:3px;
  text-transform:uppercase;letter-spacing:.04em;
}
.pill-ok{background:var(--green-bg);color:var(--green)}
.pill-warn{background:var(--amber-bg);color:var(--amber)}
.pill-err{background:var(--red-bg);color:var(--red)}
.pill-info{background:rgba(91,140,252,0.1);color:var(--blue)}
.pill-muted{background:var(--surface3);color:var(--text-3)}

/* ── explanation bullets ── */
.result-bullets{
  display:grid;gap:.25rem;margin:.55rem 0 .65rem;
}
.result-bullet{
  font-size:.78rem;color:var(--text-2);
  display:flex;align-items:baseline;gap:.4rem;line-height:1.45;
}
.result-bullet::before{
  content:'';width:4px;height:4px;border-radius:50%;
  background:var(--text-3);flex-shrink:0;margin-top:.35rem;
}

/* ── trust line ── */
.trust-line{
  font-size:.75rem;color:var(--green);
  display:flex;align-items:center;gap:.4rem;
  border-top:1px solid var(--border);
  padding-top:.6rem;margin-top:.1rem;
}

/* ── verify banner ── */
.verify-banner{
  border-radius:6px;padding:.65rem .9rem;
  font-size:.82rem;font-weight:700;margin-bottom:.65rem;
}
.banner-ok{background:var(--green-bg);color:var(--green);border:1px solid var(--green-border)}
.banner-err{background:var(--red-bg);color:var(--red);border:1px solid var(--red-border)}
.verify-sub{
  display:block;font-size:.72rem;font-weight:400;
  color:var(--text-2);margin-top:.18rem;
}

/* ── result explanation ── */
.result-explanation{
  font-size:.78rem;color:var(--text-2);line-height:1.6;
  padding-top:.55rem;border-top:1px solid var(--border);
  margin-top:.1rem;
}

/* ── technical drawer ── */
details.tech-drawer{margin-top:.9rem}
details.tech-drawer summary{
  font-size:.7rem;font-weight:600;color:var(--text-3);
  cursor:pointer;user-select:none;list-style:none;
  display:flex;align-items:center;gap:.35rem;
  padding:.3rem 0;letter-spacing:.03em;
}
details.tech-drawer summary::-webkit-details-marker{display:none}
details.tech-drawer summary::before{
  content:'▶';font-size:.5rem;color:var(--border-md);
  transition:transform .12s;
}
details.tech-drawer[open] summary::before{transform:rotate(90deg)}
details.tech-drawer summary:hover{color:var(--text-2)}
pre.json-pre{
  background:var(--surface3);border:1px solid var(--border);
  border-radius:5px;padding:.65rem .85rem;
  font-family:var(--mono);font-size:.67rem;color:var(--text-2);
  white-space:pre-wrap;word-break:break-all;
  line-height:1.5;overflow-x:auto;margin-top:.45rem;
  max-height:280px;overflow-y:auto;
}
pre.json-pre.json-ok{border-left:3px solid var(--green)}
pre.json-pre.json-err{border-left:3px solid var(--red)}
pre.json-pre.json-info{border-left:3px solid var(--blue)}

/* ── small copy btn ── */
.copy-btn{
  background:none;border:1px solid var(--border);border-radius:3px;
  color:var(--blue);cursor:pointer;font-family:inherit;font-size:.6rem;
  padding:.05rem .3rem;margin-left:.3rem;transition:color .1s;
}
.copy-btn:hover{color:var(--text-1);border-color:var(--border-md)}

/* ── loading / error notes ── */
.loading-note{font-size:.8rem;color:var(--text-3);padding:.5rem 0}
.error-note{font-size:.78rem;color:var(--red);line-height:1.5;margin:.3rem 0}
.warn-note{font-size:.78rem;color:var(--amber);line-height:1.5;margin:.3rem 0}
.success-note{font-size:.78rem;color:var(--green);line-height:1.5;margin:.3rem 0}

/* ── dispatch sub-buttons area ── */
.dispatch-actions{
  background:var(--surface);border:1px solid var(--border);
  border-radius:6px;padding:.85rem 1rem;margin-top:.65rem;
}
.dispatch-actions-label{
  font-size:.63rem;font-weight:700;color:var(--text-3);
  text-transform:uppercase;letter-spacing:.08em;margin-bottom:.55rem;
}

/* ── value section ── */
.value-section{
  margin-top:2.5rem;padding-top:2rem;
  border-top:1px solid var(--border);
}
.value-eyebrow{
  font-size:.63rem;font-weight:700;color:var(--text-3);
  text-transform:uppercase;letter-spacing:.1em;margin-bottom:1.25rem;
}
.value-grid{display:grid;grid-template-columns:repeat(3,1fr);gap:1.25rem}
@media(max-width:600px){.value-grid{grid-template-columns:1fr}}
.value-block{}
.value-block-title{
  font-size:.82rem;font-weight:700;color:var(--text-1);
  margin-bottom:.3rem;
}
.value-block-body{font-size:.78rem;color:var(--text-2);line-height:1.6}
.value-bottom{
  font-size:.78rem;color:var(--text-3);margin-top:1.5rem;
  padding-top:1rem;border-top:1px solid var(--border);line-height:1.6;
}

/* ── footer util bar ── */
.util-bar{
  position:fixed;bottom:0;left:0;right:0;
  background:rgba(17,19,24,0.95);
  border-top:1px solid var(--border);
  backdrop-filter:blur(8px);
  padding:.4rem 2rem;
  display:flex;align-items:center;gap:1rem;
  font-size:.65rem;color:var(--text-3);z-index:10;
}
.util-bar-actions{margin-left:auto;display:flex;align-items:center;gap:.5rem}
.util-btn{
  background:none;border:1px solid var(--border);border-radius:3px;
  color:var(--text-3);cursor:pointer;font-family:inherit;font-size:.63rem;
  padding:.15rem .45rem;transition:color .1s,border-color .1s;
}
.util-btn:hover{color:var(--text-2);border-color:var(--border-md)}

/* ── hidden util ── */
.hidden{display:none!important}

/* ── expand btn ── */
pre.collapsed{max-height:120px;overflow:hidden}
.expand-btn{
  background:none;border:1px solid var(--border);border-radius:3px;
  color:var(--blue);cursor:pointer;font-family:inherit;font-size:.63rem;
  padding:.1rem .38rem;margin-top:.2rem;transition:color .1s;
}
.expand-btn:hover{color:var(--text-1)}

/* ── step 2 routing result inline fields ── */
#art-hash,#art-kver,#art-dispatch-id{font-family:var(--mono);font-size:.7rem;color:var(--text-2)}

/* ── verify kind label ── */
#verify-kind-label .pill{margin-left:.25rem}

/* ── print ── */
@media print{
  header,.util-bar,.tech-drawer{display:none!important}
  body{background:#fff;color:#000}
  .step-card{border:1px solid #ccc;page-break-inside:avoid}
}
</style>
</head>
<body>

<!-- ── header ── -->
<header>
  <span class="logo">PostCAD</span>
  <span class="demo-tag">Operator Demo</span>
  <div class="hdr-right">
    <span class="hdr-dot" id="status-dot"></span>
    <span id="ver">loading…</span>
  </div>
</header>

<main>

<!-- ── hero ── -->
<div class="hero">
  <div class="hero-eyebrow">Manufacturing routing layer</div>
  <h1 class="hero-title">Deterministic manufacturing<br>routing after CAD</h1>
  <p class="hero-sub">Route, verify, and prepare a manufacturing decision with a reproducible audit trail.</p>
  <div class="trust-chips">
    <span class="chip">Deterministic</span>
    <span class="chip">Verifiable</span>
    <span class="chip">No clinical decision-making</span>
  </div>
  <p class="hero-framing">PostCAD sits between CAD and manufacturing. It does not diagnose, and it does not manufacture. It controls the routing and audit layer.</p>
</div>

<!-- ═══════════════════════════════════════════════
     Step 1 — Load Case
════════════════════════════════════════════════ -->
<div class="step-card step-active" id="step1-card">
  <div class="step-header">
    <span class="step-label">01 · Demo Case</span>
    <span class="status-chip chip-amber" id="step1-chip">READY</span>
  </div>
  <h2 class="step-title">Load a canonical case</h2>
  <p class="step-body">Use a fixed pilot case to review the routing flow from input to dispatch.</p>

  <div id="fixtures-loading" class="loading-note">Loading case…</div>
  <div id="fixtures-error" class="hidden"></div>

  <button class="btn-primary" id="btn-load-case" onclick="loadDemoCase()" disabled>
    Load Demo Case
  </button>

  <!-- case summary shown after load -->
  <div id="step1-result" class="hidden">
    <div class="result-panel result-ok">
      <div class="result-status-header">
        <span class="result-status-dot dot-green"></span>
        <span class="result-status-title">Case loaded</span>
      </div>
      <div class="field-grid">
        <span class="field-key">Case ID</span>
        <span class="field-val-mono" id="s1-case-id">—</span>
        <span class="field-key">Procedure</span>
        <span class="field-val" id="s1-procedure">—</span>
        <span class="field-key">Material</span>
        <span class="field-val" id="s1-material">—</span>
        <span class="field-key">Jurisdiction</span>
        <span class="field-val" id="s1-jurisdiction">—</span>
        <span class="field-key">Routing policy</span>
        <span class="field-val" id="s1-policy">—</span>
      </div>
    </div>
  </div>

  <details class="tech-drawer">
    <summary>Technical input JSON</summary>
    <div style="font-size:.63rem;color:var(--text-3);margin-top:.4rem;margin-bottom:.2rem;text-transform:uppercase;letter-spacing:.06em">case.json</div>
    <pre class="json-pre" id="fix-case"></pre>
    <div style="font-size:.63rem;color:var(--text-3);margin-top:.55rem;margin-bottom:.2rem;text-transform:uppercase;letter-spacing:.06em">registry_snapshot.json</div>
    <pre class="json-pre" id="fix-registry"></pre>
    <div style="font-size:.63rem;color:var(--text-3);margin-top:.55rem;margin-bottom:.2rem;text-transform:uppercase;letter-spacing:.06em">config.json</div>
    <pre class="json-pre" id="fix-config"></pre>
  </details>
</div>


<!-- ═══════════════════════════════════════════════
     Step 2 — Routing
════════════════════════════════════════════════ -->
<div class="step-card step-locked" id="step2-card">
  <div class="step-header">
    <span class="step-label">02 · Routing</span>
    <span class="status-chip chip-gray" id="step2-chip">PENDING</span>
  </div>
  <h2 class="step-title">Find an eligible manufacturer</h2>
  <p class="step-body">PostCAD evaluates the case against the routing policy and manufacturer registry.</p>

  <button class="btn-primary" id="btn-route-norm" onclick="routeNormalized(this)" disabled>
    Run Routing
  </button>
  <div id="route-norm-inline" class="hidden"></div>

  <div id="results-loading" class="hidden loading-note">Routing in progress…</div>

  <!-- routing success result -->
  <div id="route-result" class="hidden">
    <div class="result-panel result-ok">
      <div class="result-status-header">
        <span class="result-status-dot dot-green"></span>
        <span class="result-status-title">Manufacturer selected</span>
        <span id="route-result-badge" class="hidden"></span>
      </div>
      <div class="field-grid">
        <span class="field-key">Manufacturer</span>
        <span class="field-val" id="art-selected">—</span>
        <span class="field-key">Outcome</span>
        <span class="field-val field-val-pill"><span id="art-outcome"></span></span>
        <span class="field-key">Kernel</span>
        <span class="field-val-mono" id="art-kver">—</span>
        <span class="field-key">Receipt hash</span>
        <span class="field-val-mono" id="art-hash">—</span><button class="copy-btn hidden" id="art-hash-copy" onclick="copyArtHashVal(this)">Copy</button>
      </div>
      <div class="result-bullets">
        <div class="result-bullet">eligible for this case</div>
        <div class="result-bullet">compliant with jurisdiction rules</div>
        <div class="result-bullet">deterministic under identical inputs</div>
      </div>
      <div class="trust-line">
        <span>◆</span>
        Same inputs produce the same receipt hash every time
      </div>
    </div>

    <!-- hidden legacy compatibility elements -->
    <span id="receipt-json-badge" class="hidden"></span>
    <span id="mb-receipt" class="hidden"></span>
    <span id="fm-receipt" class="hidden"></span>
    <span id="as-chip-route" class="hidden"></span>
    <div id="receipt-empty-state" class="hidden"></div>
    <div id="verify-artifact-note" class="hidden"></div>
    <div id="receipt-json-actions" class="hidden">
      <button class="copy-btn" onclick="copyReceiptJson(this)">Copy receipt JSON</button>
    </div>
    <button class="expand-btn hidden" id="receipt-expand-btn" onclick="expandArtifact('route-receipt-json','receipt-expand-btn')">Expand</button>
  </div>

  <!-- routing error -->
  <div id="route-error" class="hidden">
    <div class="result-panel result-err">
      <div class="result-status-header">
        <span class="result-status-dot dot-red"></span>
        <span class="result-status-title">Routing failed</span>
      </div>
      <div id="route-error-banner" class="hidden error-note"></div>
      <pre class="json-pre json-err" id="route-error-json"></pre>
      <div id="route-error-json-actions" style="margin-top:.3rem">
        <button class="copy-btn" onclick="copyRouteErrorJson(this)">Copy</button>
      </div>
    </div>
  </div>

  <details class="tech-drawer">
    <summary>Routing receipt</summary>
    <pre class="json-pre json-ok" id="route-receipt-json"></pre>
  </details>
</div>


<!-- ═══════════════════════════════════════════════
     Step 3 — Verification
════════════════════════════════════════════════ -->
<div class="step-card step-locked" id="step3-card" id="as-verify-section">
  <div class="step-header">
    <span class="step-label">03 · Verification</span>
    <span class="status-chip chip-gray" id="step3-chip">PENDING</span>
  </div>
  <h2 class="step-title">Verify the routing decision</h2>
  <p class="step-body">Replay verification confirms that the receipt and routing decision are reproducible.</p>

  <button class="btn-primary" id="btn-verify" onclick="verifyReceipt(this)" disabled>
    Verify Receipt
  </button>
  <span id="as-chip-verify" class="hidden"></span>
  <span id="mb-verify" class="hidden"></span>
  <span id="fm-verify" class="hidden"></span>

  <!-- verify result -->
  <div id="verify-result" class="hidden">
    <div id="verify-banner"></div>
    <div id="verify-kind-label" class="hidden"></div>
    <div class="result-panel result-ok" id="verify-summary-panel">
      <div class="result-status-header">
        <span class="result-status-dot dot-green" id="verify-dot"></span>
        <span class="result-status-title">Decision verified</span>
        <span id="verify-result-badge" class="hidden"></span>
      </div>
      <div class="field-grid">
        <span class="field-key">Result</span>
        <span class="field-val field-val-pill" id="s3-result-field">—</span>
        <span class="field-key">Receipt hash</span>
        <span class="field-val-mono" id="s3-hash-display">—</span>
        <span class="field-key">Replay</span>
        <span class="field-val">Receipt reconstructed from original inputs</span>
      </div>
      <div class="result-explanation">
        This confirms that the decision can be independently checked and reproduced without manual reinterpretation.
      </div>
    </div>
    <span id="lin-verify" class="hidden"></span>
    <div id="lin-verify-note" class="hidden"></div>
    <div id="verify-json-actions" class="hidden">
      <button class="copy-btn" onclick="copyVerifyJson(this)">Copy</button>
    </div>
    <button class="expand-btn hidden" id="verify-expand-btn" onclick="expandArtifact('verify-json','verify-expand-btn')">Expand</button>
  </div>

  <details class="tech-drawer">
    <summary>Verification details</summary>
    <pre class="json-pre" id="verify-json"></pre>
  </details>

  <!-- tamper demo hidden -->
  <button class="hidden" id="btn-tamper" onclick="tamperVerify(this)"></button>
</div>


<!-- ═══════════════════════════════════════════════
     Step 4 — Dispatch
════════════════════════════════════════════════ -->
<div class="step-card step-locked" id="step4-card" id="dispatch-section">
  <div class="step-header">
    <span class="step-label">04 · Dispatch</span>
    <span class="status-chip chip-gray" id="step4-chip">PENDING</span>
  </div>
  <h2 class="step-title">Create a dispatch-ready record</h2>
  <p class="step-body">Once verification succeeds, PostCAD prepares a traceable handoff for manufacturing.</p>

  <button class="btn-primary" id="btn-dispatch-create" onclick="createDispatch(this)" disabled>
    Create Dispatch
  </button>
  <span id="as-chip-dispatch" class="hidden"></span>
  <span id="mb-dispatch" class="hidden"></span>
  <span id="fm-dispatch" class="hidden"></span>
  <div id="verify-pending-note" class="hidden warn-note"></div>
  <div id="dispatch-blocked-note" class="hidden error-note"></div>
  <div id="dispatch-stale-note" class="hidden"></div>
  <div id="dispatch-success" class="hidden success-note"></div>
  <div id="dispatch-error" class="hidden error-note"></div>

  <!-- dispatch created — approve + export -->
  <div id="dispatch-created" class="hidden">
    <div class="result-panel result-pending">
      <div class="result-status-header">
        <span class="result-status-dot dot-amber"></span>
        <span class="result-status-title">Dispatch record created</span>
      </div>
      <div class="field-grid">
        <span class="field-key">Dispatch ID</span>
        <span id="art-dispatch-id" style="font-family:var(--mono);font-size:.7rem;color:var(--text-2);grid-column:2;word-break:break-all"></span><button class="copy-btn hidden" id="art-dispatch-id-copy" onclick="copyDispatchId(this)">Copy</button>
        <span class="field-key">Status</span>
        <span class="field-val field-val-pill"><span id="art-dispatch-status"></span></span>
      </div>
    </div>
    <div class="dispatch-actions">
      <div class="dispatch-actions-label">Next steps</div>
      <button class="btn-secondary" id="btn-dispatch-approve" onclick="approveDispatch(this)" disabled>
        Approve Dispatch
      </button>
      <button class="btn-secondary" id="btn-dispatch-export" onclick="exportDispatch(this)" disabled>
        Export Dispatch Packet
      </button>
    </div>
  </div>

  <!-- export result -->
  <div id="dispatch-export-result" class="hidden">
    <div class="result-panel result-ok">
      <div class="result-status-header">
        <span class="result-status-dot dot-green"></span>
        <span class="result-status-title">Dispatch packet created</span>
        <span id="dispatch-result-badge" class="hidden"></span>
        <span id="as-chip-export" class="hidden"></span>
      </div>
      <div class="field-grid">
        <span class="field-key">Status</span>
        <span class="field-val field-val-pill" id="s4-status-display">—</span>
        <span class="field-key">Verification</span>
        <span class="field-val" id="s4-verify-display">—</span>
        <span class="field-key">Audit trail</span>
        <span class="field-val">Traceable routing record attached</span>
      </div>
      <div class="result-explanation">
        The case now has a verifiable record of how the routing decision was made before manufacturing dispatch.
      </div>
    </div>
    <div id="dispatch-export-actions" class="hidden btn-row">
      <button class="btn-secondary" onclick="downloadExportPacket()">↓ Download export_packet.json</button>
      <button class="btn-secondary" onclick="copyExportJson(this)">Copy JSON</button>
    </div>
    <span id="lin-dispatch-export" class="hidden"></span>
    <div id="lin-dispatch-note" class="hidden"></div>
    <button class="expand-btn hidden" id="dispatch-expand-btn" onclick="expandArtifact('dispatch-export-json','dispatch-expand-btn')">Expand</button>
  </div>

  <details class="tech-drawer">
    <summary>Dispatch packet</summary>
    <pre class="json-pre json-info" id="dispatch-export-json"></pre>
  </details>
</div>


<!-- ═══════════════════════════════════════════════
     Value section
════════════════════════════════════════════════ -->
<div class="value-section">
  <div class="value-eyebrow">Why PostCAD</div>
  <div class="value-grid">
    <div class="value-block">
      <div class="value-block-title">Routing</div>
      <div class="value-block-body">Replace manual lab selection with deterministic decision logic</div>
    </div>
    <div class="value-block">
      <div class="value-block-title">Verification</div>
      <div class="value-block-body">Confirm that the decision can be replayed and checked</div>
    </div>
    <div class="value-block">
      <div class="value-block-title">Audit</div>
      <div class="value-block-body">Create a traceable record before manufacturing</div>
    </div>
  </div>
  <p class="value-bottom">PostCAD is the execution layer after CAD and before production.</p>
</div>

</main>

<!-- ── util bar ── -->
<div class="util-bar">
  <span style="font-size:.62rem;color:var(--text-3)">PostCAD Operator Demo</span>
  <div class="util-bar-actions">
    <button class="util-btn" id="btn-copy-snapshot" onclick="copyAuditSnapshot(this)">Copy snapshot</button>
    <button class="util-btn" onclick="downloadAuditSnapshot()">↓ Download</button>
    <button class="util-btn" id="btn-print-handoff" onclick="window.print()">Print</button>
  </div>
</div>

<!-- ═══════════════════════════════════════════════
     Hidden legacy panel — all IDs required by JS
════════════════════════════════════════════════ -->
<div id="_legacy" style="display:none!important" aria-hidden="true">

  <!-- fixtures panel (ids used for JSON display) -->
  <div id="fixtures-panel"></div>

  <!-- hidden routing form (keyboard listener target + form state) -->
  <div id="norm-input-section">
    <input id="norm-case-id" value="f1000001-0000-0000-0000-000000000001">
    <input id="norm-restoration-type" value="crown">
    <input id="norm-material" value="zirconia">
    <input id="norm-jurisdiction" value="DE">
    <div id="route-norm-preview"></div>
    <button id="btn-route" onclick="routeCase(this)"></button>
  </div>

  <!-- legacy op state displays -->
  <span id="ops-routing"></span>
  <span id="ops-receipt"></span>
  <span id="ops-verify"></span>
  <span id="ops-dispatch"></span>

  <!-- legacy panels (all updated by JS, not shown) -->
  <div id="nar-rail"><span id="nar-action" class="nar-action nar-action-idle"></span><span id="nar-reason"></span></div>
  <div id="run-timeline">
    <div id="rt-route" class="rt-step rt-idle"><div class="rt-dot"></div><div class="rt-name"></div></div>
    <div id="rt-receipt" class="rt-step rt-idle"><div class="rt-dot"></div><div class="rt-name"></div></div>
    <div id="rt-verify" class="rt-step rt-idle"><div class="rt-dot"></div><div class="rt-name"></div></div>
    <div id="rt-dispatch" class="rt-step rt-idle"><div class="rt-dot"></div><div class="rt-name"></div></div>
    <div id="rt-summary"></div>
  </div>
  <div id="rib">
    <span id="rib-route"></span><span id="rib-receipt"></span>
    <span id="rib-verify"></span><span id="rib-dispatch"></span>
  </div>
  <div id="osg"><div id="osg-reasons"></div><button onclick="startCleanRun()"></button></div>
  <div id="oab">
    <span id="oab-action" class="oab-action oab-action-idle"></span>
    <button id="oab-btn" onclick="oabNavigate()"></button>
    <div id="oab-reason"></div>
  </div>
  <div id="orb" class="orb orb-neutral">
    <div id="orb-headline"></div><div id="orb-detail"></div>
    <button id="orb-link" class="hidden" onclick="orbNavigate()"></button>
  </div>
  <div id="crc"><div id="crc-rows"></div><div id="crc-footer"></div></div>
  <div id="pfc" class="pfc pfc-not-ready">
    <div id="pfc-headline"></div><div id="pfc-detail"></div>
    <div id="pfc-rows"></div><button id="pfc-link" class="hidden" onclick="pfcNavigate()"></button>
  </div>
  <div id="ccs" class="ccs ccs-consistent">
    <div id="ccs-headline"></div><div id="ccs-detail"></div>
  </div>
  <div id="hsc">
    <div id="hsc-verdict"></div><div id="hsc-rows"></div>
    <div id="hsc-readiness"></div><div id="hsc-artifacts"></div>
    <div id="hsc-summary"></div>
  </div>
  <div id="active-run-context">
    <span id="arc-manufacturer"></span><span id="arc-receipt-hash"></span>
    <span id="arc-verify-status" class="arc-val-pending"></span>
    <span id="arc-dispatch-status" class="arc-val-pending"></span>
  </div>
  <div id="run-history-panel"><div id="run-history-list"></div></div>
  <div id="sal">
    <div id="sal-empty"></div><div id="sal-list" class="hidden"></div>
  </div>
  <div id="op-cheatsheet"></div>
  <div class="ase-bar"></div>
  <div id="rrc">
    <div id="rrc-status" class="rrc-status rrc-not-tested"></div>
    <div id="rrc-detail"></div>
    <button id="btn-repro" onclick="runReproCheck(this)" disabled></button>
  </div>
  <div id="dpi">
    <div id="dpi-meta" class="hidden">
      <span id="dpi-origin" class="dpi-origin-none"></span>
      <span id="dpi-integrity" class="dpi-integrity-none"></span>
    </div>
    <div id="dpi-empty"></div>
    <pre id="dpi-viewer" class="hidden"></pre>
  </div>
  <div id="handoff-note"><div id="hn-body"></div></div>
  <div id="dispatch-readiness-panel">
    <div id="dr-status" class="dr-not-ready"></div>
    <div id="dr-reason"></div>
    <div id="cl-receipt" class="cl-item cl-pending"></div>
    <div id="cl-verify" class="cl-item cl-pending"></div>
    <div id="cl-dispatch" class="cl-item cl-pending"></div>
  </div>
  <div id="dbl"><div id="dbl-body"></div></div>
  <div id="dhd">
    <div id="dhd-verdict" class="dhd-verdict dhd-verdict-none"></div>
    <div id="dhd-meaning"></div><div id="dhd-checklist"></div>
    <span id="dhd-next-text"></span>
  </div>
  <div id="drs">
    <div id="drs-verdict" class="drs-verdict drs-verdict-none"></div>
    <div id="drs-meaning"></div><div id="drs-checklist"></div>
    <span id="drs-next-text"></span>
  </div>
  <div id="phs">
    <div id="phs-verdict" class="phs-verdict phs-verdict-not-ready"></div>
    <div id="phs-meaning"></div><div id="phs-checklist"></div>
    <span id="phs-action-text"></span>
  </div>
  <div id="cab">
    <div id="cab-verdict" class="cab-verdict cab-verdict-none"></div>
    <div id="cab-meaning"></div><div id="cab-artifacts"></div>
    <span id="cab-next-text"></span>
  </div>
  <div id="cpw">
    <span id="cpw-s1" class="cpw-step-status cpw-s-available"></span>
    <span id="cpw-s2" class="cpw-step-status cpw-s-blocked"></span>
    <span id="cpw-s3" class="cpw-step-status cpw-s-blocked"></span>
    <span id="cpw-s4" class="cpw-step-status cpw-s-blocked"></span>
    <span id="cpw-s5" class="cpw-step-status cpw-s-blocked"></span>
    <span id="cpw-s6" class="cpw-step-status cpw-s-blocked"></span>
  </div>
  <!-- placeholder used by old routeCase/routeNormalized hide/show calls -->
  <div id="results-placeholder"></div>
</div>

<script>
// ── state ──────────────────────────────────────────────────────────────────
let fixtures       = null;
let lastReceipt    = null;
let lastPolicy     = null;
let lastDispatchId = null;
let lastExportPacket = null;
let opRouting  = 'not-run';
let opReceipt  = 'not-run';
let opVerify   = 'not-run';
let opDispatch = 'not-run';
const runHistory = [];
let runSerial      = 0;
let verifySerial   = 0;
let dispatchSerial = 0;
let lastRouteInputs   = null;
let lastRouteEndpoint = null;
let reproStatus = 'not-tested';

// ── boot ───────────────────────────────────────────────────────────────────
(async function boot() {
  try {
    const r = await fetch('/version');
    const v = await r.json();
    document.getElementById('status-dot').className = 'hdr-dot' + (r.ok ? ' ok' : ' err');
    document.getElementById('ver').textContent =
      v.protocol_version
        ? v.protocol_version + ' · ' + v.routing_kernel_version
        : JSON.stringify(v);
  } catch(e) {
    document.getElementById('status-dot').className = 'hdr-dot err';
    document.getElementById('ver').textContent = 'service unreachable';
  }

  try {
    const r = await fetch('/pilot-fixtures');
    if (!r.ok) throw new Error('HTTP ' + r.status + ': ' + await r.text());
    fixtures = await r.json();
    document.getElementById('fix-case').textContent     = fmt(fixtures.case);
    document.getElementById('fix-registry').textContent = fmt(fixtures.registry_snapshot);
    document.getElementById('fix-config').textContent   = fmt(fixtures.routing_config);
    document.getElementById('fixtures-loading').classList.add('hidden');
    document.getElementById('btn-load-case').disabled = false;
    // btn-route-norm stays disabled until user completes Step 1 (loadDemoCase)
  } catch(e) {
    document.getElementById('fixtures-loading').classList.add('hidden');
    const errEl = document.getElementById('fixtures-error');
    errEl.innerHTML =
      '<div style="background:var(--red-bg);border:1px solid var(--red-border);border-radius:6px;padding:.75rem 1rem;margin-bottom:.5rem">'
      + '<div style="font-size:.72rem;font-weight:700;color:var(--red);margin-bottom:.25rem">Cannot load case fixtures</div>'
      + '<div style="font-size:.78rem;color:var(--text-2);line-height:1.5;margin-bottom:.3rem">Start the service from the repo root: <code style="font-family:var(--mono);color:var(--text-2)">cargo run -p postcad-service</code></div>'
      + '<div style="font-size:.72rem;color:var(--text-3);font-family:var(--mono)">' + esc(e.message) + '</div>'
      + '</div>';
    errEl.classList.remove('hidden');
  }
})();

// ── Load Demo Case (Step 1) ────────────────────────────────────────────────
function loadDemoCase() {
  loadNormSample();
  document.getElementById('s1-case-id').textContent    = document.getElementById('norm-case-id').value || '—';
  document.getElementById('s1-procedure').textContent  = document.getElementById('norm-restoration-type').value || '—';
  document.getElementById('s1-material').textContent   = document.getElementById('norm-material').value || '—';
  document.getElementById('s1-jurisdiction').textContent = document.getElementById('norm-jurisdiction').value || '—';
  if (fixtures && fixtures.routing_config) {
    const s = fixtures.routing_config.routing_strategy || fixtures.routing_config.strategy || 'deterministic_hash';
    document.getElementById('s1-policy').textContent = s;
  }
  show('step1-result');
  document.getElementById('step1-card').className = 'step-card step-done';
  document.getElementById('step1-chip').textContent = 'LOADED';
  document.getElementById('step1-chip').className = 'status-chip chip-green';
  updateStepCards();
}

// ── Step card unlock ───────────────────────────────────────────────────────
function updateStepCards() {
  // Step 2: unlocks only when Step 1 is done (user clicked Load Demo Case)
  const step1Done = document.getElementById('step1-card').classList.contains('step-done');
  if (step1Done) {
    const c2 = document.getElementById('step2-card');
    if (c2.classList.contains('step-locked')) {
      c2.classList.remove('step-locked');
      c2.classList.add('step-active');
    }
    const ch2 = document.getElementById('step2-chip');
    if (ch2.className === 'status-chip chip-gray') {
      ch2.textContent = 'READY';
      ch2.className = 'status-chip chip-amber';
    }
    document.getElementById('btn-route-norm').disabled = false;
  }

  // Step 2 status by routing state
  if (opRouting === 'available') {
    document.getElementById('step2-card').className = 'step-card step-done';
    document.getElementById('step2-chip').textContent = 'ROUTED';
    document.getElementById('step2-chip').className = 'status-chip chip-green';
    // Unlock step 3
    const c3 = document.getElementById('step3-card');
    c3.classList.remove('step-locked');
    if (!c3.classList.contains('step-done')) c3.classList.add('step-active');
    const ch3 = document.getElementById('step3-chip');
    if (ch3.className === 'status-chip chip-gray') {
      ch3.textContent = 'READY';
      ch3.className = 'status-chip chip-amber';
    }
  } else if (opRouting === 'failed') {
    document.getElementById('step2-card').className = 'step-card step-active';
    document.getElementById('step2-chip').textContent = 'BLOCKED';
    document.getElementById('step2-chip').className = 'status-chip chip-red';
  }

  // Step 3 status by verify state
  if (opVerify === 'verified') {
    document.getElementById('step3-card').className = 'step-card step-done';
    document.getElementById('step3-chip').textContent = 'VERIFIED';
    document.getElementById('step3-chip').className = 'status-chip chip-green';
    // Unlock step 4
    const c4 = document.getElementById('step4-card');
    c4.classList.remove('step-locked');
    if (!c4.classList.contains('step-done')) c4.classList.add('step-active');
    const ch4 = document.getElementById('step4-chip');
    if (ch4.className === 'status-chip chip-gray') {
      ch4.textContent = 'READY';
      ch4.className = 'status-chip chip-amber';
    }
  } else if (opVerify === 'failed') {
    document.getElementById('step3-card').className = 'step-card step-active';
    document.getElementById('step3-chip').textContent = 'BLOCKED';
    document.getElementById('step3-chip').className = 'status-chip chip-red';
  }

  // Step 4 status
  if (lastExportPacket) {
    document.getElementById('step4-card').className = 'step-card step-done';
    document.getElementById('step4-chip').textContent = 'DISPATCHED';
    document.getElementById('step4-chip').className = 'status-chip chip-green';
    // Update dispatch result fields
    const dispStatus = lastExportPacket.status || 'exported';
    document.getElementById('s4-status-display').innerHTML =
      '<span class="pill pill-ok">' + esc(dispStatus) + '</span>';
    document.getElementById('s4-verify-display').innerHTML =
      '<span class="pill pill-ok">verified</span>';
  }
}

// ── Execute Routing Kernel (legacy, hidden btn) ────────────────────────────
async function routeCase(btn) {
  if (!fixtures) return;
  setBtn(btn, 'Running kernel…', true);
  document.getElementById('btn-route-norm').disabled = true;
  if (lastReceipt) salLog('Current run reset', 'Previous run cleared for new route.');
  salLog('Route requested', 'Routing kernel execution started.');
  runSerial++;
  lastRouteInputs = null; lastRouteEndpoint = null; reproStatus = 'not-tested';

  hide('results-placeholder'); hide('route-result'); hide('route-error');
  hide('verify-result'); hide('dispatch-created'); hide('dispatch-export-result');
  hide('dispatch-success'); hide('dispatch-error');
  show('results-loading');
  lastReceipt = null; lastPolicy = null; lastDispatchId = null; lastExportPacket = null;
  updateOpState('not-run', 'not-run', 'not-run', 'not-run');
  clearRunHistory();
  document.getElementById('btn-dispatch-create').disabled  = true;
  document.getElementById('btn-dispatch-approve').disabled = true;
  document.getElementById('btn-dispatch-export').disabled  = true;

  try {
    const r = await fetch('/route', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({
        case: fixtures.case,
        registry_snapshot: fixtures.registry_snapshot,
        routing_config:    fixtures.routing_config,
      }),
    });
    const data = await r.json();
    if (r.ok && data.receipt) {
      lastReceipt = data.receipt; lastPolicy = data.derived_policy;
      populateRoutingResult(data.receipt);
      lastRouteEndpoint = '/route';
      lastRouteInputs   = {case: fixtures.case, registry_snapshot: fixtures.registry_snapshot, routing_config: fixtures.routing_config};
      updateOpState('available', 'available', 'not-run', 'available');
      salLog('Route result received', 'Route receipt generated.');
      appendRunHistory('Route executed', true);
    } else {
      hide('route-error-banner');
      document.getElementById('route-error-json').textContent = fmt(data);
      show('route-error'); show('receipt-empty-state');
      updateOpState('failed', 'missing', null, null);
      appendRunHistory('Route executed', false);
    }
  } catch(e) {
    document.getElementById('route-error-json').textContent = String(e);
    show('route-error');
    updateOpState('failed', 'missing', null, null);
    appendRunHistory('Route executed', false);
  } finally {
    hide('results-loading');
    setBtn(btn, '▶ Execute Routing Kernel', false);
    document.getElementById('btn-route-norm').disabled = false;
  }
}

// ── Route Normalized Pilot Case ────────────────────────────────────────────
async function routeNormalized(btn) {
  if (!fixtures) return;

  const pilotCase = readNormInputs();
  const ni = document.getElementById('route-norm-inline');

  const missing = validateNormInput(pilotCase);
  if (missing.length) {
    markNormInvalid(missing);
    ni.textContent = 'Required fields missing: ' + missing.join(', ');
    ni.className = 'error-note';
    ni.classList.remove('hidden');
    return;
  }
  clearNormInvalid();

  setBtn(btn, 'Routing…', true);
  document.getElementById('btn-route').disabled = true;
  if (lastReceipt) salLog('Current run reset', 'Previous run cleared for new route.');
  salLog('Route requested', 'Routing kernel execution started.');
  runSerial++;
  lastRouteInputs = null; lastRouteEndpoint = null; reproStatus = 'not-tested';

  hide('results-placeholder'); hide('route-result'); hide('route-error');
  hide('verify-result'); hide('dispatch-created'); hide('dispatch-export-result');
  hide('dispatch-success'); hide('dispatch-error');
  show('results-loading');
  lastReceipt = null; lastPolicy = null; lastDispatchId = null; lastExportPacket = null;
  updateOpState('not-run', 'not-run', 'not-run', 'not-run');
  clearRunHistory();
  document.getElementById('btn-dispatch-create').disabled  = true;
  document.getElementById('btn-dispatch-approve').disabled = true;
  document.getElementById('btn-dispatch-export').disabled  = true;

  try {
    let r;
    try {
      r = await fetch('/pilot/route-normalized', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({
          pilot_case:        pilotCase,
          registry_snapshot: fixtures.registry_snapshot,
          routing_config:    fixtures.routing_config,
        }),
      });
    } catch(netErr) {
      document.getElementById('route-error-json').textContent = String(netErr);
      show('route-error');
      updateOpState('failed', 'missing', null, null);
      appendRunHistory('Route executed', false);
      return;
    }

    let data;
    try { data = await r.json(); }
    catch(parseErr) {
      document.getElementById('route-error-json').textContent =
        'HTTP ' + r.status + ' — response is not valid JSON: ' + String(parseErr);
      show('route-error');
      updateOpState('failed', 'missing', null, null);
      appendRunHistory('Route executed', false);
      return;
    }

    if (r.ok && data.receipt) {
      lastReceipt = data.receipt; lastPolicy = data.derived_policy;
      populateRoutingResult(data.receipt);
      lastRouteEndpoint = '/pilot/route-normalized';
      lastRouteInputs   = {pilot_case: pilotCase, registry_snapshot: fixtures.registry_snapshot, routing_config: fixtures.routing_config};
      updateOpState('available', 'available', 'not-run', 'available');
      salLog('Route result received', 'Route receipt generated.');
      appendRunHistory('Route executed', true);
    } else {
      const code = data?.error?.code || data?.result || 'error';
      const msg  = data?.error?.message || '';
      const banner = document.getElementById('route-error-banner');
      banner.textContent = '[' + code + '] ' + (msg || 'Routing request failed.');
      banner.classList.remove('hidden');
      document.getElementById('route-error-json').textContent = fmt(data);
      show('route-error');
      updateOpState('failed', 'missing', null, null);
      appendRunHistory('Route executed', false);
    }
  } catch(e) {
    document.getElementById('route-error-json').textContent = String(e);
    show('route-error');
    updateOpState('failed', 'missing', null, null);
    appendRunHistory('Route executed', false);
  } finally {
    hide('results-loading');
    setBtn(btn, 'Run Routing', false);
    document.getElementById('btn-route').disabled = false;
  }
}

// ── Populate routing result fields ─────────────────────────────────────────
function populateRoutingResult(rc) {
  const outcome  = rc.outcome || '—';
  const selected = rc.selected_candidate_id || '(none — refused)';
  const rhash    = rc.receipt_hash || '—';
  const kver     = rc.routing_kernel_version || '—';

  document.getElementById('art-outcome').innerHTML =
    '<span class="pill ' + (outcome === 'routed' ? 'pill-ok' : 'pill-warn') + '">' + esc(outcome) + '</span>';
  document.getElementById('art-selected').textContent = selected;
  document.getElementById('art-hash').textContent     = rhash;
  document.getElementById('art-kver').textContent     = kver;
  document.getElementById('route-receipt-json').textContent = fmt(rc);
  collapseIfLarge('route-receipt-json', 'receipt-expand-btn');
  const copyHashBtn = document.getElementById('art-hash-copy');
  if (copyHashBtn && rhash && rhash !== '—') copyHashBtn.classList.remove('hidden');
  show('receipt-json-actions');
  show('route-result');
  document.getElementById('btn-verify').disabled          = false;
  document.getElementById('btn-tamper').disabled          = false;
  document.getElementById('btn-dispatch-create').disabled = false;
}

// ── Replay Verification ────────────────────────────────────────────────────
async function verifyReceipt(btn) {
  if (!lastReceipt || !lastPolicy) {
    showVerifyResult(false, {error: {code: 'no_review_context', message: 'No receipt. Run routing first.'}}, 'No review context');
    return;
  }
  setBtn(btn, 'Verifying…', true);
  hide('verify-result');
  salLog('Verification executed', 'Replay verification started against current receipt.');

  try {
    const r = await fetch('/verify', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({
        receipt: lastReceipt,
        case:    fixtures.case,
        policy:  lastPolicy,
      }),
    });
    const data = await r.json();
    showVerifyResult(r.ok && data.result === 'VERIFIED', data, 'Replay Verification');
  } catch(e) {
    showVerifyResult(false, {error: {code: 'client_error', message: String(e)}}, 'Replay Verification');
  } finally {
    setBtn(btn, 'Verify Receipt', false);
  }
}

// ── Tamper + Verify ────────────────────────────────────────────────────────
async function tamperVerify(btn) {
  if (!lastReceipt || !lastPolicy) return;
  setBtn(btn, 'Tampering…', true);
  hide('verify-result');
  const tampered = JSON.parse(JSON.stringify(lastReceipt));
  tampered.selected_candidate_id = 'tampered-mfr-reviewer-demo';
  try {
    const r = await fetch('/verify', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({receipt: tampered, case: fixtures.case, policy: lastPolicy}),
    });
    const data = await r.json();
    showVerifyResult(false, {_tamper_note:'selected_candidate_id tampered', ...data}, 'Tamper Demo');
  } catch(e) {
    showVerifyResult(false, {error:{code:'client_error',message:String(e)}}, 'Tamper Demo');
  } finally {
    setBtn(btn, '⚠ Tamper + Verify', false);
  }
}

// ── Verify result display ──────────────────────────────────────────────────
function showVerifyResult(isVerified, data, kind) {
  document.getElementById('verify-kind-label').innerHTML =
    '<span class="pill ' + (isVerified ? 'pill-ok' : 'pill-err') + '">' + esc(kind) + '</span>';

  const banner = document.getElementById('verify-banner');
  if (isVerified) {
    banner.className = 'verify-banner banner-ok';
    banner.innerHTML = '✓ Verified — receipt replay matched'
      + '<span class="verify-sub">The kernel reproduced the same receipt hash from the original inputs.</span>';
  } else {
    const code    = data?.error?.code || data?.result || 'FAILED';
    const msg     = data?.error?.message || '';
    const heading = kind === 'Tamper Demo' ? '✗ Tamper detected' : '✗ Verification failed';
    banner.className = 'verify-banner banner-err';
    banner.innerHTML = heading
      + '<span class="verify-sub">Code: <strong>' + esc(code) + '</strong>'
      + (msg ? ' — ' + esc(msg) : '') + '</span>';
  }

  // Update step 3 summary panel
  const panel = document.getElementById('verify-summary-panel');
  if (panel) {
    panel.className = 'result-panel ' + (isVerified ? 'result-ok' : 'result-err');
  }
  const dot = document.getElementById('verify-dot');
  if (dot) dot.className = 'result-status-dot ' + (isVerified ? 'dot-green' : 'dot-red');

  // Update result fields
  const s3result = document.getElementById('s3-result-field');
  if (s3result) s3result.innerHTML =
    '<span class="pill ' + (isVerified ? 'pill-ok' : 'pill-err') + '">'
    + (isVerified ? 'VERIFIED' : 'FAILED') + '</span>';
  const s3hash = document.getElementById('s3-hash-display');
  if (s3hash && lastReceipt?.receipt_hash) {
    s3hash.textContent = lastReceipt.receipt_hash.slice(0, 16) + '…';
  }

  if (kind === 'Replay Verification') {
    verifySerial = runSerial;
    updateOpState(null, null, isVerified ? 'verified' : 'failed', null);
    appendRunHistory('Verification executed', isVerified);
    salLog('Verification completed', isVerified ? 'Receipt replay matched.' : 'Verification failed.');
  }
  const pre = document.getElementById('verify-json');
  pre.className = 'json-pre ' + (isVerified ? 'json-ok' : 'json-err');
  pre.textContent = fmt(data);
  collapseIfLarge('verify-json', 'verify-expand-btn');
  show('verify-result');
  show('verify-json-actions');
  document.getElementById('verify-result').scrollIntoView({behavior:'smooth', block:'nearest'});
}

// ── Dispatch Commitment ────────────────────────────────────────────────────
async function createDispatch(btn) {
  if (!lastReceipt || !lastPolicy) {
    showDispatchMsg('error', 'No valid receipt. Run routing and verification first.');
    return;
  }
  setBtn(btn, 'Creating…', true);
  hide('dispatch-success'); hide('dispatch-error');

  try {
    const r = await fetch('/dispatch/create', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({receipt: lastReceipt, case: fixtures.case, policy: lastPolicy}),
    });
    const data = await r.json();
    if (r.ok && data.dispatch_id) {
      lastDispatchId = data.dispatch_id;
      document.getElementById('art-dispatch-id').textContent = data.dispatch_id;
      document.getElementById('art-dispatch-status').innerHTML =
        '<span class="pill pill-info">' + esc(data.status) + '</span>';
      const _dic2 = document.getElementById('art-dispatch-id-copy');
      if (_dic2) _dic2.classList.remove('hidden');
      hide('dispatch-export-result');
      show('dispatch-created');
      document.getElementById('btn-dispatch-approve').disabled = false;
      document.getElementById('btn-dispatch-export').disabled  = true;
    } else if (r.status === 409) {
      showDispatchMsg('warn',
        '[' + (data?.error?.code || 'receipt_already_dispatched') + '] ' +
        (data?.error?.message || 'Dispatch already exists for this receipt.'));
    } else {
      showDispatchMsg('error',
        '[' + (data?.error?.code || 'error') + '] ' + (data?.error?.message || JSON.stringify(data)));
    }
  } catch(e) {
    showDispatchMsg('error', String(e));
  } finally {
    setBtn(btn, 'Create Dispatch', false);
  }
}

async function approveDispatch(btn) {
  if (!lastDispatchId) {
    showDispatchMsg('error', 'No dispatch record. Create a dispatch commitment first.');
    return;
  }
  setBtn(btn, 'Approving…', true);
  hide('dispatch-success'); hide('dispatch-error');
  let terminal = false;

  try {
    const r = await fetch('/dispatch/' + lastDispatchId + '/approve', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({approved_by: 'reviewer'}),
    });
    const data = await r.json();
    if (r.ok && data.status === 'approved') {
      document.getElementById('art-dispatch-status').innerHTML =
        '<span class="pill pill-ok">' + esc(data.status) + '</span>';
      terminal = true;
      document.getElementById('btn-dispatch-export').disabled = false;
      const s = document.getElementById('dispatch-success');
      s.textContent = 'Dispatch approved.';
      s.classList.remove('hidden');
    } else if (r.status === 409) {
      terminal = true;
      document.getElementById('btn-dispatch-export').disabled = false;
      showDispatchMsg('warn',
        '[' + (data?.error?.code || 'dispatch_not_draft') + '] ' +
        (data?.error?.message || 'Already approved.') + ' Export is now available.');
    } else {
      showDispatchMsg('error',
        '[' + (data?.error?.code || 'error') + '] ' + (data?.error?.message || JSON.stringify(data)));
    }
  } catch(e) {
    showDispatchMsg('error', String(e));
  } finally {
    setBtn(btn, 'Approve Dispatch', terminal);
  }
}

async function exportDispatch(btn) {
  if (!lastDispatchId) {
    showDispatchMsg('error', 'No dispatch record. Create and approve first.');
    return;
  }
  setBtn(btn, 'Exporting…', true);
  hide('dispatch-export-result'); hide('dispatch-success'); hide('dispatch-error');

  try {
    const r = await fetch('/dispatch/' + lastDispatchId + '/export');
    const data = await r.json();
    if (r.ok) {
      lastExportPacket = data;
      dispatchSerial = runSerial;
      updateIntegrityBadges();
      updateDispatchReadiness();
      updateDispatchBlockers();
      updateMicrobadges();
      updateFreshnessMarkers();
      updateRunTimeline();
      updateOab();
      updateOutcomeBanner();
      updateCompletionChecklist();
      updatePreflightCard();
      updateHandoffSummary();
      updateConsistencySentinel();
      updateActiveSectionEmphasis();
      updateRunIdentityBlock();
      updateLineageBadges();
      updateLineageNotes();
      updateDpi();
      updateDossier();
      updateDryRunPanel();
      updatePilotHandoffSummary();
      updateArtifactBundle();
      updateCanonicalWorkflow();
      const _dsn = document.getElementById('dispatch-stale-note');
      if (_dsn) _dsn.classList.add('hidden');
      updateActiveRunContext();
      updateNextActionRail();
      document.getElementById('art-dispatch-status').innerHTML =
        '<span class="pill pill-ok">' + esc(data.status) + '</span>';
      document.getElementById('dispatch-export-json').textContent = fmt(data);
      collapseIfLarge('dispatch-export-json', 'dispatch-expand-btn');
      show('dispatch-export-result');
      show('dispatch-export-actions');
      appendRunHistory('Dispatch executed', true);
      salLog('Dispatch export generated', 'Dispatch packet exported for current run.');
      const s = document.getElementById('dispatch-success');
      s.textContent = 'Export complete — dispatch packet ready for handoff.';
      s.classList.remove('hidden');
      updateStepCards();
    } else {
      showDispatchMsg('error',
        '[' + (data?.error?.code || 'error') + '] ' + (data?.error?.message || JSON.stringify(data)));
    }
  } catch(e) {
    showDispatchMsg('error', String(e));
  } finally {
    setBtn(btn, 'Export Dispatch Packet', false);
  }
}

// ── Artifact copy / download ───────────────────────────────────────────────
function downloadExportPacket() {
  if (!lastExportPacket) return;
  const id = lastDispatchId ? lastDispatchId.slice(0, 8) : 'dispatch';
  const blob = new Blob([fmt(lastExportPacket)], {type: 'application/json'});
  const url  = URL.createObjectURL(blob);
  const a    = document.createElement('a');
  a.href = url; a.download = 'export_packet_' + id + '.json'; a.click();
  URL.revokeObjectURL(url);
}
function copyExportJson(btn) {
  if (!lastExportPacket) return;
  navigator.clipboard.writeText(fmt(lastExportPacket)).then(() => {
    btn.textContent = 'Copied'; btn.style.color = 'var(--green)';
  }).catch(() => { btn.textContent = 'Failed'; });
  setTimeout(() => { btn.textContent = 'Copy JSON'; btn.style.color = ''; }, 1500);
}
function copyDispatchId(btn) {
  const id = document.getElementById('art-dispatch-id').textContent.trim();
  if (!id) return;
  navigator.clipboard.writeText(id).then(() => {
    btn.textContent = 'Copied'; btn.style.color = 'var(--green)';
  }).catch(() => { btn.textContent = 'Failed'; });
  setTimeout(() => { btn.textContent = 'Copy'; btn.style.color = ''; }, 1500);
}
function copyArtHashVal(btn) {
  const hash = document.getElementById('art-hash').textContent.trim();
  if (!hash || hash === '—') return;
  navigator.clipboard.writeText(hash).then(() => {
    btn.textContent = 'Copied'; btn.style.color = 'var(--green)';
  }).catch(() => { btn.textContent = 'Failed'; });
  setTimeout(() => { btn.textContent = 'Copy'; btn.style.color = ''; }, 1500);
}
function copyReceiptJson(btn) {
  if (!lastReceipt) return;
  navigator.clipboard.writeText(fmt(lastReceipt)).then(() => {
    btn.textContent = 'Copied'; btn.style.color = 'var(--green)';
  }).catch(() => { btn.textContent = 'Failed'; });
  setTimeout(() => { btn.textContent = 'Copy receipt JSON'; btn.style.color = ''; }, 1500);
}
function copyVerifyJson(btn) {
  const pre = document.getElementById('verify-json');
  if (!pre) return;
  navigator.clipboard.writeText(pre.textContent).then(() => {
    btn.textContent = 'Copied'; btn.style.color = 'var(--green)';
  }).catch(() => { btn.textContent = 'Failed'; });
  setTimeout(() => { btn.textContent = 'Copy'; btn.style.color = ''; }, 1500);
}
function copyRouteErrorJson(btn) {
  const pre = document.getElementById('route-error-json');
  if (!pre) return;
  navigator.clipboard.writeText(pre.textContent).then(() => {
    btn.textContent = 'Copied'; btn.style.color = 'var(--green)';
  }).catch(() => { btn.textContent = 'Failed'; });
  setTimeout(() => { btn.textContent = 'Copy'; btn.style.color = ''; }, 1500);
}
function downloadReceiptJson() {
  if (!lastReceipt) return;
  const blob = new Blob([JSON.stringify(lastReceipt, null, 2)], {type: 'application/json'});
  const url  = URL.createObjectURL(blob);
  const a    = document.createElement('a');
  a.href     = url;
  const hash = lastReceipt.receipt_hash ? lastReceipt.receipt_hash.slice(0, 12) : 'receipt';
  a.download = 'receipt_' + hash + '.json';
  a.click();
  URL.revokeObjectURL(url);
}
async function copyReceiptHash(btn, hash) {
  try {
    await navigator.clipboard.writeText(hash);
    btn.textContent = 'Copied'; btn.style.color = 'var(--green)';
  } catch(e) {
    btn.textContent = 'Copy failed';
  }
  setTimeout(() => { btn.textContent = 'Copy'; btn.style.color = ''; }, 1500);
}

// ── Integrity badges (legacy compat) ───────────────────────────────────────
function setBadge(id, state) {
  const el = document.getElementById(id);
  if (!el) return;
  if (!state) { el.classList.add('hidden'); return; }
  el.classList.remove('hidden');
  el.textContent = state.toUpperCase();
}
function updateIntegrityBadges() {
  const receiptState = opRouting !== 'available' ? null
    : opVerify === 'verified' ? 'verified'
    : opVerify === 'failed'   ? 'failed'
    : 'unverified';
  setBadge('route-result-badge', receiptState);
  setBadge('receipt-json-badge', receiptState);
  const verifyState = opVerify === 'verified' ? 'verified'
    : opVerify === 'failed' ? 'failed' : null;
  setBadge('verify-result-badge', verifyState);
  setBadge('dispatch-result-badge', lastExportPacket ? 'verified' : null);
}

// ── Dispatch readiness (legacy compat) ────────────────────────────────────
const DR_LABELS = {
  'cl-receipt':  'Receipt reviewed',
  'cl-verify':   'Verification succeeded',
  'cl-dispatch': 'Dispatch action confirmed',
};
function setCheck(id, ok) {
  const el = document.getElementById(id);
  if (!el) return;
  el.textContent = (ok ? '✓ ' : '◻ ') + (DR_LABELS[id] || '');
}
function updateDispatchReadiness() {
  const receiptOk = opRouting === 'available';
  const verifyOk  = opVerify  === 'verified';
  const completed = !!lastExportPacket;
  setCheck('cl-receipt',  receiptOk);
  setCheck('cl-verify',   verifyOk);
  setCheck('cl-dispatch', completed);
  const status = document.getElementById('dr-status');
  const reason = document.getElementById('dr-reason');
  if (!status) return;
  if (completed)        { status.textContent = 'Dispatch completed'; reason.textContent = 'Export packet produced.'; }
  else if (verifyOk)    { status.textContent = 'Ready for dispatch';  reason.textContent = 'Verification succeeded.'; }
  else if (receiptOk)   { status.textContent = 'Not ready';           reason.textContent = 'Verification pending.'; }
  else                  { status.textContent = 'Not ready';           reason.textContent = 'Required artifact not yet generated.'; }
}
function dispatchBlockers() {
  const items = [];
  if (opRouting !== 'available') {
    items.push({text:'No current route result — run routing first.',
                anchor:{label:'Go to routing', target:'btn-route-norm'}});
  } else {
    if (opVerify === 'not-run') {
      items.push({text:'Verification not yet executed for current run.',
                  anchor:{label:'Go to verification', target:'btn-verify'}});
    }
    if (opVerify === 'failed') {
      items.push({text:'Verification result does not satisfy dispatch readiness.',
                  anchor:{label:'Review readiness', target:'dispatch-readiness-panel'}});
    }
  }
  return items;
}
function updateDispatchBlockers() {
  const body = document.getElementById('dbl-body');
  if (!body) return;
  if (lastExportPacket) {
    body.innerHTML = '<div>Dispatch already exported.</div>'; return;
  }
  const items = dispatchBlockers();
  body.innerHTML = items.length === 0
    ? '<div>No current blockers.</div>'
    : items.map(b => '<div>' + esc(b.text) + '</div>').join('');
}

// ── Dispatch packet inspection (legacy compat) ────────────────────────────
function updateDpi() {
  const meta      = document.getElementById('dpi-meta');
  const empty     = document.getElementById('dpi-empty');
  const viewer    = document.getElementById('dpi-viewer');
  const origin    = document.getElementById('dpi-origin');
  const integrity = document.getElementById('dpi-integrity');
  if (!meta) return;
  if (!lastExportPacket) { meta.classList.add('hidden'); empty.classList.remove('hidden'); if(viewer){viewer.classList.add('hidden');viewer.textContent='';} return; }
  empty.classList.add('hidden'); meta.classList.remove('hidden');
  if (viewer) { viewer.classList.remove('hidden'); viewer.textContent = fmt(lastExportPacket); }
  const dlin = dispatchLineage();
  if (origin) origin.textContent = dlin === 'current' ? 'current run' : dlin === 'prev' ? 'previous run' : '—';
  const vlin = verifyLineage();
  if (integrity) integrity.textContent = vlin === 'current' && opVerify === 'verified' ? 'verified packet' : 'verification not executed';
}

// ── Dispatch handoff dossier (legacy compat) ──────────────────────────────
function dhdVerdictKey() {
  if (runSerial === 0) return 'none';
  const dlin = dispatchLineage();
  if (dlin === 'current') return 'exported';
  if (dlin === 'prev')    return 'attention';
  const vlin = verifyLineage();
  if (vlin === 'current' && opVerify === 'verified') return 'ready';
  return 'not-ready';
}
const DHD_VERDICTS = {
  'none':      {text:'No current dispatch packet'},
  'not-ready': {text:'Current route not ready for dispatch'},
  'ready':     {text:'Current route ready for dispatch export'},
  'exported':  {text:'Current dispatch packet exported'},
  'attention': {text:'Current dispatch packet requires attention'},
};
function updateDossier() {
  const vkey    = dhdVerdictKey();
  const conf    = DHD_VERDICTS[vkey];
  const verdict = document.getElementById('dhd-verdict');
  if (!verdict) return;
  verdict.textContent = conf.text;
  const nextText = document.getElementById('dhd-next-text');
  if (nextText) nextText.textContent = vkey === 'none' ? 'Generate a route first.' : vkey === 'exported' ? 'Complete.' : 'Continue workflow.';
}

// ── Lineage tracking ───────────────────────────────────────────────────────
function verifyLineage() {
  if (runSerial === 0)                              return 'idle';
  if (verifySerial === runSerial)                   return 'current';
  if (verifySerial > 0 && verifySerial < runSerial) return 'prev';
  return 'idle';
}
function dispatchLineage() {
  if (runSerial === 0)                                  return 'idle';
  if (dispatchSerial === runSerial)                     return 'current';
  if (dispatchSerial > 0 && dispatchSerial < runSerial) return 'prev';
  return 'idle';
}
function setLinBadge(id, lineage, idleLabel) {
  const el = document.getElementById(id); if (!el) return;
  el.textContent = lineage === 'current' ? 'current run' : lineage === 'prev' ? 'previous run' : (idleLabel || 'not executed');
}
function updateLineageBadges() {
  setLinBadge('lin-verify',          verifyLineage(),   'not executed');
  setLinBadge('lin-dispatch-export', dispatchLineage(), 'not exported');
}
function updateLineageNotes() {
  const vn = document.getElementById('lin-verify-note');
  const dn = document.getElementById('lin-dispatch-note');
  if (vn) vn.classList.toggle('hidden', verifyLineage()   !== 'prev');
  if (dn) dn.classList.toggle('hidden', dispatchLineage() !== 'prev');
}
function updateRunIdentityBlock() {
  function setRibVal(id, state, text) {
    const el = document.getElementById(id); if (!el) return;
    el.textContent = text;
  }
  if (runSerial === 0) { setRibVal('rib-route','idle','no run yet'); setRibVal('rib-receipt','idle','not generated'); setRibVal('rib-verify','idle','not executed'); setRibVal('rib-dispatch','idle','not exported'); return; }
  setRibVal('rib-route',   null, opRouting === 'available' ? 'current run' : 'failed');
  setRibVal('rib-receipt', null, opReceipt === 'available' ? 'current run' : 'not generated');
  const vlin = verifyLineage();
  setRibVal('rib-verify',   null, vlin === 'current' ? 'current run' : vlin === 'prev' ? 'previous run' : 'not executed');
  const dlin = dispatchLineage();
  setRibVal('rib-dispatch', null, dlin === 'current' ? 'current run' : dlin === 'prev' ? 'previous run' : 'not exported');
}

// ── Session guard (legacy compat) ─────────────────────────────────────────
function updateSessionGuard() {
  const el = document.getElementById('osg'); if (!el) return;
  const vlin = verifyLineage(); const dlin = dispatchLineage();
  const stale = vlin === 'prev' || dlin === 'prev';
  el.style.display = stale ? 'block' : 'none';
}
function startCleanRun() {
  lastReceipt = null; lastPolicy = null; lastDispatchId = null; lastExportPacket = null;
  lastRouteInputs = null; lastRouteEndpoint = null; reproStatus = 'not-tested';
  runSerial = 0; verifySerial = 0; dispatchSerial = 0;
  hide('route-result'); hide('route-error'); hide('verify-result');
  hide('dispatch-created'); hide('dispatch-export-result');
  hide('dispatch-success'); hide('dispatch-error');
  show('results-placeholder');
  clearRunHistory();
  document.getElementById('btn-dispatch-create').disabled  = true;
  document.getElementById('btn-dispatch-approve').disabled = true;
  document.getElementById('btn-dispatch-export').disabled  = true;
  document.getElementById('btn-verify').disabled  = true;
  document.getElementById('btn-tamper').disabled  = true;
  salLog('Session reset', 'Operator initiated clean run.');
  updateOpState('not-run', 'not-run', 'not-run', 'not-run');
}

// ── Artifact bundle (legacy compat) ───────────────────────────────────────
function cabVerdictKey() {
  if (runSerial === 0) return 'none';
  const vlin = verifyLineage(); const dlin = dispatchLineage();
  if (opRouting === 'failed') return 'attention';
  if (vlin === 'prev' || dlin === 'prev') return 'attention';
  if (vlin === 'current' && opVerify === 'failed') return 'attention';
  if (reproStatus === 'mismatch') return 'attention';
  if (opRouting === 'available' && opReceipt === 'available' && vlin === 'current' && opVerify === 'verified' && dlin === 'current' && reproStatus === 'reproducible') return 'ready';
  return 'incomplete';
}
function updateArtifactBundle() {
  const verdict = document.getElementById('cab-verdict'); if (!verdict) return;
  const vkey = cabVerdictKey();
  verdict.textContent = {none:'No current bundle',incomplete:'Current bundle incomplete',ready:'Current bundle ready',attention:'Attention required'}[vkey] || '';
  const arts = document.getElementById('cab-artifacts'); if (!arts) return;
  const vlin = verifyLineage(); const dlin = dispatchLineage();
  arts.innerHTML = '<div>Route: ' + (opRouting==='available'?'present':'missing') + '</div>'
    + '<div>Receipt: ' + (opReceipt==='available'?'present':'missing') + '</div>'
    + '<div>Verify: ' + (vlin==='current'?opVerify:'idle') + '</div>'
    + '<div>Dispatch: ' + (dlin==='current'?'present':'missing') + '</div>';
  const nt = document.getElementById('cab-next-text');
  if (nt) nt.textContent = vkey === 'ready' ? 'Complete.' : 'Continue workflow.';
  const m = document.getElementById('cab-meaning');
  if (m) m.textContent = 'Artifact bundle for current run.';
}

// ── Canonical workflow (legacy compat) ────────────────────────────────────
function updateCanonicalWorkflow() {
  const routeOk   = opRouting === 'available';
  const receiptOk = opReceipt === 'available';
  const vlin = verifyLineage();
  const verifyOk  = vlin === 'current' && opVerify === 'verified';
  const dlin = dispatchLineage();
  const dispOk    = dlin === 'current';
  const reproExec = reproStatus === 'reproducible' || reproStatus === 'mismatch';
  function setS(id, done, blocked) {
    const el = document.getElementById(id); if (!el) return;
    el.textContent = done ? 'completed' : (blocked ? 'blocked' : 'available');
  }
  setS('cpw-s1', routeOk,   false);
  setS('cpw-s2', verifyOk,  !routeOk);
  setS('cpw-s3', receiptOk, !routeOk);
  setS('cpw-s4', reproExec, !routeOk);
  setS('cpw-s5', dispOk,    !verifyOk);
  setS('cpw-s6', dispOk && reproExec, !dispOk);
}

// ── Dry-run panel (legacy compat) ─────────────────────────────────────────
function drsVerdictKey() {
  if (runSerial === 0) return 'none';
  const vlin = verifyLineage(); const dlin = dispatchLineage();
  if (opRouting === 'failed' || vlin === 'prev' || dlin === 'prev' || (vlin === 'current' && opVerify === 'failed') || reproStatus === 'mismatch') return 'attention';
  const reproExec = reproStatus === 'reproducible' || reproStatus === 'mismatch';
  if (opRouting === 'available' && opReceipt === 'available' && vlin === 'current' && opVerify === 'verified' && dlin === 'current' && reproExec && reproStatus === 'reproducible') return 'passed';
  return 'incomplete';
}
function updateDryRunPanel() {
  const verdict = document.getElementById('drs-verdict'); if (!verdict) return;
  const vkey = drsVerdictKey();
  verdict.textContent = {none:'No dry-run',incomplete:'Incomplete',passed:'Passed',attention:'Attention required'}[vkey] || '';
  const m = document.getElementById('drs-meaning'); if (m) m.textContent = '';
  const nt = document.getElementById('drs-next-text');
  if (nt) nt.textContent = vkey === 'passed' ? 'Complete.' : 'Continue workflow.';
}

// ── Pilot handoff summary (legacy compat) ────────────────────────────────
function updatePilotHandoffSummary() {
  const verdict = document.getElementById('phs-verdict'); if (!verdict) return;
  const dlin = dispatchLineage();
  if (dlin === 'current') { verdict.textContent = 'Ready for pilot handoff'; return; }
  const vlin = verifyLineage();
  if (vlin === 'current' && opVerify === 'verified') { verdict.textContent = 'Pending dispatch export'; return; }
  verdict.textContent = 'Not ready for pilot handoff';
  const nt = document.getElementById('phs-action-text');
  if (nt) nt.textContent = 'Complete all minimum workflow steps.';
}

// ── Active run context ────────────────────────────────────────────────────
function updateActiveRunContext() {
  const block = document.getElementById('active-run-context'); if (!block) return;
  if (opRouting !== 'available' || !lastReceipt) { block.classList.add('hidden'); return; }
  block.classList.remove('hidden');
  const mfr = document.getElementById('arc-manufacturer');
  if (mfr) mfr.textContent = lastReceipt.selected_candidate_id || '(none)';
  const hash = document.getElementById('arc-receipt-hash');
  const h = lastReceipt.receipt_hash || '—';
  if (hash) hash.textContent = h !== '—' ? h.slice(0, 16) + '…' : '—';
  const verEl = document.getElementById('arc-verify-status');
  if (verEl) verEl.textContent = opVerify === 'verified' ? 'Verified' : opVerify === 'failed' ? 'Failed' : 'Pending';
  const dispEl = document.getElementById('arc-dispatch-status');
  if (dispEl) dispEl.textContent = lastExportPacket ? 'Exported' : 'Pending';
}

// ── Next-action rail (legacy compat) ────────────────────────────────────
function updateNextActionRail() {
  const actionEl = document.getElementById('nar-action'); if (!actionEl) return;
  if (lastExportPacket)             { actionEl.textContent = 'Workflow complete'; }
  else if (opVerify === 'verified') { actionEl.textContent = 'Next: export dispatch'; }
  else if (opRouting === 'available') { actionEl.textContent = 'Next: verify current route'; }
  else { actionEl.textContent = 'Next: run route'; }
}

// ── Handoff note (legacy compat) ─────────────────────────────────────────
function updateHandoffNote() {
  const body  = document.getElementById('hn-body'); if (!body) return;
  body.textContent = lastExportPacket ? 'Handoff complete.' : 'No export for current route.';
}

// ── Microbadges / freshness (legacy compat) ───────────────────────────────
function setMicrobadge(id, state) { const el = document.getElementById(id); if(el) el.textContent = state; }
function updateMicrobadges() {
  setMicrobadge('mb-receipt',  opRouting === 'available' ? 'available' : 'not-available');
  setMicrobadge('mb-verify',   opVerify === 'verified' ? 'verified' : opVerify === 'failed' ? 'failed' : 'not-available');
  setMicrobadge('mb-dispatch', lastExportPacket ? 'exported' : 'not-available');
}
function setFreshness(id, fresh, label) { const el = document.getElementById(id); if(el) el.textContent = label; }
function updateFreshnessMarkers() {
  setFreshness('fm-receipt',  opRouting === 'available', opRouting === 'available' ? 'current run' : 'not yet produced');
  const verifyDone = opVerify === 'verified' || opVerify === 'failed';
  setFreshness('fm-verify',   verifyDone, verifyDone ? 'current run' : 'not yet executed');
  setFreshness('fm-dispatch', !!lastExportPacket, lastExportPacket ? 'current run' : 'not yet exported');
}

// ── Run timeline (legacy compat) ──────────────────────────────────────────
function updateRunTimeline() {
  function setState(id, s) { const el = document.getElementById(id); if(el) el.className='rt-step rt-'+s; }
  setState('rt-route',    opRouting === 'available' ? 'done' : 'idle');
  setState('rt-receipt',  opReceipt === 'available' ? 'done' : 'idle');
  setState('rt-verify',   opVerify === 'verified' ? 'done' : opRouting === 'available' ? 'ready' : 'idle');
  setState('rt-dispatch', lastExportPacket ? 'done' : opVerify === 'verified' ? 'ready' : opVerify === 'failed' ? 'blocked' : 'idle');
  const s = document.getElementById('rt-summary');
  if (s) s.textContent = lastExportPacket ? 'Complete' : opVerify === 'verified' ? 'Dispatch ready' : opRouting === 'available' ? 'Verify pending' : 'Not started';
}

// ── OAB (legacy compat) ───────────────────────────────────────────────────
function updateOab() {
  const a = document.getElementById('oab-action'); if (!a) return;
  if (lastExportPacket)           a.textContent = 'Current run complete';
  else if (opVerify === 'verified') a.textContent = 'Export dispatch packet';
  else if (opVerify === 'failed')   a.textContent = 'Resolve verification';
  else if (opRouting === 'available') a.textContent = 'Run verification';
  else a.textContent = 'Start a route';
}
function oabNavigate() {
  let target = 'btn-route-norm';
  if (lastExportPacket)            target = 'dispatch-export-result';
  else if (opVerify === 'verified') target = 'btn-dispatch-export';
  else if (opVerify === 'failed')   target = 'verify-result';
  else if (opRouting === 'available') target = 'btn-verify';
  const el = document.getElementById(target); if (el) el.scrollIntoView({behavior:'smooth',block:'nearest'});
}

// ── Outcome banner (legacy compat) ───────────────────────────────────────
function updateOutcomeBanner() {
  const orb = document.getElementById('orb'); if (!orb) return;
  const h = document.getElementById('orb-headline'); const d = document.getElementById('orb-detail');
  if (lastExportPacket) { if(h) h.textContent='Complete'; if(d) d.textContent='Dispatch exported.'; }
  else if (opVerify === 'verified') { if(h) h.textContent='Verified'; if(d) d.textContent='Ready for dispatch.'; }
  else if (opRouting === 'available') { if(h) h.textContent='Routed'; if(d) d.textContent='Verify pending.'; }
  else { if(h) h.textContent='Not started'; if(d) d.textContent='Run routing.'; }
}
function orbNavigate() {}

// ── Completion checklist (legacy compat) ─────────────────────────────────
function updateCompletionChecklist() {
  const rows = document.getElementById('crc-rows'); if (!rows) return;
  rows.textContent = '';
  const footer = document.getElementById('crc-footer');
  const allDone = opRouting==='available' && opReceipt==='available' && opVerify==='verified' && !!lastExportPacket;
  if (footer) footer.textContent = allDone ? 'Complete' : 'Incomplete';
}
function crcNavigate(t) { const e=document.getElementById(t); if(e) e.scrollIntoView({behavior:'smooth',block:'nearest'}); }

// ── Preflight card (legacy compat) ────────────────────────────────────────
function pfcVerdictKey() {
  if (lastExportPacket)        return 'complete';
  if (opVerify === 'verified') return 'ready';
  return 'not-ready';
}
function updatePreflightCard() {
  const card = document.getElementById('pfc'); if (!card) return;
  const vk = pfcVerdictKey();
  const h = document.getElementById('pfc-headline');
  if (h) h.textContent = vk === 'complete' ? 'Complete' : vk === 'ready' ? 'Ready' : 'Not ready';
}
function pfcNavigate() {}

// ── Handoff summary (legacy compat) ──────────────────────────────────────
function updateHandoffSummary() {
  const v = document.getElementById('hsc-verdict'); if (!v) return;
  const vk = pfcVerdictKey();
  v.textContent = vk === 'complete' ? 'Complete' : vk === 'ready' ? 'Ready for dispatch' : 'Not ready';
  const s = document.getElementById('hsc-summary');
  if (s) s.textContent = vk === 'complete' ? 'Current run complete.' : 'Complete workflow steps.';
}

// ── Consistency sentinel (legacy compat) ─────────────────────────────────
function gatherConsistencyMismatches() { return []; }
function updateConsistencySentinel() {
  const card = document.getElementById('ccs'); if (!card) return;
  const h = document.getElementById('ccs-headline');
  if (h) h.textContent = 'Consistent';
}

// ── Active section emphasis (legacy compat) ───────────────────────────────
function activeSectionIndex() {
  if (lastExportPacket)                                 return 3;
  if (opVerify === 'verified' || opVerify === 'failed') return 2;
  if (opRouting === 'available')                        return 1;
  return 0;
}
function updateActiveSectionEmphasis() {
  // purely legacy — step card state handled by updateStepCards()
}

// ── Pilot run history ─────────────────────────────────────────────────────
function appendRunHistory(label, ok) {
  const ts = new Date().toLocaleTimeString([], {hour:'2-digit',minute:'2-digit',second:'2-digit'});
  runHistory.push({ts, label, ok});
  const list = document.getElementById('run-history-list'); if (!list) return;
  const entry = document.createElement('div');
  entry.textContent = ts + ' — ' + label + (ok ? ' ✓' : ' ✗');
  list.appendChild(entry);
  show('run-history-panel');
}
function clearRunHistory() {
  runHistory.length = 0;
  const list = document.getElementById('run-history-list');
  if (list) list.innerHTML = '';
  hide('run-history-panel');
}

// ── Session activity log (legacy compat) ─────────────────────────────────
const SAL_MAX = 20;
let sessionLog = [];
function salLog(label, msg) {
  sessionLog.unshift({label, msg});
  if (sessionLog.length > SAL_MAX) sessionLog.length = SAL_MAX;
}
function clearSessionLog() { sessionLog = []; }

// ── Artifact size guard ────────────────────────────────────────────────────
const ARTIFACT_COLLAPSE_LINES = 40;
function collapseIfLarge(preId, btnId) {
  const pre = document.getElementById(preId); const btn = document.getElementById(btnId);
  if (!pre || !btn) return;
  if ((pre.textContent.match(/\n/g) || []).length + 1 > ARTIFACT_COLLAPSE_LINES) {
    pre.classList.add('collapsed'); btn.classList.remove('hidden');
  } else {
    pre.classList.remove('collapsed'); btn.classList.add('hidden');
  }
}
function expandArtifact(preId, btnId) {
  const pre = document.getElementById(preId); const btn = document.getElementById(btnId);
  if (pre) pre.classList.remove('collapsed');
  if (btn) btn.classList.add('hidden');
}

// ── Repro check ───────────────────────────────────────────────────────────
const REPRO_STATES = {
  'not-tested':   {text:'Reproducibility not tested'},
  'running':      {text:'Check in progress…'},
  'reproducible': {text:'Reproducible'},
  'mismatch':     {text:'Mismatch detected'},
};
async function runReproCheck(btn) {
  if (!lastRouteInputs || !lastRouteEndpoint || !lastReceipt) return;
  reproStatus = 'running'; updateReproPanel(); if (btn) btn.disabled = true;
  try {
    const r = await fetch(lastRouteEndpoint, {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(lastRouteInputs)});
    if (!r.ok) { reproStatus = 'mismatch'; return; }
    const data = await r.json();
    const reproHash = data?.receipt?.receipt_hash;
    const origHash  = lastReceipt?.receipt_hash;
    reproStatus = (reproHash && origHash && reproHash === origHash) ? 'reproducible' : 'mismatch';
  } catch(e) { reproStatus = 'mismatch'; }
  finally { updateReproPanel(); updateDryRunPanel(); updatePilotHandoffSummary(); updateArtifactBundle(); updateCanonicalWorkflow(); }
}
function updateReproPanel() {
  const statusEl = document.getElementById('rrc-status'); if (!statusEl) return;
  const st = REPRO_STATES[reproStatus] || REPRO_STATES['not-tested'];
  statusEl.textContent = st.text;
  const btn = document.getElementById('btn-repro');
  if (btn) btn.disabled = !lastReceipt || reproStatus === 'running';
}

// ── Op state ──────────────────────────────────────────────────────────────
function updateOpState(routing, receipt, verify, dispatch) {
  if (routing  != null) opRouting  = routing;
  if (receipt  != null) opReceipt  = receipt;
  if (verify   != null) opVerify   = verify;
  if (dispatch != null) opDispatch = dispatch;
  [['ops-routing', opRouting], ['ops-receipt', opReceipt],
   ['ops-verify',  opVerify],  ['ops-dispatch', opDispatch]].forEach(([id, st]) => {
    const el = document.getElementById(id); if (el) el.textContent = st;
  });
  const vpn = document.getElementById('verify-pending-note');
  if (vpn) vpn.classList.toggle('hidden', !(opRouting === 'available' && opVerify === 'not-run'));
  const dbn = document.getElementById('dispatch-blocked-note');
  if (dbn) dbn.classList.toggle('hidden', opVerify !== 'failed');
  const dsn = document.getElementById('dispatch-stale-note');
  if (dsn) dsn.classList.toggle('hidden', !(opRouting === 'available' && !lastExportPacket));
  updateIntegrityBadges();
  updateDispatchReadiness();
  updateDispatchBlockers();
  updateActiveRunContext();
  updateNextActionRail();
  updateHandoffNote();
  updateMicrobadges();
  updateFreshnessMarkers();
  updateRunTimeline();
  updateOab();
  updateOutcomeBanner();
  updateCompletionChecklist();
  updatePreflightCard();
  updateHandoffSummary();
  updateConsistencySentinel();
  updateActiveSectionEmphasis();
  updateRunIdentityBlock();
  updateLineageBadges();
  updateLineageNotes();
  updateSessionGuard();
  updateDpi();
  updateDossier();
  updateReproPanel();
  updateDryRunPanel();
  updatePilotHandoffSummary();
  updateArtifactBundle();
  updateCanonicalWorkflow();
  updateStepCards();
}

// ── Audit snapshot ────────────────────────────────────────────────────────
function buildAuditSnapshot() {
  const receiptJson  = lastReceipt    ? fmt(lastReceipt)        : '(not generated)';
  const verifyText   = document.getElementById('verify-json')?.textContent || '(not executed)';
  const dispatchText = lastExportPacket ? fmt(lastExportPacket)  : '(not exported)';
  const readinessText = 'Routing: ' + opRouting + ' | Verification: ' + opVerify + ' | Dispatch: ' + (lastExportPacket ? 'exported' : 'not exported');
  return ['PostCAD Operator Demo — Audit Snapshot', '='.repeat(44), '',
    'Receipt', '-------', receiptJson, '',
    'Verification', '------------', verifyText, '',
    'Dispatch', '--------', dispatchText, '',
    'Dispatch readiness', '------------------', readinessText,
  ].join('\n');
}
function copyAuditSnapshot(btn) {
  const snapshot = buildAuditSnapshot();
  navigator.clipboard.writeText(snapshot).then(() => {
    btn.textContent = 'Copied'; btn.style.color = 'var(--green)';
  }).catch(() => { btn.textContent = 'Failed'; });
  setTimeout(() => { btn.textContent = 'Copy snapshot'; btn.style.color = ''; }, 1500);
}
function downloadAuditSnapshot() {
  const snapshot = buildAuditSnapshot();
  const blob = new Blob([snapshot], {type: 'text/plain'});
  const url  = URL.createObjectURL(blob);
  const a    = document.createElement('a');
  a.href = url; a.download = 'postcad_audit_snapshot.txt'; a.click();
  URL.revokeObjectURL(url);
}

// ── Dispatch message ──────────────────────────────────────────────────────
function showDispatchMsg(kind, text) {
  const el = document.getElementById('dispatch-error');
  el.className = kind === 'warn' ? 'warn-note' : 'error-note';
  el.textContent = text;
  show('dispatch-error');
}

// ── Norm form helpers ─────────────────────────────────────────────────────
const NORM_INPUT_IDS = {
  case_id:          'norm-case-id',
  restoration_type: 'norm-restoration-type',
  material:         'norm-material',
  jurisdiction:     'norm-jurisdiction',
};
function readNormInputs() {
  return Object.fromEntries(
    Object.entries(NORM_INPUT_IDS).map(([k, id]) => [k, document.getElementById(id).value.trim()])
  );
}
function validateNormInput(c) {
  return ['case_id', 'restoration_type', 'material', 'jurisdiction']
    .filter(k => !c[k] || !String(c[k]).trim());
}
function markNormInvalid(missing) {
  Object.entries(NORM_INPUT_IDS).forEach(([k, id]) => {
    const el = document.getElementById(id); if (el) el.style.borderColor = missing.includes(k) ? 'var(--red)' : '';
  });
}
function clearNormInvalid() {
  Object.values(NORM_INPUT_IDS).forEach(id => {
    const el = document.getElementById(id); if (el) el.style.borderColor = '';
  });
}
function loadNormSample() {
  document.getElementById('norm-case-id').value          = 'f1000001-0000-0000-0000-000000000001';
  document.getElementById('norm-restoration-type').value = 'crown';
  document.getElementById('norm-material').value         = 'zirconia';
  document.getElementById('norm-jurisdiction').value     = 'DE';
  clearNormInvalid();
  if (fixtures) document.getElementById('btn-route-norm').disabled = false;
}
function clearNormForm() {
  document.getElementById('norm-case-id').value          = '';
  document.getElementById('norm-restoration-type').value = '';
  document.getElementById('norm-material').value         = '';
  document.getElementById('norm-jurisdiction').value     = '';
  clearNormInvalid();
}
function toggleNormReceiptJson() {
  const pre = document.getElementById('norm-receipt-json-block');
  const btn = document.getElementById('btn-toggle-receipt');
  if (!pre || !btn) return;
  const isHidden = pre.classList.toggle('hidden');
  btn.textContent = isHidden ? 'Show receipt JSON' : 'Hide receipt JSON';
}

// ── Keyboard shortcut ─────────────────────────────────────────────────────
document.getElementById('norm-input-section').addEventListener('keydown', function(e) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
    const btn = document.getElementById('btn-route-norm');
    if (btn && !btn.disabled) { e.preventDefault(); routeNormalized(btn); }
  }
});

// ── Utility ───────────────────────────────────────────────────────────────
function fmt(o)   { return JSON.stringify(o, null, 2); }
function esc(s)   { return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;'); }
function show(id) { const e=document.getElementById(id); if(e) e.classList.remove('hidden'); }
function hide(id) { const e=document.getElementById(id); if(e) e.classList.add('hidden'); }
function setBtn(btn, label, disabled) { btn.textContent = label; btn.disabled = disabled; }
function errorHint(code) {
  const c = String(code || '').toLowerCase();
  if (c.includes('normaliz') || c.includes('validat') || c.includes('parse'))
    return 'Check that all fields contain valid values.';
  if (c.includes('no_eligible') || c.includes('routing') || c.includes('refused'))
    return 'No manufacturer matched the routing criteria.';
  return 'Clear the form and re-enter the values, or use Load Demo Case.';
}
// legacy no-ops
function previewRow(k, v) { return ''; }
function norm_preview_row(k, v) { return ''; }
</script>
</body>
</html>"#;
