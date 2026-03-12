//! Embedded showcase demo HTML.
//!
//! Form-based demo served at `GET /demo`.
//!
//! Layout (top to bottom):
//!   Header + workflow strip
//!   Two-column: Case Intake (left) | Routing Decision (right)
//!   Full-width: Constraint Evaluation  ← client-side graph, no new backend call
//!   Full-width: Determinism Check      ← hashes from receipt
//!   Full-width: Proof & Inspection     ← collapsible JSON
//!
//! Calls only the existing `/route`, `/verify`, `/protocol-manifest` endpoints.
//! No new backend logic.

pub const DEMO_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>PostCAD Pilot Demo</title>
<style>
/* ── reset ── */
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}

/* ── base ── */
body{
  font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",system-ui,sans-serif;
  background:#0d1117;color:#c9d1d9;
  min-height:100vh;font-size:14px;line-height:1.6;
}

/* ── header ── */
.site-header{
  background:#161b22;border-bottom:1px solid #21262d;
  padding:.75rem 1.5rem;display:flex;align-items:center;gap:1rem;
}
.site-header-hex{
  width:26px;height:26px;background:#1a3e2c;border:1px solid #238636;
  border-radius:5px;display:flex;align-items:center;justify-content:center;
  font-size:.8rem;color:#3fb950;flex-shrink:0;
}
.site-header-title{font-size:.95rem;font-weight:700;color:#f0f6fc;letter-spacing:-.01em}
.site-header-sub{font-size:.78rem;color:#6e7681;margin-left:.15rem}
.site-header-links{margin-left:auto;display:flex;gap:1.1rem;font-size:.73rem}
.site-header-links a{color:#484f58;text-decoration:none}
.site-header-links a:hover{color:#8b949e}

/* ── workflow strip ── */
.wf-strip{
  background:#161b22;border-bottom:1px solid #21262d;
  padding:.55rem 1.5rem;display:flex;align-items:center;gap:0;overflow-x:auto;
}
.wf-step{
  display:flex;align-items:center;gap:.4rem;
  font-size:.72rem;color:#484f58;white-space:nowrap;flex-shrink:0;
}
.wf-step.active{color:#8b949e}
.wf-num{
  width:18px;height:18px;border-radius:50%;
  background:#1c2128;border:1px solid #2d333b;
  display:flex;align-items:center;justify-content:center;
  font-size:.62rem;font-weight:700;color:#484f58;flex-shrink:0;
}
.wf-arrow{padding:0 .6rem;color:#2d333b;font-size:.65rem;flex-shrink:0}
.wf-hint{
  margin-left:auto;font-size:.68rem;color:#484f58;
  white-space:nowrap;flex-shrink:0;padding-left:1rem;
}

/* ── page grid ── */
.page{
  max-width:1060px;margin:1.5rem auto;
  padding:0 1.25rem 4rem;
  display:grid;grid-template-columns:1fr 1fr;
  gap:1.25rem;align-items:start;
}
@media(max-width:720px){.page{grid-template-columns:1fr}}

/* ── panel ── */
.panel{
  background:#161b22;border:1px solid #21262d;
  border-radius:8px;overflow:hidden;
}
.panel-head{
  padding:.6rem 1rem;border-bottom:1px solid #21262d;
  display:flex;align-items:center;gap:.5rem;
}
.panel-head-title{
  font-size:.7rem;font-weight:700;letter-spacing:.07em;
  text-transform:uppercase;color:#6e7681;
}
.panel-body{padding:.9rem 1rem 1rem}

/* ── scenario bar ── */
.scn-bar{
  display:flex;gap:.5rem;margin-bottom:.85rem;flex-wrap:wrap;
}
.btn-scn{
  padding:.28rem .7rem;font-family:inherit;font-size:.73rem;font-weight:600;
  background:#1c2128;border:1px solid #2d333b;color:#8b949e;
  border-radius:4px;cursor:pointer;transition:border-color .15s,color .15s;
}
.btn-scn:hover{border-color:#388bfd;color:#58a6ff}
.btn-scn.scn-route{border-color:#238636;color:#3fb950;background:#1a3e2c}
.btn-scn.scn-refuse{border-color:#da3633;color:#f85149;background:#3d1f1f}
.scn-label{
  font-size:.65rem;color:#484f58;align-self:center;margin-right:.1rem;
  white-space:nowrap;
}

/* ── form rows ── */
.form-row{margin-bottom:.7rem}
.form-row label{
  display:block;font-size:.68rem;font-weight:700;
  letter-spacing:.06em;text-transform:uppercase;color:#6e7681;margin-bottom:.28rem;
}
.form-row label .lbl-note{
  font-weight:400;letter-spacing:0;text-transform:none;
  color:#484f58;font-size:.65rem;
}
.form-row-pair{display:grid;grid-template-columns:1fr 1fr;gap:.55rem}

/* ── inputs ── */
.sel-wrap{position:relative}
select,input[type=text]{
  width:100%;background:#0d1117;border:1px solid #21262d;
  border-radius:5px;color:#c9d1d9;font-size:.84rem;
  padding:.42rem 2.2rem .42rem .65rem;font-family:inherit;
  cursor:pointer;appearance:none;-webkit-appearance:none;transition:border-color .15s;
}
input[type=text]{
  padding:.42rem .65rem;cursor:text;
  font-family:ui-monospace,"Menlo",monospace;
  font-size:.78rem;color:#8b949e;
}
select:focus,input[type=text]:focus{
  outline:none;border-color:#388bfd;
  box-shadow:0 0 0 3px rgba(56,139,253,.12);
}
.sel-wrap::after{
  content:"";pointer-events:none;
  position:absolute;right:.65rem;top:50%;transform:translateY(-50%);
  border:3.5px solid transparent;
  border-top-color:#6e7681;border-bottom:none;margin-top:2px;
}
.input-disabled{opacity:.45;cursor:not-allowed}

/* ── file drop placeholder ── */
.file-area{
  border:1px dashed #2d333b;border-radius:5px;
  padding:.7rem .9rem;text-align:center;
  font-size:.75rem;color:#484f58;cursor:not-allowed;
  margin-top:.65rem;line-height:1.4;
}
.file-area span{display:block;font-size:.65rem;color:#30363d;margin-top:.2rem}

/* ── JSON preview ── */
.preview-label{
  font-size:.67rem;font-weight:700;letter-spacing:.06em;
  text-transform:uppercase;color:#484f58;margin:1rem 0 .3rem;
}
pre.preview-json{
  background:#0d1117;border:1px solid #1c2128;border-radius:5px;
  padding:.55rem .7rem;
  font-family:ui-monospace,"Cascadia Mono","Menlo",monospace;
  font-size:.69rem;color:#6e7681;
  overflow-x:auto;white-space:pre-wrap;word-break:break-all;
  max-height:180px;overflow-y:auto;line-height:1.5;
}

/* ── CTA button ── */
.btn-route{
  width:100%;margin-top:1rem;padding:.72rem 1rem;
  font-family:inherit;font-size:.92rem;font-weight:700;
  background:#238636;border:1px solid #2ea043;color:#fff;
  border-radius:6px;cursor:pointer;
  display:flex;align-items:center;justify-content:center;gap:.5rem;
  transition:background .15s,transform .1s;letter-spacing:-.01em;
}
.btn-route:hover:not(:disabled){background:#2ea043;transform:translateY(-1px)}
.btn-route:active:not(:disabled){transform:translateY(0)}
.btn-route:disabled{opacity:.5;cursor:default;transform:none}

/* ── result right panel ── */
.result-empty{
  padding:2.5rem 1rem;text-align:center;
  color:#30363d;font-size:.83rem;line-height:1.6;
}
.result-empty-icon{font-size:1.4rem;margin-bottom:.5rem;display:block}
.res-outcome{padding:1rem 1rem .85rem;border-bottom:1px solid #1c2128}
.outcome-row{display:flex;align-items:center;gap:.65rem;margin-bottom:.9rem}

.badge{
  display:inline-block;padding:.15rem .55rem;
  border-radius:3px;font-size:.66rem;font-weight:800;letter-spacing:.07em;flex-shrink:0;
}
.b-routed  {background:#1a3e2c;color:#3fb950}
.b-refused {background:#3d1f1f;color:#f85149}
.b-verified{background:#1e2d45;color:#58a6ff}
.b-failed  {background:#3d1f1f;color:#f85149}
.b-pending {background:#1c2128;color:#6e7681}
.b-err     {background:#3d1f1f;color:#f85149}
.b-eligible{background:#1c2128;color:#6e7681}
.b-selected{background:#1a3e2c;color:#3fb950}
.b-noelig  {background:#2d1f1f;color:#8b6166}

.outcome-headline{font-size:.98rem;font-weight:700;color:#f0f6fc}

.res-fields{display:flex;flex-direction:column;gap:.7rem}
.rf{display:flex;flex-direction:column;gap:.1rem}
.rf .rk{
  font-size:.64rem;font-weight:700;letter-spacing:.07em;
  text-transform:uppercase;color:#484f58;
}
.rf .rv{font-size:.85rem;color:#c9d1d9}
.rf .rv-sub{font-size:.78rem;color:#6e7681}

/* ── routing path ── */
.routing-path{
  display:flex;align-items:center;gap:.45rem;
  flex-wrap:wrap;margin-bottom:.2rem;
}
.rp-node{
  font-size:.8rem;font-weight:600;color:#c9d1d9;
  background:#1c2128;border:1px solid #2d333b;
  border-radius:4px;padding:.1rem .45rem;
}
.rp-arrow{color:#484f58;font-size:.75rem}
.rp-status{font-size:.7rem;font-weight:700;letter-spacing:.05em;
           padding:.1rem .45rem;border-radius:3px}
.rp-allowed{background:#1a3e2c;color:#3fb950}
.rp-blocked{background:#3d1f1f;color:#f85149}

/* ── full-width sections ── */
.full-section{
  grid-column:1/-1;
  background:#161b22;border:1px solid #21262d;
  border-radius:8px;overflow:hidden;
}
.full-section-head{
  padding:.6rem 1rem;border-bottom:1px solid #21262d;
  display:flex;align-items:baseline;gap:.75rem;
}
.full-section-title{
  font-size:.7rem;font-weight:700;letter-spacing:.07em;
  text-transform:uppercase;color:#6e7681;
}
.full-section-sub{font-size:.72rem;color:#484f58}
.full-section-body{padding:.85rem 1rem 1rem}
.section-empty{font-size:.8rem;color:#484f58;padding:.25rem 0}

/* ── routing graph ── */
.graph-candidates{
  display:grid;grid-template-columns:repeat(3,1fr);gap:.85rem;
}
@media(max-width:720px){.graph-candidates{grid-template-columns:1fr}}

.gc{
  background:#0d1117;border:1px solid #21262d;
  border-radius:6px;overflow:hidden;
}
.gc.gc-selected{border-color:#238636}
.gc.gc-eligible{border-color:#2d333b}
.gc.gc-failed  {border-color:#2d1f1f}

.gc-head{
  padding:.5rem .75rem;
  border-bottom:1px solid #1c2128;
  display:flex;align-items:center;gap:.5rem;
}
.gc-name{font-size:.8rem;font-weight:600;color:#c9d1d9;flex:1}
.gc.gc-failed .gc-name{color:#6e7681}

.gc-checks{padding:.55rem .75rem;display:flex;flex-direction:column;gap:.3rem}
.cr{display:flex;align-items:baseline;gap:.45rem;font-size:.75rem}
.cr-icon{font-size:.7rem;flex-shrink:0;width:12px;text-align:center}
.cr-pass .cr-icon{color:#3fb950}
.cr-fail .cr-icon{color:#f85149}
.cr-pass .cr-label{color:#6e7681}
.cr-fail .cr-label{color:#8b6166}

/* ── determinism check ── */
.det-note{
  font-size:.8rem;color:#6e7681;
  background:#1c2128;border:1px solid #21262d;
  border-radius:5px;padding:.55rem .75rem;
  margin-bottom:.85rem;
}
.det-hashes{
  display:grid;grid-template-columns:160px 1fr;
  gap:.35rem .85rem;
}
.det-k{
  font-size:.67rem;font-weight:700;letter-spacing:.06em;
  text-transform:uppercase;color:#484f58;padding-top:.1rem;
}
.det-v{
  font-family:ui-monospace,"Cascadia Mono","Menlo",monospace;
  font-size:.71rem;color:#6e7681;word-break:break-all;
}

/* ── proof ── */
.proof-kv{
  display:grid;grid-template-columns:140px 1fr;
  gap:.3rem .75rem;margin-bottom:.65rem;
}
.proof-kv .pk{
  font-size:.67rem;font-weight:700;letter-spacing:.06em;
  text-transform:uppercase;color:#484f58;padding-top:.1rem;
}
.proof-kv .pv{
  font-family:ui-monospace,"Cascadia Mono","Menlo",monospace;
  font-size:.72rem;color:#6e7681;word-break:break-all;
}

/* ── disclosure ── */
details{margin-bottom:.6rem}
details:last-child{margin-bottom:0}
details summary{
  font-size:.75rem;color:#484f58;cursor:pointer;
  user-select:none;list-style:none;padding:.3rem 0;
  display:inline-flex;align-items:center;gap:.35rem;
}
details summary::marker,details summary::-webkit-details-marker{display:none}
.cv{font-size:.55rem;transition:transform .15s;display:inline-block;flex-shrink:0}
details[open] summary .cv{transform:rotate(90deg)}
details summary:hover{color:#8b949e}
pre.proof-json{
  margin-top:.4rem;background:#0d1117;border:1px solid #1c2128;
  border-radius:5px;padding:.55rem .75rem;
  font-family:ui-monospace,"Cascadia Mono","Menlo",monospace;
  font-size:.69rem;color:#6e7681;
  overflow-x:auto;white-space:pre-wrap;word-break:break-all;
  max-height:260px;overflow-y:auto;line-height:1.5;
}

/* ── spinner ── */
.spin{
  display:inline-block;width:13px;height:13px;
  border:2px solid rgba(255,255,255,.2);border-top-color:#fff;
  border-radius:50%;animation:spin .65s linear infinite;flex-shrink:0;
}
@keyframes spin{to{transform:rotate(360deg)}}
</style>
</head>
<body>

<!-- ── Header ── -->
<header class="site-header">
  <div class="site-header-hex">⬡</div>
  <div>
    <span class="site-header-title">PostCAD Pilot Demo</span>
    <span class="site-header-sub">— Deterministic routing · verifiable execution records · dental manufacturing</span>
  </div>
  <nav class="site-header-links">
    <a href="/">Operator UI</a>
    <a href="/protocol-manifest">Manifest</a>
  </nav>
</header>

<!-- ── Workflow strip ── -->
<div class="wf-strip">
  <div class="wf-step active"><div class="wf-num">1</div> Case Intake</div>
  <div class="wf-arrow">→</div>
  <div class="wf-step active"><div class="wf-num">2</div> Constraint Evaluation</div>
  <div class="wf-arrow">→</div>
  <div class="wf-step active"><div class="wf-num">3</div> Deterministic Routing</div>
  <div class="wf-arrow">→</div>
  <div class="wf-step active"><div class="wf-num">4</div> Verifiable Proof</div>
  <span class="wf-hint">Structured intake form over the existing deterministic routing kernel.</span>
</div>

<!-- ── Main grid ── -->
<div class="page">

  <!-- ── LEFT: Case Intake ── -->
  <div class="panel" id="left-panel">
    <div class="panel-head">
      <div class="panel-head-title">Case Intake</div>
    </div>
    <div class="panel-body">

      <!-- Scenario loader -->
      <div class="scn-bar">
        <span class="scn-label">Load scenario:</span>
        <button class="btn-scn scn-route"  onclick="loadScenario('routed')">Routed example</button>
        <button class="btn-scn scn-refuse" onclick="loadScenario('refusal')">Refusal example</button>
      </div>

      <!-- Case ID -->
      <div class="form-row">
        <label for="f-case-id">Case ID <span class="lbl-note">(UUID)</span></label>
        <input type="text" id="f-case-id"
          value="f1000001-0000-0000-0000-000000000001"
          oninput="updatePreview()">
      </div>

      <!-- Jurisdiction + Routing policy -->
      <div class="form-row form-row-pair">
        <div>
          <label for="f-jurisdiction">Jurisdiction</label>
          <div class="sel-wrap">
            <select id="f-jurisdiction" onchange="onJurChange(); updatePreview()">
              <option value="DE">DE — Germany</option>
              <option value="US">US — United States</option>
              <option value="JP">JP — Japan</option>
            </select>
          </div>
        </div>
        <div>
          <label for="f-policy">Routing policy</label>
          <div class="sel-wrap">
            <select id="f-policy" onchange="updatePreview()">
              <option value="allow_domestic_and_cross_border">Domestic + cross-border</option>
            </select>
          </div>
        </div>
      </div>

      <!-- Patient country + Manufacturer country -->
      <div class="form-row form-row-pair">
        <div>
          <label for="f-patient-country">Patient country <span class="lbl-note">(auto)</span></label>
          <div class="sel-wrap">
            <select id="f-patient-country" onchange="updatePreview()">
              <option value="germany">Germany</option>
              <option value="united_states">United States</option>
              <option value="japan">Japan</option>
            </select>
          </div>
        </div>
        <div>
          <label for="f-mfr-country">Manufacturer country</label>
          <div class="sel-wrap">
            <select id="f-mfr-country" onchange="updatePreview()">
              <option value="germany">Germany</option>
              <option value="united_states">United States</option>
              <option value="japan">Japan</option>
            </select>
          </div>
        </div>
      </div>

      <!-- Material + Procedure -->
      <div class="form-row form-row-pair">
        <div>
          <label for="f-material">Material</label>
          <div class="sel-wrap">
            <select id="f-material" onchange="updatePreview()">
              <option value="zirconia">Zirconia</option>
              <option value="pmma">PMMA</option>
              <option value="emax">E-Max</option>
              <option value="titanium">Titanium</option>
            </select>
          </div>
        </div>
        <div>
          <label for="f-procedure">Procedure</label>
          <div class="sel-wrap">
            <select id="f-procedure" onchange="updatePreview()">
              <option value="crown">Crown</option>
              <option value="bridge">Bridge</option>
              <option value="veneer">Veneer</option>
              <option value="implant">Implant</option>
            </select>
          </div>
        </div>
      </div>

      <!-- File type -->
      <div class="form-row">
        <label for="f-file-type">File type</label>
        <div class="sel-wrap">
          <select id="f-file-type" onchange="updatePreview()">
            <option value="stl">STL</option>
            <option value="obj">OBJ</option>
            <option value="3mf">3MF</option>
          </select>
        </div>
      </div>

      <!-- Design file placeholder -->
      <div class="file-area input-disabled" aria-disabled="true">
        Design file — demo placeholder (not processed)
        <span>Upload not wired in this demo. File type field above is used for routing.</span>
      </div>

      <!-- Live JSON preview -->
      <div class="preview-label">Structured case preview</div>
      <pre class="preview-json" id="case-preview"></pre>

      <!-- Route Case CTA -->
      <button class="btn-route" id="btn-route" onclick="runDemo(this)">
        Route Case
      </button>

    </div>
  </div>

  <!-- ── RIGHT: Routing Decision ── -->
  <div class="panel" id="right-panel">
    <div class="panel-head">
      <div class="panel-head-title">Routing Decision</div>
    </div>
    <div class="panel-body">

      <div class="result-empty" id="result-empty">
        <span class="result-empty-icon">⬡</span>
        Fill in the case form and click<br><strong>Route Case</strong> to see the decision.
      </div>

      <div id="result-card" style="display:none">

        <div class="res-outcome">
          <div class="outcome-row">
            <span id="r-badge"></span>
            <span class="outcome-headline" id="r-headline"></span>
          </div>
          <div class="res-fields">

            <div class="rf" id="r-mfr-row">
              <div class="rk">Manufacturer</div>
              <div class="rv" id="r-mfr-name"></div>
            </div>

            <div class="rf" id="r-refusal-row" style="display:none">
              <div class="rk">Refusal code</div>
              <div class="rv" id="r-refusal-code"></div>
            </div>

            <!-- Routing Path -->
            <div class="rf">
              <div class="rk">Routing Path</div>
              <div class="rv routing-path" id="r-path">
                <span class="rp-node" id="rp-origin"></span>
                <span class="rp-arrow" id="rp-arrow">→</span>
                <span class="rp-node" id="rp-dest"></span>
                <span class="rp-status" id="rp-status"></span>
              </div>
              <div class="rv-sub" id="rp-policy-line"></div>
              <div class="rv-sub" id="rp-refusal-line" style="display:none"></div>
            </div>

            <div class="rf">
              <div class="rk">Explanation</div>
              <div class="rv-sub" id="r-explanation"></div>
            </div>

            <div class="rf">
              <div class="rk">Policy</div>
              <div class="rv-sub" id="r-policy"></div>
            </div>

            <div class="rf">
              <div class="rk">Receipt hash</div>
              <div class="rv" id="r-hash"
                style="font-family:ui-monospace,monospace;font-size:.72rem;
                       word-break:break-all;color:#6e7681"></div>
            </div>

            <div class="rf" id="r-pfp-row" style="display:none">
              <div class="rk">Policy fingerprint</div>
              <div class="rv" id="r-pfp"
                style="font-family:ui-monospace,monospace;font-size:.69rem;
                       word-break:break-all;color:#6e7681"></div>
            </div>

            <div class="rf" id="r-cph-row" style="display:none">
              <div class="rk">Candidate pool hash</div>
              <div class="rv" id="r-cph"
                style="font-family:ui-monospace,monospace;font-size:.69rem;
                       word-break:break-all;color:#6e7681"></div>
            </div>

            <div class="rf">
              <div class="rk">Verification</div>
              <div class="rv" id="r-verify"></div>
            </div>

            <div class="rf">
              <div class="rk">Dispatch</div>
              <div class="rv-sub">
                Not triggered in this demo.
                <a href="/" style="color:#484f58;font-size:.78rem">Use operator UI →</a>
              </div>
            </div>

          </div>
        </div>

      </div>

    </div>
  </div>

  <!-- ── CONSTRAINT EVALUATION: full-width ── -->
  <div class="full-section" id="graph-section" style="display:none">
    <div class="full-section-head">
      <div class="full-section-title">Constraint Evaluation</div>
      <div class="full-section-sub">
        Each candidate in the registry is evaluated against the case constraints in order.
      </div>
    </div>
    <div class="full-section-body">
      <div class="graph-candidates" id="graph-candidates"></div>
    </div>
  </div>

  <!-- ── DETERMINISM CHECK: full-width ── -->
  <div class="full-section" id="det-section" style="display:none">
    <div class="full-section-head">
      <div class="full-section-title">Determinism Check</div>
    </div>
    <div class="full-section-body">
      <div class="det-note">
        Re-running this case with identical inputs produces the same routing decision.
        The hashes below commit every input and constraint that influenced the outcome.
      </div>
      <div class="det-hashes" id="det-hashes"></div>
    </div>
  </div>

  <!-- ── PROOF & INSPECTION: full-width ── -->
  <div class="full-section" id="proof-section">
    <div class="full-section-head">
      <div class="full-section-title">Proof &amp; Inspection</div>
    </div>
    <div class="full-section-body">

      <div class="section-empty" id="proof-empty">
        Proof artifacts will appear here after routing.
      </div>

      <div id="proof-content" style="display:none">

        <div class="proof-kv" id="proof-versions"></div>

        <details>
          <summary><span class="cv">▶</span> Receipt JSON</summary>
          <pre class="proof-json" id="proof-receipt-json"></pre>
        </details>

        <details>
          <summary><span class="cv">▶</span> Routing input (case sent to kernel)</summary>
          <pre class="proof-json" id="proof-input-json"></pre>
        </details>

        <details id="proof-refusal-details" style="display:none">
          <summary><span class="cv">▶</span> Refusal detail</summary>
          <pre class="proof-json" id="proof-refusal-json"></pre>
        </details>

      </div>
    </div>
  </div>

</div><!-- /.page -->

<script>
// ── Registry (examples/pilot/registry_snapshot.json) ─────────────────────────

const REGISTRY = [
  {manufacturer_id:"pilot-de-001",display_name:"Alpha Dental GmbH",
   country:"germany",is_active:true,
   capabilities:["crown","bridge"],
   materials_supported:["zirconia","pmma"],
   jurisdictions_served:["germany"],
   attestation_statuses:["verified"],sla_days:5},
  {manufacturer_id:"pilot-de-002",display_name:"Beta Zahntechnik GmbH",
   country:"germany",is_active:true,
   capabilities:["crown","veneer"],
   materials_supported:["zirconia","emax"],
   jurisdictions_served:["germany"],
   attestation_statuses:["verified"],sla_days:3},
  {manufacturer_id:"pilot-de-003",display_name:"Gamma Dental GmbH",
   country:"germany",is_active:true,
   capabilities:["crown","implant"],
   materials_supported:["zirconia","titanium"],
   jurisdictions_served:["germany"],
   attestation_statuses:["verified"],sla_days:7},
];

const MFR = Object.fromEntries(REGISTRY.map(m => [m.manufacturer_id, m]));

const JUR_TO_COUNTRY = {DE:"germany", US:"united_states", JP:"japan"};
const COUNTRY_LABEL  = {germany:"Germany", united_states:"United States", japan:"Japan"};

// ── Scenario presets ──────────────────────────────────────────────────────────

const SCENARIOS = {
  // Default pilot fixture — routes to Alpha Dental GmbH
  routed: {
    case_id:              "f1000001-0000-0000-0000-000000000001",
    jurisdiction:         "DE",
    routing_policy:       "allow_domestic_and_cross_border",
    patient_country:      "germany",
    manufacturer_country: "germany",
    material:             "zirconia",
    procedure:            "crown",
    file_type:            "stl",
  },
  // US jurisdiction — all three manufacturers serve only Germany → no_jurisdiction_match
  refusal: {
    case_id:              "f2000002-0000-0000-0000-000000000002",
    jurisdiction:         "US",
    routing_policy:       "allow_domestic_and_cross_border",
    patient_country:      "united_states",
    manufacturer_country: "germany",
    material:             "zirconia",
    procedure:            "crown",
    file_type:            "stl",
  },
};

function loadScenario(key) {
  const s = SCENARIOS[key];
  sval('f-case-id',         s.case_id);
  sval('f-jurisdiction',    s.jurisdiction);
  sval('f-policy',          s.routing_policy);
  sval('f-patient-country', s.patient_country);
  sval('f-mfr-country',     s.manufacturer_country);
  sval('f-material',        s.material);
  sval('f-procedure',       s.procedure);
  sval('f-file-type',       s.file_type);
  updatePreview();
  hideResult();
}

// ── Manifest (best-effort) ────────────────────────────────────────────────────

let manifest = null;
fetch('/protocol-manifest')
  .then(r => r.json())
  .then(d => { manifest = d; })
  .catch(() => {});

// ── Form helpers ──────────────────────────────────────────────────────────────

function val(id)     { return document.getElementById(id).value; }
function sval(id, v) { document.getElementById(id).value = v; }

function buildCaseObj() {
  return {
    case_id:              val('f-case-id').trim() || "f1000001-0000-0000-0000-000000000001",
    jurisdiction:         val('f-jurisdiction'),
    routing_policy:       val('f-policy'),
    patient_country:      val('f-patient-country'),
    manufacturer_country: val('f-mfr-country'),
    material:             val('f-material'),
    procedure:            val('f-procedure'),
    file_type:            val('f-file-type'),
  };
}

function onJurChange() {
  sval('f-patient-country', JUR_TO_COUNTRY[val('f-jurisdiction')] || 'germany');
}

function updatePreview() {
  document.getElementById('case-preview').textContent =
    JSON.stringify(buildCaseObj(), null, 2);
}

// ── Main: route + auto-verify ─────────────────────────────────────────────────

async function runDemo(btn) {
  const caseObj = buildCaseObj();
  const config  = {jurisdiction: caseObj.jurisdiction, routing_policy: caseObj.routing_policy};

  setBtn(btn, true, '<span class="spin"></span> Routing\u2026');
  hideResult();

  try {
    const rr = await post('/route', {
      case: caseObj,
      registry_snapshot: REGISTRY,
      routing_config: config,
    });

    if (!rr.ok || !rr.json.receipt) {
      const e = rr.json?.error;
      showError(e ? `${e.code}: ${e.message}` : JSON.stringify(rr.json));
      return;
    }

    const receipt = rr.json.receipt;
    const policy  = rr.json.derived_policy;

    renderResult(receipt, null, caseObj);
    renderGraph(receipt, caseObj);           // ← constraint evaluation
    renderDeterminism(receipt);              // ← hash commitments
    setBtn(btn, true, '<span class="spin"></span> Verifying\u2026');

    const vr = await post('/verify', {receipt, case: caseObj, policy});
    const ok = vr.ok && vr.json?.result === 'VERIFIED';

    renderResult(receipt, ok, caseObj);
    renderProof(receipt, caseObj);

  } catch(e) {
    showError('Network error: ' + e.message);
  } finally {
    setBtn(btn, false, 'Route Case');
  }
}

// ── Constraint evaluation (client-side, no backend call) ─────────────────────

// Mirror the kernel's stepwise filter order.
function evaluateCandidate(m, caseObj) {
  const jurCountry = JUR_TO_COUNTRY[caseObj.jurisdiction] || caseObj.jurisdiction.toLowerCase();
  return [
    {
      label: 'manufacturer active',
      pass:  m.is_active === true,
    },
    {
      label: `jurisdiction (${caseObj.jurisdiction}) served`,
      pass:  m.jurisdictions_served.includes(jurCountry)
          || m.jurisdictions_served.includes(caseObj.jurisdiction),
    },
    {
      label: `procedure (${caseObj.procedure}) supported`,
      pass:  m.capabilities.includes(caseObj.procedure),
    },
    {
      label: `material (${caseObj.material}) supported`,
      pass:  m.materials_supported.includes(caseObj.material),
    },
    {
      label: 'attestation valid',
      pass:  m.attestation_statuses.includes('verified'),
    },
  ];
}

function renderGraph(receipt, caseObj) {
  const selectedId = receipt.selected_candidate_id;
  let html = '';

  for (const m of REGISTRY) {
    const checks  = evaluateCandidate(m, caseObj);
    const allPass = checks.every(c => c.pass);
    const sel     = m.manufacturer_id === selectedId;

    const gcCls = sel ? 'gc gc-selected' : allPass ? 'gc gc-eligible' : 'gc gc-failed';
    const bCls  = sel ? 'badge b-selected' : allPass ? 'badge b-eligible' : 'badge b-noelig';
    const bTxt  = sel ? 'SELECTED' : allPass ? 'ELIGIBLE' : 'NOT ELIGIBLE';

    html += `<div class="${gcCls}">`;
    html += `<div class="gc-head">`;
    html += `<span class="${bCls}">${bTxt}</span>`;
    html += `<span class="gc-name">${esc(m.display_name)}</span>`;
    html += `</div>`;
    html += `<div class="gc-checks">`;
    for (const c of checks) {
      html += `<div class="cr ${c.pass ? 'cr-pass' : 'cr-fail'}">`;
      html += `<span class="cr-icon">${c.pass ? '✓' : '✗'}</span>`;
      html += `<span class="cr-label">${esc(c.label)}</span>`;
      html += `</div>`;
    }
    html += `</div></div>`;
  }

  document.getElementById('graph-candidates').innerHTML = html;
  document.getElementById('graph-section').style.display = '';
}

// ── Determinism check ─────────────────────────────────────────────────────────

function renderDeterminism(receipt) {
  const fields = [
    {k: 'Case fingerprint',      v: receipt.case_fingerprint},
    {k: 'Policy fingerprint',    v: receipt.policy_fingerprint},
    {k: 'Candidate pool hash',   v: receipt.candidate_pool_hash},
    {k: 'Routing decision hash', v: receipt.routing_decision_hash},
  ].filter(f => f.v);

  let html = '';
  for (const f of fields) {
    html += `<div class="det-k">${esc(f.k)}</div>`;
    html += `<div class="det-v">${esc(f.v)}</div>`;
  }

  document.getElementById('det-hashes').innerHTML = html;
  document.getElementById('det-section').style.display = '';
}

// ── Result rendering ──────────────────────────────────────────────────────────

function hideResult() {
  document.getElementById('result-empty').style.display  = '';
  document.getElementById('result-card').style.display   = 'none';
  document.getElementById('graph-section').style.display = 'none';
  document.getElementById('det-section').style.display   = 'none';
  document.getElementById('proof-empty').style.display   = '';
  document.getElementById('proof-content').style.display = 'none';
}

function renderResult(receipt, verified, caseObj) {
  document.getElementById('result-empty').style.display = 'none';
  document.getElementById('result-card').style.display  = '';

  const routed = receipt.outcome === 'routed';

  set('r-badge',    routed ? badge('ROUTED','b-routed') : badge('REFUSED','b-refused'));
  set('r-headline', routed ? 'Manufacturer selected' : 'No eligible manufacturer');

  const mfrRow     = document.getElementById('r-mfr-row');
  const refusalRow = document.getElementById('r-refusal-row');
  if (routed && receipt.selected_candidate_id) {
    const m = MFR[receipt.selected_candidate_id];
    set('r-mfr-name', esc(m ? m.display_name : receipt.selected_candidate_id));
    mfrRow.style.display     = '';
    refusalRow.style.display = 'none';
  } else {
    mfrRow.style.display = 'none';
    const code = receipt.refusal_code || 'no_eligible_candidates';
    set('r-refusal-code',
      `<code style="font-size:.8rem;color:#f85149">${esc(code)}</code>`);
    refusalRow.style.display = '';
  }

  // Routing Path
  const originLabel = COUNTRY_LABEL[caseObj.patient_country]      || caseObj.patient_country;
  const destLabel   = COUNTRY_LABEL[caseObj.manufacturer_country] || caseObj.manufacturer_country;
  document.getElementById('rp-origin').textContent = originLabel;
  document.getElementById('rp-dest').textContent   = destLabel;
  const rpStatus = document.getElementById('rp-status');
  if (routed) {
    rpStatus.textContent  = 'Allowed';
    rpStatus.className    = 'rp-status rp-allowed';
    document.getElementById('rp-refusal-line').style.display = 'none';
  } else {
    rpStatus.textContent  = 'Blocked';
    rpStatus.className    = 'rp-status rp-blocked';
    const code = receipt.refusal_code || 'no_eligible_candidates';
    document.getElementById('rp-refusal-line').textContent =
      `Refusal code: ${code}`;
    document.getElementById('rp-refusal-line').style.display = '';
  }
  document.getElementById('rp-policy-line').textContent =
    `Policy: ${caseObj.routing_policy || '\u2014'}`;

  set('r-explanation', esc(explanation(receipt, caseObj)));
  set('r-policy',      esc(caseObj.routing_policy || '\u2014'));
  set('r-hash',        esc(receipt.receipt_hash || '\u2014'));

  const pfpRow = document.getElementById('r-pfp-row');
  if (receipt.policy_fingerprint) {
    set('r-pfp', esc(receipt.policy_fingerprint));
    pfpRow.style.display = '';
  } else {
    pfpRow.style.display = 'none';
  }

  const cphRow = document.getElementById('r-cph-row');
  if (receipt.candidate_pool_hash) {
    set('r-cph', esc(receipt.candidate_pool_hash));
    cphRow.style.display = '';
  } else {
    cphRow.style.display = 'none';
  }

  if (verified === null) {
    set('r-verify', '<span style="color:#484f58">Checking\u2026</span>');
  } else {
    set('r-verify', verified
      ? badge('VERIFIED','b-verified')
        + ' <span style="font-size:.78rem;color:#6e7681">Receipt is cryptographically intact.</span>'
      : badge('FAILED','b-failed')
        + ' <span style="font-size:.78rem;color:#6e7681">Verification did not pass.</span>');
  }

  if (window.innerWidth < 720)
    document.getElementById('right-panel').scrollIntoView({behavior:'smooth',block:'nearest'});
}

function renderProof(receipt, caseObj) {
  document.getElementById('proof-empty').style.display   = 'none';
  document.getElementById('proof-content').style.display = '';

  const proto  = manifest?.protocol_version
    || (receipt.schema_version ? 'schema v' + receipt.schema_version : '\u2014');
  const kernel = receipt.routing_kernel_version
    || manifest?.routing_kernel_version || '\u2014';

  document.getElementById('proof-versions').innerHTML =
    `<div class="pk">Protocol</div><div class="pv">${esc(proto)}</div>`
  + `<div class="pk">Routing kernel</div><div class="pv">${esc(kernel)}</div>`;

  document.getElementById('proof-receipt-json').textContent =
    JSON.stringify(receipt, null, 2);
  document.getElementById('proof-input-json').textContent =
    JSON.stringify(buildCaseObj(), null, 2);

  const refDetails = document.getElementById('proof-refusal-details');
  if (receipt.outcome === 'refused') {
    document.getElementById('proof-refusal-json').textContent =
      JSON.stringify({
        outcome:      receipt.outcome,
        refusal_code: receipt.refusal_code,
        receipt_hash: receipt.receipt_hash,
      }, null, 2);
    refDetails.style.display = '';
  } else {
    refDetails.style.display = 'none';
  }
}

function showError(msg) {
  document.getElementById('result-empty').style.display = 'none';
  document.getElementById('result-card').style.display  = '';
  set('r-badge',    badge('ERROR','b-err'));
  set('r-headline', 'Request failed');
  document.getElementById('r-mfr-row').style.display     = 'none';
  document.getElementById('r-refusal-row').style.display = 'none';
  set('r-explanation', esc(msg));
  set('r-hash',   '\u2014');
  set('r-verify', '\u2014');
}

// ── Plain-language explanations ───────────────────────────────────────────────

function explanation(receipt, caseObj) {
  if (receipt.outcome === 'routed') {
    const m = MFR[receipt.selected_candidate_id];
    return m
      ? `${m.display_name} is active in `
        + `${COUNTRY_LABEL[caseObj.patient_country] || caseObj.patient_country}, `
        + `supports ${caseObj.procedure} in ${caseObj.material}, `
        + `and holds current attestations. `
        + `Selected by deterministic eligibility and policy checks.`
      : 'Selected by deterministic eligibility and policy checks.';
  }
  const code = receipt.refusal_code || '';
  if (code === 'no_jurisdiction_match')
    return `No manufacturer in this registry serves `
         + `${COUNTRY_LABEL[caseObj.patient_country] || caseObj.patient_country}. `
         + 'No eligible route under current policy and constraints.';
  if (code === 'no_capability_match')
    return `No manufacturer in this registry performs ${caseObj.procedure} procedures. `
         + 'No eligible route under current policy and constraints.';
  if (code === 'no_material_match')
    return `No manufacturer in this registry works with ${caseObj.material}. `
         + 'No eligible route under current policy and constraints.';
  if (code === 'attestation_failed')
    return 'No manufacturer holds current attestations for this jurisdiction.';
  if (code === 'no_active_manufacturer')
    return 'No active manufacturer is available in this registry.';
  return `No eligible route under current policy and constraints `
       + `(${code || 'no candidates'}).`;
}

// ── Utilities ─────────────────────────────────────────────────────────────────

async function post(path, body) {
  const r = await fetch(path, {
    method:'POST',
    headers:{'Content-Type':'application/json'},
    body:JSON.stringify(body),
  });
  let json;
  try { json = await r.json(); } catch { json = {}; }
  return {ok: r.ok, json};
}

function set(id, html) { document.getElementById(id).innerHTML = html; }

function badge(text, cls) {
  return `<span class="badge ${cls}">${esc(text)}</span>`;
}

function setBtn(btn, disabled, html) {
  btn.disabled  = disabled;
  btn.innerHTML = html;
}

function esc(s) {
  return String(s)
    .replace(/&/g,'&amp;').replace(/</g,'&lt;')
    .replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}

// ── Bootstrap ─────────────────────────────────────────────────────────────────

updatePreview();
</script>
</body>
</html>"#;
