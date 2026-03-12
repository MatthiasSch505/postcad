//! Minimal reviewer shell — single page, two actions, real kernel output.
//!
//! Served at `GET /reviewer`. Auto-loads pilot fixtures from `examples/pilot/`
//! via `GET /pilot-fixtures`. Calls `POST /route` and `POST /verify` — the same
//! real pilot endpoints used by run_pilot.sh. No mock data, no fake outputs.

pub const REVIEWER_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PostCAD Reviewer</title>
<style>
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
body{font-family:ui-monospace,"Cascadia Mono","Menlo",monospace;
     background:#0f1117;color:#c9d1d9;min-height:100vh;font-size:13px}

header{background:#161b22;border-bottom:1px solid #30363d;
       padding:.55rem 1.2rem;display:flex;align-items:center;gap:.75rem}
h1{font-size:.95rem;font-weight:700;color:#f0f6fc}
#ver{margin-left:auto;font-size:.7rem;color:#6e7681}
#status-dot{width:7px;height:7px;border-radius:50%;background:#d29922;flex-shrink:0}
#status-dot.ok{background:#3fb950}
#status-dot.err{background:#f85149}

main{max-width:1100px;margin:1.2rem auto;padding:0 1rem;display:grid;gap:1rem}

.card{background:#161b22;border:1px solid #30363d;border-radius:8px;padding:.9rem 1.1rem}
.card-title{font-size:.68rem;font-weight:700;color:#8b949e;letter-spacing:.07em;
            text-transform:uppercase;margin-bottom:.65rem}

/* overview */
.overview-grid{display:grid;grid-template-columns:1fr 1fr 1fr;gap:.75rem}
@media(max-width:640px){.overview-grid{grid-template-columns:1fr}}
.ov-box{background:#0d1117;border:1px solid #21262d;border-radius:6px;padding:.65rem .8rem}
.ov-label{font-size:.65rem;color:#8b949e;text-transform:uppercase;letter-spacing:.06em;margin-bottom:.3rem}
.ov-text{font-size:.78rem;color:#c9d1d9;line-height:1.55}

/* two-col layout */
.two-col{display:grid;grid-template-columns:340px 1fr;gap:1rem}
@media(max-width:800px){.two-col{grid-template-columns:1fr}}

/* inputs panel */
details{margin-bottom:.5rem}
summary{font-size:.72rem;color:#58a6ff;cursor:pointer;padding:.2rem 0;user-select:none}
summary:hover{color:#79c0ff}
pre.fixture{background:#0d1117;border:1px solid #21262d;border-radius:5px;
            padding:.45rem .65rem;font-size:.68rem;overflow-x:auto;
            white-space:pre-wrap;word-break:break-all;max-height:180px;
            overflow-y:auto;line-height:1.45;margin-top:.3rem;color:#8b949e}

/* buttons */
.btn{display:inline-flex;align-items:center;gap:.3rem;padding:.38rem .9rem;
     font-family:inherit;font-size:.78rem;border-radius:5px;
     border:1px solid transparent;cursor:pointer;transition:opacity .12s;font-weight:600}
.btn:hover:not(:disabled){opacity:.82}
.btn:disabled{opacity:.4;cursor:default}
.btn-route  {background:#238636;border-color:#2ea043;color:#fff;width:100%;
             justify-content:center;margin-top:.4rem}
.btn-verify {background:#6e40c9;border-color:#8957e5;color:#fff;width:100%;
             justify-content:center;margin-top:.6rem}
.btn-sm{padding:.18rem .55rem;font-size:.7rem;width:auto;justify-content:initial}

/* result summary */
.kv-grid{display:grid;grid-template-columns:max-content 1fr;gap:.2rem .75rem;
         align-items:baseline;margin:.5rem 0}
.kv-key{font-size:.68rem;color:#8b949e;white-space:nowrap}
.kv-val{font-size:.78rem;color:#f0f6fc;word-break:break-all}
.kv-hash{font-size:.7rem;color:#58a6ff;word-break:break-all}
.pill{display:inline-block;padding:.06rem .4rem;border-radius:3px;
      font-size:.68rem;font-weight:700}
.pill-ok   {background:#1a3e2c;color:#3fb950}
.pill-err  {background:#3d1f1f;color:#f85149}
.pill-info {background:#1e2d45;color:#58a6ff}
.pill-warn {background:#2d2009;color:#d29922}

pre.result{background:#0d1117;border:1px solid #21262d;border-radius:5px;
           padding:.5rem .7rem;font-size:.68rem;overflow-x:auto;
           white-space:pre-wrap;word-break:break-all;max-height:360px;
           overflow-y:auto;line-height:1.45;margin-top:.5rem}
pre.result-ok  {border-left:3px solid #3fb950}
pre.result-err {border-left:3px solid #f85149}
pre.result-info{border-left:3px solid #388bfd}

.verify-banner{border-radius:6px;padding:.6rem .9rem;font-size:.85rem;
               font-weight:700;margin-top:.75rem;text-align:center}
.banner-ok  {background:#1a3e2c;color:#3fb950;border:1px solid #2ea043}
.banner-err {background:#3d1f1f;color:#f85149;border:1px solid #f85149}

.dimmed{color:#6e7681;font-size:.75rem}
.hidden{display:none!important}
.section-title{font-size:.68rem;font-weight:700;color:#8b949e;text-transform:uppercase;
               letter-spacing:.06em;margin:.75rem 0 .35rem}
.error-note{font-size:.75rem;color:#f85149;margin-top:.4rem}
</style>
</head>
<body>

<header>
  <span id="status-dot"></span>
  <h1>PostCAD · Reviewer Shell</h1>
  <span id="ver">loading…</span>
</header>

<main>

  <!-- 1. Overview -->
  <div class="card">
    <div class="card-title">What this is</div>
    <div class="overview-grid">
      <div class="ov-box">
        <div class="ov-label">Operator side — Route</div>
        <div class="ov-text">Submit a dental CAD case with a manufacturer registry.
          The routing kernel selects an eligible manufacturer and
          produces a cryptographically committed receipt.</div>
      </div>
      <div class="ov-box">
        <div class="ov-label">Verifier side — Verify</div>
        <div class="ov-text">Given the same case + policy inputs, replay the routing
          decision. The receipt hash must match exactly. No trust in
          the server state — only in the inputs and the kernel.</div>
      </div>
      <div class="ov-box">
        <div class="ov-label">The guarantee</div>
        <div class="ov-text">Deterministic: same inputs always produce the same
          receipt hash. Every decision carries a reason code.
          The audit chain is hash-linked and append-only.</div>
      </div>
    </div>
  </div>

  <!-- 2. Two-column: inputs + results -->
  <div class="two-col">

    <!-- LEFT: Inputs -->
    <div class="card">
      <div class="card-title">Pilot inputs — examples/pilot/</div>
      <p id="fixtures-loading" class="dimmed">Loading fixtures…</p>
      <div id="fixtures-panel" class="hidden">
        <details open>
          <summary>case.json</summary>
          <pre class="fixture" id="fix-case"></pre>
        </details>
        <details>
          <summary>registry_snapshot.json</summary>
          <pre class="fixture" id="fix-registry"></pre>
        </details>
        <details>
          <summary>config.json (routing config)</summary>
          <pre class="fixture" id="fix-config"></pre>
        </details>
      </div>
      <div id="fixtures-error" class="hidden error-note"></div>

      <button class="btn btn-route" id="btn-route" onclick="routeCase(this)" disabled>
        Route Case →
      </button>
    </div>

    <!-- RIGHT: Results -->
    <div class="card">
      <div id="results-placeholder" class="dimmed" style="padding:.5rem 0">
        Run "Route Case" to see real kernel output.
      </div>

      <!-- Routing summary -->
      <div id="route-result" class="hidden">
        <div class="card-title">Routing result</div>
        <div class="kv-grid" id="route-kv"></div>

        <div class="section-title">Full receipt JSON</div>
        <pre class="result result-ok" id="route-receipt-json"></pre>

        <button class="btn btn-verify" id="btn-verify" onclick="verifyReceipt(this)">
          Verify Receipt ↩
        </button>
      </div>

      <!-- Route error -->
      <div id="route-error" class="hidden">
        <div class="card-title">Route error</div>
        <pre class="result result-err" id="route-error-json"></pre>
      </div>

      <!-- Verify result -->
      <div id="verify-result" class="hidden">
        <div class="section-title">Verification result</div>
        <div id="verify-banner"></div>
        <pre class="result" id="verify-json"></pre>
      </div>
    </div>

  </div>

</main>

<script>
// ── state ──────────────────────────────────────────────────────────────────
let fixtures = null;   // {case, registry_snapshot, routing_config}
let lastReceipt = null;
let lastPolicy  = null;

// ── boot ───────────────────────────────────────────────────────────────────
(async function boot() {
  // Version
  try {
    const r = await fetch('/version');
    const v = await r.json();
    const dot = document.getElementById('status-dot');
    dot.className = r.ok ? 'ok' : 'err';
    document.getElementById('ver').textContent =
      v.protocol_version
        ? v.protocol_version + ' · ' + v.routing_kernel_version
        : JSON.stringify(v);
  } catch(e) {
    document.getElementById('status-dot').className = 'err';
    document.getElementById('ver').textContent = 'service unreachable';
  }

  // Fixtures
  try {
    const r = await fetch('/pilot-fixtures');
    if (!r.ok) throw new Error('HTTP ' + r.status + ': ' + await r.text());
    fixtures = await r.json();
    document.getElementById('fix-case').textContent     = fmt(fixtures.case);
    document.getElementById('fix-registry').textContent = fmt(fixtures.registry_snapshot);
    document.getElementById('fix-config').textContent   = fmt(fixtures.routing_config);
    document.getElementById('fixtures-loading').classList.add('hidden');
    document.getElementById('fixtures-panel').classList.remove('hidden');
    document.getElementById('btn-route').disabled = false;
  } catch(e) {
    document.getElementById('fixtures-loading').classList.add('hidden');
    const errEl = document.getElementById('fixtures-error');
    errEl.textContent = 'Could not load pilot fixtures: ' + e.message +
      '\n\nStart the service from the repo root so examples/pilot/ is reachable.';
    errEl.classList.remove('hidden');
  }
})();

// ── Route Case ─────────────────────────────────────────────────────────────
async function routeCase(btn) {
  if (!fixtures) return;
  const orig = btn.textContent;
  btn.disabled = true; btn.textContent = 'Routing…';

  // Reset results
  hide('results-placeholder');
  hide('route-result'); hide('route-error');
  hide('verify-result');
  lastReceipt = null; lastPolicy = null;

  try {
    const r = await fetch('/route', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({
        case: fixtures.case,
        registry_snapshot: fixtures.registry_snapshot,
        routing_config: fixtures.routing_config,
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

      document.getElementById('route-kv').innerHTML =
        kv('outcome',        pill(outcome, outcome === 'routed' ? 'pill-ok' : 'pill-warn')) +
        kv('selected',       esc(selected)) +
        kv('receipt_hash',   `<span class="kv-hash">${esc(rhash)}</span>`) +
        kv('kernel_version', esc(kver));

      document.getElementById('route-receipt-json').textContent = fmt(rc);
      show('route-result');
      document.getElementById('btn-verify').disabled = false;
    } else {
      document.getElementById('route-error-json').textContent = fmt(data);
      show('route-error');
    }
  } catch(e) {
    document.getElementById('route-error-json').textContent = String(e);
    show('route-error');
  } finally {
    btn.disabled = false; btn.textContent = orig;
  }
}

// ── Verify Receipt ─────────────────────────────────────────────────────────
async function verifyReceipt(btn) {
  if (!lastReceipt || !lastPolicy) return;
  const orig = btn.textContent;
  btn.disabled = true; btn.textContent = 'Verifying…';
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
    const isVerified = r.ok && data.result === 'VERIFIED';

    const banner = document.getElementById('verify-banner');
    banner.className = 'verify-banner ' + (isVerified ? 'banner-ok' : 'banner-err');
    banner.textContent = isVerified ? '✓ VERIFIED — receipt replay matched' : '✗ FAILED — ' + (data.error?.code || 'unknown');

    const pre = document.getElementById('verify-json');
    pre.className = 'result ' + (isVerified ? 'result-ok' : 'result-err');
    pre.textContent = fmt(data);
    show('verify-result');
  } catch(e) {
    const pre = document.getElementById('verify-json');
    pre.className = 'result result-err';
    pre.textContent = String(e);
    show('verify-result');
  } finally {
    btn.disabled = false; btn.textContent = orig;
  }
}

// ── helpers ────────────────────────────────────────────────────────────────
function fmt(o)   { return JSON.stringify(o, null, 2); }
function esc(s)   { return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;'); }
function show(id) { document.getElementById(id).classList.remove('hidden'); }
function hide(id) { document.getElementById(id).classList.add('hidden'); }
function pill(t, cls) { return `<span class="pill ${cls}">${esc(t)}</span>`; }
function kv(k, vHtml) {
  return `<span class="kv-key">${esc(k)}</span><span class="kv-val">${vHtml}</span>`;
}
</script>
</body>
</html>"#;
