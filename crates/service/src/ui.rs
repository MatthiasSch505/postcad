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
  }
  body{background:var(--bg);color:var(--text);font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;min-height:100vh;display:flex;flex-direction:column;align-items:center;padding:0 20px 64px}

  header{width:100%;max-width:480px;display:flex;align-items:center;justify-content:space-between;padding:20px 0 0}
  .brand{font-size:.85rem;font-weight:700;letter-spacing:.08em;color:var(--muted);text-transform:uppercase}
  .brand span{color:var(--green)}
  .lang-toggle{display:flex;gap:2px;background:var(--surface);border:1px solid var(--border);border-radius:6px;overflow:hidden}
  .lang-btn{padding:4px 10px;font-size:.75rem;font-weight:600;border:none;background:transparent;color:var(--muted);cursor:pointer;transition:color .15s}
  .lang-btn.active{background:var(--surface2);color:var(--text)}

  main{width:100%;max-width:480px}

  /* Hero */
  .hero{padding:52px 0 44px;text-align:center}
  .hero h1{font-size:2rem;font-weight:900;line-height:1.2;margin-bottom:12px}
  .hero-sub{font-size:.92rem;color:var(--muted);line-height:1.6;margin-bottom:32px;max-width:360px;margin-left:auto;margin-right:auto}
  .cta-row{display:flex;flex-direction:column;gap:10px;align-items:center}
  .cta-primary{
    display:block;padding:14px 32px;border-radius:10px;
    background:var(--green);color:#fff;font-size:.95rem;font-weight:700;
    text-decoration:none;letter-spacing:.01em;transition:opacity .15s;width:100%;text-align:center;
  }
  .cta-primary:hover{opacity:.9}
  .cta-secondary{font-size:.82rem;color:var(--muted);text-decoration:none;padding:6px 0;transition:color .15s}
  .cta-secondary:hover{color:var(--text)}

  /* Preview */
  .preview-wrap{margin-bottom:44px}
  .preview-card{
    background:var(--surface);border:1.5px solid var(--border);border-radius:16px;
    padding:24px 22px;position:relative;
  }
  .live-badge{position:absolute;top:18px;right:18px;display:flex;align-items:center;gap:5px;font-size:.65rem;font-weight:700;color:var(--green);letter-spacing:.06em;text-transform:uppercase}
  .live-dot{width:6px;height:6px;border-radius:50%;background:var(--green);animation:pulse 1.8s ease-in-out infinite}
  @keyframes pulse{0%,100%{opacity:1;transform:scale(1)}50%{opacity:.4;transform:scale(.8)}}
  .preview-label{font-size:.7rem;font-weight:700;letter-spacing:.1em;color:var(--muted);text-transform:uppercase;margin-bottom:14px}
  .mini-scenario{display:flex;align-items:center;gap:10px;margin-bottom:16px}
  .mini-sc-icon{font-size:1.1rem}
  .mini-sc-body{flex:1}
  .mini-sc-title{font-size:.84rem;font-weight:700;margin-bottom:2px}
  .mini-sc-desc{font-size:.73rem;color:var(--muted)}
  .mini-result{border-top:1px solid var(--border);padding-top:16px;text-align:center}
  .mini-verdict{font-size:1.5rem;font-weight:900;color:var(--green);letter-spacing:.04em;margin-bottom:3px}
  .mini-sub{font-size:.76rem;color:var(--muted)}
  .preview-link{display:block;text-align:center;font-size:.8rem;color:var(--muted);text-decoration:none;margin-top:12px;transition:color .15s}
  .preview-link:hover{color:var(--text)}

  /* Support rows */
  .supports{display:flex;flex-direction:column}
  .support-row{display:flex;align-items:flex-start;gap:12px;padding:14px 0;border-top:1px solid var(--border)}
  .support-icon{font-size:1rem;flex-shrink:0;margin-top:1px;color:var(--muted)}
  .support-title{font-size:.85rem;font-weight:700;margin-bottom:2px}
  .support-desc{font-size:.78rem;color:var(--muted);line-height:1.5}
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
    <h1 id="t-h1">Digitale Fälle sicher<br>weitergeben.</h1>
    <p class="hero-sub" id="t-sub">PostCAD prüft, ob ein Fall zulässig ist — bevor er in die Fertigung geht.</p>
    <div class="cta-row">
      <a class="cta-primary" href="/reviewer" id="t-cta-primary">Live-Demo starten</a>
      <a class="cta-secondary" href="#" id="t-cta-secondary">Für Pilot vormerken</a>
    </div>
  </section>

  <!-- Preview -->
  <div class="preview-wrap">
    <a href="/reviewer" style="text-decoration:none">
      <div class="preview-card">
        <div class="live-badge"><div class="live-dot"></div><span id="t-live">Live</span></div>
        <div class="preview-label" id="t-preview-label">Fallprüfung</div>
        <div class="mini-scenario">
          <div class="mini-sc-icon">🦷</div>
          <div class="mini-sc-body">
            <div class="mini-sc-title" id="t-mini-title">Standardfall</div>
            <div class="mini-sc-desc" id="t-mini-desc">Krone · Zirkon · Deutschland</div>
          </div>
        </div>
        <div class="mini-result">
          <div class="mini-verdict" id="t-mini-verdict">✓ FREIGEGEBEN</div>
          <div class="mini-sub" id="t-mini-sub">Weitergabe möglich</div>
        </div>
      </div>
    </a>
    <a class="preview-link" href="/reviewer" id="t-preview-link">Demo öffnen →</a>
  </div>

  <!-- Support rows -->
  <div class="supports">
    <div class="support-row">
      <div class="support-icon">✓</div>
      <div>
        <div class="support-title" id="t-b1-title">Prüft Regeln</div>
        <div class="support-desc" id="t-b1-desc">CE-Kennzeichnung, FDA-Zulassung, ISO 13485 — automatisch und nachvollziehbar.</div>
      </div>
    </div>
    <div class="support-row">
      <div class="support-icon">✕</div>
      <div>
        <div class="support-title" id="t-b2-title">Blockiert Fehler</div>
        <div class="support-desc" id="t-b2-desc">Unzulässige Jurisdiktionen und nicht verfügbare Materialien werden vor der Weitergabe gestoppt.</div>
      </div>
    </div>
    <div class="support-row">
      <div class="support-icon">◎</div>
      <div>
        <div class="support-title" id="t-b3-title">Dokumentiert Entscheidung</div>
        <div class="support-desc" id="t-b3-desc">Jede Freigabe und jede Ablehnung wird mit Begründung gespeichert.</div>
      </div>
    </div>
  </div>

</main>

<script>
const LP = {
  DE: {
    h1: 'Digitale Fälle sicher<br>weitergeben.',
    sub: 'PostCAD prüft, ob ein Fall zulässig ist — bevor er in die Fertigung geht.',
    ctaPrimary: 'Live-Demo starten',
    ctaSecondary: 'Für Pilot vormerken',
    live: 'Live',
    previewLabel: 'Fallprüfung',
    miniTitle: 'Standardfall', miniDesc: 'Krone · Zirkon · Deutschland',
    miniVerdict: '\u2713 FREIGEGEBEN', miniSub: 'Weitergabe möglich',
    previewLink: 'Demo öffnen →',
    b1Title: 'Prüft Regeln', b1Desc: 'CE-Kennzeichnung, FDA-Zulassung, ISO 13485 — automatisch und nachvollziehbar.',
    b2Title: 'Blockiert Fehler', b2Desc: 'Unzulässige Jurisdiktionen und nicht verfügbare Materialien werden vor der Weitergabe gestoppt.',
    b3Title: 'Dokumentiert Entscheidung', b3Desc: 'Jede Freigabe und jede Ablehnung wird mit Begründung gespeichert.',
  },
  EN: {
    h1: 'Pass digital cases<br>safely forward.',
    sub: 'PostCAD checks whether a case is allowed before it enters manufacturing.',
    ctaPrimary: 'Start live demo',
    ctaSecondary: 'Register for pilot',
    live: 'Live',
    previewLabel: 'Case Review',
    miniTitle: 'Standard case', miniDesc: 'Crown · Zirconia · Germany',
    miniVerdict: '\u2713 APPROVED', miniSub: 'Safe to proceed',
    previewLink: 'Open demo →',
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
  document.getElementById('t-h1').innerHTML = t.h1;
  document.getElementById('t-sub').textContent = t.sub;
  document.getElementById('t-cta-primary').textContent = t.ctaPrimary;
  document.getElementById('t-cta-secondary').textContent = t.ctaSecondary;
  document.getElementById('t-live').textContent = t.live;
  document.getElementById('t-preview-label').textContent = t.previewLabel;
  document.getElementById('t-mini-title').textContent = t.miniTitle;
  document.getElementById('t-mini-desc').textContent = t.miniDesc;
  document.getElementById('t-mini-verdict').textContent = t.miniVerdict;
  document.getElementById('t-mini-sub').textContent = t.miniSub;
  document.getElementById('t-preview-link').textContent = t.previewLink;
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
