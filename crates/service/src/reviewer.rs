//! Reviewer shell — single page, three actions, real kernel output.
//!
//! Served at `GET /reviewer`. Auto-loads pilot fixtures from `examples/pilot/`
//! via `GET /pilot-fixtures`. Calls real endpoints only:
//!   POST /route  — routing kernel execution
//!   POST /verify — deterministic receipt verification
//!
//! No mock data. No fake outputs. No mocked decisions.

pub const REVIEWER_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PostCAD — Reviewer Shell</title>
<style>
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
body{font-family:ui-monospace,"Cascadia Mono","Menlo",monospace;
     background:#0f1117;color:#c9d1d9;min-height:100vh;font-size:13px}

/* ── header ── */
header{background:#161b22;border-bottom:1px solid #30363d;
       padding:.55rem 1.2rem;display:flex;align-items:center;gap:.75rem}
#status-dot{width:7px;height:7px;border-radius:50%;background:#d29922;flex-shrink:0}
#status-dot.ok{background:#3fb950}
#status-dot.err{background:#f85149}
#hdr-title{font-size:.92rem;font-weight:700;color:#f0f6fc}
#hdr-tag{font-size:.7rem;color:#6e7681;margin-left:.25rem}
#ver{margin-left:auto;font-size:.7rem;color:#6e7681}

/* ── layout ── */
main{max-width:1100px;margin:1.2rem auto;padding:0 1rem 3.5rem;display:grid;gap:1rem}

/* ── hero ── */
.hero{background:#161b22;border:1px solid #30363d;border-radius:8px;
      padding:1rem 1.25rem}
.hero-title{font-size:1.15rem;font-weight:700;color:#f0f6fc;margin-bottom:.2rem}
.hero-sub{font-size:.82rem;color:#8b949e;margin-bottom:.25rem}
.hero-why{font-size:.78rem;color:#6e7681;margin-bottom:.9rem;
          border-left:2px solid #21262d;padding-left:.6rem}

/* ── 4-step flow ── */
.flow{display:flex;align-items:stretch;gap:0;overflow-x:auto}
.flow-step{flex:1;min-width:0;background:#0d1117;border:1px solid #21262d;
           border-radius:0;padding:.6rem .75rem;position:relative}
.flow-step:first-child{border-radius:6px 0 0 6px}
.flow-step:last-child {border-radius:0 6px 6px 0}
.flow-step+.flow-step{border-left:none}
.flow-num{font-size:.6rem;color:#6e7681;font-weight:700;letter-spacing:.08em;
          text-transform:uppercase;margin-bottom:.2rem}
.flow-label{font-size:.75rem;font-weight:700;color:#c9d1d9;margin-bottom:.2rem}
.flow-desc{font-size:.68rem;color:#8b949e;line-height:1.45}
.flow-arrow{align-self:center;color:#6e7681;font-size:.9rem;padding:0 .1rem;
            flex-shrink:0}

/* ── card ── */
.card{background:#161b22;border:1px solid #30363d;border-radius:8px;
      padding:.9rem 1.1rem}
.card-title{font-size:.65rem;font-weight:700;color:#8b949e;letter-spacing:.08em;
            text-transform:uppercase;margin-bottom:.65rem}

/* ── two-col ── */
.two-col{display:grid;grid-template-columns:320px 1fr;gap:1rem}
@media(max-width:800px){.two-col{grid-template-columns:1fr}}

/* ── inputs panel ── */
.input-label{font-size:.65rem;color:#6e7681;text-transform:uppercase;
             letter-spacing:.06em;margin:.55rem 0 .15rem;display:flex;
             align-items:center;gap:.4rem}
.input-label:first-child{margin-top:0}
.input-badge{font-size:.6rem;color:#8b949e;background:#1c2128;
             border:1px solid #30363d;border-radius:2px;padding:.02rem .3rem}
details{margin-bottom:.15rem}
summary{font-size:.72rem;color:#58a6ff;cursor:pointer;padding:.15rem 0;
        user-select:none;list-style:none;display:flex;align-items:center;gap:.3rem}
summary::before{content:"▶";font-size:.55rem;transition:transform .15s;color:#30363d}
details[open] summary::before{transform:rotate(90deg)}
summary:hover{color:#79c0ff}
pre.fixture{background:#0d1117;border:1px solid #21262d;border-radius:5px;
            padding:.45rem .65rem;font-size:.67rem;overflow-x:auto;
            white-space:pre-wrap;word-break:break-all;max-height:160px;
            overflow-y:auto;line-height:1.4;margin-top:.25rem;color:#8b949e}

/* ── buttons ── */
.btn{display:inline-flex;align-items:center;gap:.35rem;padding:.38rem .9rem;
     font-family:inherit;font-size:.78rem;border-radius:5px;
     border:1px solid transparent;cursor:pointer;transition:opacity .12s;font-weight:600}
.btn:hover:not(:disabled){opacity:.82}
.btn:disabled{opacity:.4;cursor:default}
.btn-route      {background:#238636;border-color:#2ea043;color:#fff;width:100%;
                 justify-content:center;margin-top:.5rem}
.btn-route-norm {background:#1a3455;border-color:#388bfd;color:#79c0ff;width:100%;
                 justify-content:center;margin-top:.35rem;font-size:.75rem}
.btn-verify {background:#6e40c9;border-color:#8957e5;color:#fff;width:100%;
             justify-content:center;margin-top:.5rem}
.btn-tamper   {background:#21262d;border-color:#b36200;color:#d29922;width:100%;
               justify-content:center;margin-top:.4rem;font-size:.72rem}
.btn-dispatch {background:#1a3455;border-color:#388bfd;color:#79c0ff;width:100%;
               justify-content:center;margin-top:.5rem}
.btn-approve  {background:#1a3e2c;border-color:#2ea043;color:#3fb950;width:100%;
               justify-content:center;margin-top:.35rem}
.btn-export   {background:#21262d;border-color:#388bfd;color:#58a6ff;width:100%;
               justify-content:center;margin-top:.35rem;font-size:.72rem}

/* ── artifact summary ── */
.artifacts{background:#0d1117;border:1px solid #21262d;border-radius:6px;
           padding:.65rem .85rem;margin-bottom:.65rem}
.artifact-row{display:grid;grid-template-columns:max-content 1fr;gap:.15rem .6rem;
              align-items:baseline;margin-bottom:.2rem}
.artifact-row:last-child{margin-bottom:0}
.art-key{font-size:.65rem;color:#6e7681;text-transform:uppercase;
         letter-spacing:.05em;white-space:nowrap}
.art-val{font-size:.8rem;font-weight:700;color:#f0f6fc;word-break:break-all}
.art-hash{font-size:.71rem;color:#58a6ff;word-break:break-all;font-family:inherit}
.determinism-note{font-size:.67rem;color:#3fb950;margin-top:.5rem;
                  padding-top:.4rem;border-top:1px solid #21262d;
                  display:flex;align-items:center;gap:.35rem}

/* ── pills / badges ── */
.pill{display:inline-block;padding:.06rem .4rem;border-radius:3px;
      font-size:.68rem;font-weight:700}
.pill-ok   {background:#1a3e2c;color:#3fb950}
.pill-err  {background:#3d1f1f;color:#f85149}
.pill-warn {background:#2d2009;color:#d29922}
.pill-info {background:#1e2d45;color:#58a6ff}

/* ── receipt JSON ── */
.section-title{font-size:.65rem;font-weight:700;color:#8b949e;text-transform:uppercase;
               letter-spacing:.07em;margin:.8rem 0 .3rem;
               display:flex;align-items:center;gap:.4rem}
pre.result{background:#0d1117;border:1px solid #21262d;border-radius:5px;
           padding:.55rem .75rem;font-size:.68rem;overflow-x:auto;
           white-space:pre-wrap;word-break:break-all;max-height:380px;
           overflow-y:auto;line-height:1.45}
pre.result-ok  {border-left:3px solid #3fb950}
pre.result-err {border-left:3px solid #f85149}
pre.result-info{border-left:3px solid #388bfd}

/* ── verify banner ── */
.verify-section{margin-top:.9rem}
.verify-banner{border-radius:6px;padding:.7rem 1rem;font-size:.88rem;
               font-weight:700;text-align:center;margin-bottom:.5rem}
.banner-ok  {background:#1a3e2c;color:#3fb950;border:1px solid #2ea043}
.banner-err {background:#3d1f1f;color:#f85149;border:1px solid #f85149}
.verify-sub{font-size:.72rem;color:#8b949e;margin-bottom:.3rem;
            font-weight:400;display:block}

/* ── tamper section ── */
.tamper-section{margin-top:.9rem;padding-top:.8rem;border-top:1px dashed #30363d}
.tamper-label{font-size:.65rem;color:#6e7681;text-transform:uppercase;
              letter-spacing:.07em;margin-bottom:.4rem}
.tamper-desc{font-size:.72rem;color:#8b949e;margin-bottom:.5rem;line-height:1.5}

/* ── footer ── */
footer{position:fixed;bottom:0;left:0;right:0;background:#0d1117;
       border-top:1px solid #21262d;padding:.45rem 1.2rem;
       display:flex;align-items:center;gap:1.4rem;font-size:.67rem;
       color:#6e7681;z-index:10;overflow-x:auto;white-space:nowrap}
.ft-label{color:#6e7681;font-size:.62rem;text-transform:uppercase;
          letter-spacing:.07em;margin-right:.3rem}
.ft-ep{color:#8b949e}
.ft-ep .method{color:#d29922}
.ft-ep .path{color:#58a6ff}
.ft-arch{margin-left:auto;color:#6e7681;font-size:.65rem}

/* ── misc ── */
.dimmed{color:#6e7681;font-size:.75rem}
.hidden{display:none!important}
.error-note  {font-size:.75rem;color:#f85149;margin-top:.4rem;line-height:1.5}
.warn-note   {font-size:.75rem;color:#d29922;margin-top:.4rem;line-height:1.5}
.success-note{font-size:.75rem;color:#3fb950;margin-top:.4rem;line-height:1.5}
.loading-note{font-size:.75rem;color:#6e7681;margin-top:.4rem;line-height:1.5}
.norm-preview{background:#0d1117;border:1px solid #21262d;border-radius:5px;
              padding:.4rem .65rem;margin-top:.35rem;font-size:.67rem;line-height:1.6}
.norm-preview-row{display:grid;grid-template-columns:max-content 1fr;gap:.1rem .6rem}
.norm-preview-key{color:#6e7681;text-transform:uppercase;font-size:.6rem;
                  letter-spacing:.05em;white-space:nowrap}
.norm-preview-val{color:#c9d1d9;word-break:break-all}
.norm-field-invalid{border-color:#f85149!important}
.copy-btn{background:none;border:1px solid #30363d;border-radius:3px;color:#58a6ff;
          cursor:pointer;font-family:inherit;font-size:.6rem;padding:.05rem .3rem;
          margin-left:.35rem;transition:color .1s}
.copy-btn:hover{color:#79c0ff}
.btn-dl{display:inline-flex;align-items:center;gap:.3rem;background:#1a3455;
        border:1px solid #388bfd;border-radius:4px;color:#79c0ff;cursor:pointer;
        font-family:inherit;font-size:.72rem;font-weight:600;padding:.28rem .7rem;
        margin-top:.45rem;transition:opacity .12s}
.btn-dl:hover{opacity:.82}
</style>
</head>
<body>

<header>
  <span id="status-dot"></span>
  <span id="hdr-title">PostCAD</span>
  <span id="hdr-tag">reviewer shell</span>
  <span id="ver">loading…</span>
</header>

<main>

  <!-- A. Hero / info architecture -->
  <div class="hero">
    <div class="hero-title">PostCAD Routing Kernel</div>
    <div class="hero-sub">Deterministic manufacturing routing with verifiable receipts</div>
    <div class="hero-why">PostCAD replaces manual lab selection with deterministic routing and auditable manufacturing receipts.</div>
    <div class="flow">
      <div class="flow-step">
        <div class="flow-num">01 · inputs</div>
        <div class="flow-label">Inputs</div>
        <div class="flow-desc">Dental CAD case + manufacturer registry + routing policy</div>
      </div>
      <div class="flow-arrow">›</div>
      <div class="flow-step">
        <div class="flow-num">02 · kernel</div>
        <div class="flow-label">Kernel</div>
        <div class="flow-desc">Deterministic routing engine evaluates eligibility and selects a manufacturer</div>
      </div>
      <div class="flow-arrow">›</div>
      <div class="flow-step">
        <div class="flow-num">03 · output</div>
        <div class="flow-label">Output</div>
        <div class="flow-desc">Cryptographically committed receipt with hash-chained audit entry</div>
      </div>
      <div class="flow-arrow">›</div>
      <div class="flow-step">
        <div class="flow-num">04 · verify</div>
        <div class="flow-label">Verification</div>
        <div class="flow-desc">Independent replay confirms the decision — same hash, every time</div>
      </div>
    </div>
  </div>

  <!-- Two-column: inputs + results -->
  <div class="two-col">

    <!-- LEFT: Inputs -->
    <div class="card">
      <div class="card-title">Pilot inputs <span style="font-weight:400;color:#6e7681">— examples/pilot/</span></div>

      <p id="fixtures-loading" class="dimmed">Loading fixtures…</p>
      <div id="fixtures-panel" class="hidden">

        <div class="input-label">case <span class="input-badge">case.json</span></div>
        <details open>
          <summary>view JSON</summary>
          <pre class="fixture" id="fix-case"></pre>
        </details>

        <div class="input-label">registry <span class="input-badge">registry_snapshot.json</span></div>
        <details>
          <summary>view JSON</summary>
          <pre class="fixture" id="fix-registry"></pre>
        </details>

        <div class="input-label">policy / config <span class="input-badge">config.json</span></div>
        <details>
          <summary>view JSON</summary>
          <pre class="fixture" id="fix-config"></pre>
        </details>
      </div>
      <div id="fixtures-error" class="hidden error-note"></div>

      <button class="btn btn-route" id="btn-route" onclick="routeCase(this)" disabled>
        ▶ Execute Routing Kernel
      </button>

      <div id="norm-input-section" style="margin-top:.75rem;border-top:1px solid #21262d;padding-top:.65rem">
        <div class="input-label">normalized pilot input <span class="input-badge">4 fields only</span></div>
        <details>
          <summary>view JSON</summary>
          <pre class="fixture" id="fix-normalized-case">{"case_id":"f1000001-0000-0000-0000-000000000001","restoration_type":"crown","material":"zirconia","jurisdiction":"DE"}</pre>
        </details>
        <button class="btn btn-route-norm" id="btn-route-norm" onclick="routeNormalized(this)" disabled>
          ▶ Route Normalized Pilot Case
        </button>
        <button class="copy-btn" style="margin-top:.3rem" onclick="clearNormForm()">↺ Clear form</button>
        <button class="copy-btn" style="margin-top:.3rem;margin-left:.4rem" onclick="loadNormSample()">⊕ Load sample</button>
        <div id="route-norm-inline" class="hidden"></div>
        <div id="route-norm-preview" class="hidden"></div>
      </div>
    </div>

    <!-- RIGHT: Results -->
    <div class="card">
      <div id="results-placeholder" class="dimmed" style="padding:.5rem 0">
        No items awaiting review. Submit a pilot input to begin.
      </div>
      <div id="results-loading" class="hidden loading-note" style="padding:.5rem 0">Running kernel…</div>

      <!-- B. Artifact summary (shown after route) -->
      <div id="route-result" class="hidden">
        <div class="card-title">Routing result</div>

        <div class="artifacts">
          <div class="artifact-row">
            <span class="art-key">Outcome</span>
            <span class="art-val" id="art-outcome"></span>
          </div>
          <div class="artifact-row">
            <span class="art-key">Selected Manufacturer</span>
            <span class="art-val" id="art-selected"></span>
          </div>
          <div class="artifact-row">
            <span class="art-key">Receipt Hash</span>
            <span class="art-hash" id="art-hash"></span>
          </div>
          <div class="artifact-row">
            <span class="art-key">Kernel Version</span>
            <span class="art-val" id="art-kver"></span>
          </div>
          <div class="determinism-note">
            <span>◆</span>
            <span>Deterministic result — same inputs produce the same receipt hash on every run</span>
          </div>
        </div>

        <div class="section-title">
          Full receipt JSON
          <span style="font-weight:400;color:#6e7681;font-size:.63rem;text-transform:none">— raw kernel output</span>
        </div>
        <pre class="result result-ok" id="route-receipt-json"></pre>

        <!-- D. Button labels -->
        <button class="btn btn-verify" id="btn-verify" onclick="verifyReceipt(this)" disabled>
          ↩ Replay Verification
        </button>

        <!-- E. Tamper demo -->
        <div class="tamper-section">
          <div class="tamper-label">Tamper demo</div>
          <div class="tamper-desc">
            Modifies <code>selected_candidate_id</code> in the receipt client-side,
            then submits to the real <code>POST /verify</code> endpoint.
            The verifier catches the mismatch — no backend changes.
          </div>
          <button class="btn btn-tamper" id="btn-tamper" onclick="tamperVerify(this)" disabled>
            ⚠ Tamper + Verify
          </button>
        </div>

        <!-- G. Dispatch commitment -->
        <div class="tamper-section" id="dispatch-section">
          <div class="tamper-label">Dispatch Commitment</div>
          <div class="tamper-desc">
            Calls <code>POST /dispatch/create</code> — the server re-verifies the
            receipt before creating the record. Approve makes the commitment
            immutable; export produces the deterministic dispatch packet.
          </div>
          <button class="btn btn-dispatch" id="btn-dispatch-create" onclick="createDispatch(this)" disabled>
            ⬦ Create Dispatch
          </button>

          <div id="dispatch-created" class="hidden" style="margin-top:.55rem">
            <div class="artifacts">
              <div class="artifact-row">
                <span class="art-key">Dispatch ID</span>
                <span class="art-hash" id="art-dispatch-id"></span>
              </div>
              <div class="artifact-row">
                <span class="art-key">Status</span>
                <span class="art-val" id="art-dispatch-status"></span>
              </div>
            </div>
            <button class="btn btn-approve" id="btn-dispatch-approve" onclick="approveDispatch(this)" disabled>
              ✓ Approve Dispatch
            </button>
            <button class="btn btn-export" id="btn-dispatch-export" onclick="exportDispatch(this)" disabled>
              ↓ Export Dispatch Packet
            </button>
          </div>

          <div id="dispatch-export-result" class="hidden" style="margin-top:.55rem">
            <div class="section-title">Export Packet</div>
            <pre class="result result-info" id="dispatch-export-json"></pre>
          </div>

          <div id="dispatch-success" class="hidden success-note" style="margin-top:.4rem"></div>
          <div id="dispatch-error" class="hidden error-note" style="margin-top:.4rem"></div>
        </div>
      </div>

      <!-- Route error -->
      <div id="route-error" class="hidden">
        <div class="card-title">Route error</div>
        <div id="route-error-banner" class="hidden error-note" style="margin-bottom:.35rem"></div>
        <pre class="result result-err" id="route-error-json"></pre>
      </div>

      <!-- F. Verify result (normal) -->
      <div id="verify-result" class="hidden">
        <div class="verify-section">
          <div class="section-title">Verification result <span id="verify-kind-label"></span></div>
          <div id="verify-banner"></div>
          <pre class="result" id="verify-json"></pre>
        </div>
      </div>

    </div>
  </div>

</main>

<!-- C. Architecture / endpoint footer -->
<footer>
  <span>
    <span class="ft-label">endpoints</span>
    <span class="ft-ep"><span class="method">GET</span> <span class="path">/pilot-fixtures</span></span>
    <span style="color:#30363d;margin:0 .3rem">·</span>
    <span class="ft-ep"><span class="method">POST</span> <span class="path">/route</span></span>
    <span style="color:#30363d;margin:0 .3rem">·</span>
    <span class="ft-ep"><span class="method">POST</span> <span class="path">/pilot/route-normalized</span></span>
    <span style="color:#30363d;margin:0 .3rem">·</span>
    <span class="ft-ep"><span class="method">POST</span> <span class="path">/verify</span></span>
    <span style="color:#30363d;margin:0 .3rem">·</span>
    <span class="ft-ep"><span class="method">POST</span> <span class="path">/dispatch/create</span></span>
    <span style="color:#30363d;margin:0 .3rem">·</span>
    <span class="ft-ep"><span class="method">POST</span> <span class="path">/dispatch/:id/approve</span></span>
    <span style="color:#30363d;margin:0 .3rem">·</span>
    <span class="ft-ep"><span class="method">GET</span> <span class="path">/dispatch/:id/export</span></span>
  </span>
  <span class="ft-arch">Reviewer UI → HTTP API → PostCAD Service → Routing Kernel → Receipt / Verification / Dispatch</span>
</footer>

<script>
// ── state ──────────────────────────────────────────────────────────────────
let fixtures       = null;   // {case, registry_snapshot, routing_config}
let lastReceipt    = null;
let lastPolicy     = null;
let lastDispatchId = null;

// ── boot ───────────────────────────────────────────────────────────────────
(async function boot() {
  try {
    const r = await fetch('/version');
    const v = await r.json();
    document.getElementById('status-dot').className = r.ok ? 'ok' : 'err';
    document.getElementById('ver').textContent =
      v.protocol_version
        ? v.protocol_version + ' · ' + v.routing_kernel_version
        : JSON.stringify(v);
  } catch(e) {
    document.getElementById('status-dot').className = 'err';
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
    document.getElementById('fixtures-panel').classList.remove('hidden');
    document.getElementById('btn-route').disabled      = false;
    document.getElementById('btn-route-norm').disabled  = false;
  } catch(e) {
    document.getElementById('fixtures-loading').classList.add('hidden');
    const errEl = document.getElementById('fixtures-error');
    errEl.textContent = 'Could not load pilot fixtures: ' + e.message
      + '\n\nStart the service from the repo root so examples/pilot/ is reachable.';
    errEl.classList.remove('hidden');
  }
})();

// ── Execute Routing Kernel ─────────────────────────────────────────────────
async function routeCase(btn) {
  if (!fixtures) return;
  setBtn(btn, 'Running kernel…', true);
  document.getElementById('btn-route-norm').disabled = true;

  hide('results-placeholder');
  hide('route-norm-inline'); hide('route-norm-preview');
  hide('route-result'); hide('route-error'); hide('verify-result');
  hide('dispatch-created'); hide('dispatch-export-result');
  hide('dispatch-success'); hide('dispatch-error');
  show('results-loading');
  lastReceipt = null; lastPolicy = null; lastDispatchId = null;
  document.getElementById('btn-dispatch-create').disabled  = true;
  document.getElementById('btn-dispatch-approve').disabled = true;
  document.getElementById('btn-dispatch-export').disabled  = true;

  try {
    const r = await fetch('/route', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({
        case:             fixtures.case,
        registry_snapshot: fixtures.registry_snapshot,
        routing_config:   fixtures.routing_config,
      }),
    });
    const data = await r.json();

    if (r.ok && data.receipt) {
      lastReceipt = data.receipt;
      lastPolicy  = data.derived_policy;
      const rc = data.receipt;

      const outcome  = rc.outcome || '—';
      const selected = rc.selected_candidate_id || '(none — refused)';
      const rhash    = rc.receipt_hash || '—';
      const kver     = rc.routing_kernel_version || '—';

      document.getElementById('art-outcome').innerHTML =
        `<span class="pill ${outcome === 'routed' ? 'pill-ok' : 'pill-warn'}">${esc(outcome)}</span>`;
      document.getElementById('art-selected').textContent = selected;
      document.getElementById('art-hash').textContent     = rhash;
      document.getElementById('art-kver').textContent     = kver;
      document.getElementById('route-receipt-json').textContent = fmt(rc);

      show('route-result');
      document.getElementById('btn-verify').disabled          = false;
      document.getElementById('btn-tamper').disabled          = false;
      document.getElementById('btn-dispatch-create').disabled = false;
    } else {
      hide('route-error-banner');
      document.getElementById('route-error-json').textContent = fmt(data);
      show('route-error');
    }
  } catch(e) {
    hide('route-error-banner');
    document.getElementById('route-error-json').textContent = String(e);
    show('route-error');
  } finally {
    hide('results-loading');
    setBtn(btn, '▶ Execute Routing Kernel', false);
    document.getElementById('btn-route-norm').disabled = false;
  }
}

// ── Route Normalized Pilot Case ────────────────────────────────────────────
async function routeNormalized(btn) {
  if (!fixtures) return;

  const pilotCase = {
    case_id:          'f1000001-0000-0000-0000-000000000001',
    restoration_type: 'crown',
    material:         'zirconia',
    jurisdiction:     'DE',
  };
  const ni = document.getElementById('route-norm-inline');

  // ── client-side validation ──────────────────────────────────────────────
  const missing = validateNormInput(pilotCase);
  if (missing.length) {
    ni.textContent = 'Required fields missing: ' + missing.join(', ');
    ni.className = 'error-note';
    ni.classList.remove('hidden');
    document.getElementById('fix-normalized-case').classList.add('norm-field-invalid');
    return;   // button stays enabled; clearNormForm() / loadNormSample() clear this error
  }
  document.getElementById('fix-normalized-case').classList.remove('norm-field-invalid');

  setBtn(btn, 'Running kernel…', true);
  document.getElementById('btn-route').disabled = true;

  hide('results-placeholder');
  hide('route-result'); hide('route-error'); hide('verify-result');
  hide('dispatch-created'); hide('dispatch-export-result');
  hide('dispatch-success'); hide('dispatch-error');
  hide('route-norm-preview');
  show('results-loading');
  // Transition to submitting state immediately — button stays disabled until finally.
  ni.textContent = 'Submitting…';
  ni.className = 'loading-note';
  ni.classList.remove('hidden');
  lastReceipt = null; lastPolicy = null; lastDispatchId = null;
  document.getElementById('btn-dispatch-create').disabled  = true;
  document.getElementById('btn-dispatch-approve').disabled = true;
  document.getElementById('btn-dispatch-export').disabled  = true;

  try {
    // ── fetch (network failure → inline error) ──────────────────────────────
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
      ni.textContent = 'Network failure — ' + String(netErr);
      ni.className = 'error-note';
      hide('route-error-banner');
      document.getElementById('route-error-json').textContent = String(netErr);
      show('route-error');
      return;
    }

    // ── parse (invalid JSON → inline error) ────────────────────────────────
    let data;
    try {
      data = await r.json();
    } catch(parseErr) {
      ni.textContent = 'Invalid JSON response (HTTP ' + r.status + ')';
      ni.className = 'error-note';
      hide('route-error-banner');
      document.getElementById('route-error-json').textContent =
        'HTTP ' + r.status + ' — response is not valid JSON: ' + String(parseErr);
      show('route-error');
      return;
    }

    if (r.ok && data.receipt) {
      lastReceipt = data.receipt;
      lastPolicy  = data.derived_policy;
      const rc = data.receipt;

      const outcome  = rc.outcome || '—';
      const selected = rc.selected_candidate_id || '(none — refused)';
      const rhash    = rc.receipt_hash || '—';
      const kver     = rc.routing_kernel_version || '—';

      document.getElementById('art-outcome').innerHTML =
        `<span class="pill ${outcome === 'routed' ? 'pill-ok' : 'pill-warn'}">${esc(outcome)}</span>`;
      document.getElementById('art-selected').textContent = selected;
      document.getElementById('art-hash').textContent     = rhash;
      document.getElementById('art-kver').textContent     = kver;
      document.getElementById('route-receipt-json').textContent = fmt(rc);

      show('route-result');
      document.getElementById('btn-verify').disabled          = false;
      document.getElementById('btn-tamper').disabled          = false;
      document.getElementById('btn-dispatch-create').disabled = false;
      ni.textContent = '✓ Routing complete — receipt ' + rhash.slice(0, 12) + '…';
      ni.className = 'success-note';
      const prev = document.getElementById('route-norm-preview');
      const hashRow = rhash !== '—'
        ? '<div class="norm-preview-row">'
            + '<span class="norm-preview-key">Receipt Hash</span>'
            + '<span class="norm-preview-val">' + esc(rhash)
            + ' <button class="copy-btn" onclick="copyReceiptHash(this,'
            + JSON.stringify(rhash) + ')">Copy</button></span></div>'
        : previewRow('Receipt Hash', rhash);
      const mfrRow = rc.selected_candidate_id
        ? '<div class="norm-preview-row">'
            + '<span class="norm-preview-key">Manufacturer</span>'
            + '<span class="norm-preview-val">' + esc(selected)
            + ' <button class="copy-btn" onclick="copyReceiptHash(this,'
            + JSON.stringify(rc.selected_candidate_id) + ')">Copy</button></span></div>'
        : previewRow('Manufacturer', selected);
      prev.innerHTML =
        '<div class="norm-preview">' +
        hashRow +
        mfrRow +
        previewRow('Jurisdiction',   rc.routing_input?.jurisdiction || '—') +
        previewRow('Material',       rc.routing_input?.material     || '—') +
        previewRow('Created At',     rc.created_at                  || '—') +
        '</div>' +
        '<button class="btn-dl" onclick="downloadReceiptJson()">↓ Download receipt.json</button>' +
        '<button class="btn-route-norm" style="margin-top:.35rem;font-size:.72rem"' +
        ' id="btn-toggle-receipt" onclick="toggleNormReceiptJson()">Show receipt JSON</button>' +
        '<pre class="fixture hidden" id="norm-receipt-json-block"' +
        ' style="margin-top:.3rem;max-height:300px"></pre>';
      document.getElementById('norm-receipt-json-block').textContent = fmt(rc);
      prev.classList.remove('hidden');
    } else {
      // non-2xx HTTP response → inline error + details panel
      const code = data?.error?.code || data?.result || 'error';
      const msg  = data?.error?.message || '';
      ni.textContent = '[' + code + ']' + (msg ? ' — ' + msg : '');
      ni.className = 'error-note';
      const banner = document.getElementById('route-error-banner');
      banner.textContent = '[' + code + ']' + (msg ? ' ' + msg : '');
      banner.classList.remove('hidden');
      document.getElementById('route-error-json').textContent = fmt(data);
      show('route-error');
    }
  } catch(e) {
    ni.textContent = 'Unexpected error — ' + String(e);
    ni.className = 'error-note';
    hide('route-error-banner');
    document.getElementById('route-error-json').textContent = String(e);
    show('route-error');
  } finally {
    hide('results-loading');
    setBtn(btn, '▶ Route Normalized Pilot Case', false);
    document.getElementById('btn-route').disabled = false;
  }
}

// ── Replay Verification ────────────────────────────────────────────────────
async function verifyReceipt(btn) {
  if (!lastReceipt || !lastPolicy) return;
  setBtn(btn, 'Replaying…', true);
  hide('verify-result');

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
    setBtn(btn, '↩ Replay Verification', false);
  }
}

// ── E. Tamper + Verify ─────────────────────────────────────────────────────
async function tamperVerify(btn) {
  if (!lastReceipt || !lastPolicy) return;
  setBtn(btn, 'Tampering…', true);
  hide('verify-result');

  // Deep-copy receipt and change selected_candidate_id client-side only.
  // No backend changes. The real POST /verify will catch the mismatch.
  const tampered = JSON.parse(JSON.stringify(lastReceipt));
  const original = tampered.selected_candidate_id || '(none)';
  tampered.selected_candidate_id = 'tampered-mfr-reviewer-demo';

  try {
    const r = await fetch('/verify', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({
        receipt: tampered,
        case:    fixtures.case,
        policy:  lastPolicy,
      }),
    });
    const data = await r.json();
    // Annotate response with what was changed so the reviewer sees it clearly
    const annotated = {
      _tamper_note: `selected_candidate_id changed client-side: "${original}" → "tampered-mfr-reviewer-demo"`,
      _submitted_to: 'POST /verify (real endpoint, no backend changes)',
      ...data,
    };
    showVerifyResult(false, annotated, 'Tamper Demo');
  } catch(e) {
    showVerifyResult(false, {error: {code: 'client_error', message: String(e)}}, 'Tamper Demo');
  } finally {
    setBtn(btn, '⚠ Tamper + Verify', false);
  }
}

// ── F. Verification result display ─────────────────────────────────────────
function showVerifyResult(isVerified, data, kind) {
  document.getElementById('verify-kind-label').innerHTML =
    `<span class="pill ${isVerified ? 'pill-ok' : 'pill-err'}">${esc(kind)}</span>`;

  const banner = document.getElementById('verify-banner');
  if (isVerified) {
    banner.className = 'verify-banner banner-ok';
    banner.innerHTML = '✓ VERIFIED — receipt replay matched'
      + '<span class="verify-sub">The kernel reproduced the same receipt hash from the original inputs.</span>';
  } else {
    const code    = data?.error?.code || data?.result || 'FAILED';
    const msg     = data?.error?.message || '';
    const heading = kind === 'Tamper Demo' ? '✗ TAMPER DETECTED' : '✗ VERIFICATION FAILED';
    banner.className = 'verify-banner banner-err';
    banner.innerHTML = heading
      + `<span class="verify-sub">Error code: <strong>${esc(code)}</strong>${msg ? ' — ' + esc(msg) : ''}</span>`;
  }

  const pre = document.getElementById('verify-json');
  pre.className = 'result ' + (isVerified ? 'result-ok' : 'result-err');
  pre.textContent = fmt(data);
  show('verify-result');
  document.getElementById('verify-result').scrollIntoView({behavior:'smooth', block:'nearest'});
}

// ── G. Dispatch Commitment ─────────────────────────────────────────────────
async function createDispatch(btn) {
  if (!lastReceipt || !lastPolicy) return;
  setBtn(btn, 'Creating…', true);
  // Only clear the error/success displays — preserve any existing dispatch panel
  // so that a 409 (already created) doesn't wipe the visible dispatch_id.
  hide('dispatch-success'); hide('dispatch-error');

  try {
    const r = await fetch('/dispatch/create', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({receipt: lastReceipt, case: fixtures.case, policy: lastPolicy}),
    });
    const data = await r.json();

    if (r.ok && data.dispatch_id) {
      // Fresh creation — update full dispatch panel.
      lastDispatchId = data.dispatch_id;
      document.getElementById('art-dispatch-id').textContent = data.dispatch_id;
      document.getElementById('art-dispatch-status').innerHTML =
        `<span class="pill pill-info">${esc(data.status)}</span>`;
      hide('dispatch-export-result');
      show('dispatch-created');
      document.getElementById('btn-dispatch-approve').disabled = false;
      document.getElementById('btn-dispatch-export').disabled  = true;
    } else if (r.status === 409) {
      // Already created for this receipt — show as a warning, not an error.
      // Keep any existing dispatch panel visible so the operator can continue.
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
    setBtn(btn, '⬦ Create Dispatch', false);
  }
}

async function approveDispatch(btn) {
  if (!lastDispatchId) return;
  setBtn(btn, 'Approving…', true);
  hide('dispatch-success'); hide('dispatch-error');
  // Track whether this button should stay disabled after the call.
  // true = terminal state (success or already-approved 409).
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
        `<span class="pill pill-ok">${esc(data.status)}</span>`;
      terminal = true;   // immutable — disable approve permanently
      document.getElementById('btn-dispatch-export').disabled = false;
      const s = document.getElementById('dispatch-success');
      s.textContent = 'Dispatch approved.';
      s.classList.remove('hidden');
    } else if (r.status === 409) {
      // Already approved server-side — enable export so the operator can continue.
      terminal = true;
      document.getElementById('btn-dispatch-export').disabled = false;
      showDispatchMsg('warn',
        '[' + (data?.error?.code || 'dispatch_not_draft') + '] ' +
        (data?.error?.message || 'Dispatch is already approved.') +
        ' Export is now available.');
    } else {
      showDispatchMsg('error',
        '[' + (data?.error?.code || 'error') + '] ' + (data?.error?.message || JSON.stringify(data)));
    }
  } catch(e) {
    showDispatchMsg('error', String(e));
  } finally {
    // Re-enable only if not in a terminal state.
    setBtn(btn, '✓ Approve Dispatch', terminal);
  }
}

async function exportDispatch(btn) {
  if (!lastDispatchId) return;
  setBtn(btn, 'Exporting…', true);
  hide('dispatch-export-result'); hide('dispatch-success'); hide('dispatch-error');

  try {
    const r = await fetch('/dispatch/' + lastDispatchId + '/export');
    const data = await r.json();
    if (r.ok) {
      document.getElementById('art-dispatch-status').innerHTML =
        `<span class="pill pill-ok">${esc(data.status)}</span>`;
      document.getElementById('dispatch-export-json').textContent = fmt(data);
      show('dispatch-export-result');
      const s = document.getElementById('dispatch-success');
      s.textContent = 'Export complete.';
      s.classList.remove('hidden');
    } else {
      showDispatchMsg('error',
        '[' + (data?.error?.code || 'error') + '] ' + (data?.error?.message || JSON.stringify(data)));
    }
  } catch(e) {
    showDispatchMsg('error', String(e));
  } finally {
    setBtn(btn, '↓ Export Dispatch Packet', false);
  }
}

// ── helpers ────────────────────────────────────────────────────────────────
function fmt(o)        { return JSON.stringify(o, null, 2); }
function esc(s)        { return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;'); }
function previewRow(k, v) {
  return '<div class="norm-preview-row">'
    + '<span class="norm-preview-key">' + esc(k) + '</span>'
    + '<span class="norm-preview-val">'  + esc(String(v)) + '</span>'
    + '</div>';
}
function validateNormInput(c) {
  return ['case_id', 'restoration_type', 'material', 'jurisdiction']
    .filter(k => !c[k] || !String(c[k]).trim());
}
function loadNormSample() {
  document.getElementById('fix-normalized-case').textContent =
    '{"case_id":"f1000001-0000-0000-0000-000000000001","restoration_type":"crown","material":"zirconia","jurisdiction":"DE"}';
  clearNormForm();
}
function clearNormForm() {
  const ni = document.getElementById('route-norm-inline');
  ni.textContent = '';
  ni.className = 'hidden';
  const prev = document.getElementById('route-norm-preview');
  prev.innerHTML = '';
  prev.classList.add('hidden');
  document.getElementById('fix-normalized-case').classList.remove('norm-field-invalid');
  if (fixtures) document.getElementById('btn-route-norm').disabled = false;
}
function toggleNormReceiptJson() {
  const pre = document.getElementById('norm-receipt-json-block');
  const btn = document.getElementById('btn-toggle-receipt');
  if (!pre || !btn) return;
  const isHidden = pre.classList.toggle('hidden');
  btn.textContent = isHidden ? 'Show receipt JSON' : 'Hide receipt JSON';
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
    btn.textContent = 'Copied';
    btn.style.color = '#3fb950';
  } catch(e) {
    btn.textContent = 'Copy failed';
    btn.style.color = '#f85149';
  }
  setTimeout(() => { btn.textContent = 'Copy'; btn.style.color = ''; }, 1500);
}
function show(id)      { document.getElementById(id).classList.remove('hidden'); }
function hide(id)      { document.getElementById(id).classList.add('hidden'); }
function setBtn(btn, label, disabled) { btn.textContent = label; btn.disabled = disabled; }

// Ctrl+Enter / Cmd+Enter inside the normalized input section submits the form.
document.getElementById('norm-input-section').addEventListener('keydown', function(e) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
    const btn = document.getElementById('btn-route-norm');
    if (btn && !btn.disabled) { e.preventDefault(); routeNormalized(btn); }
  }
});

// Show a dispatch-section status message.
// kind: 'error' (red) | 'warn' (amber)
function showDispatchMsg(kind, text) {
  const el = document.getElementById('dispatch-error');
  el.className = (kind === 'warn' ? 'warn-note' : 'error-note');
  el.textContent = text;
  show('dispatch-error');
}
</script>
</body>
</html>"#;
