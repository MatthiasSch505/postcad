//! Public landing page for PostCAD.
//!
//! Served at `GET /`. Minimal hero: headline, subtitle, safety note, CTA to /reviewer, feature list.

pub const OPERATOR_UI_HTML: &str = r##"<!DOCTYPE html>
<html lang="de">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PostCAD</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
:root{
  --bg:#f8fafc;
  --border:#e2e8f0;
  --text:#0f172a;
  --sub:#475569;
  --dim:#94a3b8;
  --accent:#1d4ed8;
}
html{scroll-behavior:smooth}
body{background:var(--bg);color:var(--text);font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;min-height:100vh}
header{position:fixed;top:0;left:0;right:0;display:flex;align-items:center;justify-content:space-between;padding:18px 32px;z-index:100;background:var(--bg);border-bottom:1px solid var(--border)}
.brand{font-size:.78rem;font-weight:700;letter-spacing:.12em;color:var(--dim);text-transform:uppercase}
.brand span{color:var(--sub)}
.lang-toggle{display:flex;gap:1px}
.lang-btn{padding:3px 9px;font-size:.7rem;font-weight:600;border:1px solid var(--border);background:transparent;color:var(--dim);cursor:pointer;border-radius:4px;transition:color .15s,border-color .15s}
.lang-btn.active{color:var(--sub);border-color:var(--dim)}
#hero{min-height:100vh;display:flex;align-items:center;padding:0 24px}
.inner{max-width:600px;margin:0 auto}
#hero .inner{padding:96px 0 80px}
.hero-h1{font-size:clamp(2rem,5.5vw,3.2rem);font-weight:900;line-height:1.1;letter-spacing:-.02em;margin-bottom:20px}
.hero-sub{font-size:1rem;color:var(--sub);line-height:1.65;margin-bottom:24px;max-width:480px}
.safety-note{font-size:.8rem;color:var(--dim);line-height:1.6;margin-bottom:40px;max-width:480px;border-left:2px solid var(--border);padding-left:12px}
.hero-cta{display:inline-flex;align-items:center;gap:10px;font-size:.95rem;font-weight:600;color:#fff;background:var(--accent);text-decoration:none;padding:13px 24px;border-radius:8px;transition:background .15s}
.hero-cta:hover{background:#1e3a8a}
.cta-arrow{transition:transform .15s}
.hero-cta:hover .cta-arrow{transform:translateX(3px)}
#features{padding:80px 24px 100px;border-top:1px solid var(--border)}
.features-heading{font-size:.62rem;font-weight:700;letter-spacing:.12em;color:var(--dim);text-transform:uppercase;margin-bottom:32px}
.feature-item{display:flex;gap:16px;align-items:flex-start;margin-bottom:24px}
.feature-dot{width:6px;height:6px;border-radius:50%;background:var(--accent);flex-shrink:0;margin-top:7px}
.feature-text{font-size:1rem;color:var(--sub);line-height:1.5}
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

<section id="hero">
  <div class="inner">
    <h1 class="hero-h1" id="t-h1">Digitale Fälle vor der Herstellung klar dokumentieren.</h1>
    <p class="hero-sub" id="t-sub">PostCAD hilft Praxis und Labor, STL-/Scan-Fälle vor der Fertigung visuell zu klären, Hinweise festzuhalten und Entscheidungen nachvollziehbar zu dokumentieren.</p>
    <p class="safety-note" id="t-safety">PostCAD erkennt keine medizinischen oder technischen Fehler und gibt keine Herstellung frei.</p>
    <a class="hero-cta" href="/reviewer">
      <span id="t-cta">Reviewer öffnen</span>
      <span class="cta-arrow">→</span>
    </a>
  </div>
</section>

<section id="features">
  <div class="inner">
    <div class="features-heading" id="t-features-heading">Funktionen</div>
    <div class="feature-item">
      <div class="feature-dot"></div>
      <div class="feature-text" id="t-f1">STL/Scan im Browser ansehen</div>
    </div>
    <div class="feature-item">
      <div class="feature-dot"></div>
      <div class="feature-text" id="t-f2">Hinweis an die Praxis dokumentieren</div>
    </div>
    <div class="feature-item">
      <div class="feature-dot"></div>
      <div class="feature-text" id="t-f3">Entscheidungsnachweis erzeugen</div>
    </div>
  </div>
</section>

<script>
const T = {
  DE: {
    h1: 'Digitale Fälle vor der Herstellung klar dokumentieren.',
    sub: 'PostCAD hilft Praxis und Labor, STL-/Scan-Fälle vor der Fertigung visuell zu klären, Hinweise festzuhalten und Entscheidungen nachvollziehbar zu dokumentieren.',
    safety: 'PostCAD erkennt keine medizinischen oder technischen Fehler und gibt keine Herstellung frei.',
    cta: 'Reviewer öffnen',
    featuresHeading: 'Funktionen',
    f1: 'STL/Scan im Browser ansehen',
    f2: 'Hinweis an die Praxis dokumentieren',
    f3: 'Entscheidungsnachweis erzeugen',
  },
  EN: {
    h1: 'Clearly document digital cases before manufacturing.',
    sub: 'PostCAD helps practices and labs visually review STL/scan cases before manufacturing, record notes, and document decisions traceably.',
    safety: 'PostCAD does not detect medical or technical errors and does not release manufacturing.',
    cta: 'Open Reviewer',
    featuresHeading: 'Features',
    f1: 'View STL/scan in the browser',
    f2: 'Document a note to the practice',
    f3: 'Generate a decision receipt',
  },
};
let lang = 'DE';
function setLang(l) {
  lang = l;
  document.getElementById('btn-de').classList.toggle('active', l === 'DE');
  document.getElementById('btn-en').classList.toggle('active', l === 'EN');
  const t = T[l];
  document.getElementById('t-h1').textContent = t.h1;
  document.getElementById('t-sub').textContent = t.sub;
  document.getElementById('t-safety').textContent = t.safety;
  document.getElementById('t-cta').textContent = t.cta;
  document.getElementById('t-features-heading').textContent = t.featuresHeading;
  document.getElementById('t-f1').textContent = t.f1;
  document.getElementById('t-f2').textContent = t.f2;
  document.getElementById('t-f3').textContent = t.f3;
}
</script>
</body>
</html>
"##;
