//! Embedded operator UI HTML.
//!
//! A single static string served at `GET /`. The page calls only existing
//! service endpoints via `fetch`; no backend logic lives here.

/// Full single-page operator UI, embedded at compile time.
pub const OPERATOR_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PostCAD Operator UI</title>
<style>
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
  body { font-family: ui-monospace, "Cascadia Mono", "Menlo", monospace;
         background: #0f1117; color: #c9d1d9; min-height: 100vh; }
  a { color: #58a6ff; }
  h1 { font-size: 1.1rem; font-weight: 600; color: #f0f6fc; }
  h2 { font-size: 0.85rem; font-weight: 600; color: #8b949e; letter-spacing: .06em;
       text-transform: uppercase; margin-bottom: .75rem; }
  header { background: #161b22; border-bottom: 1px solid #30363d;
           padding: .6rem 1.2rem; display: flex; align-items: center; gap: 1rem; }
  #status-bar { margin-left: auto; font-size: .75rem; color: #8b949e; display: flex; gap: 1rem; }
  #status-bar span { display: flex; align-items: center; gap: .3rem; }
  .dot { width: 7px; height: 7px; border-radius: 50%; background: #3fb950; }
  .dot.err { background: #f85149; }
  main { max-width: 1080px; margin: 1.5rem auto; padding: 0 1rem;
         display: grid; gap: 1.25rem; }
  .card { background: #161b22; border: 1px solid #30363d; border-radius: 8px;
          padding: 1rem 1.25rem; }
  label { font-size: .8rem; color: #8b949e; display: block; margin-bottom: .3rem; }
  textarea, input[type=text] {
    width: 100%; background: #0d1117; border: 1px solid #30363d; border-radius: 5px;
    color: #c9d1d9; font-family: inherit; font-size: .78rem; padding: .5rem .7rem;
    resize: vertical; }
  textarea:focus, input:focus { outline: none; border-color: #58a6ff; }
  .btn { display: inline-block; padding: .35rem .8rem; font-family: inherit;
         font-size: .78rem; border-radius: 5px; border: 1px solid transparent;
         cursor: pointer; transition: opacity .15s; }
  .btn:hover { opacity: .85; }
  .btn-primary { background: #238636; border-color: #2ea043; color: #fff; }
  .btn-secondary { background: #21262d; border-color: #30363d; color: #c9d1d9; }
  .btn-dispatch { background: #1f6feb; border-color: #388bfd; color: #fff; }
  .btn-verify { background: #6e40c9; border-color: #8957e5; color: #fff; }
  .btn-route { background: #b08800; border-color: #d29922; color: #fff; }
  .btn-sm { padding: .2rem .55rem; font-size: .72rem; }
  pre { background: #0d1117; border: 1px solid #21262d; border-radius: 5px;
        padding: .6rem .8rem; font-size: .72rem; overflow-x: auto;
        white-space: pre-wrap; word-break: break-all; max-height: 320px;
        overflow-y: auto; margin-top: .5rem; }
  .result-ok  { border-left: 3px solid #3fb950; }
  .result-err { border-left: 3px solid #f85149; }
  .result-info { border-left: 3px solid #388bfd; }
  table { width: 100%; border-collapse: collapse; font-size: .78rem; }
  th { text-align: left; color: #8b949e; font-weight: 600; padding: .4rem .5rem;
       border-bottom: 1px solid #21262d; }
  td { padding: .4rem .5rem; border-bottom: 1px solid #161b22; vertical-align: top; }
  tr:hover td { background: #1c2128; }
  .hash { font-size: .7rem; color: #8b949e; word-break: break-all; }
  .badge { display: inline-block; padding: .1rem .4rem; border-radius: 3px;
           font-size: .68rem; font-weight: 600; }
  .badge-routed   { background: #1a3e2c; color: #3fb950; }
  .badge-refused  { background: #3d1f1f; color: #f85149; }
  .badge-verified { background: #1e2d45; color: #58a6ff; }
  .badge-invalid  { background: #3d1f1f; color: #f85149; }
  .badge-dispatched { background: #2d2500; color: #d29922; }
  .row-actions { display: flex; gap: .35rem; flex-wrap: wrap; }
  .section-header { display: flex; align-items: center; gap: .75rem; margin-bottom: .75rem; }
  .section-header h2 { margin: 0; }
  .spacer { flex: 1; }
  #route-modal-overlay { display: none; position: fixed; inset: 0;
    background: rgba(0,0,0,.6); z-index: 100; align-items: center; justify-content: center; }
  #route-modal-overlay.open { display: flex; }
  #route-modal { background: #161b22; border: 1px solid #30363d; border-radius: 8px;
                 padding: 1.25rem; width: 560px; max-width: 95vw; max-height: 90vh;
                 overflow-y: auto; }
  #route-modal h2 { margin-bottom: .75rem; }
  .modal-actions { display: flex; gap: .5rem; justify-content: flex-end; margin-top: .75rem; }
  .hidden { display: none; }
  .empty-note { font-size: .78rem; color: #8b949e; padding: .5rem 0; }
</style>
</head>
<body>

<header>
  <h1>⬡ PostCAD Operator</h1>
  <div id="status-bar">
    <span><span class="dot" id="health-dot"></span><span id="health-label">—</span></span>
    <span id="version-label" style="color:#8b949e">—</span>
  </div>
</header>

<main>

  <!-- A. Case Intake -->
  <div class="card" id="section-intake">
    <div class="section-header">
      <h2>A · Case Intake</h2>
      <button class="btn btn-secondary btn-sm" onclick="loadCases()">↻ Refresh cases</button>
    </div>
    <label for="case-json-input">Case JSON</label>
    <textarea id="case-json-input" rows="7" placeholder='{"case_id":"...","patient_country":"germany","manufacturer_country":"germany","material":"zirconia","procedure":"crown","file_type":"stl"}'></textarea>
    <div style="display:flex;gap:.5rem;margin-top:.6rem;align-items:center;">
      <button class="btn btn-primary" onclick="submitCase()">POST /cases</button>
      <span id="intake-status" style="font-size:.75rem;color:#8b949e;"></span>
    </div>
    <pre id="intake-result" class="hidden"></pre>
  </div>

  <!-- B. Cases -->
  <div class="card" id="section-cases">
    <div class="section-header">
      <h2>B · Cases</h2>
      <span class="spacer"></span>
      <button class="btn btn-secondary btn-sm" onclick="loadCases()">↻ Refresh</button>
    </div>
    <div id="cases-content"><p class="empty-note">Loading…</p></div>
  </div>

  <!-- C. Receipts -->
  <div class="card" id="section-receipts">
    <div class="section-header">
      <h2>C · Receipts</h2>
      <span class="spacer"></span>
      <button class="btn btn-secondary btn-sm" onclick="loadReceipts()">↻ Refresh</button>
    </div>
    <div id="receipts-content"><p class="empty-note">Loading…</p></div>
    <pre id="receipt-detail" class="hidden result-info"></pre>
  </div>

  <!-- D. Route History -->
  <div class="card" id="section-history">
    <div class="section-header">
      <h2>D · Route History</h2>
      <span class="spacer"></span>
      <button class="btn btn-secondary btn-sm" onclick="loadHistory()">↻ Refresh</button>
    </div>
    <div id="history-content"><p class="empty-note">Loading…</p></div>
  </div>

  <!-- E. Status -->
  <div class="card" id="section-status">
    <h2>E · Service Status</h2>
    <pre id="status-detail" class="result-info" style="margin-top:.5rem"></pre>
  </div>

</main>

<!-- Route modal -->
<div id="route-modal-overlay">
  <div id="route-modal">
    <h2>Route case <span id="route-modal-case-id" style="color:#58a6ff;font-size:.85rem;"></span></h2>
    <label for="route-registry-input">Registry snapshot JSON (array)</label>
    <textarea id="route-registry-input" rows="6" placeholder='[{"manufacturer_id":"mfr-de-001","...":"..."}]'></textarea>
    <label for="route-config-input" style="margin-top:.6rem;">Routing config JSON</label>
    <textarea id="route-config-input" rows="4" placeholder='{"jurisdiction":"DE","routing_policy":"allow_domestic_and_cross_border"}'></textarea>
    <div class="modal-actions">
      <button class="btn btn-secondary" onclick="closeRouteModal()">Cancel</button>
      <button class="btn btn-route" onclick="submitRoute()">POST /cases/:id/route</button>
    </div>
    <pre id="route-modal-result" class="hidden" style="margin-top:.75rem;"></pre>
  </div>
</div>

<script>
// ── Utility ──────────────────────────────────────────────────────────────────

function fmt(obj) { return JSON.stringify(obj, null, 2); }

function showPre(el, text, cls) {
  el.className = 'pre ' + (cls || '');
  el.textContent = text;
  el.classList.remove('hidden');
}

async function api(method, path, body) {
  const opts = { method, headers: {} };
  if (body !== undefined) {
    opts.body = JSON.stringify(body);
    opts.headers['Content-Type'] = 'application/json';
  }
  const r = await fetch(path, opts);
  const text = await r.text();
  let json;
  try { json = JSON.parse(text); } catch { json = { _raw: text }; }
  return { status: r.status, ok: r.ok, json };
}

function badge(label, cls) {
  return `<span class="badge ${cls}">${label}</span>`;
}

function escapedHash(h) {
  return `<span class="hash" title="${h}">${h.slice(0,16)}…</span>`;
}

// ── Status (E) ────────────────────────────────────────────────────────────────

async function loadStatus() {
  try {
    const [h, v] = await Promise.all([api('GET','/health'), api('GET','/version')]);
    const dot = document.getElementById('health-dot');
    const label = document.getElementById('health-label');
    if (h.ok) {
      dot.className = 'dot';
      label.textContent = 'healthy';
    } else {
      dot.className = 'dot err';
      label.textContent = 'unreachable';
    }
    document.getElementById('version-label').textContent =
      v.json.protocol_version ? `${v.json.protocol_version} · ${v.json.routing_kernel_version}` : '—';
    document.getElementById('status-detail').textContent =
      fmt({ health: h.json, version: v.json });
  } catch(e) {
    document.getElementById('health-dot').className = 'dot err';
    document.getElementById('health-label').textContent = 'error';
  }
}

// ── Case intake (A) ───────────────────────────────────────────────────────────

async function submitCase() {
  const raw = document.getElementById('case-json-input').value.trim();
  const el = document.getElementById('intake-result');
  const st = document.getElementById('intake-status');
  if (!raw) { st.textContent = 'paste case JSON first'; return; }
  let body;
  try { body = JSON.parse(raw); } catch(e) { st.textContent = 'invalid JSON'; return; }
  st.textContent = 'posting…';
  const r = await api('POST', '/cases', body);
  const cls = r.ok ? 'result-ok' : 'result-err';
  showPre(el, fmt(r.json), cls);
  st.textContent = `HTTP ${r.status}`;
  if (r.ok) loadCases();
}

// ── Cases (B) ─────────────────────────────────────────────────────────────────

async function loadCases() {
  const r = await api('GET', '/cases');
  const el = document.getElementById('cases-content');
  if (!r.ok) { el.innerHTML = `<pre class="result-err">${fmt(r.json)}</pre>`; return; }
  const ids = r.json.case_ids || [];
  if (ids.length === 0) { el.innerHTML = '<p class="empty-note">No cases stored yet.</p>'; return; }
  let rows = ids.map(id => `
    <tr>
      <td><code>${id}</code></td>
      <td><div class="row-actions">
        <button class="btn btn-secondary btn-sm" onclick="viewCase('${id}')">View</button>
        <button class="btn btn-route btn-sm" onclick="openRouteModal('${id}')">Route →</button>
      </div></td>
    </tr>`).join('');
  el.innerHTML = `<table><thead><tr><th>case_id</th><th>actions</th></tr></thead><tbody>${rows}</tbody></table>
    <pre id="case-detail" class="hidden result-info" style="margin-top:.5rem"></pre>`;
}

async function viewCase(id) {
  const r = await api('GET', `/cases/${id}`);
  const el = document.getElementById('case-detail');
  showPre(el, fmt(r.json), r.ok ? 'result-info' : 'result-err');
}

// ── Route modal ───────────────────────────────────────────────────────────────

let _routeCaseId = null;

function openRouteModal(caseId) {
  _routeCaseId = caseId;
  document.getElementById('route-modal-case-id').textContent = caseId;
  document.getElementById('route-modal-result').classList.add('hidden');
  document.getElementById('route-modal-overlay').classList.add('open');
}

function closeRouteModal() {
  document.getElementById('route-modal-overlay').classList.remove('open');
  _routeCaseId = null;
}

async function submitRoute() {
  const regRaw = document.getElementById('route-registry-input').value.trim();
  const cfgRaw = document.getElementById('route-config-input').value.trim();
  const el = document.getElementById('route-modal-result');
  if (!regRaw || !cfgRaw) { showPre(el, 'registry and config JSON required', 'result-err'); return; }
  let registry, config;
  try { registry = JSON.parse(regRaw); config = JSON.parse(cfgRaw); }
  catch(e) { showPre(el, 'invalid JSON: ' + e.message, 'result-err'); return; }
  const r = await api('POST', `/cases/${_routeCaseId}/route`, { registry, config });
  showPre(el, fmt(r.json), r.ok ? 'result-ok' : 'result-err');
  if (r.ok) { loadReceipts(); loadHistory(); }
}

// ── Receipts (C) ──────────────────────────────────────────────────────────────

async function loadReceipts() {
  const r = await api('GET', '/receipts');
  const el = document.getElementById('receipts-content');
  if (!r.ok) { el.innerHTML = `<pre class="result-err">${fmt(r.json)}</pre>`; return; }
  const hashes = r.json.receipts || [];
  if (hashes.length === 0) { el.innerHTML = '<p class="empty-note">No receipts stored yet.</p>'; return; }
  let rows = hashes.map(h => `
    <tr>
      <td>${escapedHash(h)}</td>
      <td><div class="row-actions">
        <button class="btn btn-secondary btn-sm" onclick="viewReceipt('${h}')">View</button>
        <button class="btn btn-dispatch btn-sm" onclick="dispatchReceipt('${h}')">Dispatch</button>
        <button class="btn btn-verify btn-sm" onclick="verifyReceipt('${h}')">Verify</button>
      </div></td>
    </tr>`).join('');
  el.innerHTML = `<table><thead><tr><th>receipt_hash</th><th>actions</th></tr></thead><tbody>${rows}</tbody></table>`;
}

async function viewReceipt(hash) {
  const r = await api('GET', `/receipts/${hash}`);
  const el = document.getElementById('receipt-detail');
  showPre(el, fmt(r.json), r.ok ? 'result-info' : 'result-err');
  el.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
}

async function dispatchReceipt(hash) {
  const r = await api('POST', `/dispatch/${hash}`);
  const el = document.getElementById('receipt-detail');
  showPre(el, fmt(r.json), r.ok ? 'result-ok' : 'result-err');
}

async function verifyReceipt(hash) {
  const r = await api('POST', `/dispatch/${hash}/verify`);
  const el = document.getElementById('receipt-detail');
  const result = r.json.result;
  let cls = 'result-info';
  if (result === 'VERIFIED') cls = 'result-ok';
  if (result === 'INVALID')  cls = 'result-err';
  showPre(el, fmt(r.json), cls);
}

// ── Route History (D) ─────────────────────────────────────────────────────────

async function loadHistory() {
  const r = await api('GET', '/routes');
  const el = document.getElementById('history-content');
  if (!r.ok) { el.innerHTML = `<pre class="result-err">${fmt(r.json)}</pre>`; return; }
  const routes = r.json.routes || [];
  if (routes.length === 0) { el.innerHTML = '<p class="empty-note">No routes recorded yet.</p>'; return; }
  let rows = routes.map(e => {
    const outcomeBadge = e.selected_candidate_id
      ? badge('routed','badge-routed')
      : badge('refused','badge-refused');
    return `<tr>
      <td>${outcomeBadge}</td>
      <td><code style="font-size:.72rem">${e.case_id}</code></td>
      <td>${escapedHash(e.receipt_hash)}</td>
      <td><code style="font-size:.72rem">${e.selected_candidate_id || '—'}</code></td>
      <td style="font-size:.7rem;color:#8b949e">${e.timestamp}</td>
    </tr>`;
  }).join('');
  el.innerHTML = `<table><thead><tr>
    <th>outcome</th><th>case_id</th><th>receipt_hash</th>
    <th>manufacturer</th><th>timestamp</th>
  </tr></thead><tbody>${rows}</tbody></table>`;
}

// ── Bootstrap ─────────────────────────────────────────────────────────────────

document.getElementById('route-modal-overlay').addEventListener('click', function(e) {
  if (e.target === this) closeRouteModal();
});

loadStatus();
loadCases();
loadReceipts();
loadHistory();
</script>
</body>
</html>"#;
