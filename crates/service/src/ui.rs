//! Public landing page for PostCAD.
//!
//! Served at `GET /`. Single-page scroll: hero → flow diagram → interactive demo.

pub const OPERATOR_UI_HTML: &str = r##"<!DOCTYPE html>
<html lang="de">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PostCAD</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
:root{
  --bg:#090b0e;
  --surface:#0f1216;
  --border:#1a2030;
  --green:#22c55e;
  --red:#ef4444;
  --text:#f1f5f9;
  --sub:#64748b;
  --dim:#374151;
}
html{scroll-behavior:smooth}
body{background:var(--bg);color:var(--text);font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif}

/* ── HEADER ── */
header{
  position:fixed;top:0;left:0;right:0;
  display:flex;align-items:center;justify-content:space-between;
  padding:18px 32px;z-index:100;
  background:var(--bg);
}
.brand{font-size:.78rem;font-weight:700;letter-spacing:.12em;color:var(--dim);text-transform:uppercase}
.brand span{color:var(--sub)}
.lang-toggle{display:flex;gap:1px}
.lang-btn{padding:3px 9px;font-size:.7rem;font-weight:600;border:1px solid var(--border);background:transparent;color:var(--dim);cursor:pointer;border-radius:4px;transition:color .15s,border-color .15s}
.lang-btn.active{color:var(--sub);border-color:var(--dim)}

/* ── LAYOUT ── */
section{padding:0 24px}
.inner{max-width:580px;margin:0 auto}

/* ── SECTION 1: HERO ── */
#hero{min-height:100vh;display:flex;align-items:center}
#hero .inner{padding:96px 0 80px}
.hero-h1{font-size:clamp(2.4rem,6.5vw,4rem);font-weight:900;line-height:1.08;letter-spacing:-.02em;margin-bottom:20px}
.hero-sub{font-size:1rem;color:var(--sub);line-height:1.65;margin-bottom:44px;max-width:420px}
.hero-cta{
  display:inline-flex;align-items:center;gap:10px;
  font-size:.9rem;font-weight:600;color:var(--sub);text-decoration:none;
  transition:color .15s;
}
.hero-cta:hover{color:var(--text)}
.hero-cta-arrow{font-size:1rem;transition:transform .15s}
.hero-cta:hover .hero-cta-arrow{transform:translateY(3px)}

/* ── SECTION 2: FLOW ── */
#flow{padding-top:100px;padding-bottom:120px;border-top:1px solid var(--border)}
#flow .inner{}
.flow-heading{font-size:.65rem;font-weight:700;letter-spacing:.12em;color:var(--dim);text-transform:uppercase;margin-bottom:48px}
.flow-track{display:flex;flex-direction:column}
.flow-node{display:flex;gap:20px;align-items:flex-start}
.flow-spine{display:flex;flex-direction:column;align-items:center;flex-shrink:0;padding-top:6px}
.flow-dot{width:8px;height:8px;border-radius:50%;background:var(--border);flex-shrink:0}
.flow-dot.lit{background:var(--sub)}
.flow-line{width:1px;height:40px;background:var(--border);margin:3px 0}
.flow-body{padding-bottom:8px}
.flow-n{font-size:.6rem;font-weight:700;letter-spacing:.1em;color:var(--dim);margin-bottom:4px;text-transform:uppercase}
.flow-label{font-size:1.05rem;font-weight:600;color:var(--sub)}
.flow-label.primary{color:var(--text)}
.flow-outcomes{display:flex;gap:36px;padding-top:4px;padding-left:28px}
.flow-outcome-blocked .outcome-v{font-size:1.5rem;font-weight:900;color:var(--red);letter-spacing:.03em}
.flow-outcome-ok .outcome-v{font-size:1.5rem;font-weight:900;color:var(--green);letter-spacing:.03em}
.outcome-sub{font-size:.72rem;color:var(--dim);margin-top:3px}

/* ── SECTION 3: DEMO ── */
#demo{padding-top:80px;padding-bottom:120px;border-top:1px solid var(--border)}
.demo-heading{font-size:.65rem;font-weight:700;letter-spacing:.12em;color:var(--dim);text-transform:uppercase;margin-bottom:32px}

/* Scenario rows */
.scenario-rows{display:flex;flex-direction:column}
.scenario-row{
  display:flex;align-items:center;gap:16px;
  padding:18px 0;border-top:1px solid var(--border);cursor:pointer;
  transition:opacity .12s;
}
.scenario-row:last-child{border-bottom:1px solid var(--border)}
.scenario-row:hover{opacity:.65}
.sc-body{flex:1;min-width:0}
.sc-title{font-size:.95rem;font-weight:700;margin-bottom:3px}
.sc-desc{font-size:.78rem;color:var(--sub)}
.sc-signal{font-size:.72rem;font-weight:700;flex-shrink:0}
.sig-ok{color:var(--green)}
.sig-block{color:var(--red)}

/* Check phase */
#phase-check{display:none;padding:64px 0;text-align:center}
.check-text{font-size:.9rem;color:var(--sub);transition:opacity .2s}
.check-text.fade{opacity:0}

/* Result phase */
#phase-result{display:none;padding:48px 0 0;animation:fadein .28s ease-out}
@keyframes fadein{from{opacity:0}to{opacity:1}}
.result-verdict{font-size:3.6rem;font-weight:900;letter-spacing:.02em;line-height:1;margin-bottom:8px}
.result-ok .result-verdict{color:var(--green)}
.result-blocked .result-verdict{color:var(--red)}
.result-sub{font-size:.88rem;color:var(--sub);margin-bottom:36px}

.result-reason{margin-bottom:36px}
.result-reason-text{font-size:.95rem;font-weight:600;color:var(--text);margin-bottom:8px}
.result-impact{font-size:.82rem;color:var(--sub);font-style:italic}

.reset-btn{
  background:none;border:1px solid var(--border);border-radius:8px;
  color:var(--sub);font-size:.85rem;font-weight:600;
  padding:12px 20px;cursor:pointer;transition:border-color .15s,color .15s;
}
.reset-btn:hover{border-color:var(--sub);color:var(--text)}
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

<!-- ────────────────── HERO ────────────────── -->
<section id="hero">
  <div class="inner">
    <h1 class="hero-h1" id="t-h1">Nicht jeder Fall<br>darf produziert werden.</h1>
    <p class="hero-sub" id="t-sub">PostCAD entscheidet das automatisch.</p>
    <a class="hero-cta" href="#demo">
      <span id="t-cta">Demo ansehen</span>
      <span class="hero-cta-arrow">↓</span>
    </a>
  </div>
</section>

<!-- ────────────────── FLOW ────────────────── -->
<section id="flow">
  <div class="inner">
    <div class="flow-heading" id="t-flow-heading">Ablauf</div>
    <div class="flow-track">

      <div class="flow-node">
        <div class="flow-spine">
          <div class="flow-dot lit"></div>
          <div class="flow-line"></div>
        </div>
        <div class="flow-body">
          <div class="flow-n">01</div>
          <div class="flow-label" id="t-f1">CAD-Datei erstellt</div>
        </div>
      </div>

      <div class="flow-node">
        <div class="flow-spine">
          <div class="flow-dot lit"></div>
          <div class="flow-line"></div>
        </div>
        <div class="flow-body">
          <div class="flow-n">02</div>
          <div class="flow-label primary" id="t-f2">PostCAD prüft automatisch</div>
        </div>
      </div>

      <div class="flow-outcomes">
        <div class="flow-outcome-blocked">
          <div class="outcome-v" id="t-f3-blocked">BLOCKIERT</div>
          <div class="outcome-sub" id="t-f3-blocked-sub">Weitergabe nicht möglich</div>
        </div>
        <div class="flow-outcome-ok">
          <div class="outcome-v" id="t-f3-ok">FREIGEGEBEN</div>
          <div class="outcome-sub" id="t-f3-ok-sub">Weitergabe möglich</div>
        </div>
      </div>

    </div>
  </div>
</section>

<!-- ────────────────── DEMO ────────────────── -->
<section id="demo">
  <div class="inner">
    <div class="demo-heading" id="t-demo-heading">Interaktive Demo</div>

    <!-- Phase: select -->
    <div id="phase-select">
      <div class="scenario-rows">
        <div class="scenario-row" onclick="startFlow(0)">
          <div class="sc-body">
            <div class="sc-title" id="t-s0-title">Zulässig</div>
            <div class="sc-desc" id="t-s0-desc">Krone · Zirkon · Deutschland</div>
          </div>
          <div class="sc-signal sig-ok" id="t-s0-chip">Freigegeben</div>
        </div>
        <div class="scenario-row" onclick="startFlow(1)">
          <div class="sc-body">
            <div class="sc-title" id="t-s1-title">Nicht zulässig</div>
            <div class="sc-desc" id="t-s1-desc">Krone · Zirkon · USA</div>
          </div>
          <div class="sc-signal sig-block" id="t-s1-chip">Blockiert</div>
        </div>
        <div class="scenario-row" onclick="startFlow(2)">
          <div class="sc-body">
            <div class="sc-title" id="t-s2-title">Nicht erfüllbar</div>
            <div class="sc-desc" id="t-s2-desc">Brücke · E.max · Deutschland</div>
          </div>
          <div class="sc-signal sig-block" id="t-s2-chip">Blockiert</div>
        </div>
      </div>
    </div>

    <!-- Phase: check -->
    <div id="phase-check">
      <div class="check-text" id="check-text"></div>
    </div>

    <!-- Phase: result -->
    <div id="phase-result">
      <div id="result-block">
        <div class="result-verdict" id="result-verdict"></div>
        <div class="result-sub" id="result-sub"></div>
      </div>
      <div id="result-reason">
        <div class="result-reason-text" id="grund-text"></div>
        <div class="result-impact" id="t-impact"></div>
      </div>
      <button class="reset-btn" onclick="reset()"><span id="t-reset">Anderen Fall prüfen</span></button>
    </div>

  </div>
</section>

<script>
const T = {
  DE: {
    h1: 'Nicht jeder Fall<br>darf produziert werden.',
    sub: 'PostCAD entscheidet das automatisch.',
    cta: 'Demo ansehen',
    flowHeading: 'Ablauf',
    f1: 'CAD-Datei erstellt',
    f2: 'PostCAD prüft automatisch',
    f3Blocked: 'BLOCKIERT', f3BlockedSub: 'Weitergabe nicht möglich',
    f3Ok: 'FREIGEGEBEN', f3OkSub: 'Weitergabe möglich',
    demoHeading: 'Interaktive Demo',
    s0Title: 'Zulässig', s0Desc: 'Krone · Zirkon · Deutschland', s0Chip: 'Freigegeben',
    s1Title: 'Nicht zulässig', s1Desc: 'Krone · Zirkon · USA', s1Chip: 'Blockiert',
    s2Title: 'Nicht erfüllbar', s2Desc: 'Brücke · E.max · Deutschland', s2Chip: 'Blockiert',
    checkStep: 'Prüfe Fall \u2026',
    verdictOk: 'FREIGEGEBEN', verdictBlock: 'BLOCKIERT',
    subOk: 'Weitergabe möglich', subBlock: 'Weitergabe nicht möglich',
    impact: 'Dieser Fall wäre sonst in die Fertigung gegangen.',
    reasons: {
      no_jurisdiction_match: 'Jurisdiktion nicht zulässig',
      no_material_match: 'Kein passender Fertigungspartner',
      default: 'Eingabedaten unvollständig',
    },
    reset: 'Anderen Fall prüfen',
  },
  EN: {
    h1: 'Not every case<br>may be manufactured.',
    sub: 'PostCAD decides automatically.',
    cta: 'View demo',
    flowHeading: 'Process',
    f1: 'CAD file created',
    f2: 'PostCAD checks automatically',
    f3Blocked: 'BLOCKED', f3BlockedSub: 'Cannot proceed',
    f3Ok: 'APPROVED', f3OkSub: 'Safe to proceed',
    demoHeading: 'Interactive Demo',
    s0Title: 'Permissible', s0Desc: 'Crown · Zirconia · Germany', s0Chip: 'Approved',
    s1Title: 'Not permissible', s1Desc: 'Crown · Zirconia · USA', s1Chip: 'Blocked',
    s2Title: 'Cannot be fulfilled', s2Desc: 'Bridge · E.max · Germany', s2Chip: 'Blocked',
    checkStep: 'Checking case \u2026',
    verdictOk: 'APPROVED', verdictBlock: 'BLOCKED',
    subOk: 'Safe to proceed', subBlock: 'Cannot proceed',
    impact: 'This case would otherwise have entered manufacturing.',
    reasons: {
      no_jurisdiction_match: 'Jurisdiction not allowed',
      no_material_match: 'No matching manufacturing partner',
      default: 'Input data incomplete',
    },
    reset: 'Check another case',
  },
};

const CASES = [
  {
    pilot_case:{case_id:'f1000001',patient_ref:'P-001',procedure_type:'Crown',material:'Zirconia',file_type:'STL',source_country:'DE',destination_country:'DE'},
    routing_config:{strategy:'HighestPriority',jurisdiction:'DE'},
  },
  {
    pilot_case:{case_id:'f1000002',patient_ref:'P-002',procedure_type:'Crown',material:'Zirconia',file_type:'STL',source_country:'US',destination_country:'US'},
    routing_config:{strategy:'HighestPriority',jurisdiction:'US'},
  },
  {
    pilot_case:{case_id:'f1000003',patient_ref:'P-003',procedure_type:'Bridge',material:'Emax',file_type:'STL',source_country:'DE',destination_country:'DE'},
    routing_config:{strategy:'HighestPriority',jurisdiction:'DE'},
  },
];

let lang = 'DE';

function setLang(l) {
  lang = l;
  document.getElementById('btn-de').classList.toggle('active', l === 'DE');
  document.getElementById('btn-en').classList.toggle('active', l === 'EN');
  const t = T[l];
  document.getElementById('t-h1').innerHTML = t.h1;
  document.getElementById('t-sub').textContent = t.sub;
  document.getElementById('t-cta').textContent = t.cta;
  document.getElementById('t-flow-heading').textContent = t.flowHeading;
  document.getElementById('t-f1').textContent = t.f1;
  document.getElementById('t-f2').textContent = t.f2;
  document.getElementById('t-f3-blocked').textContent = t.f3Blocked;
  document.getElementById('t-f3-blocked-sub').textContent = t.f3BlockedSub;
  document.getElementById('t-f3-ok').textContent = t.f3Ok;
  document.getElementById('t-f3-ok-sub').textContent = t.f3OkSub;
  document.getElementById('t-demo-heading').textContent = t.demoHeading;
  document.getElementById('t-s0-title').textContent = t.s0Title;
  document.getElementById('t-s0-desc').textContent = t.s0Desc;
  document.getElementById('t-s0-chip').textContent = t.s0Chip;
  document.getElementById('t-s1-title').textContent = t.s1Title;
  document.getElementById('t-s1-desc').textContent = t.s1Desc;
  document.getElementById('t-s1-chip').textContent = t.s1Chip;
  document.getElementById('t-s2-title').textContent = t.s2Title;
  document.getElementById('t-s2-desc').textContent = t.s2Desc;
  document.getElementById('t-s2-chip').textContent = t.s2Chip;
  document.getElementById('t-reset').textContent = t.reset;
}

async function callAPIs(c) {
  try {
    const routeRes = await fetch('/pilot/route-normalized', {
      method:'POST', headers:{'Content-Type':'application/json'}, body:JSON.stringify(c),
    });
    const routeData = await routeRes.json();
    if (routeData.outcome !== 'routed') {
      return {ok:false, refusalCode: routeData.refusal_code || 'default'};
    }
    const verifyRes = await fetch('/verify', {
      method:'POST', headers:{'Content-Type':'application/json'},
      body:JSON.stringify(routeData.receipt || routeData),
    });
    const verifyData = await verifyRes.json();
    try {
      await fetch('/dispatch/create', {
        method:'POST', headers:{'Content-Type':'application/json'},
        body:JSON.stringify({receipt_hash: verifyData.receipt_hash || verifyData.hash || 'unknown'}),
      });
    } catch(_) {}
    return {ok:true, refusalCode:'ok'};
  } catch(e) {
    return {ok:false, refusalCode:'default'};
  }
}

async function startFlow(i) {
  const t = T[lang];
  const checkEl = document.getElementById('check-text');
  checkEl.classList.remove('fade');
  checkEl.textContent = t.checkStep;

  document.getElementById('phase-select').style.display = 'none';
  document.getElementById('phase-check').style.display = 'block';
  document.getElementById('phase-result').style.display = 'none';

  const [result] = await Promise.all([
    callAPIs(CASES[i]),
    (async () => { await delay(700); checkEl.classList.add('fade'); await delay(200); })(),
  ]);

  showResult(result);
}

function showResult(result) {
  const t = T[lang];
  document.getElementById('phase-check').style.display = 'none';
  document.getElementById('phase-result').style.display = 'block';

  const block = document.getElementById('result-block');
  if (result.ok) {
    block.className = 'result-ok';
    document.getElementById('result-verdict').textContent = t.verdictOk;
    document.getElementById('result-sub').textContent = t.subOk;
    document.getElementById('result-reason').style.display = 'none';
  } else {
    block.className = 'result-blocked';
    document.getElementById('result-verdict').textContent = t.verdictBlock;
    document.getElementById('result-sub').textContent = t.subBlock;
    const key = result.refusalCode in t.reasons ? result.refusalCode : 'default';
    document.getElementById('grund-text').textContent = t.reasons[key];
    document.getElementById('t-impact').textContent = t.impact;
    document.getElementById('result-reason').style.display = 'block';
  }
}

function reset() {
  document.getElementById('phase-result').style.display = 'none';
  document.getElementById('phase-check').style.display = 'none';
  document.getElementById('phase-select').style.display = 'block';
}

function delay(ms) { return new Promise(r => setTimeout(r, ms)); }
</script>
</body>
</html>
"##;
