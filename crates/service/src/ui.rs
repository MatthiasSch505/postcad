//! Embedded operator UI HTML.
//!
//! A single static string served at `GET /`. The page calls only existing
//! service endpoints via `fetch`; no new backend logic lives here.

/// Full single-page operator UI, embedded at compile time.
pub const OPERATOR_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PostCAD Operator</title>
<style>
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
body{font-family:ui-monospace,"Cascadia Mono","Menlo",monospace;
     background:#0f1117;color:#c9d1d9;min-height:100vh;font-size:14px}
a{color:#58a6ff}

/* ── header ── */
header{background:#161b22;border-bottom:1px solid #30363d;
       padding:.55rem 1.2rem;display:flex;align-items:center;gap:.75rem;
       position:sticky;top:0;z-index:10}
h1{font-size:1rem;font-weight:700;color:#f0f6fc;letter-spacing:-.01em}
#header-status{margin-left:auto;font-size:.72rem;color:#8b949e;
               display:flex;align-items:center;gap:.9rem}
.dot{width:7px;height:7px;border-radius:50%;background:#3fb950;flex-shrink:0}
.dot.err{background:#f85149}
.dot.warn{background:#d29922}
#header-ver{max-width:380px;overflow:hidden;white-space:nowrap;
            text-overflow:ellipsis;color:#6e7681}

/* ── layout ── */
main{max-width:1100px;margin:1.2rem auto;padding:0 1rem;display:grid;gap:1rem}

/* ── card ── */
.card{background:#161b22;border:1px solid #30363d;border-radius:8px;
      padding:.9rem 1.1rem}
.card-header{display:flex;align-items:center;gap:.6rem;margin-bottom:.75rem}
.card-header h2{font-size:.72rem;font-weight:700;color:#8b949e;
                letter-spacing:.07em;text-transform:uppercase;margin:0}
.card-header .spacer{flex:1}

/* ── form ── */
label{font-size:.75rem;color:#8b949e;display:block;margin-bottom:.25rem;
      margin-top:.55rem}
label:first-child{margin-top:0}
textarea,input[type=text]{width:100%;background:#0d1117;border:1px solid #30363d;
  border-radius:5px;color:#c9d1d9;font-family:inherit;font-size:.75rem;
  padding:.45rem .65rem;resize:vertical}
textarea:focus,input:focus{outline:none;border-color:#58a6ff}

/* ── buttons ── */
.btn{display:inline-flex;align-items:center;gap:.3rem;padding:.32rem .75rem;
     font-family:inherit;font-size:.75rem;border-radius:5px;
     border:1px solid transparent;cursor:pointer;transition:opacity .12s;
     white-space:nowrap;font-weight:500}
.btn:hover:not(:disabled){opacity:.82}
.btn:disabled{opacity:.45;cursor:default}
.btn-primary  {background:#238636;border-color:#2ea043;color:#fff}
.btn-secondary{background:#21262d;border-color:#30363d;color:#c9d1d9}
.btn-dispatch {background:#1f6feb;border-color:#388bfd;color:#fff}
.btn-verify   {background:#6e40c9;border-color:#8957e5;color:#fff}
.btn-route    {background:#9e6a03;border-color:#d29922;color:#fff}
.btn-sm{padding:.18rem .5rem;font-size:.7rem}

/* ── result pre ── */
pre{background:#0d1117;border:1px solid #21262d;border-radius:5px;
    padding:.55rem .75rem;font-size:.71rem;overflow-x:auto;
    white-space:pre-wrap;word-break:break-all;max-height:340px;
    overflow-y:auto;line-height:1.5}
.result-label{font-size:.7rem;color:#6e7681;margin-bottom:.25rem;
              margin-top:.65rem;display:flex;align-items:center;gap:.4rem}
.result-label .status-pill{padding:.05rem .35rem;border-radius:3px;
  font-size:.65rem;font-weight:700}
.pill-ok  {background:#1a3e2c;color:#3fb950}
.pill-err {background:#3d1f1f;color:#f85149}
.pill-info{background:#1e2d45;color:#58a6ff}
pre.result-ok  {border-left:3px solid #3fb950}
pre.result-err {border-left:3px solid #f85149}
pre.result-info{border-left:3px solid #388bfd}
.hidden{display:none!important}

/* ── table ── */
table{width:100%;border-collapse:collapse;font-size:.75rem}
th{text-align:left;color:#8b949e;font-weight:600;padding:.35rem .45rem;
   border-bottom:1px solid #21262d;white-space:nowrap}
td{padding:.35rem .45rem;border-bottom:1px solid #1c2128;vertical-align:middle}
tr:last-child td{border-bottom:none}
tr:hover td{background:#1c2128}
.row-actions{display:flex;gap:.3rem;flex-wrap:wrap}

/* ── misc ── */
.badge{display:inline-block;padding:.08rem .4rem;border-radius:3px;
       font-size:.65rem;font-weight:700}
.badge-routed  {background:#1a3e2c;color:#3fb950}
.badge-refused {background:#3d1f1f;color:#f85149}
.badge-verified{background:#1e2d45;color:#58a6ff}
.badge-invalid {background:#3d1f1f;color:#f85149}
.empty-note{font-size:.75rem;color:#6e7681;padding:.4rem 0}
.hash-full{font-size:.69rem;color:#8b949e;word-break:break-all}
.hash-short{font-size:.69rem;color:#8b949e;cursor:pointer;
            border-bottom:1px dashed #30363d}
.hash-short:hover{color:#58a6ff}
code{font-family:inherit;font-size:.73rem}

/* ── route modal ── */
#route-modal-overlay{display:none;position:fixed;inset:0;
  background:rgba(0,0,0,.65);z-index:100;
  align-items:center;justify-content:center}
#route-modal-overlay.open{display:flex}
#route-modal{background:#161b22;border:1px solid #30363d;border-radius:8px;
             padding:1.1rem 1.25rem;width:580px;max-width:96vw;
             max-height:92vh;overflow-y:auto}
.modal-title{font-size:.85rem;font-weight:700;color:#f0f6fc;margin-bottom:.75rem}
.modal-case-id{color:#58a6ff;font-size:.78rem}
.modal-actions{display:flex;gap:.5rem;justify-content:flex-end;margin-top:.75rem}
</style>
</head>
<body>

<header>
  <h1>⬡ PostCAD Operator</h1>
  <div id="header-status">
    <span style="display:flex;align-items:center;gap:.35rem">
      <span class="dot warn" id="health-dot"></span>
      <span id="health-label">connecting…</span>
    </span>
    <span id="header-ver">—</span>
    <button class="btn btn-secondary btn-sm" onclick="refreshAll()" title="Refresh all sections">↻ Refresh all</button>
  </div>
</header>

<main>

  <!-- E. Service Status (shown first — operator sees health before acting) -->
  <div class="card" id="section-status">
    <div class="card-header">
      <h2>Service Status</h2>
      <span class="spacer"></span>
      <button class="btn btn-secondary btn-sm" onclick="loadStatus()">↻ Refresh</button>
    </div>
    <pre id="status-detail" class="result-info" style="margin:0"></pre>
  </div>

  <!-- A. Case Intake -->
  <div class="card" id="section-intake">
    <div class="card-header">
      <h2>Case Intake</h2>
    </div>
    <label for="case-json-input">Case JSON</label>
    <textarea id="case-json-input" rows="6"
      placeholder='{"case_id":"f1000001-0000-0000-0000-000000000001","patient_country":"germany","manufacturer_country":"germany","material":"zirconia","procedure":"crown","file_type":"stl"}'></textarea>
    <div style="display:flex;gap:.5rem;align-items:center;margin-top:.6rem">
      <button class="btn btn-primary" id="btn-store-case" onclick="submitCase(this)">Store Case</button>
      <span id="intake-inline" style="font-size:.72rem;color:#6e7681"></span>
    </div>
    <div id="intake-result-area" class="hidden">
      <div class="result-label" id="intake-result-label"></div>
      <pre id="intake-result"></pre>
    </div>
  </div>

  <!-- B. Cases -->
  <div class="card" id="section-cases">
    <div class="card-header">
      <h2>Cases</h2>
      <span class="spacer"></span>
      <button class="btn btn-secondary btn-sm" onclick="loadCases()">↻ Refresh</button>
    </div>
    <div id="cases-content"><p class="empty-note">Loading…</p></div>
    <!-- Stable case viewer — not recreated on list refresh -->
    <div id="case-viewer-area" class="hidden">
      <div class="result-label" id="case-viewer-label"></div>
      <pre id="case-detail" class="result-info"></pre>
    </div>
  </div>

  <!-- C. Receipts -->
  <div class="card" id="section-receipts">
    <div class="card-header">
      <h2>Receipts</h2>
      <span class="spacer"></span>
      <button class="btn btn-secondary btn-sm" onclick="loadReceipts()">↻ Refresh</button>
    </div>
    <div id="receipts-content"><p class="empty-note">Loading…</p></div>
    <!-- Stable receipt action viewer -->
    <div id="receipt-action-area" class="hidden">
      <div class="result-label" id="receipt-action-label"></div>
      <pre id="receipt-detail"></pre>
    </div>
  </div>

  <!-- D. Route History -->
  <div class="card" id="section-history">
    <div class="card-header">
      <h2>Route History</h2>
      <span class="spacer"></span>
      <button class="btn btn-secondary btn-sm" onclick="loadHistory()">↻ Refresh</button>
    </div>
    <div id="history-content"><p class="empty-note">Loading…</p></div>
  </div>

</main>

<!-- Route modal (inline, no external framework) -->
<div id="route-modal-overlay">
  <div id="route-modal">
    <div class="modal-title">
      Route case
      <span class="modal-case-id" id="route-modal-case-id"></span>
    </div>
    <label for="route-registry-input">Registry snapshot JSON (array of manufacturer records)</label>
    <textarea id="route-registry-input" rows="7"
      placeholder='[{"manufacturer_id":"mfr-de-001","country":"germany","active":true,"supported_materials":["zirconia"],"supported_procedures":["crown"],"compliance_snapshots":[]}]'></textarea>
    <label for="route-config-input">Routing config JSON</label>
    <textarea id="route-config-input" rows="3"
      placeholder='{"jurisdiction":"DE","routing_policy":"allow_domestic_and_cross_border"}'></textarea>
    <div class="modal-actions">
      <button class="btn btn-secondary" onclick="closeRouteModal()">Cancel</button>
      <button class="btn btn-route" id="btn-submit-route" onclick="submitRoute(this)">Route This Case →</button>
    </div>
    <div id="route-modal-result-area" class="hidden" style="margin-top:.75rem">
      <div class="result-label" id="route-modal-result-label"></div>
      <pre id="route-modal-result"></pre>
    </div>
  </div>
</div>

<script>
// ─────────────────────────────────────────────────────────────────────────────
// Utilities
// ─────────────────────────────────────────────────────────────────────────────

function fmt(obj) {
  return JSON.stringify(obj, null, 2);
}

/** Show a result <pre> with correct border colour and remove hidden class. */
function showResult(preEl, text, cls) {
  preEl.className = cls || '';       // FIX: was 'pre ' + cls, which set wrong classes
  preEl.textContent = text;
}

/** Populate a result area (label div + pre) and make it visible. */
function showResultArea(areaEl, labelEl, preEl, labelText, statusPillHtml, text, preCls) {
  labelEl.innerHTML = labelText + ' ' + statusPillHtml;
  showResult(preEl, text, preCls);
  areaEl.classList.remove('hidden');
  preEl.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
}

function pill(label, cls) {
  return `<span class="status-pill ${cls}">${label}</span>`;
}

function badge(label, cls) {
  return `<span class="badge ${cls}">${label}</span>`;
}

/** Format an error response for display. */
function fmtApiError(status, json) {
  const code = json?.error?.code;
  const msg  = json?.error?.message;
  if (code || msg) {
    return `HTTP ${status} · ${code || 'error'}\n${msg || fmt(json)}`;
  }
  return `HTTP ${status}\n${fmt(json)}`;
}

/** Run a fetch with loading state on a button. Restores button on completion. */
async function withBtn(btn, label, fn) {
  if (!btn) { await fn(); return; }
  const orig = btn.textContent;
  btn.disabled = true;
  btn.textContent = label + '…';
  try { await fn(); }
  finally { btn.disabled = false; btn.textContent = orig; }
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

// ─────────────────────────────────────────────────────────────────────────────
// E · Service Status
// ─────────────────────────────────────────────────────────────────────────────

async function loadStatus() {
  try {
    const [h, v] = await Promise.all([api('GET', '/health'), api('GET', '/version')]);
    const dot   = document.getElementById('health-dot');
    const label = document.getElementById('health-label');
    if (h.ok) {
      dot.className = 'dot';
      label.textContent = 'healthy';
    } else {
      dot.className = 'dot err';
      label.textContent = 'unhealthy';
    }
    const ver = v.json;
    document.getElementById('header-ver').textContent =
      ver.protocol_version
        ? `${ver.protocol_version} · ${ver.routing_kernel_version}`
        : JSON.stringify(ver);
    document.getElementById('status-detail').textContent =
      fmt({ health: h.json, version: v.json });
  } catch(e) {
    document.getElementById('health-dot').className = 'dot err';
    document.getElementById('health-label').textContent = 'unreachable';
    document.getElementById('status-detail').textContent = String(e);
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// A · Case Intake
// ─────────────────────────────────────────────────────────────────────────────

async function submitCase(btn) {
  const raw = document.getElementById('case-json-input').value.trim();
  const inline = document.getElementById('intake-inline');
  const area   = document.getElementById('intake-result-area');
  const lbl    = document.getElementById('intake-result-label');
  const pre    = document.getElementById('intake-result');

  if (!raw) { inline.textContent = 'paste case JSON first'; return; }
  let body;
  try { body = JSON.parse(raw); }
  catch(e) { inline.textContent = 'invalid JSON: ' + e.message; return; }
  inline.textContent = '';

  await withBtn(btn, 'Storing', async () => {
    const r = await api('POST', '/cases', body);
    if (r.ok) {
      const cid = r.json.case_id || '—';
      showResultArea(area, lbl, pre,
        `POST /cases`,
        pill(`HTTP ${r.status} · stored`, 'pill-ok'),
        fmt(r.json), 'result-ok');
      inline.textContent = `✓ ${cid}`;
      loadCases();
    } else {
      showResultArea(area, lbl, pre,
        `POST /cases`,
        pill(`HTTP ${r.status}`, 'pill-err'),
        fmtApiError(r.status, r.json), 'result-err');
      inline.textContent = `✗ HTTP ${r.status}`;
    }
  });
}

// ─────────────────────────────────────────────────────────────────────────────
// B · Cases
// ─────────────────────────────────────────────────────────────────────────────

async function loadCases() {
  const el = document.getElementById('cases-content');
  const r = await api('GET', '/cases');
  if (!r.ok) {
    el.innerHTML = `<pre class="result-err">${fmtApiError(r.status, r.json)}</pre>`;
    return;
  }
  const ids = r.json.case_ids || [];
  if (ids.length === 0) {
    el.innerHTML = '<p class="empty-note">No cases stored yet.</p>';
    return;
  }
  const rows = ids.map(id => `
    <tr>
      <td><code>${escHtml(id)}</code></td>
      <td class="row-actions">
        <button class="btn btn-secondary btn-sm" onclick="viewCase('${escAttr(id)}', this)">View JSON</button>
        <button class="btn btn-route btn-sm" onclick="openRouteModal('${escAttr(id)}')">Route This Case →</button>
      </td>
    </tr>`).join('');
  el.innerHTML = `<table>
    <thead><tr><th>case_id</th><th>actions</th></tr></thead>
    <tbody>${rows}</tbody>
  </table>`;
}

async function viewCase(id, btn) {
  await withBtn(btn, 'Loading', async () => {
    const r = await api('GET', `/cases/${id}`);
    const area = document.getElementById('case-viewer-area');
    const lbl  = document.getElementById('case-viewer-label');
    const pre  = document.getElementById('case-detail');
    if (r.ok) {
      showResultArea(area, lbl, pre,
        `GET /cases/${id}`,
        pill('200', 'pill-info'),
        fmt(r.json), 'result-info');
    } else {
      showResultArea(area, lbl, pre,
        `GET /cases/${id}`,
        pill(`${r.status}`, 'pill-err'),
        fmtApiError(r.status, r.json), 'result-err');
    }
  });
}

// ─────────────────────────────────────────────────────────────────────────────
// B · Route modal
// ─────────────────────────────────────────────────────────────────────────────

let _routeCaseId = null;

function openRouteModal(caseId) {
  _routeCaseId = caseId;
  document.getElementById('route-modal-case-id').textContent = caseId;
  document.getElementById('route-modal-result-area').classList.add('hidden');
  document.getElementById('route-modal-overlay').classList.add('open');
}

function closeRouteModal() {
  document.getElementById('route-modal-overlay').classList.remove('open');
  _routeCaseId = null;
}

async function submitRoute(btn) {
  const regRaw = document.getElementById('route-registry-input').value.trim();
  const cfgRaw = document.getElementById('route-config-input').value.trim();
  const area   = document.getElementById('route-modal-result-area');
  const lbl    = document.getElementById('route-modal-result-label');
  const pre    = document.getElementById('route-modal-result');

  if (!regRaw || !cfgRaw) {
    showResultArea(area, lbl, pre, 'Validation', pill('error', 'pill-err'),
      'Registry snapshot and routing config JSON are both required.', 'result-err');
    return;
  }
  let registry, config;
  try { registry = JSON.parse(regRaw); config = JSON.parse(cfgRaw); }
  catch(e) {
    showResultArea(area, lbl, pre, 'Parse error', pill('error', 'pill-err'),
      'Invalid JSON: ' + e.message, 'result-err');
    return;
  }

  await withBtn(btn, 'Routing', async () => {
    const r = await api('POST', `/cases/${_routeCaseId}/route`, { registry, config });
    const path = `/cases/${_routeCaseId}/route`;
    if (r.ok) {
      const hash = r.json.receipt_hash || '—';
      showResultArea(area, lbl, pre,
        `POST ${path}`,
        pill(`HTTP ${r.status} · receipt stored`, 'pill-ok'),
        fmt(r.json), 'result-ok');
      loadReceipts();
      loadHistory();
    } else {
      showResultArea(area, lbl, pre,
        `POST ${path}`,
        pill(`HTTP ${r.status}`, 'pill-err'),
        fmtApiError(r.status, r.json), 'result-err');
    }
  });
}

// ─────────────────────────────────────────────────────────────────────────────
// C · Receipts
// ─────────────────────────────────────────────────────────────────────────────

async function loadReceipts() {
  const el = document.getElementById('receipts-content');
  const r = await api('GET', '/receipts');
  if (!r.ok) {
    el.innerHTML = `<pre class="result-err">${fmtApiError(r.status, r.json)}</pre>`;
    return;
  }
  const hashes = r.json.receipts || [];
  if (hashes.length === 0) {
    el.innerHTML = '<p class="empty-note">No receipts stored yet.</p>';
    return;
  }
  const rows = hashes.map(h => {
    const short = h.slice(0, 20) + '…';
    return `<tr>
      <td>
        <span class="hash-short" title="${escAttr(h)}" onclick="toggleHash(this,'${escAttr(h)}')">${short}</span>
      </td>
      <td class="row-actions">
        <button class="btn btn-secondary btn-sm" onclick="viewReceipt('${escAttr(h)}',this)">View JSON</button>
        <button class="btn btn-dispatch btn-sm" onclick="dispatchReceipt('${escAttr(h)}',this)">Dispatch</button>
        <button class="btn btn-verify btn-sm" onclick="verifyReceipt('${escAttr(h)}',this)">Verify Integrity</button>
      </td>
    </tr>`;
  }).join('');
  el.innerHTML = `<table>
    <thead><tr><th>receipt_hash</th><th>actions</th></tr></thead>
    <tbody>${rows}</tbody>
  </table>`;
}

function toggleHash(span, full) {
  if (span.classList.contains('hash-short')) {
    span.className = 'hash-full';
    span.textContent = full;
  } else {
    span.className = 'hash-short';
    span.textContent = full.slice(0, 20) + '…';
  }
}

async function viewReceipt(hash, btn) {
  await withBtn(btn, 'Loading', async () => {
    const r = await api('GET', `/receipts/${hash}`);
    showReceiptResult(
      `GET /receipts/${hash.slice(0,12)}…`,
      r.ok ? pill('200', 'pill-info') : pill(`${r.status}`, 'pill-err'),
      r.ok ? fmt(r.json) : fmtApiError(r.status, r.json),
      r.ok ? 'result-info' : 'result-err');
  });
}

async function dispatchReceipt(hash, btn) {
  await withBtn(btn, 'Dispatching', async () => {
    const r = await api('POST', `/dispatch/${hash}`);
    const ok = r.ok;
    showReceiptResult(
      `POST /dispatch/${hash.slice(0,12)}…`,
      ok ? pill(`HTTP ${r.status} · dispatched`, 'pill-ok') : pill(`HTTP ${r.status}`, 'pill-err'),
      ok ? fmt(r.json) : fmtApiError(r.status, r.json),
      ok ? 'result-ok' : 'result-err');
  });
}

async function verifyReceipt(hash, btn) {
  await withBtn(btn, 'Verifying', async () => {
    const r = await api('POST', `/dispatch/${hash}/verify`);
    const result = r.json?.result;
    let preCls = 'result-info';
    let statusHtml = pill(`HTTP ${r.status}`, 'pill-info');
    if (!r.ok) {
      preCls = 'result-err';
      statusHtml = pill(`HTTP ${r.status}`, 'pill-err');
    } else if (result === 'VERIFIED') {
      preCls = 'result-ok';
      statusHtml = pill('VERIFIED', 'pill-ok');
    } else if (result === 'INVALID') {
      preCls = 'result-err';
      statusHtml = pill('INVALID', 'pill-err');
    }
    showReceiptResult(
      `POST /dispatch/${hash.slice(0,12)}…/verify`,
      statusHtml,
      r.ok ? fmt(r.json) : fmtApiError(r.status, r.json),
      preCls);
  });
}

function showReceiptResult(labelText, statusHtml, text, preCls) {
  showResultArea(
    document.getElementById('receipt-action-area'),
    document.getElementById('receipt-action-label'),
    document.getElementById('receipt-detail'),
    labelText, statusHtml, text, preCls);
}

// ─────────────────────────────────────────────────────────────────────────────
// D · Route History
// ─────────────────────────────────────────────────────────────────────────────

async function loadHistory() {
  const el = document.getElementById('history-content');
  const r = await api('GET', '/routes');
  if (!r.ok) {
    el.innerHTML = `<pre class="result-err">${fmtApiError(r.status, r.json)}</pre>`;
    return;
  }
  const routes = r.json.routes || [];
  if (routes.length === 0) {
    el.innerHTML = '<p class="empty-note">No routes recorded yet.</p>';
    return;
  }
  const rows = routes.map(e => {
    const outcome = e.selected_candidate_id
      ? badge('routed', 'badge-routed')
      : badge('refused', 'badge-refused');
    const mfr = e.selected_candidate_id
      ? `<code>${escHtml(e.selected_candidate_id)}</code>`
      : '<span style="color:#6e7681">—</span>';
    const ts = e.timestamp ? e.timestamp.replace('T', ' ').replace('Z', ' UTC') : '—';
    return `<tr>
      <td>${outcome}</td>
      <td><code>${escHtml(e.case_id)}</code></td>
      <td><span class="hash-short" title="${escAttr(e.receipt_hash)}"
               onclick="toggleHash(this,'${escAttr(e.receipt_hash)}')">${e.receipt_hash.slice(0,16)}…</span></td>
      <td>${mfr}</td>
      <td style="color:#6e7681;font-size:.69rem">${ts}</td>
    </tr>`;
  }).join('');
  el.innerHTML = `<table>
    <thead><tr>
      <th>outcome</th><th>case_id</th><th>receipt_hash</th>
      <th>manufacturer</th><th>routed at</th>
    </tr></thead>
    <tbody>${rows}</tbody>
  </table>`;
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

function escHtml(s) {
  return String(s)
    .replace(/&/g,'&amp;').replace(/</g,'&lt;')
    .replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}
function escAttr(s) { return escHtml(s); }

function refreshAll() {
  loadStatus();
  loadCases();
  loadReceipts();
  loadHistory();
}

// ─────────────────────────────────────────────────────────────────────────────
// Bootstrap
// ─────────────────────────────────────────────────────────────────────────────

document.getElementById('route-modal-overlay').addEventListener('click', function(e) {
  if (e.target === this) closeRouteModal();
});

refreshAll();
</script>
</body>
</html>"#;
