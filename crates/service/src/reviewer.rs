pub const REVIEWER_HTML: &str = r##"<!DOCTYPE html>
<html lang="de">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PostCAD · Fallprüfung</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
:root{
  --bg:#0d0f12;
  --surface:#131720;
  --border:#1e2535;
  --green:#22c55e;
  --red:#ef4444;
  --text:#f1f5f9;
  --sub:#94a3b8;
  --dim:#4b5a6e;
}
body{background:var(--bg);color:var(--text);font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;min-height:100vh;display:flex;flex-direction:column;align-items:center;padding:0 24px 64px}
header{width:100%;max-width:560px;display:flex;align-items:center;justify-content:space-between;padding:20px 0 44px}
.brand{font-size:.78rem;font-weight:700;letter-spacing:.12em;color:var(--dim);text-transform:uppercase}
.brand span{color:var(--sub)}
.lang-toggle{display:flex;gap:1px}
.lang-btn{padding:3px 9px;font-size:.7rem;font-weight:600;border:1px solid var(--border);background:transparent;color:var(--dim);cursor:pointer;border-radius:4px;transition:color .15s,border-color .15s}
.lang-btn.active{color:var(--sub);border-color:var(--dim)}
main{width:100%;max-width:560px}

/* Upload */
.upload-zone{border:1px dashed #2d3d54;border-radius:8px;padding:44px 24px;text-align:center;cursor:pointer;display:block;transition:border-color .15s;margin-bottom:16px;text-decoration:none;color:inherit}
.upload-zone:hover,.upload-zone.drag-over{border-color:var(--sub)}
.upload-icon{font-size:1.4rem;color:var(--dim);margin-bottom:12px}
.upload-title{font-size:1.05rem;font-weight:700;margin-bottom:6px}
.upload-sub{font-size:.85rem;color:var(--sub)}
.demo-files{display:flex;gap:8px;flex-wrap:wrap}
.demo-file-btn{padding:8px 14px;border:1px solid var(--border);border-radius:6px;background:transparent;color:var(--sub);font-size:.8rem;font-family:'SF Mono','Fira Code',monospace;cursor:pointer;transition:border-color .15s,color .15s}
.demo-file-btn:hover{border-color:var(--sub);color:var(--text)}

/* Processing */
#phase-processing{display:none;padding:52px 0}
.proc-filename{font-size:.78rem;color:var(--dim);font-family:'SF Mono','Fira Code',monospace;margin-bottom:20px}
.proc-step{font-size:.95rem;color:var(--sub);padding:6px 0;opacity:0;transition:opacity .15s}
.proc-step.visible{opacity:1}

/* Result */
#phase-result{display:none;animation:fadein .25s ease-out}
@keyframes fadein{from{opacity:0}to{opacity:1}}
.res-section{padding:32px 0}
.res-label{font-size:.68rem;font-weight:700;letter-spacing:.12em;color:var(--dim);text-transform:uppercase;margin-bottom:16px}
.case-proc{font-size:1.35rem;font-weight:700;margin-bottom:14px;line-height:1.2}
.case-row{font-size:.92rem;color:var(--sub);margin-bottom:6px;display:flex;gap:8px}
.case-row-lbl{color:var(--dim)}
.result-verdict{font-size:clamp(2.4rem,8vw,3.2rem);font-weight:900;letter-spacing:.02em;line-height:1;margin-bottom:10px}
.verdict-ok{color:var(--green)}
.verdict-blocked{color:var(--red)}
.result-sub{font-size:1rem;color:var(--sub);margin-bottom:6px}
.result-explanation{font-size:.88rem;color:var(--dim);margin-top:10px;line-height:1.5}
.check-row{display:flex;justify-content:space-between;align-items:center;padding:11px 0;font-size:1rem}
.check-row-lbl{color:var(--sub)}
.chk-ok{color:var(--green);font-weight:700;font-size:1.1rem}
.chk-fail{color:var(--red);font-weight:700;font-size:1.1rem}
.check-ergebnis{font-size:.9rem;color:var(--sub);margin-top:14px;font-weight:600}
.lab-item{font-size:1.05rem;color:var(--sub);padding:6px 0}
.lab-item::before{content:'— ';color:var(--dim)}
.audit-row{display:flex;font-size:.85rem;padding:5px 0;gap:12px}
.audit-row-lbl{color:var(--dim);flex-shrink:0;min-width:80px}
.audit-row-val{color:var(--sub);font-family:'SF Mono','Fira Code',monospace;font-size:.78rem}
.reset-btn{background:none;border:1px solid var(--border);border-radius:8px;color:var(--sub);font-size:.9rem;font-weight:600;padding:13px 22px;cursor:pointer;transition:border-color .15s,color .15s}
.reset-btn:hover{border-color:var(--sub);color:var(--text)}

#_legacy{display:none!important}
</style>
</head>
<body>

<header>
  <div class="brand">Post<span>CAD</span></div>
  <div class="lang-toggle">
    <button class="lang-btn active" id="btn-de" onclick="setLang('DE')">DE</button>
    <button class="lang-btn" id="btn-en" onclick="setLang('EN')">EN</button>
  </div>
</header>

<main>

  <div id="phase-upload">
    <label class="upload-zone" id="upload-zone" for="file-input">
      <div class="upload-icon">↑</div>
      <div class="upload-title" id="t-upload-title">CAD-Datei hochladen</div>
      <div class="upload-sub" id="t-upload-sub">Datei hier ablegen oder auswählen (STL, OBJ)</div>
    </label>
    <input type="file" id="file-input" accept=".stl,.obj" style="display:none" onchange="onFileInput(this)">
    <div class="demo-files">
      <button class="demo-file-btn" onclick="loadDemo('Krone_3-6_DE.stl')">Krone_3-6_DE.stl</button>
      <button class="demo-file-btn" onclick="loadDemo('Bruecke_USA.stl')">Bruecke_USA.stl</button>
    </div>
  </div>

  <div id="phase-processing">
    <div class="proc-filename" id="proc-filename"></div>
    <div class="proc-step" id="pstep-0"></div>
    <div class="proc-step" id="pstep-1"></div>
    <div class="proc-step" id="pstep-2"></div>
    <div class="proc-step" id="pstep-3"></div>
  </div>

  <div id="phase-result">

    <div class="res-section">
      <div class="res-label" id="t-case-label">Fall erkannt</div>
      <div class="case-proc" id="res-proc"></div>
      <div class="case-row"><span class="case-row-lbl" id="t-material-lbl">Material</span><span id="res-material"></span></div>
      <div class="case-row"><span class="case-row-lbl" id="t-land-lbl">Land</span><span id="res-land"></span></div>
      <div class="case-row"><span class="case-row-lbl" id="t-indication-lbl">Indikation</span><span id="res-indication"></span></div>
    </div>

    <div class="res-section">
      <div class="res-label" id="t-decision-label">Entscheidung</div>
      <div class="result-verdict" id="result-verdict"></div>
      <div class="result-sub" id="result-sub"></div>
      <div class="result-explanation" id="result-explanation"></div>
    </div>

    <div class="res-section">
      <div class="res-label" id="t-pruefung-label">Prüfung</div>
      <div class="check-row">
        <span class="check-row-lbl" id="t-chk-material">Material zugelassen</span>
        <span id="chk-material"></span>
      </div>
      <div class="check-row">
        <span class="check-row-lbl" id="t-chk-jurisdiction">Jurisdiktion zulässig</span>
        <span id="chk-jurisdiction"></span>
      </div>
      <div class="check-row">
        <span class="check-row-lbl" id="t-chk-manufacturing">Fertigung verfügbar</span>
        <span id="chk-manufacturing"></span>
      </div>
      <div class="check-ergebnis" id="check-ergebnis"></div>
    </div>

    <div class="res-section" id="labs-section" style="display:none">
      <div class="res-label" id="t-fertigung-label">Mögliche Fertigung</div>
      <div id="labs-list"></div>
    </div>

    <div class="res-section">
      <div class="res-label" id="t-audit-label">Audit</div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-audit-id-lbl">Audit-ID</span><span class="audit-row-val" id="audit-id"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-audit-time-lbl">Zeitpunkt</span><span class="audit-row-val" id="audit-time"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-audit-status-lbl">Status</span><span class="audit-row-val" id="audit-status"></span></div>
    </div>

    <div class="res-section" style="border-top:none;padding-top:8px">
      <button class="reset-btn" onclick="resetDemo()"><span id="t-reset">Neue Datei hochladen</span></button>
    </div>

  </div>

</main>

<div id="_legacy">
  <div id="step1-card"></div><div id="step2-card"></div><div id="step3-card"></div>
  <div id="result-area"></div><div id="case-data-display"></div>
  <button id="btn-route-norm"></button><button id="btn-verify"></button><button id="btn-dispatch"></button>
  <div id="block-cad"></div><div id="block-routing"></div><div id="block-verify"></div><div id="block-dispatch"></div>
  <div id="fixture-select"></div><select id="fixture-dropdown"></select>
</div>

<script>
const T = {
  DE: {
    uploadTitle: 'CAD-Datei hochladen',
    uploadSub: 'Datei hier ablegen oder auswählen (STL, OBJ)',
    procSteps: ['Datei erkannt', 'Falldaten werden gelesen', 'Routing wird gepr\u00fcft', 'Entscheidung wird erstellt'],
    caseLabel: 'Fall erkannt',
    materialLbl: 'Material', landLbl: 'Land', indicationLbl: 'Indikation',
    decisionLabel: 'Entscheidung',
    verdictOk: 'FREIGEGEBEN', verdictBlock: 'BLOCKIERT',
    subOk: 'Weitergabe m\u00f6glich', subBlock: 'Weitergabe nicht m\u00f6glich',
    explanationOk: 'Zul\u00e4ssig unter MDR-konformer Fertigung in Deutschland.',
    explanationBlock: 'Versto\u00df gegen Jurisdiktionsanforderungen.',
    pruefungLabel: 'Pr\u00fcfung',
    chkMaterial: 'Material zugelassen', chkJurisdiction: 'Jurisdiktion zul\u00e4ssig', chkManufacturing: 'Fertigung verf\u00fcgbar',
    ergebnisOk: 'Ergebnis: Freigegeben', ergebnisBlock: 'Ergebnis: Blockiert',
    fertigungLabel: 'M\u00f6gliche Fertigung',
    auditLabel: 'Audit', auditIdLbl: 'Audit-ID', auditTimeLbl: 'Zeitpunkt', auditStatusLbl: 'Status',
    reset: 'Neue Datei hochladen',
  },
  EN: {
    uploadTitle: 'Upload CAD file',
    uploadSub: 'Drop file here or select (STL, OBJ)',
    procSteps: ['File detected', 'Case data being read', 'Routing being checked', 'Decision being created'],
    caseLabel: 'Case detected',
    materialLbl: 'Material', landLbl: 'Country', indicationLbl: 'Indication',
    decisionLabel: 'Decision',
    verdictOk: 'APPROVED', verdictBlock: 'BLOCKED',
    subOk: 'Safe to proceed', subBlock: 'Cannot proceed',
    explanationOk: 'Permissible under MDR-compliant manufacturing in Germany.',
    explanationBlock: 'Violation of jurisdiction requirements.',
    pruefungLabel: 'Checks',
    chkMaterial: 'Material approved', chkJurisdiction: 'Jurisdiction valid', chkManufacturing: 'Manufacturing available',
    ergebnisOk: 'Result: Approved', ergebnisBlock: 'Result: Blocked',
    fertigungLabel: 'Manufacturing options',
    auditLabel: 'Audit', auditIdLbl: 'Audit ID', auditTimeLbl: 'Time', auditStatusLbl: 'Status',
    reset: 'Upload new file',
  },
};

const FILE_CASES = {
  'krone_3-6_de.stl': {
    proc: 'Krone \u00b7 Zahn 3\u20136',
    material: 'E.max', land: 'Deutschland', indication: 'Standardversorgung',
    ok: true,
    checks: {material: true, jurisdiction: true, manufacturing: true},
    labs: ['Labor Berlin', 'Labor M\u00fcnchen', 'Industriepool EU'],
  },
  'bruecke_usa.stl': {
    proc: 'Br\u00fccke',
    material: 'Zirkon', land: 'USA', indication: 'Standardversorgung',
    ok: false,
    checks: {material: true, jurisdiction: false, manufacturing: true},
    labs: [],
  },
};

function getCaseData(filename) {
  return FILE_CASES[filename.toLowerCase()] || {
    proc: filename, material: '\u2014', land: '\u2014', indication: '\u2014',
    ok: false, checks: {material: false, jurisdiction: false, manufacturing: false}, labs: [],
  };
}

let lang = 'DE';
let lastResultOk = null;

function setLang(l) {
  lang = l;
  document.getElementById('btn-de').classList.toggle('active', l === 'DE');
  document.getElementById('btn-en').classList.toggle('active', l === 'EN');
  const t = T[l];
  document.getElementById('t-upload-title').textContent = t.uploadTitle;
  document.getElementById('t-upload-sub').textContent = t.uploadSub;
  document.getElementById('t-case-label').textContent = t.caseLabel;
  document.getElementById('t-material-lbl').textContent = t.materialLbl;
  document.getElementById('t-land-lbl').textContent = t.landLbl;
  document.getElementById('t-indication-lbl').textContent = t.indicationLbl;
  document.getElementById('t-decision-label').textContent = t.decisionLabel;
  document.getElementById('t-pruefung-label').textContent = t.pruefungLabel;
  document.getElementById('t-chk-material').textContent = t.chkMaterial;
  document.getElementById('t-chk-jurisdiction').textContent = t.chkJurisdiction;
  document.getElementById('t-chk-manufacturing').textContent = t.chkManufacturing;
  document.getElementById('t-fertigung-label').textContent = t.fertigungLabel;
  document.getElementById('t-audit-label').textContent = t.auditLabel;
  document.getElementById('t-audit-id-lbl').textContent = t.auditIdLbl;
  document.getElementById('t-audit-time-lbl').textContent = t.auditTimeLbl;
  document.getElementById('t-audit-status-lbl').textContent = t.auditStatusLbl;
  document.getElementById('t-reset').textContent = t.reset;
  if (lastResultOk !== null) {
    document.getElementById('result-explanation').textContent = lastResultOk ? t.explanationOk : t.explanationBlock;
  }
}

function onFileInput(input) {
  if (input.files[0]) { loadDemo(input.files[0].name); input.value = ''; }
}

function loadDemo(filename) {
  startProcessing(filename);
}

async function startProcessing(filename) {
  const t = T[lang];
  document.getElementById('proc-filename').textContent = filename;
  for (let i = 0; i < 4; i++) {
    const el = document.getElementById('pstep-' + i);
    el.textContent = t.procSteps[i];
    el.classList.remove('visible');
  }
  document.getElementById('phase-upload').style.display = 'none';
  document.getElementById('phase-processing').style.display = 'block';
  document.getElementById('phase-result').style.display = 'none';

  for (let i = 0; i < 4; i++) {
    await delay(i === 0 ? 80 : 260);
    document.getElementById('pstep-' + i).classList.add('visible');
  }
  await delay(220);
  showResult(filename);
}

function showResult(filename) {
  const t = T[lang];
  const c = getCaseData(filename);

  document.getElementById('res-proc').textContent = c.proc;
  document.getElementById('res-material').textContent = c.material;
  document.getElementById('res-land').textContent = c.land;
  document.getElementById('res-indication').textContent = c.indication;

  lastResultOk = c.ok;
  const vEl = document.getElementById('result-verdict');
  if (c.ok) {
    vEl.className = 'result-verdict verdict-ok';
    vEl.textContent = t.verdictOk;
    document.getElementById('result-sub').textContent = t.subOk;
    document.getElementById('result-explanation').textContent = t.explanationOk;
  } else {
    vEl.className = 'result-verdict verdict-blocked';
    vEl.textContent = t.verdictBlock;
    document.getElementById('result-sub').textContent = t.subBlock;
    document.getElementById('result-explanation').textContent = t.explanationBlock;
  }

  setCheck('chk-material', c.checks.material);
  setCheck('chk-jurisdiction', c.checks.jurisdiction);
  setCheck('chk-manufacturing', c.checks.manufacturing);
  document.getElementById('check-ergebnis').textContent = c.ok ? t.ergebnisOk : t.ergebnisBlock;

  const labsSec = document.getElementById('labs-section');
  if (c.ok && c.labs.length > 0) {
    labsSec.style.display = 'block';
    document.getElementById('labs-list').innerHTML = c.labs.map(l => '<div class="lab-item">' + l + '</div>').join('');
  } else {
    labsSec.style.display = 'none';
  }

  document.getElementById('audit-id').textContent = 'PC-2026-' + String(Math.floor(Math.random() * 99999)).padStart(5, '0');
  document.getElementById('audit-time').textContent = new Date().toLocaleTimeString('de-DE', {hour:'2-digit', minute:'2-digit', second:'2-digit'});
  document.getElementById('audit-status').textContent = c.ok ? t.verdictOk : t.verdictBlock;

  document.getElementById('phase-processing').style.display = 'none';
  document.getElementById('phase-result').style.display = 'block';
}

function setCheck(id, pass) {
  const el = document.getElementById(id);
  el.textContent = pass ? '\u2713' : '\u2715';
  el.className = pass ? 'chk-ok' : 'chk-fail';
}

function resetDemo() {
  lastResultOk = null;
  document.getElementById('phase-result').style.display = 'none';
  document.getElementById('phase-processing').style.display = 'none';
  document.getElementById('phase-upload').style.display = 'block';
}

function delay(ms) { return new Promise(r => setTimeout(r, ms)); }

(function() {
  const zone = document.getElementById('upload-zone');
  zone.addEventListener('dragover', e => { e.preventDefault(); zone.classList.add('drag-over'); });
  zone.addEventListener('dragleave', e => { if (!zone.contains(e.relatedTarget)) zone.classList.remove('drag-over'); });
  zone.addEventListener('drop', e => {
    e.preventDefault(); zone.classList.remove('drag-over');
    if (e.dataTransfer.files[0]) loadDemo(e.dataTransfer.files[0].name);
  });
})();

function loadFixtures(){}
function routeNormalized(){}
function verifyReceipt(){}
function dispatchReceipt(){}
</script>
</body>
</html>
"##;
