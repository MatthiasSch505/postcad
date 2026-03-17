//! Public landing page for PostCAD.
//!
//! Served at `GET /`. Static marketing surface — no backend calls, no state.
//! Links to `GET /reviewer` for the live operator demo.

pub const OPERATOR_UI_HTML: &str = r##"<!DOCTYPE html>
<html lang="de">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PostCAD</title>
<style>
  *{box-sizing:border-box;margin:0;padding:0}
  :root{
    --bg:#0d0f12;
    --surface:#161a1f;
    --surface2:#1d2229;
    --border:#252b34;
    --green:#22c55e;
    --green-dim:#15803d;
    --text:#f1f5f9;
    --muted:#64748b;
    --accent:#3b82f6;
  }
  body{background:var(--bg);color:var(--text);font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;min-height:100vh;display:flex;flex-direction:column;align-items:center;padding:0 20px 64px}

  header{width:100%;max-width:480px;display:flex;align-items:center;justify-content:space-between;padding:20px 0 0}
  .brand{font-size:.85rem;font-weight:700;letter-spacing:.08em;color:var(--muted);text-transform:uppercase}
  .brand span{color:var(--green)}
  .lang-toggle{display:flex;gap:2px;background:var(--surface);border:1px solid var(--border);border-radius:6px;overflow:hidden}
  .lang-btn{padding:4px 10px;font-size:.75rem;font-weight:600;border:none;background:transparent;color:var(--muted);cursor:pointer;transition:all .15s}
  .lang-btn.active{background:var(--surface2);color:var(--text)}

  main{width:100%;max-width:480px}

  /* Hero */
  .hero{padding:52px 0 44px;text-align:center}
  .hero-eyebrow{font-size:.7rem;font-weight:700;letter-spacing:.14em;color:var(--accent);text-transform:uppercase;margin-bottom:12px}
  .hero h1{font-size:2rem;font-weight:900;line-height:1.18;margin-bottom:14px}
  .hero-sub{font-size:.95rem;color:var(--muted);line-height:1.6;margin-bottom:32px;max-width:380px;margin-left:auto;margin-right:auto}
  .cta-row{display:flex;flex-direction:column;gap:10px;align-items:center}
  .cta-primary{
    display:inline-block;padding:15px 32px;border-radius:10px;
    background:var(--green);color:#fff;font-size:1rem;font-weight:700;
    text-decoration:none;letter-spacing:.01em;transition:opacity .15s;width:100%;text-align:center;
  }
  .cta-primary:hover{opacity:.9}
  .cta-secondary{font-size:.85rem;color:var(--muted);text-decoration:none;padding:6px 0;border-bottom:1px solid transparent;transition:color .15s,border-color .15s}
  .cta-secondary:hover{color:var(--text);border-bottom-color:var(--border)}

  /* Phone preview card */
  .phone-wrap{margin-bottom:44px}
  .phone-card{
    background:var(--surface);border:1.5px solid var(--border);border-radius:20px;
    padding:22px 20px;position:relative;overflow:hidden;
  }
  .phone-bar{display:flex;align-items:center;gap:8px;margin-bottom:18px}
  .phone-dot{width:8px;height:8px;border-radius:50%}
  .phone-title{font-size:.78rem;font-weight:600;color:var(--muted);flex:1;text-align:center;padding-right:16px}
  .mini-scenario{background:var(--surface2);border:1px solid var(--border);border-radius:10px;padding:12px 14px;display:flex;align-items:center;gap:10px;margin-bottom:10px}
  .mini-sc-icon{font-size:1.1rem}
  .mini-sc-body{flex:1}
  .mini-sc-title{font-size:.82rem;font-weight:700;margin-bottom:2px}
  .mini-sc-desc{font-size:.72rem;color:var(--muted)}
  .mini-cta{width:100%;padding:11px;border-radius:8px;background:var(--green);border:none;color:#fff;font-size:.85rem;font-weight:700;margin-bottom:14px}
  .mini-result{background:linear-gradient(135deg,#052e16,#14532d);border:1px solid var(--green-dim);border-radius:10px;padding:14px;text-align:center}
  .mini-result-verdict{font-size:1.1rem;font-weight:900;color:var(--green);letter-spacing:.06em;margin-bottom:2px}
  .mini-result-sub{font-size:.72rem;color:#86efac}
  .live-badge{position:absolute;top:16px;right:16px;display:flex;align-items:center;gap:5px;font-size:.65rem;font-weight:700;color:var(--green);letter-spacing:.06em;text-transform:uppercase}
  .live-dot{width:6px;height:6px;border-radius:50%;background:var(--green);animation:pulse 1.8s ease-in-out infinite}
  @keyframes pulse{0%,100%{opacity:1;transform:scale(1)}50%{opacity:.4;transform:scale(.8)}}
  .phone-link{display:block;text-align:center;font-size:.8rem;color:var(--muted);text-decoration:none;margin-top:12px;transition:color .15s}
  .phone-link:hover{color:var(--text)}

  /* Support blocks */
  .blocks{display:flex;flex-direction:column;gap:10px}
  .block{background:var(--surface);border:1px solid var(--border);border-radius:12px;padding:16px 18px;display:flex;align-items:flex-start;gap:14px}
  .block-icon{font-size:1.3rem;flex-shrink:0;margin-top:1px}
  .block-title{font-size:.88rem;font-weight:700;margin-bottom:3px}
  .block-desc{font-size:.8rem;color:var(--muted);line-height:1.45}
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

  <!-- Hero -->
  <section class="hero">
    <div class="hero-eyebrow" id="t-eyebrow">Dental · Fertigung · Compliance</div>
    <h1 id="t-h1">Digitale Fälle sicher<br>in die Fertigung geben.</h1>
    <p class="hero-sub" id="t-sub">PostCAD prüft jeden Fall automatisch auf Regularien, Materialeignung und Laborzulassung — bevor die Fertigung beginnt.</p>
    <div class="cta-row">
      <a class="cta-primary" href="/reviewer" id="t-cta-primary">Live-Demo starten</a>
      <a class="cta-secondary" href="#" id="t-cta-secondary">Für Pilot vormerken</a>
    </div>
  </section>

  <!-- Phone preview -->
  <div class="phone-wrap">
    <a href="/reviewer" style="text-decoration:none">
      <div class="phone-card">
        <div class="live-badge"><div class="live-dot"></div><span id="t-live">Live</span></div>
        <div class="phone-bar">
          <div class="phone-dot" style="background:#ef4444"></div>
          <div class="phone-dot" style="background:#f59e0b"></div>
          <div class="phone-dot" style="background:#22c55e"></div>
          <div class="phone-title" id="t-phone-title">Fallprüfung vor Fertigung</div>
        </div>
        <div class="mini-scenario">
          <div class="mini-sc-icon">🦷</div>
          <div class="mini-sc-body">
            <div class="mini-sc-title" id="t-mini-title">Standardfall</div>
            <div class="mini-sc-desc" id="t-mini-desc">Krone · Zirkon · Deutschland</div>
          </div>
        </div>
        <button class="mini-cta" id="t-mini-cta">Diesen Fall prüfen</button>
        <div class="mini-result">
          <div class="mini-result-verdict" id="t-mini-verdict">✓ FREIGEGEBEN</div>
          <div class="mini-result-sub" id="t-mini-sub">Weitergabe möglich</div>
        </div>
      </div>
    </a>
    <a class="phone-link" href="/reviewer" id="t-phone-link">Demo öffnen →</a>
  </div>

  <!-- Support blocks -->
  <div class="blocks">
    <div class="block">
      <div class="block-icon">✓</div>
      <div>
        <div class="block-title" id="t-b1-title">Prüft Regeln</div>
        <div class="block-desc" id="t-b1-desc">CE-Kennzeichnung, FDA-Zulassung, ISO 13485 — automatisch und nachvollziehbar.</div>
      </div>
    </div>
    <div class="block">
      <div class="block-icon">✕</div>
      <div>
        <div class="block-title" id="t-b2-title">Blockiert Fehler</div>
        <div class="block-desc" id="t-b2-desc">Unzulässige Jurisdiktionen und nicht verfügbare Materialien werden vor der Weitergabe gestoppt.</div>
      </div>
    </div>
    <div class="block">
      <div class="block-icon">📋</div>
      <div>
        <div class="block-title" id="t-b3-title">Dokumentiert Entscheidung</div>
        <div class="block-desc" id="t-b3-desc">Jede Freigabe und jede Ablehnung wird mit Begründung gespeichert.</div>
      </div>
    </div>
  </div>

</main>

<script>
const LP = {
  DE: {
    eyebrow: 'Dental · Fertigung · Compliance',
    h1: 'Digitale Fälle sicher<br>in die Fertigung geben.',
    sub: 'PostCAD prüft jeden Fall automatisch auf Regularien, Materialeignung und Laborzulassung — bevor die Fertigung beginnt.',
    ctaPrimary: 'Live-Demo starten',
    ctaSecondary: 'Für Pilot vormerken',
    live: 'Live',
    phoneTitle: 'Fallprüfung vor Fertigung',
    miniTitle: 'Standardfall', miniDesc: 'Krone · Zirkon · Deutschland',
    miniCta: 'Diesen Fall prüfen',
    miniVerdict: '\u2713 FREIGEGEBEN', miniSub: 'Weitergabe möglich',
    phoneLink: 'Demo öffnen →',
    b1Title: 'Prüft Regeln', b1Desc: 'CE-Kennzeichnung, FDA-Zulassung, ISO 13485 — automatisch und nachvollziehbar.',
    b2Title: 'Blockiert Fehler', b2Desc: 'Unzulässige Jurisdiktionen und nicht verfügbare Materialien werden vor der Weitergabe gestoppt.',
    b3Title: 'Dokumentiert Entscheidung', b3Desc: 'Jede Freigabe und jede Ablehnung wird mit Begründung gespeichert.',
  },
  EN: {
    eyebrow: 'Dental · Manufacturing · Compliance',
    h1: 'Pass digital cases safely<br>into manufacturing.',
    sub: 'PostCAD automatically checks every case for regulations, material suitability, and lab certification — before manufacturing begins.',
    ctaPrimary: 'Start live demo',
    ctaSecondary: 'Register for pilot',
    live: 'Live',
    phoneTitle: 'Pre-Manufacturing Case Review',
    miniTitle: 'Standard case', miniDesc: 'Crown · Zirconia · Germany',
    miniCta: 'Check this case',
    miniVerdict: '\u2713 APPROVED', miniSub: 'Safe to proceed',
    phoneLink: 'Open demo →',
    b1Title: 'Checks rules', b1Desc: 'CE marking, FDA clearance, ISO 13485 — automatically and traceably.',
    b2Title: 'Blocks errors', b2Desc: 'Invalid jurisdictions and unavailable materials are stopped before handoff.',
    b3Title: 'Documents decisions', b3Desc: 'Every approval and refusal is stored with its reason.',
  },
};

let lang = 'DE';

function setLang(l) {
  lang = l;
  document.getElementById('btn-de').classList.toggle('active', l === 'DE');
  document.getElementById('btn-en').classList.toggle('active', l === 'EN');
  const t = LP[l];
  document.getElementById('t-eyebrow').textContent = t.eyebrow;
  document.getElementById('t-h1').innerHTML = t.h1;
  document.getElementById('t-sub').textContent = t.sub;
  document.getElementById('t-cta-primary').textContent = t.ctaPrimary;
  document.getElementById('t-cta-secondary').textContent = t.ctaSecondary;
  document.getElementById('t-live').textContent = t.live;
  document.getElementById('t-phone-title').textContent = t.phoneTitle;
  document.getElementById('t-mini-title').textContent = t.miniTitle;
  document.getElementById('t-mini-desc').textContent = t.miniDesc;
  document.getElementById('t-mini-cta').textContent = t.miniCta;
  document.getElementById('t-mini-verdict').textContent = t.miniVerdict;
  document.getElementById('t-mini-sub').textContent = t.miniSub;
  document.getElementById('t-phone-link').textContent = t.phoneLink;
  document.getElementById('t-b1-title').textContent = t.b1Title;
  document.getElementById('t-b1-desc').textContent = t.b1Desc;
  document.getElementById('t-b2-title').textContent = t.b2Title;
  document.getElementById('t-b2-desc').textContent = t.b2Desc;
  document.getElementById('t-b3-title').textContent = t.b3Title;
  document.getElementById('t-b3-desc').textContent = t.b3Desc;
}
</script>
</body>
</html>
"##;
