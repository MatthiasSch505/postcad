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
    --green-dim:#15803d;
    --red:#ef4444;
    --red-dim:#991b1b;
    --text:#f1f5f9;
    --muted:#64748b;
    --accent:#3b82f6;
  }
  body{background:var(--bg);color:var(--text);font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;min-height:100vh;display:flex;flex-direction:column;align-items:center;padding:0 16px 48px}
  header{width:100%;max-width:440px;display:flex;align-items:center;justify-content:space-between;padding:20px 0 28px}
  .brand{font-size:.85rem;font-weight:700;letter-spacing:.08em;color:var(--muted);text-transform:uppercase}
  .brand span{color:var(--green)}
  .lang-toggle{display:flex;gap:2px;background:var(--surface);border:1px solid var(--border);border-radius:6px;overflow:hidden}
  .lang-btn{padding:4px 10px;font-size:.75rem;font-weight:600;border:none;background:transparent;color:var(--muted);cursor:pointer;transition:all .15s}
  .lang-btn.active{background:var(--surface2);color:var(--text)}

  main{width:100%;max-width:440px}

  #phase-select{}
  .hero-label{font-size:.7rem;font-weight:700;letter-spacing:.12em;color:var(--accent);text-transform:uppercase;margin-bottom:8px}
  h1{font-size:1.6rem;font-weight:800;line-height:1.25;margin-bottom:8px}
  .subline{font-size:.9rem;color:var(--muted);margin-bottom:28px;line-height:1.5}

  .scenario-label{font-size:.7rem;font-weight:700;letter-spacing:.1em;color:var(--muted);text-transform:uppercase;margin-bottom:10px}
  .scenario-cards{display:flex;flex-direction:column;gap:10px;margin-bottom:24px}
  .scenario-card{
    background:var(--surface);border:1.5px solid var(--border);border-radius:12px;
    padding:16px 18px;cursor:pointer;transition:border-color .15s,background .15s;
    display:flex;align-items:center;gap:14px;
  }
  .scenario-card:hover{border-color:var(--accent);background:var(--surface2)}
  .scenario-card.selected{border-color:var(--accent);background:var(--surface2)}
  .sc-icon{font-size:1.4rem;flex-shrink:0}
  .sc-body{flex:1;min-width:0}
  .sc-title{font-size:.95rem;font-weight:700;margin-bottom:3px}
  .sc-desc{font-size:.8rem;color:var(--muted);line-height:1.4}
  .sc-chip{flex-shrink:0;font-size:.7rem;font-weight:700;padding:3px 8px;border-radius:20px;letter-spacing:.04em}
  .chip-ok{background:#14532d;color:#86efac}
  .chip-block{background:#450a0a;color:#fca5a5}

  .cta{
    width:100%;padding:16px;border:none;border-radius:12px;
    background:var(--green);color:#fff;font-size:1rem;font-weight:700;
    cursor:pointer;transition:opacity .15s;letter-spacing:.01em;
  }
  .cta:hover{opacity:.9}
  .cta:disabled{opacity:.4;cursor:not-allowed}

  #phase-check{display:none;text-align:center;padding:40px 0}
  .spinner{width:44px;height:44px;border:3px solid var(--border);border-top-color:var(--green);border-radius:50%;animation:spin .7s linear infinite;margin:0 auto 28px}
  @keyframes spin{to{transform:rotate(360deg)}}
  .check-step{font-size:1rem;font-weight:600;color:var(--text);transition:opacity .3s;min-height:1.4em}
  .check-step.fade{opacity:0}

  #phase-result{display:none}
  .result-card{
    border-radius:16px;padding:28px 24px 24px;text-align:center;
    animation:scalein .25s ease-out;margin-bottom:14px;
  }
  @keyframes scalein{from{transform:scale(.95);opacity:0}to{transform:scale(1);opacity:1}}
  .result-card.ok{background:linear-gradient(135deg,#052e16,#14532d);border:1.5px solid var(--green-dim)}
  .result-card.blocked{background:linear-gradient(135deg,#1c0404,#450a0a);border:1.5px solid var(--red-dim)}
  .result-icon{font-size:2.2rem;margin-bottom:10px}
  .result-verdict{font-size:1.8rem;font-weight:900;letter-spacing:.04em;margin-bottom:4px}
  .result-card.ok .result-verdict{color:var(--green)}
  .result-card.blocked .result-verdict{color:var(--red)}
  .result-sub{font-size:.9rem;color:#94a3b8}

  .reason-card{background:var(--surface);border:1px solid var(--border);border-radius:12px;padding:16px 18px;margin-bottom:14px}
  .reason-label{font-size:.68rem;font-weight:700;letter-spacing:.1em;color:var(--muted);text-transform:uppercase;margin-bottom:6px}
  .reason-text{font-size:.95rem;color:var(--text);line-height:1.5}

  .tech-toggle{background:none;border:none;color:var(--muted);font-size:.8rem;cursor:pointer;padding:4px 0;margin-bottom:8px;text-decoration:underline;text-underline-offset:3px}
  .tech-box{display:none;background:var(--surface);border:1px solid var(--border);border-radius:10px;padding:14px;margin-bottom:14px;overflow-x:auto}
  .tech-box.open{display:block}
  .tech-box pre{font-size:.72rem;color:var(--muted);white-space:pre-wrap;word-break:break-all;font-family:'SF Mono','Fira Code',monospace}

  .reset-btn{width:100%;padding:14px;border:1.5px solid var(--border);border-radius:12px;background:transparent;color:var(--muted);font-size:.9rem;font-weight:600;cursor:pointer;transition:border-color .15s,color .15s}
  .reset-btn:hover{border-color:var(--text);color:var(--text)}

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

<!-- PHASE: SELECT -->
<div id="phase-select">
  <div class="hero-label" id="t-hero-label">Fallprüfung</div>
  <h1 id="t-h1">Fallprüfung<br>vor Fertigung</h1>
  <p class="subline" id="t-subline">Wählen Sie ein Szenario. Das System prüft den Fall in Echtzeit und zeigt Ihnen die Entscheidung.</p>

  <div class="scenario-label" id="t-sc-label">Szenario wählen</div>
  <div class="scenario-cards">
    <div class="scenario-card" id="card-0" onclick="selectCard(0)">
      <div class="sc-icon">🦷</div>
      <div class="sc-body">
        <div class="sc-title" id="t-s0-title">Standardfall</div>
        <div class="sc-desc" id="t-s0-desc">Krone · Zirkon · Deutschland</div>
      </div>
      <div class="sc-chip chip-ok" id="t-s0-chip">Freigegeben</div>
    </div>
    <div class="scenario-card" id="card-1" onclick="selectCard(1)">
      <div class="sc-icon">🌍</div>
      <div class="sc-body">
        <div class="sc-title" id="t-s1-title">Falsche Jurisdiktion</div>
        <div class="sc-desc" id="t-s1-desc">Krone · Zirkon · USA</div>
      </div>
      <div class="sc-chip chip-block" id="t-s1-chip">Blockiert</div>
    </div>
    <div class="scenario-card" id="card-2" onclick="selectCard(2)">
      <div class="sc-icon">🔬</div>
      <div class="sc-body">
        <div class="sc-title" id="t-s2-title">Kein passendes Labor</div>
        <div class="sc-desc" id="t-s2-desc">Brücke · E.max · Deutschland</div>
      </div>
      <div class="sc-chip chip-block" id="t-s2-chip">Blockiert</div>
    </div>
  </div>

  <button class="cta" id="cta-run" disabled onclick="startFlow()">
    <span id="t-cta">Szenario wählen</span>
  </button>
</div>

<!-- PHASE: CHECK -->
<div id="phase-check">
  <div class="spinner"></div>
  <div class="check-step" id="check-step-text"></div>
</div>

<!-- PHASE: RESULT -->
<div id="phase-result">
  <div class="result-card" id="result-card">
    <div class="result-icon" id="result-icon"></div>
    <div class="result-verdict" id="result-verdict"></div>
    <div class="result-sub" id="result-sub"></div>
  </div>

  <div class="reason-card">
    <div class="reason-label" id="t-reason-label">Begründung</div>
    <div class="reason-text" id="reason-text"></div>
  </div>

  <button class="tech-toggle" onclick="toggleTech()">
    <span id="t-tech-toggle">Technische Details anzeigen</span>
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
    heroLabel: 'Fallprüfung',
    h1: 'Fallprüfung<br>vor Fertigung',
    subline: 'Wählen Sie ein Szenario. Das System prüft den Fall in Echtzeit und zeigt Ihnen die Entscheidung.',
    scLabel: 'Szenario wählen',
    s0Title: 'Standardfall', s0Desc: 'Krone · Zirkon · Deutschland', s0Chip: 'Freigegeben',
    s1Title: 'Falsche Jurisdiktion', s1Desc: 'Krone · Zirkon · USA', s1Chip: 'Blockiert',
    s2Title: 'Kein passendes Labor', s2Desc: 'Brücke · E.max · Deutschland', s2Chip: 'Blockiert',
    cta: 'Diesen Fall prüfen',
    ctaDisabled: 'Szenario wählen',
    steps: ['Fall wird geprüft \u2026', 'Regeln werden geprüft \u2026', 'Labor wird abgeglichen \u2026', 'Nachvollziehbarkeit wird erstellt \u2026'],
    verdictOk: 'FREIGEGEBEN',
    verdictBlock: 'BLOCKIERT',
    subOk: 'Weitergabe möglich',
    subBlock: 'Weitergabe nicht möglich',
    reasonLabel: 'Begründung',
    reasons: {
      ok: 'Der Fall erfüllt alle Anforderungen. Ein zertifiziertes Labor wurde zugewiesen.',
      no_jurisdiction_match: 'Für diese Jurisdiktion ist kein zugelassenes Labor registriert. Bitte prüfen Sie das Zielland.',
      no_material_match: 'Kein Labor kann dieses Material in der gewünschten Kombination fertigen.',
      default: 'Die Prüfung hat einen Fehler festgestellt. Bitte versuchen Sie es erneut.',
    },
    techToggleShow: 'Technische Details anzeigen',
    techToggleHide: 'Technische Details ausblenden',
    reset: 'Anderen Fall prüfen',
  },
  EN: {
    heroLabel: 'Case Review',
    h1: 'Pre-Manufacturing<br>Case Review',
    subline: 'Select a scenario. The system checks the case in real time and shows you the decision.',
    scLabel: 'Choose a scenario',
    s0Title: 'Standard case', s0Desc: 'Crown · Zirconia · Germany', s0Chip: 'Approved',
    s1Title: 'Wrong jurisdiction', s1Desc: 'Crown · Zirconia · USA', s1Chip: 'Blocked',
    s2Title: 'No matching lab', s2Desc: 'Bridge · E.max · Germany', s2Chip: 'Blocked',
    cta: 'Check this case',
    ctaDisabled: 'Choose a scenario',
    steps: ['Submitting case \u2026', 'Checking rules \u2026', 'Matching lab \u2026', 'Creating audit record \u2026'],
    verdictOk: 'APPROVED',
    verdictBlock: 'BLOCKED',
    subOk: 'Safe to proceed',
    subBlock: 'Cannot proceed',
    reasonLabel: 'Reason',
    reasons: {
      ok: 'The case meets all requirements. A certified lab has been assigned.',
      no_jurisdiction_match: 'No approved lab is registered for this jurisdiction. Please check the target country.',
      no_material_match: 'No lab can manufacture this material in the requested combination.',
      default: 'The check encountered an error. Please try again.',
    },
    techToggleShow: 'Show technical details',
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
let selected = -1;
let techOpen = false;

function setLang(l) {
  lang = l;
  document.getElementById('btn-de').classList.toggle('active', l === 'DE');
  document.getElementById('btn-en').classList.toggle('active', l === 'EN');
  applyLang();
}

function applyLang() {
  const t = T[lang];
  document.getElementById('t-hero-label').textContent = t.heroLabel;
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
  document.getElementById('t-cta').textContent = selected >= 0 ? t.cta : t.ctaDisabled;
  document.getElementById('t-reason-label').textContent = t.reasonLabel;
  document.getElementById('t-tech-toggle').textContent = techOpen ? t.techToggleHide : t.techToggleShow;
  document.getElementById('t-reset').textContent = t.reset;
}

function selectCard(i) {
  selected = i;
  document.querySelectorAll('.scenario-card').forEach((c,j) => c.classList.toggle('selected', j === i));
  document.getElementById('cta-run').disabled = false;
  document.getElementById('t-cta').textContent = T[lang].cta;
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

async function startFlow() {
  if (selected < 0) return;
  const c = CASES[selected];
  const t = T[lang];

  const apiPromise = callAPIs(c);

  document.getElementById('phase-select').style.display = 'none';
  document.getElementById('phase-check').style.display = 'block';
  document.getElementById('phase-result').style.display = 'none';

  const stepEl = document.getElementById('check-step-text');
  for (let i = 0; i < t.steps.length; i++) {
    stepEl.classList.remove('fade');
    stepEl.textContent = t.steps[i];
    await delay(220);
    if (i < t.steps.length - 1) {
      stepEl.classList.add('fade');
      await delay(80);
    }
  }

  const result = await apiPromise;
  showResult(result);
}

function showResult(result) {
  const t = T[lang];
  document.getElementById('phase-check').style.display = 'none';
  document.getElementById('phase-result').style.display = 'block';

  const card = document.getElementById('result-card');
  if (result.ok) {
    card.className = 'result-card ok';
    document.getElementById('result-icon').textContent = '\u2713';
    document.getElementById('result-verdict').textContent = t.verdictOk;
    document.getElementById('result-sub').textContent = t.subOk;
  } else {
    card.className = 'result-card blocked';
    document.getElementById('result-icon').textContent = '\u2715';
    document.getElementById('result-verdict').textContent = t.verdictBlock;
    document.getElementById('result-sub').textContent = t.subBlock;
  }

  const reasonKey = result.refusalCode in t.reasons ? result.refusalCode : 'default';
  document.getElementById('reason-text').textContent = t.reasons[reasonKey];
  document.getElementById('tech-content').textContent = JSON.stringify(result.tech, null, 2);

  techOpen = false;
  document.getElementById('tech-box').classList.remove('open');
  document.getElementById('t-tech-toggle').textContent = t.techToggleShow;
}

function toggleTech() {
  techOpen = !techOpen;
  document.getElementById('tech-box').classList.toggle('open', techOpen);
  document.getElementById('t-tech-toggle').textContent = techOpen ? T[lang].techToggleHide : T[lang].techToggleShow;
}

function reset() {
  selected = -1;
  techOpen = false;
  document.querySelectorAll('.scenario-card').forEach(c => c.classList.remove('selected'));
  document.getElementById('cta-run').disabled = true;
  document.getElementById('t-cta').textContent = T[lang].ctaDisabled;
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
