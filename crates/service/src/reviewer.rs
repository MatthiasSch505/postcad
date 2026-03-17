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
    --surface:#161a1f;
    --surface2:#1d2229;
    --border:#252b34;
    --green:#22c55e;
    --red:#ef4444;
    --text:#f1f5f9;
    --muted:#64748b;
  }
  body{background:var(--bg);color:var(--text);font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;min-height:100vh;display:flex;flex-direction:column;align-items:center;padding:0 20px 56px}

  header{width:100%;max-width:440px;display:flex;align-items:center;justify-content:space-between;padding:20px 0 36px}
  .brand{font-size:.85rem;font-weight:700;letter-spacing:.08em;color:var(--muted);text-transform:uppercase}
  .brand span{color:var(--green)}
  .lang-toggle{display:flex;gap:2px;background:var(--surface);border:1px solid var(--border);border-radius:6px;overflow:hidden}
  .lang-btn{padding:4px 10px;font-size:.75rem;font-weight:600;border:none;background:transparent;color:var(--muted);cursor:pointer;transition:color .15s}
  .lang-btn.active{background:var(--surface2);color:var(--text)}

  main{width:100%;max-width:440px}

  /* SELECT */
  h1{font-size:1.4rem;font-weight:800;line-height:1.25;margin-bottom:6px}
  .subline{font-size:.85rem;color:var(--muted);margin-bottom:28px;line-height:1.5}
  .scenario-label{font-size:.65rem;font-weight:700;letter-spacing:.1em;color:var(--muted);text-transform:uppercase;margin-bottom:10px}
  .scenario-cards{display:flex;flex-direction:column;gap:2px}
  .scenario-card{
    border-radius:8px;padding:14px 14px;cursor:pointer;
    transition:background .12s;
    display:flex;align-items:center;gap:12px;
  }
  .scenario-card:hover{background:var(--surface)}
  .sc-icon{font-size:1.2rem;flex-shrink:0}
  .sc-body{flex:1;min-width:0}
  .sc-title{font-size:.9rem;font-weight:700;margin-bottom:2px}
  .sc-desc{font-size:.76rem;color:var(--muted);line-height:1.4}
  .sc-chip{flex-shrink:0;font-size:.68rem;font-weight:700;letter-spacing:.04em}
  .chip-ok{color:var(--green)}
  .chip-block{color:var(--red)}

  /* CHECK */
  #phase-check{display:none;text-align:center;padding:72px 0}
  .check-step{font-size:.9rem;color:var(--muted);transition:opacity .2s;min-height:1.4em}
  .check-step.fade{opacity:0}

  /* RESULT */
  #phase-result{display:none;animation:fadein .25s ease-out}
  @keyframes fadein{from{opacity:0}to{opacity:1}}
  .result-block{text-align:center;padding:44px 0 32px}
  .result-verdict{font-size:3.2rem;font-weight:900;letter-spacing:.03em;line-height:1;margin-bottom:8px}
  .result-ok .result-verdict{color:var(--green)}
  .result-blocked .result-verdict{color:var(--red)}
  .result-sub{font-size:.88rem;color:var(--muted)}

  hr.sep{border:none;border-top:1px solid var(--border);margin:24px 0}

  .grund-label{font-size:.63rem;font-weight:700;letter-spacing:.1em;color:var(--muted);text-transform:uppercase;margin-bottom:6px}
  .grund-text{font-size:.95rem;color:var(--text);line-height:1.5;margin-bottom:10px;font-weight:600}
  .grund-impact{font-size:.82rem;color:var(--muted);line-height:1.5;font-style:italic;margin-bottom:20px}

  .tech-toggle{background:none;border:none;color:var(--border);font-size:.75rem;cursor:pointer;padding:0;margin-bottom:10px;text-decoration:underline;text-underline-offset:3px;display:block;transition:color .15s}
  .tech-toggle:hover{color:var(--muted)}
  .tech-box{display:none;border:1px solid var(--border);border-radius:8px;padding:12px;margin-bottom:16px;overflow-x:auto}
  .tech-box.open{display:block}
  .tech-box pre{font-size:.68rem;color:var(--muted);white-space:pre-wrap;word-break:break-all;font-family:'SF Mono','Fira Code',monospace}

  .reset-btn{width:100%;padding:13px;border:1px solid var(--border);border-radius:10px;background:transparent;color:var(--muted);font-size:.88rem;font-weight:600;cursor:pointer;transition:border-color .15s,color .15s}
  .reset-btn:hover{border-color:#3d4a5c;color:var(--text)}

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

<div id="phase-select">
  <h1 id="t-h1">Fallprüfung<br>vor Fertigung</h1>
  <p class="subline" id="t-subline">Tippen Sie auf ein Szenario.</p>
  <div class="scenario-label" id="t-sc-label">Szenario wählen</div>
  <div class="scenario-cards">
    <div class="scenario-card" onclick="startFlow(0)">
      <div class="sc-icon">🦷</div>
      <div class="sc-body">
        <div class="sc-title" id="t-s0-title">Standardfall</div>
        <div class="sc-desc" id="t-s0-desc">Krone · Zirkon · Deutschland</div>
      </div>
      <div class="sc-chip chip-ok" id="t-s0-chip">Freigegeben</div>
    </div>
    <div class="scenario-card" onclick="startFlow(1)">
      <div class="sc-icon">🌍</div>
      <div class="sc-body">
        <div class="sc-title" id="t-s1-title">Falsche Jurisdiktion</div>
        <div class="sc-desc" id="t-s1-desc">Krone · Zirkon · USA</div>
      </div>
      <div class="sc-chip chip-block" id="t-s1-chip">Blockiert</div>
    </div>
    <div class="scenario-card" onclick="startFlow(2)">
      <div class="sc-icon">🔬</div>
      <div class="sc-body">
        <div class="sc-title" id="t-s2-title">Kein passendes Labor</div>
        <div class="sc-desc" id="t-s2-desc">Brücke · E.max · Deutschland</div>
      </div>
      <div class="sc-chip chip-block" id="t-s2-chip">Blockiert</div>
    </div>
  </div>
</div>

<div id="phase-check">
  <div class="check-step" id="check-step-text"></div>
</div>

<div id="phase-result">
  <div class="result-block" id="result-block">
    <div class="result-verdict" id="result-verdict"></div>
    <div class="result-sub" id="result-sub"></div>
  </div>

  <hr class="sep">

  <div id="result-reason">
    <div class="grund-label" id="t-grund-label">Grund</div>
    <div class="grund-text" id="grund-text"></div>
    <div class="grund-impact" id="t-impact"></div>
  </div>

  <button class="tech-toggle" onclick="toggleTech()">
    <span id="t-tech-toggle">Technische Details</span>
  </button>
  <div class="tech-box" id="tech-box">
    <pre id="tech-content"></pre>
  </div>

  <button class="reset-btn" onclick="reset()">
    <span id="t-reset">Anderen Fall prüfen</span>
  </button>
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
    h1: 'Fallprüfung<br>vor Fertigung',
    subline: 'Tippen Sie auf ein Szenario.',
    scLabel: 'Szenario wählen',
    s0Title: 'Standardfall', s0Desc: 'Krone · Zirkon · Deutschland', s0Chip: 'Freigegeben',
    s1Title: 'Falsche Jurisdiktion', s1Desc: 'Krone · Zirkon · USA', s1Chip: 'Blockiert',
    s2Title: 'Kein passendes Labor', s2Desc: 'Brücke · E.max · Deutschland', s2Chip: 'Blockiert',
    checkStep: 'Prüfe Fall \u2026',
    verdictOk: 'FREIGEGEBEN', verdictBlock: 'BLOCKIERT',
    subOk: 'Weitergabe möglich', subBlock: 'Weitergabe nicht möglich',
    grundLabel: 'Grund',
    impact: 'Dieser Fall wäre sonst in die Fertigung gegangen.',
    reasons: {
      no_jurisdiction_match: 'Jurisdiktion nicht zulässig',
      no_material_match: 'Kein passender Fertigungspartner',
      default: 'Eingabedaten unvollständig',
    },
    techToggle: 'Technische Details',
    techToggleHide: 'Technische Details ausblenden',
    reset: 'Anderen Fall prüfen',
  },
  EN: {
    h1: 'Pre-Manufacturing<br>Case Review',
    subline: 'Tap a scenario to run it.',
    scLabel: 'Choose a scenario',
    s0Title: 'Standard case', s0Desc: 'Crown · Zirconia · Germany', s0Chip: 'Approved',
    s1Title: 'Wrong jurisdiction', s1Desc: 'Crown · Zirconia · USA', s1Chip: 'Blocked',
    s2Title: 'No matching lab', s2Desc: 'Bridge · E.max · Germany', s2Chip: 'Blocked',
    checkStep: 'Checking case \u2026',
    verdictOk: 'APPROVED', verdictBlock: 'BLOCKED',
    subOk: 'Safe to proceed', subBlock: 'Cannot proceed',
    grundLabel: 'Reason',
    impact: 'This case would otherwise have entered manufacturing.',
    reasons: {
      no_jurisdiction_match: 'Jurisdiction not allowed',
      no_material_match: 'No matching manufacturing partner',
      default: 'Input data incomplete',
    },
    techToggle: 'Technical details',
    techToggleHide: 'Hide technical details',
    reset: 'Check another case',
  }
};

const CASES = [
  {
    pilot_case: {case_id:'f1000001',patient_ref:'P-001',procedure_type:'Crown',material:'Zirconia',file_type:'STL',source_country:'DE',destination_country:'DE'},
    routing_config: {strategy:'HighestPriority',jurisdiction:'DE'},
  },
  {
    pilot_case: {case_id:'f1000002',patient_ref:'P-002',procedure_type:'Crown',material:'Zirconia',file_type:'STL',source_country:'US',destination_country:'US'},
    routing_config: {strategy:'HighestPriority',jurisdiction:'US'},
  },
  {
    pilot_case: {case_id:'f1000003',patient_ref:'P-003',procedure_type:'Bridge',material:'Emax',file_type:'STL',source_country:'DE',destination_country:'DE'},
    routing_config: {strategy:'HighestPriority',jurisdiction:'DE'},
  },
];

let lang = 'DE';
let techOpen = false;

function setLang(l) {
  lang = l;
  document.getElementById('btn-de').classList.toggle('active', l === 'DE');
  document.getElementById('btn-en').classList.toggle('active', l === 'EN');
  applyLang();
}

function applyLang() {
  const t = T[lang];
  document.getElementById('t-h1').innerHTML = t.h1;
  document.getElementById('t-subline').textContent = t.subline;
  document.getElementById('t-sc-label').textContent = t.scLabel;
  document.getElementById('t-s0-title').textContent = t.s0Title;
  document.getElementById('t-s0-desc').textContent = t.s0Desc;
  document.getElementById('t-s0-chip').textContent = t.s0Chip;
  document.getElementById('t-s1-title').textContent = t.s1Title;
  document.getElementById('t-s1-desc').textContent = t.s1Desc;
  document.getElementById('t-s1-chip').textContent = t.s1Chip;
  document.getElementById('t-s2-title').textContent = t.s2Title;
  document.getElementById('t-s2-desc').textContent = t.s2Desc;
  document.getElementById('t-s2-chip').textContent = t.s2Chip;
  document.getElementById('t-grund-label').textContent = t.grundLabel;
  document.getElementById('t-impact').textContent = t.impact;
  document.getElementById('t-tech-toggle').textContent = techOpen ? t.techToggleHide : t.techToggle;
  document.getElementById('t-reset').textContent = t.reset;
}

async function callAPIs(c) {
  try {
    const routeRes = await fetch('/pilot/route-normalized', {
      method: 'POST',
      headers: {'Content-Type':'application/json'},
      body: JSON.stringify(c),
    });
    const routeData = await routeRes.json();

    if (routeData.outcome !== 'routed') {
      return {ok: false, refusalCode: routeData.refusal_code || 'default', tech: routeData};
    }

    const verifyRes = await fetch('/verify', {
      method: 'POST',
      headers: {'Content-Type':'application/json'},
      body: JSON.stringify(routeData.receipt || routeData),
    });
    const verifyData = await verifyRes.json();

    let dispatchData = null;
    try {
      const dispatchRes = await fetch('/dispatch/create', {
        method: 'POST',
        headers: {'Content-Type':'application/json'},
        body: JSON.stringify({receipt_hash: verifyData.receipt_hash || verifyData.hash || 'unknown'}),
      });
      dispatchData = await dispatchRes.json();
    } catch(_) {}

    return {ok: true, refusalCode: 'ok', tech: {route: routeData, verify: verifyData, dispatch: dispatchData}};
  } catch(e) {
    return {ok: false, refusalCode: 'default', tech: {error: e.message}};
  }
}

async function startFlow(i) {
  const t = T[lang];
  const stepEl = document.getElementById('check-step-text');
  stepEl.classList.remove('fade');
  stepEl.textContent = t.checkStep;

  document.getElementById('phase-select').style.display = 'none';
  document.getElementById('phase-check').style.display = 'block';
  document.getElementById('phase-result').style.display = 'none';

  const [result] = await Promise.all([
    callAPIs(CASES[i]),
    (async () => {
      await delay(700);
      stepEl.classList.add('fade');
      await delay(200);
    })(),
  ]);

  showResult(result);
}

function showResult(result) {
  const t = T[lang];
  document.getElementById('phase-check').style.display = 'none';
  document.getElementById('phase-result').style.display = 'block';

  const block = document.getElementById('result-block');
  if (result.ok) {
    block.className = 'result-block result-ok';
    document.getElementById('result-verdict').textContent = t.verdictOk;
    document.getElementById('result-sub').textContent = t.subOk;
    document.getElementById('result-reason').style.display = 'none';
  } else {
    block.className = 'result-block result-blocked';
    document.getElementById('result-verdict').textContent = t.verdictBlock;
    document.getElementById('result-sub').textContent = t.subBlock;
    const reasonKey = result.refusalCode in t.reasons ? result.refusalCode : 'default';
    document.getElementById('grund-text').textContent = t.reasons[reasonKey];
    document.getElementById('t-impact').textContent = t.impact;
    document.getElementById('result-reason').style.display = 'block';
  }

  document.getElementById('tech-content').textContent = JSON.stringify(result.tech, null, 2);
  techOpen = false;
  document.getElementById('tech-box').classList.remove('open');
  document.getElementById('t-tech-toggle').textContent = t.techToggle;
}

function toggleTech() {
  techOpen = !techOpen;
  document.getElementById('tech-box').classList.toggle('open', techOpen);
  document.getElementById('t-tech-toggle').textContent = techOpen ? T[lang].techToggleHide : T[lang].techToggle;
}

function reset() {
  techOpen = false;
  document.getElementById('phase-result').style.display = 'none';
  document.getElementById('phase-check').style.display = 'none';
  document.getElementById('phase-select').style.display = 'block';
}

function delay(ms) { return new Promise(r => setTimeout(r, ms)); }

function loadFixtures(){}
function routeNormalized(){}
function verifyReceipt(){}
function dispatchReceipt(){}
</script>
</body>
</html>
"##;
