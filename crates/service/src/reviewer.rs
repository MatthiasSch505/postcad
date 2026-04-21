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
  --amber:#f59e0b;
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

/* Decision gate */
#phase-decision{display:none;padding:40px 0;animation:fadein .2s ease-out}
.gate-badge{font-size:.63rem;font-weight:800;letter-spacing:.16em;color:var(--amber);text-transform:uppercase;margin-bottom:10px}
.gate-title{font-size:1.22rem;font-weight:800;margin-bottom:8px;line-height:1.25}
.gate-sub{font-size:.88rem;color:var(--sub);line-height:1.55;margin-bottom:24px}
.gate-case-ctx{font-size:.8rem;color:var(--dim);font-family:'SF Mono','Fira Code',monospace;padding:9px 13px;background:var(--surface);border:1px solid var(--border);border-radius:6px;margin-bottom:24px}
.decision-choices{display:flex;flex-direction:column;gap:8px;margin-bottom:22px}
.choice-btn{padding:13px 16px;border:1px solid var(--border);border-radius:8px;background:transparent;color:var(--sub);font-size:.93rem;font-weight:600;text-align:left;cursor:pointer;transition:border-color .15s,color .15s,background .15s}
.choice-btn:hover{border-color:var(--sub);color:var(--text)}
.choice-btn.sel-proceed{border-color:var(--green);color:var(--text);background:rgba(34,197,94,.07)}
.choice-btn.sel-risk{border-color:var(--amber);color:#fbbf24;background:rgba(245,158,11,.07)}
.choice-btn.sel-block{border-color:var(--red);color:var(--red);background:rgba(239,68,68,.07)}
.reason-row{margin-bottom:22px;display:none}
.reason-label{font-size:.7rem;font-weight:700;letter-spacing:.1em;color:var(--dim);text-transform:uppercase;display:block;margin-bottom:8px}
#reason-code{width:100%;padding:10px 13px;background:var(--surface);border:1px solid var(--border);border-radius:6px;color:var(--text);font-size:.9rem;cursor:pointer;appearance:none}
#reason-code:focus{outline:none;border-color:var(--sub)}
.confirm-btn{width:100%;padding:15px;border:none;border-radius:8px;background:var(--green);color:#0d0f12;font-size:1rem;font-weight:800;cursor:pointer;transition:opacity .15s;letter-spacing:.01em}
.confirm-btn:disabled{opacity:.3;cursor:not-allowed}
.confirm-btn:not(:disabled):hover{opacity:.85}
.gate-error{font-size:.82rem;color:var(--red);margin-top:12px;text-align:center;display:none}

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

  <div id="phase-decision">
    <div class="gate-badge" id="t-gate-badge">PRODUCTION GATE</div>
    <div class="gate-title" id="t-gate-title">Production Decision Required</div>
    <div class="gate-sub" id="t-gate-sub">This case may only enter production with an explicit accountable decision.</div>
    <div class="gate-case-ctx" id="gate-case-ctx"></div>
    <div class="decision-choices">
      <button class="choice-btn" id="choice-proceed" onclick="selectDecision('proceed')" id="t-opt-proceed">Proceed</button>
      <button class="choice-btn" id="choice-proceed_with_risk" onclick="selectDecision('proceed_with_risk')" id="t-opt-risk">Proceed with risk</button>
      <button class="choice-btn" id="choice-request_correction" onclick="selectDecision('request_correction')" id="t-opt-correction">Request correction</button>
    </div>
    <div class="reason-row" id="reason-row">
      <label class="reason-label" id="t-reason-label" for="reason-code">Reason (required)</label>
      <select id="reason-code" onchange="updateConfirmState()">
        <option value="">— select —</option>
        <option value="incomplete_scan" id="rc-incomplete_scan">Incomplete scan</option>
        <option value="unclear_margin" id="rc-unclear_margin">Unclear margin</option>
        <option value="prep_uncertainty" id="rc-prep_uncertainty">Prep uncertainty</option>
        <option value="time_pressure" id="rc-time_pressure">Time pressure</option>
        <option value="other" id="rc-other">Other</option>
      </select>
    </div>
    <button class="confirm-btn" id="confirm-btn" onclick="confirmDecision()" disabled id="t-confirm-btn">Confirm Decision</button>
    <div class="gate-error" id="gate-error"></div>
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

    <div class="res-section" id="proof-section" style="display:none">
      <div class="res-label">Proof &amp; Receipt</div>
      <details open>
        <summary style="font-size:.83rem;color:var(--sub);cursor:pointer;padding:4px 0">Receipt JSON</summary>
        <pre id="proof-receipt-json" style="margin-top:8px;font-size:.7rem;color:var(--sub);font-family:'SF Mono','Fira Code',monospace;white-space:pre-wrap;word-break:break-all;line-height:1.5;background:var(--bg);border:1px solid var(--border);border-radius:6px;padding:12px;max-height:260px;overflow-y:auto"></pre>
      </details>
    </div>

    <div class="res-section" style="border-top:none;padding-top:8px">
      <button class="reset-btn" onclick="resetDemo()"><span id="t-reset">Neue Datei hochladen</span></button>
    </div>

  </div>

</main>

<div id="_legacy">
  <div id="step1-card"></div><div id="step2-card"></div><div id="step3-card"></div>
  <div id="result-area"></div><div id="case-data-display"></div>
  <button id="btn-route-norm">Route Normalized Pilot Case</button><button id="btn-verify"></button><button id="btn-dispatch"></button>
  <div id="block-cad"></div><div id="block-routing"></div><div id="block-verify"></div><div id="block-dispatch"></div>
  <div id="fixture-select"></div><select id="fixture-dropdown"></select>
  <span data-endpoint="/pilot/route-normalized"></span>
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
    decisionBlockedSub: 'Routing blockiert \u2014 Korrektur angefordert',
    decisionBlockedExplanation: 'Der Reviewer hat Korrektur angefordert. Kein Routing wird ausgef\u00fchrt.',
    pruefungLabel: 'Pr\u00fcfung',
    chkMaterial: 'Material zugelassen', chkJurisdiction: 'Jurisdiktion zul\u00e4ssig', chkManufacturing: 'Fertigung verf\u00fcgbar',
    ergebnisOk: 'Ergebnis: Freigegeben', ergebnisBlock: 'Ergebnis: Blockiert',
    fertigungLabel: 'M\u00f6gliche Fertigung',
    auditLabel: 'Audit', auditIdLbl: 'Audit-ID', auditTimeLbl: 'Zeitpunkt', auditStatusLbl: 'Status',
    reset: 'Neue Datei hochladen',
    gateBadge: 'PRODUKTIONSGATE',
    gateTitle: 'Produktionsentscheidung erforderlich',
    gateSub: 'Dieser Fall darf nur in Produktion gehen, wenn eine explizite Entscheidung vorliegt.',
    optProceed: 'Freigeben',
    optRisk: 'Freigeben mit Vorbehalt',
    optCorrection: 'Korrektur anfordern',
    reasonLabel: 'Begr\u00fcndung (erforderlich)',
    reasonSelect: '\u2014 ausw\u00e4hlen \u2014',
    rcIncompleteScan: 'Unvollst\u00e4ndiger Scan',
    rcUnclearMargin: 'Unklare Pr\u00e4p.-Grenze',
    rcPrepUncertainty: 'Pr\u00e4p.-Unsicherheit',
    rcTimePressure: 'Zeitdruck',
    rcOther: 'Sonstiges',
    confirmBtn: 'Entscheidung best\u00e4tigen',
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
    decisionBlockedSub: 'Routing blocked \u2014 correction requested',
    decisionBlockedExplanation: 'The reviewer requested correction. No routing will proceed.',
    pruefungLabel: 'Checks',
    chkMaterial: 'Material approved', chkJurisdiction: 'Jurisdiction valid', chkManufacturing: 'Manufacturing available',
    ergebnisOk: 'Result: Approved', ergebnisBlock: 'Result: Blocked',
    fertigungLabel: 'Manufacturing options',
    auditLabel: 'Audit', auditIdLbl: 'Audit ID', auditTimeLbl: 'Time', auditStatusLbl: 'Status',
    reset: 'Upload new file',
    gateBadge: 'PRODUCTION GATE',
    gateTitle: 'Production Decision Required',
    gateSub: 'This case may only enter production with an explicit accountable decision.',
    optProceed: 'Proceed',
    optRisk: 'Proceed with risk',
    optCorrection: 'Request correction',
    reasonLabel: 'Reason (required)',
    reasonSelect: '\u2014 select \u2014',
    rcIncompleteScan: 'Incomplete scan',
    rcUnclearMargin: 'Unclear margin',
    rcPrepUncertainty: 'Prep uncertainty',
    rcTimePressure: 'Time pressure',
    rcOther: 'Other',
    confirmBtn: 'Confirm Decision',
  },
};

const REGISTRY = [
  {manufacturer_id:"pilot-de-001",display_name:"Alpha Dental GmbH",country:"germany",is_active:true,capabilities:["crown","bridge"],materials_supported:["zirconia","pmma"],jurisdictions_served:["germany"],attestation_statuses:["verified"],sla_days:5},
  {manufacturer_id:"pilot-de-002",display_name:"Beta Zahntechnik GmbH",country:"germany",is_active:true,capabilities:["crown","veneer"],materials_supported:["zirconia","emax"],jurisdictions_served:["germany"],attestation_statuses:["verified"],sla_days:3},
  {manufacturer_id:"pilot-de-003",display_name:"Gamma Dental GmbH",country:"germany",is_active:true,capabilities:["crown","implant"],materials_supported:["zirconia","titanium"],jurisdictions_served:["germany"],attestation_statuses:["verified"],sla_days:7},
];

const FILE_CASES_API = {
  'krone_3-6_de.stl': {case_id:'f3000003-0000-0000-0000-000000000003',jurisdiction:'DE',routing_policy:'allow_domestic_and_cross_border',patient_country:'germany',manufacturer_country:'germany',material:'emax',procedure:'crown',file_type:'stl'},
  'bruecke_usa.stl':  {case_id:'f4000004-0000-0000-0000-000000000004',jurisdiction:'US',routing_policy:'allow_domestic_and_cross_border',patient_country:'united_states',manufacturer_country:'germany',material:'zirconia',procedure:'bridge',file_type:'stl'},
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
let currentFilename = null;
let selectedDecision = null;

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
  document.getElementById('t-gate-badge').textContent = t.gateBadge;
  document.getElementById('t-gate-title').textContent = t.gateTitle;
  document.getElementById('t-gate-sub').textContent = t.gateSub;
  document.getElementById('choice-proceed').textContent = t.optProceed;
  document.getElementById('choice-proceed_with_risk').textContent = t.optRisk;
  document.getElementById('choice-request_correction').textContent = t.optCorrection;
  document.getElementById('t-reason-label').textContent = t.reasonLabel;
  document.getElementById('confirm-btn').textContent = t.confirmBtn;
  const sel = document.getElementById('reason-code');
  sel.options[0].text = t.reasonSelect;
  sel.options[1].text = t.rcIncompleteScan;
  sel.options[2].text = t.rcUnclearMargin;
  sel.options[3].text = t.rcPrepUncertainty;
  sel.options[4].text = t.rcTimePressure;
  sel.options[5].text = t.rcOther;
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
  document.getElementById('phase-decision').style.display = 'none';
  document.getElementById('phase-result').style.display = 'none';

  for (let i = 0; i < 4; i++) {
    await delay(i === 0 ? 80 : 260);
    document.getElementById('pstep-' + i).classList.add('visible');
  }
  await delay(220);
  showDecisionGate(filename);
}

function showDecisionGate(filename) {
  currentFilename = filename;
  selectedDecision = null;
  const t = T[lang];
  const c = getCaseData(filename);

  document.getElementById('gate-case-ctx').textContent = c.proc + ' \u00b7 ' + c.material + ' \u00b7 ' + c.land;
  ['proceed', 'proceed_with_risk', 'request_correction'].forEach(d => {
    document.getElementById('choice-' + d).className = 'choice-btn';
  });
  document.getElementById('reason-row').style.display = 'none';
  document.getElementById('reason-code').value = '';
  document.getElementById('confirm-btn').disabled = true;
  document.getElementById('confirm-btn').textContent = t.confirmBtn;
  document.getElementById('gate-error').style.display = 'none';
  document.getElementById('gate-error').textContent = '';

  document.getElementById('phase-processing').style.display = 'none';
  document.getElementById('phase-decision').style.display = 'block';
}

function selectDecision(type) {
  selectedDecision = type;
  const classMap = {proceed: 'sel-proceed', proceed_with_risk: 'sel-risk', request_correction: 'sel-block'};
  ['proceed', 'proceed_with_risk', 'request_correction'].forEach(d => {
    document.getElementById('choice-' + d).className = 'choice-btn' + (d === type ? ' ' + classMap[d] : '');
  });
  const needsReason = type === 'proceed_with_risk' || type === 'request_correction';
  document.getElementById('reason-row').style.display = needsReason ? 'block' : 'none';
  updateConfirmState();
}

function updateConfirmState() {
  if (!selectedDecision) { document.getElementById('confirm-btn').disabled = true; return; }
  const needsReason = selectedDecision === 'proceed_with_risk' || selectedDecision === 'request_correction';
  const hasReason = document.getElementById('reason-code').value !== '';
  document.getElementById('confirm-btn').disabled = needsReason && !hasReason;
}

async function confirmDecision() {
  const caseObj = FILE_CASES_API[currentFilename.toLowerCase()];
  if (!caseObj || !selectedDecision) return;

  const t = T[lang];
  const btn = document.getElementById('confirm-btn');
  btn.disabled = true;
  btn.textContent = '\u2026';
  document.getElementById('gate-error').style.display = 'none';

  try {
    await fetch('/cases', {method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(caseObj)});

    const reasonVal = document.getElementById('reason-code').value;
    const decBody = {
      case_id: caseObj.case_id,
      actor_role: 'reviewer',
      actor_id: 'demo-reviewer',
      decision_type: selectedDecision,
    };
    if (reasonVal) decBody.reason_code = reasonVal;

    const decRes = await fetch('/decisions', {method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(decBody)});

    if (!decRes.ok) {
      const err = await decRes.json().catch(() => ({}));
      showGateError(err.error && err.error.message ? err.error.message : 'Decision failed');
      btn.disabled = false;
      btn.textContent = t.confirmBtn;
      return;
    }

    document.getElementById('phase-decision').style.display = 'none';

    if (selectedDecision === 'request_correction') {
      showResultBlocked(currentFilename);
    } else {
      showResult(currentFilename);
      fetchAndRenderProof(currentFilename);
    }
  } catch(e) {
    showGateError('Network error');
    btn.disabled = false;
    btn.textContent = t.confirmBtn;
  }
}

function showGateError(msg) {
  const el = document.getElementById('gate-error');
  el.textContent = msg;
  el.style.display = 'block';
}

function showResultBlocked(filename) {
  const t = T[lang];
  const c = getCaseData(filename);

  document.getElementById('res-proc').textContent = c.proc;
  document.getElementById('res-material').textContent = c.material;
  document.getElementById('res-land').textContent = c.land;
  document.getElementById('res-indication').textContent = c.indication;

  lastResultOk = false;
  const vEl = document.getElementById('result-verdict');
  vEl.className = 'result-verdict verdict-blocked';
  vEl.textContent = t.verdictBlock;
  document.getElementById('result-sub').textContent = t.decisionBlockedSub;
  document.getElementById('result-explanation').textContent = t.decisionBlockedExplanation;

  setCheck('chk-material', false);
  setCheck('chk-jurisdiction', false);
  setCheck('chk-manufacturing', false);
  document.getElementById('check-ergebnis').textContent = t.ergebnisBlock;
  document.getElementById('labs-section').style.display = 'none';

  document.getElementById('audit-id').textContent = 'PC-2026-' + String(Math.floor(Math.random() * 99999)).padStart(5, '0');
  document.getElementById('audit-time').textContent = new Date().toLocaleTimeString('de-DE', {hour:'2-digit', minute:'2-digit', second:'2-digit'});
  document.getElementById('audit-status').textContent = t.verdictBlock;

  document.getElementById('proof-section').style.display = 'none';
  document.getElementById('phase-result').style.display = 'block';
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

  document.getElementById('phase-result').style.display = 'block';
}

async function fetchAndRenderProof(filename) {
  const caseObj = FILE_CASES_API[filename.toLowerCase()];
  if (!caseObj) return;
  try {
    const routeRes = await fetch('/cases/' + caseObj.case_id + '/route', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({
        registry: REGISTRY,
        config: {jurisdiction: caseObj.jurisdiction, routing_policy: caseObj.routing_policy},
      }),
    });
    if (!routeRes.ok) return;
    const routeData = await routeRes.json();
    const receiptHash = routeData.receipt_hash;
    if (!receiptHash) return;
    const receiptRes = await fetch('/receipts/' + receiptHash);
    if (!receiptRes.ok) return;
    const receipt = await receiptRes.json();
    document.getElementById('proof-receipt-json').textContent = JSON.stringify(receipt, null, 2);
    document.getElementById('proof-section').style.display = '';
  } catch(e) {}
}

function setCheck(id, pass) {
  const el = document.getElementById(id);
  el.textContent = pass ? '\u2713' : '\u2715';
  el.className = pass ? 'chk-ok' : 'chk-fail';
}

function resetDemo() {
  lastResultOk = null;
  currentFilename = null;
  selectedDecision = null;
  document.getElementById('phase-result').style.display = 'none';
  document.getElementById('phase-processing').style.display = 'none';
  document.getElementById('phase-decision').style.display = 'none';
  document.getElementById('phase-upload').style.display = 'block';
  document.getElementById('proof-section').style.display = 'none';
  document.getElementById('proof-receipt-json').textContent = '';
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
