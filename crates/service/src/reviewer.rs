//! Reviewer shell — interactive guided demo, engine-connected.
//!
//! Served at `GET /reviewer`. Auto-loads pilot fixtures from `examples/pilot/`
//! via `GET /pilot-fixtures`. Calls real endpoints only:
//!   POST /pilot/route-normalized  — routing kernel execution
//!   POST /verify                  — deterministic receipt verification
//!   POST /dispatch/create         — dispatch commitment
//!
//! No mock data. No fake outputs. No mocked decisions.

pub const REVIEWER_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PostCAD — Operator Demo</title>
<style>
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}

:root{
  --bg:#0e1117;
  --surface:#151c28;
  --surface2:#1a2235;
  --surface3:#1f2840;
  --border:rgba(255,255,255,0.08);
  --border-md:rgba(255,255,255,0.14);
  --border-strong:rgba(255,255,255,0.22);
  --text-1:#eaf0f9;
  --text-2:#7a8fa8;
  --text-3:#3a4d63;
  --green:#2fcf7a;
  --green-bg:rgba(47,207,122,0.09);
  --green-border:rgba(47,207,122,0.25);
  --amber:#e8a020;
  --amber-bg:rgba(232,160,32,0.09);
  --amber-border:rgba(232,160,32,0.25);
  --red:#e05555;
  --blue:#5b8cfc;
  --blue-bg:rgba(91,140,252,0.09);
  --blue-border:rgba(91,140,252,0.25);
  --mono:'ui-monospace','Cascadia Code','Menlo',monospace;
}

body{
  font-family:-apple-system,BlinkMacSystemFont,'Inter','Segoe UI',sans-serif;
  background:var(--bg);color:var(--text-1);min-height:100vh;
  font-size:15px;line-height:1.6;-webkit-font-smoothing:antialiased;
}

/* ── header ── */
.demo-header{
  border-bottom:1px solid var(--border);
  padding:.75rem 1.5rem;
  display:flex;align-items:center;justify-content:space-between;
  background:rgba(14,17,23,0.94);
  backdrop-filter:blur(14px);
  position:sticky;top:0;z-index:10;
}
.demo-header-logo{
  font-size:.9rem;font-weight:700;color:var(--text-1);
  text-decoration:none;letter-spacing:-.01em;
}
.demo-header-right{
  display:flex;align-items:center;gap:.75rem;
  font-size:.72rem;color:var(--text-3);
}
.demo-header-dot{
  width:5px;height:5px;border-radius:50%;
  background:var(--amber);flex-shrink:0;
}
.demo-header-dot.ok{background:var(--green)}
.demo-header-dot.err{background:var(--red)}
.demo-header-version{
  font-family:var(--mono);font-size:.65rem;color:var(--text-3);
}
.demo-header-back{
  font-size:.75rem;color:var(--text-3);text-decoration:none;
  transition:color .12s;
}
.demo-header-back:hover{color:var(--text-2)}

/* ── demo wrap ── */
.demo-wrap{
  max-width:520px;
  margin:0 auto;
  padding:3rem 1.5rem 5rem;
}

/* ── top section ── */
.demo-top{
  text-align:center;
  margin-bottom:3rem;
}
.demo-top-title{
  font-size:1.7rem;font-weight:700;
  letter-spacing:-.03em;color:var(--text-1);
  margin-bottom:.5rem;
}
.demo-top-sub{
  font-size:.95rem;color:var(--text-2);
  line-height:1.6;margin-bottom:.75rem;
}
.demo-top-hint{
  font-size:.78rem;color:var(--text-3);
}

/* ── case selection ── */
.case-select{margin-bottom:1.5rem}
.case-select-label{
  font-size:.65rem;font-weight:700;color:var(--text-3);
  text-transform:uppercase;letter-spacing:.1em;
  margin-bottom:.85rem;text-align:center;
}
.case-btn{
  display:flex;align-items:flex-start;justify-content:space-between;
  width:100%;background:var(--surface);
  border:1.5px solid var(--border);border-radius:10px;
  padding:1rem 1.25rem;cursor:pointer;
  font-family:inherit;text-align:left;
  transition:border-color .15s,background .15s;
  margin-bottom:.6rem;
  gap:1rem;
}
.case-btn:last-child{margin-bottom:0}
.case-btn:hover{border-color:var(--border-md);background:var(--surface2)}
.case-btn.selected{
  border-color:var(--blue);
  background:var(--blue-bg);
}
.case-btn-left{}
.case-btn-name{
  display:block;font-size:.92rem;font-weight:600;
  color:var(--text-1);margin-bottom:.18rem;
}
.case-btn-detail{
  display:block;font-size:.78rem;color:var(--text-2);
  font-family:var(--mono);
}
.case-btn-right{}
.case-btn-expect{
  font-size:.7rem;font-family:var(--mono);
  white-space:nowrap;padding-top:.15rem;
}
.expect-ok{color:var(--green)}
.expect-warn{color:var(--amber)}

/* ── run button ── */
.run-btn{
  display:block;width:100%;
  font-family:inherit;font-size:1rem;font-weight:700;
  background:var(--text-1);color:#0d1018;
  border:none;border-radius:10px;
  padding:1rem;cursor:pointer;
  transition:opacity .12s;
  margin-bottom:2.5rem;
}
.run-btn:hover:not(:disabled){opacity:.88}
.run-btn:disabled{opacity:.3;cursor:default}
.run-btn.running{opacity:.6;cursor:default}

/* ── flow blocks ── */
.flow-blocks{display:flex;flex-direction:column;gap:1rem}

.flow-block{
  background:var(--surface);
  border:1.5px solid var(--border);
  border-radius:12px;
  padding:1.5rem 1.75rem;
  transition:border-color .3s,opacity .3s,background .3s;
}

/* states */
.flow-block.locked{
  opacity:.3;pointer-events:none;
}
.flow-block.active{
  border-color:var(--blue);
  background:var(--surface2);
  animation:pulse-blue .6s ease-in-out;
}
.flow-block.done-ok{
  border-color:var(--green-border);
  background:var(--surface);
  opacity:1;pointer-events:auto;
}
.flow-block.done-err{
  border-color:var(--amber-border);
  background:var(--surface);
  opacity:1;pointer-events:auto;
}

@keyframes pulse-blue{
  0%{box-shadow:0 0 0 0 rgba(91,140,252,0)}
  40%{box-shadow:0 0 0 6px rgba(91,140,252,0.12)}
  100%{box-shadow:0 0 0 0 rgba(91,140,252,0)}
}

.block-step{
  font-size:.62rem;font-weight:700;color:var(--text-3);
  text-transform:uppercase;letter-spacing:.1em;
  font-family:var(--mono);margin-bottom:.75rem;
}
.block-status{
  font-size:1.1rem;font-weight:700;
  letter-spacing:-.015em;
  min-height:1.65rem;
  margin-bottom:.4rem;
}
.block-detail{
  font-size:.82rem;color:var(--text-2);
  line-height:1.5;min-height:1.2rem;
}

/* status colours */
.status-ok{color:var(--green)}
.status-err{color:var(--amber)}
.status-pending{color:var(--blue)}
.status-neutral{color:var(--text-2)}

/* ── tech details ── */
.tech-details{margin-top:1rem}
.tech-details summary{
  font-size:.72rem;font-weight:600;color:var(--text-3);
  cursor:pointer;user-select:none;list-style:none;
  display:inline-flex;align-items:center;gap:.3rem;
  padding:.25rem 0;letter-spacing:.03em;transition:color .12s;
}
.tech-details summary::-webkit-details-marker{display:none}
.tech-details summary::before{
  content:'▶';font-size:.45rem;color:var(--text-3);
  transition:transform .12s;
}
.tech-details[open] summary::before{transform:rotate(90deg)}
.tech-details summary:hover{color:var(--text-2)}
.tech-pre{
  background:var(--surface3);border:1px solid var(--border);
  border-radius:6px;padding:.65rem .85rem;
  font-family:var(--mono);font-size:.65rem;color:var(--text-2);
  white-space:pre-wrap;word-break:break-all;line-height:1.5;
  overflow-x:auto;margin-top:.4rem;
  max-height:260px;overflow-y:auto;
}

/* ── reset button ── */
.reset-btn{
  display:block;width:100%;
  font-family:inherit;font-size:.88rem;font-weight:600;
  background:transparent;color:var(--text-2);
  border:1.5px solid var(--border-md);border-radius:10px;
  padding:.85rem;cursor:pointer;margin-top:1.5rem;
  transition:border-color .12s,color .12s;
}
.reset-btn:hover{border-color:var(--border-strong);color:var(--text-1)}

/* ── loading / error states ── */
.load-note{
  font-size:.82rem;color:var(--text-3);text-align:center;
  padding:1rem 0;
}
.error-banner{
  background:var(--amber-bg);border:1px solid var(--amber-border);
  border-radius:8px;padding:.9rem 1.1rem;margin-bottom:1.5rem;
  font-size:.82rem;color:var(--text-1);line-height:1.5;
}
.error-banner strong{color:var(--amber);display:block;margin-bottom:.3rem}

/* ── hidden ── */
.hidden{display:none!important}

/* ── print ── */
@media print{
  .demo-header{display:none}
  body{background:#fff;color:#000}
  .flow-block{border:1px solid #ccc;page-break-inside:avoid}
}
</style>
</head>
<body>

<!-- ── header ── -->
<div class="demo-header">
  <a class="demo-header-logo" href="/">PostCAD</a>
  <div class="demo-header-right">
    <span class="demo-header-dot" id="status-dot"></span>
    <span class="demo-header-version" id="ver">connecting…</span>
    <a class="demo-header-back" href="/">← Back</a>
  </div>
</div>

<!-- ── demo wrap ── -->
<div class="demo-wrap">

  <!-- top -->
  <div class="demo-top">
    <div class="demo-top-title">PostCAD</div>
    <div class="demo-top-sub">Deterministic routing from CAD to production</div>
    <div class="demo-top-hint">Run a real case through the system</div>
  </div>

  <!-- error / loading -->
  <div id="fixtures-loading" class="load-note">Loading…</div>
  <div id="fixtures-error" class="hidden"></div>

  <!-- case selection -->
  <div id="case-select" class="case-select hidden">
    <div class="case-select-label">Choose a case</div>

    <button class="case-btn" id="sc-valid" onclick="selectCase('valid')">
      <div class="case-btn-left">
        <span class="case-btn-name">Standard routing</span>
        <span class="case-btn-detail">Crown · Zirconia · Germany</span>
      </div>
      <div class="case-btn-right">
        <span class="case-btn-expect expect-ok">→ Routed</span>
      </div>
    </button>

    <button class="case-btn" id="sc-jurisdiction" onclick="selectCase('jurisdiction')">
      <div class="case-btn-left">
        <span class="case-btn-name">Invalid jurisdiction</span>
        <span class="case-btn-detail">Crown · Zirconia · United States</span>
      </div>
      <div class="case-btn-right">
        <span class="case-btn-expect expect-warn">→ Refused</span>
      </div>
    </button>

    <button class="case-btn" id="sc-capability" onclick="selectCase('capability')">
      <div class="case-btn-left">
        <span class="case-btn-name">Manufacturer not eligible</span>
        <span class="case-btn-detail">Bridge · E.max · Germany</span>
      </div>
      <div class="case-btn-right">
        <span class="case-btn-expect expect-warn">→ Refused</span>
      </div>
    </button>
  </div>

  <!-- run button -->
  <button class="run-btn hidden" id="run-btn" onclick="runFullFlow()" disabled>
    Run live case
  </button>

  <!-- flow blocks -->
  <div class="flow-blocks hidden" id="flow-blocks">

    <div class="flow-block" id="block-cad">
      <div class="block-step">01 · CAD Case</div>
      <div class="block-status" id="cad-status"></div>
      <div class="block-detail" id="cad-detail"></div>
    </div>

    <div class="flow-block locked" id="block-routing">
      <div class="block-step">02 · Routing</div>
      <div class="block-status" id="routing-status"></div>
      <div class="block-detail" id="routing-detail"></div>
      <details class="tech-details hidden" id="routing-tech">
        <summary>Show technical details</summary>
        <pre class="tech-pre" id="routing-json"></pre>
      </details>
    </div>

    <div class="flow-block locked" id="block-verify">
      <div class="block-step">03 · Verification</div>
      <div class="block-status" id="verify-status"></div>
      <div class="block-detail" id="verify-detail"></div>
      <details class="tech-details hidden" id="verify-tech">
        <summary>Show technical details</summary>
        <pre class="tech-pre" id="verify-json-disp"></pre>
      </details>
    </div>

    <div class="flow-block locked" id="block-dispatch">
      <div class="block-step">04 · Dispatch</div>
      <div class="block-status" id="dispatch-status"></div>
      <div class="block-detail" id="dispatch-detail"></div>
      <details class="tech-details hidden" id="dispatch-tech">
        <summary>Show technical details</summary>
        <pre class="tech-pre" id="dispatch-json-disp"></pre>
      </details>
    </div>

  </div><!-- /flow-blocks -->

  <!-- reset -->
  <button class="reset-btn hidden" id="reset-btn" onclick="resetFlow()">
    Try another case
  </button>

</div><!-- /demo-wrap -->

<!--
  Legacy compatibility layer — hidden elements required by
  any retained helper functions (fmt, esc, etc.)
  Not visible; not part of the demo surface.
-->
<div id="_legacy" style="display:none!important" aria-hidden="true">
  <div id="norm-input-section">
    <input id="norm-case-id">
    <input id="norm-restoration-type">
    <input id="norm-material">
    <input id="norm-jurisdiction">
    <div id="route-norm-preview"></div>
    <button id="btn-route-norm"></button>
    <button id="btn-route"></button>
  </div>
  <button id="btn-load-case"></button>
  <button id="btn-verify"></button>
  <button id="btn-tamper"></button>
  <button id="btn-dispatch-create"></button>
  <button id="btn-dispatch-approve"></button>
  <button id="btn-dispatch-export"></button>
  <span id="step1-chip"></span>
  <span id="step2-chip"></span>
  <span id="step3-chip"></span>
  <span id="step4-chip"></span>
  <pre id="fix-case"></pre>
  <pre id="fix-registry"></pre>
  <pre id="fix-config"></pre>
  <pre id="route-receipt-json"></pre>
  <pre id="verify-json"></pre>
  <pre id="dispatch-export-json"></pre>
  <div id="step1-result"></div>
  <div id="step2-card"></div>
  <div id="step3-card"></div>
  <div id="step4-card"></div>
  <div id="route-result"></div>
  <div id="route-error"></div>
  <div id="verify-result"></div>
  <div id="dispatch-created"></div>
  <div id="dispatch-export-result"></div>
  <div id="dispatch-success"></div>
  <div id="dispatch-error"></div>
  <div id="results-loading"></div>
  <div id="results-placeholder"></div>
  <div id="fixtures-panel"></div>
  <div id="route-norm-inline"></div>
  <span id="ops-routing"></span><span id="ops-receipt"></span>
  <span id="ops-verify"></span><span id="ops-dispatch"></span>
  <span id="art-selected"></span><span id="art-outcome"></span>
  <span id="art-hash"></span><span id="art-kver"></span>
  <span id="art-dispatch-id"></span><span id="art-dispatch-status"></span>
  <span id="s1-case-id"></span><span id="s1-procedure"></span>
  <span id="s1-material"></span><span id="s1-jurisdiction"></span>
  <span id="s1-policy"></span>
  <span id="s3-result-field"></span><span id="s3-hash-display"></span>
  <span id="s4-status-display"></span><span id="s4-verify-display"></span>
  <span id="status-dot-legacy"></span>
  <div id="nar-rail"><span id="nar-action"></span><span id="nar-reason"></span></div>
  <div id="run-timeline">
    <div id="rt-route"><div class="rt-dot"></div><div class="rt-name"></div></div>
    <div id="rt-receipt"><div class="rt-dot"></div><div class="rt-name"></div></div>
    <div id="rt-verify"><div class="rt-dot"></div><div class="rt-name"></div></div>
    <div id="rt-dispatch"><div class="rt-dot"></div><div class="rt-name"></div></div>
    <div id="rt-summary"></div>
  </div>
  <div id="rib">
    <span id="rib-route"></span><span id="rib-receipt"></span>
    <span id="rib-verify"></span><span id="rib-dispatch"></span>
  </div>
  <div id="osg"><div id="osg-reasons"></div><button onclick="startCleanRun()"></button></div>
  <div id="oab">
    <span id="oab-action"></span>
    <button id="oab-btn" onclick="oabNavigate()"></button>
    <div id="oab-reason"></div>
  </div>
  <div id="orb"><div id="orb-headline"></div><div id="orb-detail"></div>
    <button id="orb-link" onclick="orbNavigate()"></button></div>
  <div id="crc"><div id="crc-rows"></div><div id="crc-footer"></div></div>
  <div id="pfc"><div id="pfc-headline"></div><div id="pfc-detail"></div>
    <div id="pfc-rows"></div><button id="pfc-link" onclick="pfcNavigate()"></button></div>
  <div id="ccs"><div id="ccs-headline"></div><div id="ccs-detail"></div></div>
  <div id="hsc"><div id="hsc-verdict"></div><div id="hsc-rows"></div>
    <div id="hsc-readiness"></div><div id="hsc-artifacts"></div>
    <div id="hsc-summary"></div></div>
  <div id="active-run-context">
    <span id="arc-manufacturer"></span><span id="arc-receipt-hash"></span>
    <span id="arc-verify-status"></span><span id="arc-dispatch-status"></span>
  </div>
  <div id="run-history-panel"><div id="run-history-list"></div></div>
  <div id="sal"><div id="sal-empty"></div><div id="sal-list"></div></div>
  <div id="op-cheatsheet"></div>
  <div id="rrc">
    <div id="rrc-status"></div><div id="rrc-detail"></div>
    <button id="btn-repro" onclick="runReproCheck(this)"></button>
  </div>
  <div id="dpi">
    <div id="dpi-meta"><span id="dpi-origin"></span><span id="dpi-integrity"></span></div>
    <div id="dpi-empty"></div><pre id="dpi-viewer"></pre>
  </div>
  <div id="handoff-note"><div id="hn-body"></div></div>
  <div id="dispatch-readiness-panel">
    <div id="dr-status"></div><div id="dr-reason"></div>
    <div id="cl-receipt"></div><div id="cl-verify"></div><div id="cl-dispatch"></div>
  </div>
  <div id="dbl"><div id="dbl-body"></div></div>
  <div id="dhd">
    <div id="dhd-verdict"></div><div id="dhd-meaning"></div>
    <div id="dhd-checklist"></div><span id="dhd-next-text"></span>
  </div>
  <div id="drs">
    <div id="drs-verdict"></div><div id="drs-meaning"></div>
    <div id="drs-checklist"></div><span id="drs-next-text"></span>
  </div>
  <div id="phs">
    <div id="phs-verdict"></div><div id="phs-meaning"></div>
    <div id="phs-checklist"></div><span id="phs-action-text"></span>
  </div>
  <div id="cab">
    <div id="cab-verdict"></div><div id="cab-meaning"></div>
    <div id="cab-artifacts"></div><span id="cab-next-text"></span>
  </div>
  <div id="cpw">
    <span id="cpw-s1"></span><span id="cpw-s2"></span><span id="cpw-s3"></span>
    <span id="cpw-s4"></span><span id="cpw-s5"></span><span id="cpw-s6"></span>
  </div>
  <div id="lin-verify-note"></div>
  <div id="lin-dispatch-note"></div>
  <span id="lin-verify"></span>
  <span id="lin-dispatch-export"></span>
  <span id="mb-receipt"></span><span id="mb-verify"></span><span id="mb-dispatch"></span>
  <span id="fm-receipt"></span><span id="fm-verify"></span><span id="fm-dispatch"></span>
  <span id="as-chip-route"></span><span id="as-chip-verify"></span><span id="as-chip-dispatch"></span>
  <span id="as-chip-export"></span>
  <span id="receipt-json-badge"></span>
  <span id="route-result-badge"></span>
  <span id="verify-result-badge"></span>
  <span id="dispatch-result-badge"></span>
  <div id="receipt-empty-state"></div>
  <div id="verify-artifact-note"></div>
  <div id="receipt-json-actions"></div>
  <div id="verify-json-actions"></div>
  <div id="dispatch-export-actions"></div>
  <div id="verify-pending-note"></div>
  <div id="dispatch-blocked-note"></div>
  <div id="dispatch-stale-note"></div>
  <div id="verify-banner"></div>
  <div id="verify-kind-label"></div>
  <div id="verify-summary-panel"><span id="verify-dot"></span></div>
  <div id="verify-result-inner"></div>
  <button id="receipt-expand-btn" onclick="expandArtifact('route-receipt-json','receipt-expand-btn')"></button>
  <button id="verify-expand-btn" onclick="expandArtifact('verify-json','verify-expand-btn')"></button>
  <button id="dispatch-expand-btn" onclick="expandArtifact('dispatch-export-json','dispatch-expand-btn')"></button>
  <button id="art-hash-copy" onclick="copyArtHashVal(this)"></button>
  <button id="art-dispatch-id-copy" onclick="copyDispatchId(this)"></button>
  <button id="btn-copy-snapshot" onclick="copyAuditSnapshot(this)"></button>
  <button id="btn-print-handoff" onclick="window.print()"></button>
</div>

<script>
// ─────────────────────────────────────────────────────────────────────────────
// State
// ─────────────────────────────────────────────────────────────────────────────
let fixtures       = null;
let selectedCaseId = null;
let lastReceipt    = null;
let lastPolicy     = null;
let flowRunning    = false;

// Legacy state (kept for any retained helper functions)
let lastDispatchId   = null;
let lastExportPacket = null;
let opRouting  = 'not-run';
let opReceipt  = 'not-run';
let opVerify   = 'not-run';
let opDispatch = 'not-run';
let runSerial      = 0;
let verifySerial   = 0;
let dispatchSerial = 0;
let lastRouteInputs   = null;
let lastRouteEndpoint = null;
let reproStatus = 'not-tested';
const runHistory = [];
let sessionLog = [];

// ─────────────────────────────────────────────────────────────────────────────
// Scenario definitions — real inputs against real engine
// ─────────────────────────────────────────────────────────────────────────────
const CASES = {
  valid: {
    case_id:          'f1000001-0000-0000-0000-000000000001',
    restoration_type: 'crown',
    material:         'zirconia',
    jurisdiction:     'DE',
    routing_config:   {jurisdiction:'DE', routing_policy:'allow_domestic_and_cross_border'},
  },
  jurisdiction: {
    case_id:          'f1000002-0000-0000-0000-000000000002',
    restoration_type: 'crown',
    material:         'zirconia',
    jurisdiction:     'US',
    routing_config:   {jurisdiction:'US', routing_policy:'allow_domestic_and_cross_border'},
  },
  capability: {
    case_id:          'f1000003-0000-0000-0000-000000000003',
    restoration_type: 'bridge',
    material:         'emax',
    jurisdiction:     'DE',
    routing_config:   {jurisdiction:'DE', routing_policy:'allow_domestic_and_cross_border'},
  },
};

const REFUSAL_MSGS = {
  'no_jurisdiction_match': 'No manufacturers serve this jurisdiction',
  'no_material_match':     'No manufacturer supports this material and procedure',
  'no_active_manufacturer':'No active manufacturers available',
};

// ─────────────────────────────────────────────────────────────────────────────
// Boot
// ─────────────────────────────────────────────────────────────────────────────
(async function boot() {
  try {
    const r = await fetch('/version');
    const v = await r.json();
    const dot = document.getElementById('status-dot');
    const ver = document.getElementById('ver');
    dot.className = 'demo-header-dot' + (r.ok ? ' ok' : ' err');
    ver.textContent = v.protocol_version
      ? v.protocol_version + ' · ' + v.routing_kernel_version
      : JSON.stringify(v);
  } catch(e) {
    const dot = document.getElementById('status-dot');
    if (dot) { dot.className = 'demo-header-dot err'; }
    const ver = document.getElementById('ver');
    if (ver) ver.textContent = 'offline';
  }

  try {
    const r = await fetch('/pilot-fixtures');
    if (!r.ok) throw new Error('HTTP ' + r.status + ': ' + await r.text());
    fixtures = await r.json();
    // Populate legacy tech preview elements
    const fc = document.getElementById('fix-case');
    const fr = document.getElementById('fix-registry');
    const ff = document.getElementById('fix-config');
    if (fc) fc.textContent = fmt(fixtures.case);
    if (fr) fr.textContent = fmt(fixtures.registry_snapshot);
    if (ff) ff.textContent = fmt(fixtures.routing_config);

    document.getElementById('fixtures-loading').classList.add('hidden');
    document.getElementById('case-select').classList.remove('hidden');
    document.getElementById('run-btn').classList.remove('hidden');
    document.getElementById('flow-blocks').classList.remove('hidden');
  } catch(e) {
    document.getElementById('fixtures-loading').classList.add('hidden');
    const errEl = document.getElementById('fixtures-error');
    errEl.innerHTML =
      '<div class="error-banner"><strong>Cannot load fixtures</strong>'
      + 'Start the service: <code style="font-family:var(--mono)">cargo run -p postcad-service</code><br>'
      + '<span style="font-size:.7rem;color:var(--text-3);font-family:var(--mono)">' + esc(e.message) + '</span></div>';
    errEl.classList.remove('hidden');
  }
})();

// ─────────────────────────────────────────────────────────────────────────────
// Case selection
// ─────────────────────────────────────────────────────────────────────────────
function selectCase(id) {
  if (flowRunning) return;
  selectedCaseId = id;
  document.querySelectorAll('.case-btn').forEach(b => b.classList.remove('selected'));
  const btn = document.getElementById('sc-' + id);
  if (btn) btn.classList.add('selected');
  const runBtn = document.getElementById('run-btn');
  if (runBtn && fixtures) runBtn.disabled = false;
}

// ─────────────────────────────────────────────────────────────────────────────
// Run full flow
// ─────────────────────────────────────────────────────────────────────────────
async function runFullFlow() {
  if (!selectedCaseId || !fixtures || flowRunning) return;
  const c = CASES[selectedCaseId];
  if (!c) return;

  flowRunning = true;
  lastReceipt = null; lastPolicy = null;

  const runBtn = document.getElementById('run-btn');
  runBtn.disabled = true;
  runBtn.classList.add('running');
  runBtn.textContent = 'Running…';

  document.getElementById('reset-btn').classList.add('hidden');

  // ── Step 1: CAD block ────────────────────────────────────────────────────
  setBlock('block-cad', 'active');
  setText('cad-status',
    '<span class="status-ok">✔ Case ready</span>');
  setText('cad-detail',
    cap(c.restoration_type) + ' · ' + cap(c.material) + ' · ' + jurisdictionLabel(c.jurisdiction));

  await delay(350);

  // ── Step 2: Routing ──────────────────────────────────────────────────────
  setBlock('block-routing', 'active');
  setText('routing-status', '<span class="status-pending">Routing…</span>');
  setText('routing-detail', '');

  let routed = false;
  let routeReceiptHash = '';

  try {
    const r = await fetch('/pilot/route-normalized', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({
        pilot_case: {
          case_id:          c.case_id,
          restoration_type: c.restoration_type,
          material:         c.material,
          jurisdiction:     c.jurisdiction,
        },
        registry_snapshot: fixtures.registry_snapshot,
        routing_config:    c.routing_config,
      }),
    });
    const data = await r.json();

    if (r.ok && data.receipt) {
      lastReceipt = data.receipt;
      lastPolicy  = data.derived_policy;
      const outcome = data.receipt.outcome;
      routed = outcome === 'routed';
      routeReceiptHash = data.receipt.receipt_hash || '';

      if (routed) {
        const mfr = data.receipt.selected_candidate_id || '—';
        setBlock('block-routing', 'done-ok');
        setText('routing-status',
          '<span class="status-ok">✔ Routed to eligible manufacturer</span>');
        setText('routing-detail', mfr);
      } else {
        const code = data.receipt.refusal_code || '';
        setBlock('block-routing', 'done-err');
        setText('routing-status', '<span class="status-err">✖ Cannot route</span>');
        setText('routing-detail', REFUSAL_MSGS[code] || 'No eligible manufacturer found');
      }
      document.getElementById('routing-json').textContent = fmt(data.receipt);
      document.getElementById('routing-tech').classList.remove('hidden');
    } else {
      setBlock('block-routing', 'done-err');
      setText('routing-status', '<span class="status-err">✖ Routing error</span>');
      setText('routing-detail', data?.error?.message || 'Request failed');
      document.getElementById('routing-json').textContent = fmt(data);
      document.getElementById('routing-tech').classList.remove('hidden');
    }
  } catch(e) {
    setBlock('block-routing', 'done-err');
    setText('routing-status', '<span class="status-err">✖ Network error</span>');
    setText('routing-detail', String(e));
  }

  await delay(300);

  // ── Step 3: Verification ─────────────────────────────────────────────────
  setBlock('block-verify', 'active');
  setText('verify-status', '<span class="status-pending">Verifying…</span>');
  setText('verify-detail', '');

  if (lastReceipt && lastPolicy) {
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
      const isVerified = r.ok && data.result === 'VERIFIED';
      setBlock('block-verify', isVerified ? 'done-ok' : 'done-err');
      if (isVerified) {
        setText('verify-status',
          '<span class="status-ok">✔ Decision reproducible</span>');
        setText('verify-detail', 'Same inputs produce the same receipt');
      } else {
        setText('verify-status',
          '<span class="status-err">✖ Verification failed</span>');
        setText('verify-detail', data?.error?.message || 'Replay mismatch');
      }
      document.getElementById('verify-json-disp').textContent = fmt(data);
      document.getElementById('verify-tech').classList.remove('hidden');
    } catch(e) {
      setBlock('block-verify', 'done-err');
      setText('verify-status', '<span class="status-err">✖ Error</span>');
      setText('verify-detail', String(e));
    }
  } else {
    setBlock('block-verify', 'done-err');
    setText('verify-status', '<span class="status-neutral">— Skipped</span>');
    setText('verify-detail', 'No receipt available to verify');
  }

  await delay(300);

  // ── Step 4: Dispatch ─────────────────────────────────────────────────────
  setBlock('block-dispatch', 'active');

  if (routed && lastReceipt && lastPolicy) {
    setText('dispatch-status',
      '<span class="status-pending">Creating dispatch record…</span>');
    setText('dispatch-detail', '');
    try {
      const r = await fetch('/dispatch/create', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({
          receipt: lastReceipt,
          case:    fixtures.case,
          policy:  lastPolicy,
        }),
      });
      const data = await r.json();
      if (r.ok && data.dispatch_id) {
        setBlock('block-dispatch', 'done-ok');
        setText('dispatch-status',
          '<span class="status-ok">✔ Dispatch ready</span>');
        setText('dispatch-detail',
          'Record: ' + data.dispatch_id.slice(0, 14) + '…');
        document.getElementById('dispatch-json-disp').textContent = fmt(data);
        document.getElementById('dispatch-tech').classList.remove('hidden');
      } else if (r.status === 409) {
        setBlock('block-dispatch', 'done-ok');
        setText('dispatch-status',
          '<span class="status-ok">✔ Dispatch record exists</span>');
        setText('dispatch-detail', 'This receipt was already committed');
      } else {
        setBlock('block-dispatch', 'done-err');
        setText('dispatch-status',
          '<span class="status-err">✖ Dispatch failed</span>');
        setText('dispatch-detail', data?.error?.message || 'Error');
        document.getElementById('dispatch-json-disp').textContent = fmt(data);
        document.getElementById('dispatch-tech').classList.remove('hidden');
      }
    } catch(e) {
      setBlock('block-dispatch', 'done-err');
      setText('dispatch-status', '<span class="status-err">✖ Network error</span>');
      setText('dispatch-detail', String(e));
    }
  } else {
    setBlock('block-dispatch', 'done-err');
    setText('dispatch-status',
      '<span class="status-neutral">✖ Not available</span>');
    setText('dispatch-detail', 'Routing must succeed before dispatch');
  }

  // ── Done ─────────────────────────────────────────────────────────────────
  flowRunning = false;
  runBtn.classList.remove('running');
  runBtn.textContent = 'Run live case';
  document.getElementById('reset-btn').classList.remove('hidden');
}

// ─────────────────────────────────────────────────────────────────────────────
// Reset
// ─────────────────────────────────────────────────────────────────────────────
function resetFlow() {
  lastReceipt = null; lastPolicy = null;
  selectedCaseId = null; flowRunning = false;

  document.querySelectorAll('.case-btn').forEach(b => b.classList.remove('selected'));

  setBlock('block-cad',      '');
  setBlock('block-routing',  'locked');
  setBlock('block-verify',   'locked');
  setBlock('block-dispatch', 'locked');

  ['cad-status','routing-status','verify-status','dispatch-status'].forEach(id => {
    document.getElementById(id).innerHTML = '';
  });
  ['cad-detail','routing-detail','verify-detail','dispatch-detail'].forEach(id => {
    document.getElementById(id).textContent = '';
  });
  ['routing-tech','verify-tech','dispatch-tech'].forEach(id => {
    const el = document.getElementById(id);
    if (el) { el.removeAttribute('open'); el.classList.add('hidden'); }
  });

  const runBtn = document.getElementById('run-btn');
  runBtn.disabled = true;
  runBtn.textContent = 'Run live case';
  document.getElementById('reset-btn').classList.add('hidden');
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────
function setBlock(id, state) {
  const el = document.getElementById(id);
  if (!el) return;
  el.className = 'flow-block' + (state ? ' ' + state : '');
}
function setText(id, html) {
  const el = document.getElementById(id);
  if (el) el.innerHTML = html;
}
function delay(ms) { return new Promise(r => setTimeout(r, ms)); }
function fmt(o)   { return JSON.stringify(o, null, 2); }
function esc(s)   { return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;'); }
function cap(s)   { return s ? s.charAt(0).toUpperCase() + s.slice(1) : s; }
function jurisdictionLabel(j) {
  const m = {DE:'Germany',US:'United States',GB:'UK',FR:'France',JP:'Japan'};
  return m[j] || j;
}

// ─────────────────────────────────────────────────────────────────────────────
// Legacy no-ops — kept so any retained HTML onclick attributes don't throw
// ─────────────────────────────────────────────────────────────────────────────
function show(id) { const e=document.getElementById(id); if(e) e.classList.remove('hidden'); }
function hide(id) { const e=document.getElementById(id); if(e) e.classList.add('hidden'); }
function setBtn(btn, label, disabled) { if(btn){btn.textContent=label;btn.disabled=disabled;} }
function loadDemoCase() {}
function routeNormalized() {}
function verifyReceipt() {}
function createDispatch() {}
function approveDispatch() {}
function exportDispatch() {}
function tamperVerify() {}
function startCleanRun() {}
function runReproCheck() {}
function oabNavigate() {}
function orbNavigate() {}
function pfcNavigate() {}
function copyArtHashVal() {}
function copyDispatchId() {}
function copyReceiptJson() {}
function copyVerifyJson() {}
function copyRouteErrorJson() {}
function copyExportJson() {}
function copyAuditSnapshot() {}
function downloadAuditSnapshot() {}
function downloadExportPacket() {}
function expandArtifact() {}
function loadNormSample() {}
function clearNormForm() {}
function routeCase() {}
</script>
</body>
</html>"##;
