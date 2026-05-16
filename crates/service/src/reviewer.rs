pub const REVIEWER_HTML: &str = r##"<!DOCTYPE html>
<html lang="de">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PostCAD · Klärung vor Herstellung</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
:root{
  --bg:#f4f6f8;
  --surface:#ffffff;
  --border:#dde3ea;
  --green:#059669;
  --amber:#b45309;
  --red:#dc2626;
  --text:#1a2332;
  --sub:#4e6078;
  --dim:#8a9bb0;
}
body{background:var(--bg);color:var(--text);font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;min-height:100vh;display:flex;flex-direction:column;align-items:center;padding:0 24px 64px}
header{width:100%;max-width:560px;display:flex;align-items:center;justify-content:space-between;padding:20px 0 44px}
.brand{font-size:.78rem;font-weight:700;letter-spacing:.12em;color:var(--dim);text-transform:uppercase}
.brand span{color:var(--sub)}
.brand-sub{font-size:.68rem;font-weight:600;letter-spacing:.1em;color:var(--dim);text-transform:uppercase;margin-top:3px}
.lang-toggle{display:flex;gap:1px}
.lang-btn{padding:3px 9px;font-size:.7rem;font-weight:600;border:1px solid var(--border);background:transparent;color:var(--dim);cursor:pointer;border-radius:4px;transition:color .15s,border-color .15s}
.lang-btn.active{color:var(--sub);border-color:var(--dim)}
main{width:100%;max-width:560px}

/* Upload */
.upload-zone{border:1px dashed var(--border);border-radius:8px;padding:44px 24px;text-align:center;cursor:pointer;display:block;transition:border-color .15s;margin-bottom:16px;text-decoration:none;color:inherit}
.upload-zone:hover,.upload-zone.drag-over{border-color:var(--sub)}
.upload-icon{font-size:1.4rem;color:var(--dim);margin-bottom:12px}
.upload-title{font-size:1.05rem;font-weight:700;margin-bottom:6px}
.upload-sub{font-size:.85rem;color:var(--sub)}
.demo-files{display:flex;gap:8px;flex-wrap:wrap}
.demo-file-btn{padding:10px 18px;border:1px solid var(--border);border-radius:6px;background:var(--surface);color:var(--sub);font-size:.9rem;font-weight:600;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;cursor:pointer;transition:border-color .15s,color .15s,box-shadow .15s}
.demo-file-btn:hover{border-color:var(--sub);color:var(--text);box-shadow:0 1px 4px rgba(26,35,50,.08)}

/* Processing */
#phase-processing{display:none;padding:52px 0;animation:fadein .15s ease-out}
.proc-filename{font-size:.85rem;color:var(--sub);font-family:'SF Mono','Fira Code',monospace;margin-bottom:20px}
.proc-step{font-size:.95rem;color:var(--sub);padding:6px 0;opacity:0;transition:opacity .3s}
.proc-step.visible{opacity:1}

/* Decision gate */
#phase-decision{display:none;padding:40px 0;animation:fadein .2s ease-out}
.gate-badge{font-size:.63rem;font-weight:800;letter-spacing:.16em;color:var(--amber);text-transform:uppercase;margin-bottom:10px}
.gate-title{font-size:1.7rem;font-weight:900;margin-bottom:10px;line-height:1.2}
.gate-sub{font-size:.88rem;color:var(--sub);line-height:1.55;margin-bottom:24px}
.gate-case-ctx{font-size:.8rem;color:var(--dim);font-family:'SF Mono','Fira Code',monospace;padding:9px 13px;background:var(--surface);border:1px solid var(--border);border-radius:6px;margin-bottom:24px}
.decision-choices{display:flex;flex-direction:column;gap:8px;margin-bottom:22px}
.choice-btn{padding:13px 16px;border:1px solid var(--border);border-radius:8px;background:transparent;color:var(--sub);font-size:.93rem;font-weight:600;text-align:left;cursor:pointer;transition:border-color .15s,color .15s,background .15s}
.choice-btn:hover{border-color:var(--sub);color:var(--text);background:rgba(78,96,120,.06)}
.choice-btn.sel-proceed{border:2px solid var(--green);color:var(--text);background:rgba(5,150,105,.10)}
.choice-btn.sel-risk{border:2px solid var(--amber);color:var(--amber);background:rgba(180,83,9,.08)}
.choice-btn.sel-block{border:2px solid var(--red);color:var(--red);background:rgba(220,38,38,.08)}
.reason-row{margin-bottom:22px;display:none}
.reason-label{font-size:.7rem;font-weight:700;letter-spacing:.1em;color:var(--dim);text-transform:uppercase;display:block;margin-bottom:8px}
#reason-code{width:100%;padding:10px 13px;background:var(--surface);border:1px solid var(--border);border-radius:6px;color:var(--text);font-size:.9rem;cursor:pointer;appearance:none}
#reason-code:focus{outline:none;border-color:var(--sub)}
.confirm-btn{width:100%;padding:15px;border:none;border-radius:8px;background:var(--green);color:#fff;font-size:1rem;font-weight:800;cursor:pointer;transition:opacity .15s;letter-spacing:.01em}
.confirm-btn:disabled{opacity:.25;cursor:not-allowed}
.confirm-btn:not(:disabled):hover{opacity:.85}
.confirm-hint{font-size:.78rem;color:var(--dim);margin-top:8px;text-align:center;min-height:1.1em}
.choice-hint{font-size:.78rem;color:var(--dim);padding:2px 16px 8px;line-height:1.4}
.aha-line{font-size:.85rem;color:var(--sub);line-height:1.55;font-style:italic}
.gate-error{font-size:.82rem;color:var(--red);margin-top:12px;text-align:center;display:none}

/* Result */
#phase-result{display:none;animation:fadein .25s ease-out}
@keyframes fadein{from{opacity:0}to{opacity:1}}
.res-section{padding:32px 0}
.res-label{font-size:.68rem;font-weight:700;letter-spacing:.12em;color:var(--dim);text-transform:uppercase;margin-bottom:16px}
.case-proc{font-size:1.35rem;font-weight:700;margin-bottom:14px;line-height:1.2}
.case-row{font-size:.92rem;color:var(--sub);margin-bottom:6px;display:flex;gap:8px}
.case-row-lbl{color:var(--dim)}
.result-verdict{font-size:clamp(2.4rem,8vw,3.2rem);font-weight:900;letter-spacing:.02em;line-height:1.1;margin-bottom:14px}
.verdict-ok{color:var(--green)}
.verdict-blocked{color:var(--red)}
.verdict-risk{color:var(--amber)}
.result-sub{font-size:1rem;color:var(--sub);margin-bottom:6px}
.result-explanation{font-size:.88rem;color:var(--dim);margin-top:10px;line-height:1.5}
.check-row{display:flex;justify-content:space-between;align-items:center;padding:11px 0;font-size:1rem}
.check-row-lbl{color:var(--sub)}
.chk-ok{color:var(--green);font-weight:700;font-size:1.1rem}
.chk-fail{color:var(--red);font-weight:700;font-size:1.1rem}
.check-ergebnis{font-size:.9rem;color:var(--sub);margin-top:14px;font-weight:600}
.check-grund{font-size:.78rem;color:var(--red);padding:2px 0 10px;line-height:1.4;display:none}
.result-next-step{font-size:.85rem;color:var(--red);margin-top:10px;line-height:1.5;display:none}
.lab-item{font-size:1.05rem;color:var(--sub);padding:6px 0}
.lab-item::before{content:'— ';color:var(--dim)}
.audit-row{display:flex;font-size:.85rem;padding:5px 0;gap:12px}
.audit-row-lbl{color:var(--dim);flex-shrink:0;min-width:120px}
.audit-row-val{color:var(--sub);font-family:'SF Mono','Fira Code',monospace;font-size:.78rem;word-break:break-word}
.reset-btn{background:none;border:1px solid var(--border);border-radius:8px;color:var(--sub);font-size:.9rem;font-weight:600;padding:13px 22px;cursor:pointer;transition:border-color .15s,color .15s}
.reset-btn:hover{border-color:var(--sub);color:var(--text)}
.intro-line{font-size:.83rem;color:var(--sub);line-height:1.55;margin-bottom:28px;max-width:480px}

#_legacy{display:none!important}

/* Visual clarification step */
#phase-visual{display:none;padding:40px 0;animation:fadein .2s ease-out}
.visual-placeholder-box{background:var(--surface);border:1px solid var(--border);border-radius:8px;padding:32px 24px;text-align:center;margin-bottom:12px}
.visual-tooth-icon{width:52px;height:64px;background:#eef1f5;border:1px solid var(--border);border-radius:5px 5px 9px 9px;margin:0 auto 14px;display:flex;align-items:center;justify-content:center;font-size:.7rem;color:var(--sub);letter-spacing:.06em;font-weight:700}
.visual-placeholder-lbl{font-size:1rem;font-weight:700;color:var(--sub);margin-bottom:6px}
.visual-placeholder-hint{font-size:.8rem;color:var(--dim);line-height:1.5}
.comment-row{margin-bottom:22px}
.comment-label{font-size:.7rem;font-weight:700;letter-spacing:.1em;color:var(--dim);text-transform:uppercase;display:block;margin-bottom:8px}
.comment-area{width:100%;padding:10px 13px;background:var(--surface);border:1px solid var(--border);border-radius:6px;color:var(--text);font-size:.9rem;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;resize:vertical;min-height:80px;line-height:1.5}
.comment-area:focus{outline:none;border-color:var(--sub)}
.comment-area::placeholder{color:var(--dim)}
.res-section+.res-section{border-top:1px solid var(--border)}
.visual-disclaimer{font-size:.75rem;color:var(--dim);line-height:1.5;margin-bottom:20px;padding:9px 12px;background:var(--surface);border:1px solid var(--border);border-radius:6px;border-left:3px solid var(--border)}

/* STL viewer */
.stl-viewer-wrap{background:#eceff4;border:1px solid var(--border);border-radius:8px;overflow:hidden;margin-bottom:8px;position:relative}
#stl-canvas{display:block;cursor:grab;touch-action:none}
#stl-canvas:active{cursor:grabbing}
.stl-viewer-bar{display:flex;justify-content:space-between;align-items:center;padding:7px 12px;background:var(--bg);border-top:1px solid var(--border)}
.stl-viewer-label-txt{font-size:.65rem;font-weight:700;letter-spacing:.1em;color:var(--dim);text-transform:uppercase}
.stl-viewer-hint-txt{font-size:.68rem;color:var(--dim)}
.stl-viewer-fallback{background:var(--surface);border:1px solid var(--border);border-radius:8px;padding:32px 24px;text-align:center;margin-bottom:8px;display:none}

/* Case metadata form */
.case-meta-form{background:var(--surface);border:1px solid var(--border);border-radius:8px;overflow:hidden;margin-bottom:20px}
.case-meta-form>*+*{border-top:1px solid var(--border)}
.case-meta-row{display:flex;align-items:center;gap:12px;padding:9px 14px}
.case-meta-label{font-size:.68rem;font-weight:700;letter-spacing:.08em;color:var(--dim);text-transform:uppercase;min-width:110px;flex-shrink:0}
.case-meta-input{flex:1;background:transparent;border:none;padding:2px 0;font-size:.9rem;color:var(--text);font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;outline:none}
.case-meta-input::placeholder{color:var(--dim)}
.case-meta-privacy{font-size:.7rem;color:var(--dim);padding:8px 14px;font-style:italic}
.case-meta-value{flex:1;padding:2px 0;font-size:.82rem;color:var(--sub);font-family:'SF Mono','Fira Code',monospace}
.comment-sub{font-size:.75rem;color:var(--dim);margin:-2px 0 10px;line-height:1.45}
.ereignis-tag{font-size:.68rem;font-weight:700;letter-spacing:.08em;color:var(--dim);text-transform:uppercase;margin-bottom:20px;display:inline-block;padding:4px 9px;background:var(--surface);border:1px solid var(--border);border-radius:4px}
.upload-privacy{font-size:.7rem;color:var(--dim);margin-top:10px;text-align:center;font-style:italic}
.demo-notice{font-size:.7rem;color:var(--dim);margin-top:6px;text-align:center;font-style:italic}
.stl-loaded-banner{background:#ecfdf5;border:1px solid #a7f3d0;border-radius:6px;padding:9px 14px;margin-bottom:16px;font-size:.88rem;color:var(--green);font-weight:700}
.stl-loaded-sub{font-size:.75rem;color:var(--sub);font-weight:400;font-family:'SF Mono','Fira Code',monospace;margin-top:3px}
.case-meta-label.required::after{content:' *';color:var(--amber);font-size:.75em}
.comment-label.required::after{content:' *';color:var(--amber);font-size:.75em}
.nachweis-subtitle{font-size:.78rem;color:var(--sub);line-height:1.5;margin-bottom:14px;font-style:italic}
</style>
</head>
<body>

<header>
  <div>
    <div class="brand">Post<span>CAD</span></div>
    <div class="brand-sub" id="t-brand-tagline">Klärung vor Herstellung</div>
  </div>
  <div class="lang-toggle">
    <button class="lang-btn active" id="btn-de" onclick="setLang('DE')">DE</button>
    <button class="lang-btn" id="btn-en" onclick="setLang('EN')">EN</button>
  </div>
</header>

<main>

  <p class="intro-line" id="t-intro-line">STL-Datei hochladen, Fall kurz pr&#252;fen, Hinweis dokumentieren und Entscheidung vor Herstellung festhalten.</p>

  <div id="phase-upload">
    <label class="upload-zone" id="upload-zone" for="file-input">
      <div class="upload-icon">↑</div>
      <div class="upload-title" id="t-upload-title">STL-Datei hochladen</div>
      <div class="upload-sub" id="t-upload-sub">STL-Datei aus Scanner/CAD hier ablegen oder ausw&#228;hlen</div>
    </label>
    <input type="file" id="file-input" accept=".stl,.obj" style="display:none" onchange="onFileInput(this)">
    <p class="upload-privacy" id="t-upload-privacy">Die Datei bleibt lokal im Browser und wird nicht auf dem Server gespeichert.</p>
    <div class="demo-files">
      <button class="demo-file-btn" onclick="loadDemo('Krone_Zahn_36_DE.stl')">Demo-Fall ansehen</button>
    </div>
    <p class="demo-notice" id="t-demo-notice">Nur Beispiel &#8211; f&#252;r echte F&#228;lle bitte STL-Datei hochladen.</p>
  </div>

  <div id="phase-processing">
    <div class="proc-filename" id="proc-filename"></div>
    <div class="proc-step" id="pstep-0"></div>
    <div class="proc-step" id="pstep-1"></div>
    <div class="proc-step" id="pstep-2"></div>
    <div class="proc-step" id="pstep-3"></div>
    <div style="margin-top:32px">
      <button class="reset-btn" onclick="resetDemo()"><span id="t-proc-reset">Neuen Laborfall öffnen</span></button>
    </div>
  </div>

  <div id="phase-visual">
    <div id="stl-loaded-banner" class="stl-loaded-banner" style="display:none">
      <div id="t-stl-loaded-text">STL geladen · bereit zur Kl&#228;rung</div>
      <div class="stl-loaded-sub" id="stl-loaded-details"></div>
    </div>
    <div class="gate-badge" id="t-visual-badge">VISUELLE KLÄRUNG</div>
    <div class="gate-title" id="t-visual-title">Visuelle Klärung vor Herstellung</div>
    <div class="gate-sub" id="t-visual-sub">Halten Sie fest, was der Praxis vor Herstellung verständlich gemacht werden soll.</div>
    <div class="gate-case-ctx" id="visual-case-ctx"></div>
    <div class="ereignis-tag" id="t-ereignis-lbl">Ereignis 1 · Klärung vor Herstellung</div>
    <div class="stl-viewer-wrap" id="stl-viewer-wrap">
      <canvas id="stl-canvas"></canvas>
      <div class="stl-viewer-bar">
        <span class="stl-viewer-label-txt" id="t-viewer-label">Demo-Ansicht · schematische Darstellung</span>
        <span class="stl-viewer-hint-txt" id="t-viewer-hint">Ziehen zum Drehen &middot; Scrollen zum Zoomen</span>
      </div>
    </div>
    <div class="stl-viewer-fallback" id="stl-viewer-fallback">
      <div class="visual-tooth-icon">36</div>
      <div class="visual-placeholder-lbl" id="t-visual-placeholder-lbl">Krone &middot; Zahn 36</div>
      <div class="visual-placeholder-hint" id="t-visual-placeholder-hint">3D-Ansicht nicht verf&#252;gbar.</div>
    </div>
    <div class="visual-disclaimer" id="t-visual-disclaimer">Die 3D-Ansicht dient der gemeinsamen Kl&#228;rung und Dokumentation. Keine automatische technische Pr&#252;fung.</div>
    <div class="case-meta-form" id="case-meta-form">
      <div class="case-meta-row">
        <span class="case-meta-label" id="t-meta-caseid-lbl">Fall-ID</span>
        <span class="case-meta-value" id="case-id-display">—</span>
      </div>
      <div class="case-meta-row">
        <span class="case-meta-label" id="t-meta-datei-lbl">Datei</span>
        <span class="case-meta-value" id="datei-display">—</span>
      </div>
      <div class="case-meta-row">
        <span class="case-meta-label required" id="t-meta-bezeichnung-lbl">Fallbezeichnung</span>
        <input class="case-meta-input" id="meta-bezeichnung" type="text" placeholder="z.&#x202F;B. Krone Zahn 36">
      </div>
      <div class="case-meta-row">
        <span class="case-meta-label required" id="t-meta-zahn-lbl">Zahn / Region</span>
        <input class="case-meta-input" id="meta-zahn" type="text" placeholder="z.&#x202F;B. 36">
      </div>
      <div class="case-meta-row">
        <span class="case-meta-label required" id="t-meta-material-lbl">Material</span>
        <input class="case-meta-input" id="meta-material" type="text" placeholder="z.&#x202F;B. E.max">
      </div>
      <div class="case-meta-row">
        <span class="case-meta-label required" id="t-meta-praxis-lbl">Praxis / Kunde</span>
        <input class="case-meta-input" id="meta-praxis" type="text" placeholder="z.&#x202F;B. Praxis M&#252;ller">
      </div>
      <div class="case-meta-privacy" id="t-meta-privacy">* Pflichtfeld &#183; Bitte keine Patientennamen eingeben.</div>
    </div>
    <div class="gate-error" id="meta-error" style="display:none">Bitte Falldaten vollst&#228;ndig ausf&#252;llen.</div>
    <div class="comment-row">
      <label class="comment-label required" id="t-lab-comment-label" for="lab-comment">Hinweis an die Praxis</label>
      <div class="comment-sub" id="t-lab-comment-sub">Was soll vor Herstellung verständlich gemacht oder bestätigt werden?</div>
      <textarea class="comment-area" id="lab-comment" placeholder="z.&#x202F;B. kritischer Randbereich, zu geringer Abstand&#8230;"></textarea>
      <div class="gate-error" id="comment-error" style="display:none">Bitte Hinweis an die Praxis erg&#228;nzen.</div>
    </div>
    <button class="confirm-btn" id="visual-next-btn" onclick="proceedToDecision()">Weiter zur Entscheidung</button>
    <div style="margin-top:20px;text-align:center">
      <button class="reset-btn" onclick="resetDemo()"><span id="t-visual-reset">Neuen Laborfall öffnen</span></button>
    </div>
  </div>

  <div id="phase-decision">
    <div class="gate-badge" id="t-gate-badge">ENTSCHEIDUNG VOR HERSTELLUNG</div>
    <div class="gate-title" id="t-gate-title">Welche Entscheidung wird vor Herstellung dokumentiert?</div>
    <div class="gate-sub" id="t-gate-sub">Vor Produktionsstart ist eine explizite Entscheidung erforderlich.</div>
    <div class="gate-case-ctx" id="gate-case-ctx"></div>
    <div class="decision-choices">
      <button class="choice-btn" id="choice-proceed" onclick="selectDecision('proceed')">Fortsetzung dokumentiert</button>
      <div class="choice-hint" id="hint-proceed">Die verantwortliche Person dokumentiert, dass die Ausgangslage ausreichend geklärt ist.</div>
      <button class="choice-btn" id="choice-proceed_with_risk" onclick="selectDecision('proceed_with_risk')">Fortsetzung mit Hinweis</button>
      <div class="choice-hint" id="hint-proceed_with_risk">Die verantwortliche Person dokumentiert, dass die Fertigung fortgesetzt wird, aber ein relevanter Hinweis, eine Annahme oder eine Einschränkung besteht.</div>
      <button class="choice-btn" id="choice-request_correction" onclick="selectDecision('request_correction')">Klärung erforderlich</button>
      <div class="choice-hint" id="hint-request_correction">Die verantwortliche Person dokumentiert, dass vor der Fertigung eine Rückfrage, Ergänzung oder Korrektur durch die Praxis erforderlich ist.</div>
    </div>
    <div class="reason-row" id="reason-row">
      <label class="reason-label" id="t-reason-label" for="reason-code">Grund (erforderlich)</label>
      <select id="reason-code" onchange="updateConfirmState()">
        <option value="">&#x2014; auswählen &#x2014;</option>
        <option value="incomplete_scan">Unvollständiger Scan</option>
        <option value="unclear_margin">Unklare Präp.-Grenze</option>
        <option value="prep_uncertainty">Präp.-Unsicherheit</option>
        <option value="time_pressure">Zeitdruck</option>
        <option value="other">Sonstiges</option>
      </select>
    </div>
    <button class="confirm-btn" id="confirm-btn" onclick="confirmDecision()" disabled>Entscheidung bestätigen</button>
    <div class="confirm-hint" id="confirm-hint"></div>
    <div class="gate-error" id="gate-error"></div>
    <div style="margin-top:20px;text-align:center">
      <button class="reset-btn" onclick="resetDemo()"><span id="t-gate-reset">Neuen Laborfall öffnen</span></button>
    </div>
  </div>

  <div id="phase-result">

    <div class="res-section">
      <div class="res-label" id="t-decision-label">Entscheidung</div>
      <div class="result-verdict" id="result-verdict"></div>
      <div class="result-sub" id="result-sub"></div>
      <div class="result-explanation" id="result-explanation"></div>
      <div class="result-next-step" id="result-next-step"></div>
    </div>

    <div class="res-section">
      <div class="res-label" id="t-case-label">Fall erkannt</div>
      <div class="case-proc" id="res-proc"></div>
      <div class="case-row"><span class="case-row-lbl" id="t-material-lbl">Material</span><span id="res-material"></span></div>
      <div class="case-row"><span class="case-row-lbl" id="t-land-lbl">Land</span><span id="res-land"></span></div>
      <div class="case-row"><span class="case-row-lbl" id="t-indication-lbl">Indikation</span><span id="res-indication"></span></div>
    </div>

    <div class="res-section">
      <div class="res-label" id="t-pruefung-label">Falldaten</div>
      <div class="check-row">
        <span class="check-row-lbl" id="t-chk-material">Materialangabe</span>
        <span id="chk-material"></span>
      </div>
      <div class="check-grund" id="grund-material"></div>
      <div class="check-row">
        <span class="check-row-lbl" id="t-chk-jurisdiction">Länderangabe</span>
        <span id="chk-jurisdiction"></span>
      </div>
      <div class="check-grund" id="grund-jurisdiction"></div>
      <div class="check-row">
        <span class="check-row-lbl" id="t-chk-manufacturing">Herstellungsangabe</span>
        <span id="chk-manufacturing"></span>
      </div>
      <div class="check-grund" id="grund-manufacturing"></div>
      <div class="check-ergebnis" id="check-ergebnis"></div>
    </div>

    <div class="res-section" id="labs-section" style="display:none">
      <div class="res-label" id="t-fertigung-label">Entscheidungsgrundlage</div>
      <div id="labs-list"></div>
    </div>

    <div class="res-section">
      <div class="res-label" id="t-audit-label">Entscheidungsnachweis erstellt</div>
      <div class="nachweis-subtitle" id="t-nachweis-subtitle">Dieser Nachweis dokumentiert die Kl&#228;rung vor Herstellung. Keine automatische technische Pr&#252;fung.</div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-nachweis-rcid-lbl">Fall-ID</span><span class="audit-row-val" id="nachweis-caseid"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-nachweis-datei-lbl">Datei</span><span class="audit-row-val" id="nachweis-datei"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-nachweis-ereignis-lbl">Ereignis</span><span class="audit-row-val" id="nachweis-ereignis"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-nachweis-bezeichnung-lbl">Fallbezeichnung</span><span class="audit-row-val" id="nachweis-bezeichnung"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-nachweis-zahn-lbl">Zahn / Region</span><span class="audit-row-val" id="nachweis-zahn"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-nachweis-material-lbl">Material</span><span class="audit-row-val" id="nachweis-material"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-nachweis-praxis-lbl">Praxis / Kunde</span><span class="audit-row-val" id="nachweis-praxis"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-nachweis-fall-lbl">Fall</span><span class="audit-row-val" id="nachweis-fall"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-nachweis-decision-lbl">Entscheidung</span><span class="audit-row-val" id="nachweis-decision"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-nachweis-grundlage-lbl">Entscheidungsgrundlage</span><span class="audit-row-val" id="nachweis-grundlage"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-nachweis-visual-lbl">Visuelle Klärung</span><span class="audit-row-val" id="nachweis-visual"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-nachweis-kommentar-lbl">Laborkommentar</span><span class="audit-row-val" id="nachweis-kommentar"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-nachweis-person-lbl">Verantwortliche Person</span><span class="audit-row-val" id="nachweis-person"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-audit-time-lbl">Zeitpunkt</span><span class="audit-row-val" id="audit-time"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-audit-id-lbl">Audit-ID</span><span class="audit-row-val" id="audit-id"></span></div>
      <div class="audit-row"><span class="audit-row-lbl" id="t-audit-status-lbl">Status</span><span class="audit-row-val" id="audit-status"></span></div>
      <div style="font-size:.82rem;color:var(--dim);line-height:1.55;margin-top:12px" id="t-nachweis-body">Die Entscheidung wird durch die verantwortliche Person dokumentiert. Ausgangslage und Entscheidungsgrundlage werden nachvollziehbar festgehalten.</div>
    </div>

    <div class="res-section" id="proof-section" style="display:none">
      <div class="res-label" id="t-proof-label">Technischer Nachweis / Protokollansicht</div>
      <details open>
        <summary style="font-size:.83rem;color:var(--sub);cursor:pointer;padding:4px 0">Receipt JSON</summary>
        <pre id="proof-receipt-json" style="margin-top:8px;font-size:.7rem;color:var(--sub);font-family:'SF Mono','Fira Code',monospace;white-space:pre-wrap;word-break:break-all;line-height:1.5;background:var(--bg);border:1px solid var(--border);border-radius:6px;padding:12px;max-height:260px;overflow-y:auto"></pre>
      </details>
    </div>

    <div class="res-section">
      <p class="aha-line" id="aha-line">PostCAD erkennt keine medizinischen oder technischen Fehler und gibt keine Herstellung frei. Das System strukturiert die Kommunikation zwischen Praxis und Labor und dokumentiert, welche verantwortliche Person auf welcher Grundlage entschieden hat.</p>
    </div>

    <div class="res-section" style="border-top:none;padding-top:8px;display:flex;gap:12px;flex-wrap:wrap">
      <button class="reset-btn" onclick="backToDecision()"><span id="t-back-decision">Zurück zur Entscheidung</span></button>
      <button class="reset-btn" onclick="resetDemo()"><span id="t-reset">Neuen Laborfall öffnen</span></button>
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

<script src="https://cdn.jsdelivr.net/npm/three@0.158.0/build/three.min.js"></script>
<script>
const T = {
  DE: {
    uploadTitle: 'STL-Datei hochladen',
    uploadSub: 'STL-Datei aus Scanner/CAD hier ablegen oder auswählen',
    uploadPrivacy: 'Die Datei bleibt lokal im Browser und wird nicht auf dem Server gespeichert.',
    demoNotice: 'Nur Beispiel – für echte Fälle bitte STL-Datei hochladen.',
    stlLoadedBanner: 'STL geladen · bereit zur Klärung',
    demoLoadedBanner: 'Demo-Fall geladen · bereit zur Klärung',
    procSteps: ['Datei empfangen', 'Falldaten werden gelesen', 'Angaben werden gelesen', 'Entscheidung vor Herstellung erforderlich'],
    caseLabel: 'Fall erkannt',
    materialLbl: 'Material', landLbl: 'Land', indicationLbl: 'Indikation',
    decisionLabel: 'Entscheidung',
    verdictOk: 'FORTSETZUNG DOKUMENTIERT', verdictBlock: 'KL\u00c4RUNG ERFORDERLICH',
    verdictRisk: 'FORTSETZUNG MIT HINWEIS',
    subOk: 'Fortsetzung dokumentiert', subBlock: 'Kl\u00e4rung erforderlich',
    subRisk: 'Fortsetzung mit Hinweis',
    explanationOk: 'Die verantwortliche Person hat dokumentiert, dass die Ausgangslage ausreichend gekl\u00e4rt ist.',
    explanationRisk: 'Die verantwortliche Person hat dokumentiert, dass die Fertigung fortgesetzt wird, aber ein relevanter Hinweis, eine Annahme oder eine Einschr\u00e4nkung besteht.',
    explanationBlock: 'Die verantwortliche Person hat dokumentiert, dass vor der Fertigung eine Kl\u00e4rung erforderlich ist.',
    decisionBlockedSub: 'Kl\u00e4rung erforderlich. Herstellung wird nicht gestartet.',
    decisionBlockedExplanation: 'Vor der Fertigung ist eine R\u00fcckfrage, Erg\u00e4nzung oder Korrektur durch die Praxis erforderlich.',
    pruefungLabel: 'Falldaten',
    chkMaterial: 'Materialangabe', chkJurisdiction: 'L\u00e4nderangabe', chkManufacturing: 'Herstellungsangabe',
    ergebnisOk: 'Entscheidung: Fortsetzung dokumentiert', ergebnisRisk: 'Entscheidung: Fortsetzung mit Hinweis', ergebnisBlock: 'Entscheidung: Kl\u00e4rung erforderlich',
    fertigungLabel: 'Entscheidungsgrundlage',
    auditLabel: 'Entscheidungsnachweis erstellt', nachweisSubtitle: 'Dieser Nachweis dokumentiert die Klärung vor Herstellung. Keine automatische technische Prüfung.', auditIdLbl: 'Audit-ID', auditTimeLbl: 'Zeitpunkt', auditStatusLbl: 'Status',
    nachweisFallLbl: 'Fall', nachweisDecisionLbl: 'Entscheidung', nachweisGrundlageLbl: 'Entscheidungsgrundlage',
    nachweisBody: 'Die Entscheidung wird durch die verantwortliche Person dokumentiert. Ausgangslage und Entscheidungsgrundlage werden nachvollziehbar festgehalten.',
    proofLabel: 'Technischer Nachweis / Protokollansicht',
    reset: 'Neuen Laborfall öffnen',
    gateBadge: 'ENTSCHEIDUNG VOR HERSTELLUNG',
    gateTitle: 'Welche Entscheidung wird vor Herstellung dokumentiert?',
    gateSub: 'Vor Produktionsstart ist eine explizite Entscheidung erforderlich.',
    optProceed: 'Fortsetzung dokumentiert',
    optRisk: 'Fortsetzung mit Hinweis',
    optCorrection: 'Klärung erforderlich',
    reasonLabel: 'Grund (erforderlich)',
    reasonSelect: '\u2014 ausw\u00e4hlen \u2014',
    rcIncompleteScan: 'Unvollst\u00e4ndiger Scan',
    rcUnclearMargin: 'Unklare Pr\u00e4p.-Grenze',
    rcPrepUncertainty: 'Pr\u00e4p.-Unsicherheit',
    rcTimePressure: 'Zeitdruck',
    rcOther: 'Sonstiges',
    confirmBtn: 'Entscheidung best\u00e4tigen',
    reasonHint: 'W\u00e4hlen Sie einen Grund, um fortzufahren.',
    demoOnlyError: 'Nur Demo-Dateien werden unterst\u00fctzt. Bitte Demo-Datei ausw\u00e4hlen.',
    backToDecision: 'Zur\u00fcck zur Entscheidung',
    hintProceed: 'Die verantwortliche Person dokumentiert, dass die Ausgangslage ausreichend geklärt ist.',
    hintRisk: 'Die verantwortliche Person dokumentiert, dass die Fertigung fortgesetzt wird, aber ein relevanter Hinweis, eine Annahme oder eine Einschränkung besteht.',
    hintCorrection: 'Die verantwortliche Person dokumentiert, dass vor der Fertigung eine Rückfrage, Ergänzung oder Korrektur durch die Praxis erforderlich ist.',
    ahaLine: 'PostCAD erkennt keine medizinischen oder technischen Fehler und gibt keine Herstellung frei. Das System strukturiert die Kommunikation zwischen Praxis und Labor und dokumentiert, welche verantwortliche Person auf welcher Grundlage entschieden hat.',
    grundMaterial: 'Hinweis: Material ist f\u00fcr diesen Produkttyp in den Systemdaten nicht hinterlegt.',
    grundJurisdiction: 'Hinweis: Die hinterlegten Systemdaten decken diese Jurisdiktion nicht ab.',
    grundManufacturing: 'Hinweis: Kein geeigneter Herstellungspartner in den Systemdaten hinterlegt.',
    nextStepBlock: 'N\u00e4chster Schritt: Vor der Fertigung ist eine Kl\u00e4rung erforderlich.',
    brandTagline: 'Kl\u00e4rung vor Herstellung',
    introLine: 'STL-Datei hochladen, Fall kurz pr\u00fcfen, Hinweis dokumentieren und Entscheidung vor Herstellung festhalten.',
    fertigungBody: 'Die Entscheidung wird durch die verantwortliche Person dokumentiert. Ausgangslage und Entscheidungsgrundlage werden nachvollziehbar festgehalten.',
    visualBadge: 'VISUELLE KLÄRUNG',
    visualTitle: 'Visuelle Klärung vor Herstellung',
    visualSub: 'Halten Sie fest, was der Praxis vor Herstellung verständlich gemacht werden soll.',
    visualPlaceholderLbl: 'Krone · Zahn 36',
    visualPlaceholderHint: 'Demo-Ansicht: hier kann später ein Scan-/CAD-Ausschnitt, Screenshot oder kurzer Clip dokumentiert werden.',
    visualDisclaimer: 'Die 3D-Ansicht dient der gemeinsamen Klärung und Dokumentation. Keine automatische technische Prüfung.',
    labCommentLabel: 'Hinweis an die Praxis',
    labCommentSub: 'Was soll vor Herstellung verständlich gemacht oder bestätigt werden?',
    labCommentPlaceholder: 'z. B. kritischer Randbereich, zu geringer Abstand…',
    visualNextBtn: 'Weiter zur Entscheidung',
    visualClarificationSummary: 'Klärungshinweis dokumentiert',
    nachweisVisualLbl: 'Visuelle Klärung',
    nachweisKommentarLbl: 'Laborkommentar',
    viewerLabelDemo: 'Demo-Ansicht · schematische Darstellung',
    viewerLabelLocal: 'Lokale STL-Datei · nur im Browser dargestellt',
    stlParseError: 'STL konnte lokal nicht dargestellt werden. Bitte Datei prüfen.',
    viewerHint: 'Ziehen zum Drehen · Scrollen zum Zoomen',
    viewerFallbackHint: '3D-Ansicht nicht verfügbar.',
    metaBezeichnungLbl: 'Fallbezeichnung',
    metaZahnLbl: 'Zahn / Region',
    metaMaterialLbl: 'Material',
    metaPraxisLbl: 'Praxis / Kunde',
    metaPrivacy: '* Pflichtfeld · Bitte keine Patientennamen eingeben.',
    nachweisBezeichnungLbl: 'Fallbezeichnung',
    nachweisZahnLbl: 'Zahn / Region',
    nachweisMetaMaterialLbl: 'Material',
    nachweisMetaPraxisLbl: 'Praxis / Kunde',
    metaCaseidLbl: 'Fall-ID',
    metaDateiLbl: 'Datei',
    ereignisLbl: 'Ereignis 1 · Klärung vor Herstellung',
    ereignisValue: 'Ereignis 1 · Klärung vor Herstellung',
    nachweisRcIdLbl: 'Fall-ID',
    nachweisDateiLbl: 'Datei',
    nachweisEreignisLbl: 'Ereignis',
    nachweisPersonLbl: 'Verantwortliche Person',
    nachweisPersonValue: 'Reviewer · Labor',
  },
  EN: {
    uploadTitle: 'Upload STL file',
    uploadSub: 'Drop or select STL file from scanner/CAD',
    uploadPrivacy: 'The file stays local in your browser and is not stored on the server.',
    demoNotice: 'Demo only – for real cases please upload an STL file.',
    stlLoadedBanner: 'STL loaded · ready for clarification',
    demoLoadedBanner: 'Demo case loaded · ready for clarification',
    procSteps: ['File received', 'Case data being read', 'Reading details', 'Decision before manufacturing required'],
    caseLabel: 'Case detected',
    materialLbl: 'Material', landLbl: 'Country', indicationLbl: 'Indication',
    decisionLabel: 'Decision',
    verdictOk: 'PROCEED DOCUMENTED', verdictBlock: 'CLARIFICATION REQUIRED',
    verdictRisk: 'PROCEED WITH NOTE',
    subOk: 'Proceed documented', subBlock: 'Clarification required',
    subRisk: 'Proceed with note',
    explanationOk: 'The responsible person documented that the starting situation is sufficiently clarified.',
    explanationRisk: 'The responsible person documented that manufacturing proceeds, but a relevant note, assumption, or limitation exists.',
    explanationBlock: 'The responsible person documented that clarification is required before manufacturing.',
    decisionBlockedSub: 'Clarification required. Manufacturing is not started.',
    decisionBlockedExplanation: 'A follow-up, addition or correction by the practice is required before manufacturing.',
    pruefungLabel: 'Case data',
    chkMaterial: 'Material information', chkJurisdiction: 'Country information', chkManufacturing: 'Manufacturing information',
    ergebnisOk: 'Decision: Proceed documented', ergebnisRisk: 'Decision: Proceed with note', ergebnisBlock: 'Decision: Clarification required',
    fertigungLabel: 'Decision basis',
    auditLabel: 'Decision record created', nachweisSubtitle: 'This record documents the clarification before manufacturing. No automatic technical inspection.', auditIdLbl: 'Audit ID', auditTimeLbl: 'Time', auditStatusLbl: 'Status',
    nachweisFallLbl: 'Case', nachweisDecisionLbl: 'Decision', nachweisGrundlageLbl: 'Case basis',
    nachweisBody: 'The decision is documented by the responsible person. Initial situation and decision basis are recorded traceably.',
    proofLabel: 'Technical record / Protocol view',
    reset: 'Open new lab case',
    gateBadge: 'DECISION BEFORE MANUFACTURING',
    gateTitle: 'Which decision is documented before manufacturing?',
    gateSub: 'An explicit decision is required before production can start.',
    optProceed: 'Proceed documented',
    optRisk: 'Proceed with note',
    optCorrection: 'Clarification required',
    reasonLabel: 'Reason (required)',
    reasonSelect: '\u2014 select \u2014',
    rcIncompleteScan: 'Incomplete scan',
    rcUnclearMargin: 'Unclear margin',
    rcPrepUncertainty: 'Prep uncertainty',
    rcTimePressure: 'Time pressure',
    rcOther: 'Other',
    confirmBtn: 'Confirm Decision',
    reasonHint: 'Select a reason to continue.',
    demoOnlyError: 'Only demo files are supported. Please select a demo file.',
    backToDecision: 'Back to Decision',
    hintProceed: 'The situation has been documented by the responsible person as sufficiently clarified.',
    hintRisk: 'Manufacturing proceeds, but a relevant note, assumption or constraint is documented.',
    hintCorrection: 'A follow-up, addition or correction by the practice is required before manufacturing.',
    ahaLine: 'PostCAD does not detect medical or technical errors and does not release manufacturing. The system structures communication between practice and lab, and documents which responsible person decided on which basis.',
    grundMaterial: 'Note: Material is not recorded for this procedure type in the system data.',
    grundJurisdiction: 'Note: The system data does not cover this jurisdiction.',
    grundManufacturing: 'Note: No eligible manufacturing partner recorded in the system data.',
    nextStepBlock: 'Next step: Clarification is required before manufacturing.',
    brandTagline: 'Clarification before manufacturing',
    introLine: 'Upload STL file, briefly review the case, document a note, and record the decision before manufacturing.',
    fertigungBody: 'The decision is documented by the responsible person. Initial situation and decision basis are recorded traceably.',
    visualBadge: 'VISUAL CLARIFICATION',
    visualTitle: 'Visual clarification before manufacturing',
    visualSub: 'Document what should be communicated to the practice before manufacturing.',
    visualPlaceholderLbl: 'Crown · Tooth 36',
    visualPlaceholderHint: 'Demo view: a scan/CAD excerpt, screenshot or short clip can be documented here.',
    visualDisclaimer: 'The 3D view is for joint clarification and documentation. No automatic technical inspection.',
    labCommentLabel: 'Note to practice',
    labCommentSub: 'What should be clarified or confirmed before manufacturing?',
    labCommentPlaceholder: 'e.g. critical margin area, insufficient clearance…',
    visualNextBtn: 'Proceed to decision',
    visualClarificationSummary: 'Clarification note documented',
    nachweisVisualLbl: 'Visual clarification',
    nachweisKommentarLbl: 'Lab comment',
    viewerLabelDemo: 'Demo view · schematic representation',
    viewerLabelLocal: 'Local STL file · browser only',
    stlParseError: 'STL could not be displayed locally. Please check the file.',
    viewerHint: 'Drag to rotate · Scroll to zoom',
    viewerFallbackHint: '3D view unavailable.',
    metaBezeichnungLbl: 'Case name',
    metaZahnLbl: 'Tooth / Region',
    metaMaterialLbl: 'Material',
    metaPraxisLbl: 'Practice / Client',
    metaPrivacy: '* Required field · Please do not enter patient names.',
    nachweisBezeichnungLbl: 'Case name',
    nachweisZahnLbl: 'Tooth / Region',
    nachweisMetaMaterialLbl: 'Material',
    nachweisMetaPraxisLbl: 'Practice / Client',
    metaCaseidLbl: 'Case ID',
    metaDateiLbl: 'File',
    ereignisLbl: 'Event 1 · Clarification before manufacturing',
    ereignisValue: 'Event 1 · Clarification before manufacturing',
    nachweisRcIdLbl: 'Case ID',
    nachweisDateiLbl: 'File',
    nachweisEreignisLbl: 'Event',
    nachweisPersonLbl: 'Responsible person',
    nachweisPersonValue: 'Reviewer · Lab',
  },
};

const REGISTRY = [
  {manufacturer_id:"pilot-de-001",display_name:"Alpha Dental GmbH",country:"germany",is_active:true,capabilities:["crown","bridge"],materials_supported:["zirconia","pmma"],jurisdictions_served:["germany"],attestation_statuses:["verified"],sla_days:5},
  {manufacturer_id:"pilot-de-002",display_name:"Beta Zahntechnik GmbH",country:"germany",is_active:true,capabilities:["crown","veneer"],materials_supported:["zirconia","emax"],jurisdictions_served:["germany"],attestation_statuses:["verified"],sla_days:3},
  {manufacturer_id:"pilot-de-003",display_name:"Gamma Dental GmbH",country:"germany",is_active:true,capabilities:["crown","implant"],materials_supported:["zirconia","titanium"],jurisdictions_served:["germany"],attestation_statuses:["verified"],sla_days:7},
];

const FILE_CASES_API = {
  'krone_zahn_36_de.stl': {case_id:'f3000003-0000-0000-0000-000000000003',jurisdiction:'DE',routing_policy:'allow_domestic_and_cross_border',patient_country:'germany',manufacturer_country:'germany',material:'emax',procedure:'crown',file_type:'stl'},
  'bruecke_usa.stl':  {case_id:'f4000004-0000-0000-0000-000000000004',jurisdiction:'US',routing_policy:'allow_domestic_and_cross_border',patient_country:'united_states',manufacturer_country:'germany',material:'zirconia',procedure:'bridge',file_type:'stl'},
};

const FILE_CASES = {
  'krone_zahn_36_de.stl': {
    proc: 'Krone \u00b7 Zahn 36',
    material: 'E.max', land: 'Deutschland', indication: 'Standardversorgung',
    ok: true,
    checks: {material: true, jurisdiction: true, manufacturing: true},
    labs: [],
  },
  'bruecke_usa.stl': {
    proc: 'Br\u00fccke',
    material: 'Zirkon', land: 'USA', indication: 'Standardversorgung',
    ok: false,
    checks: {material: true, jurisdiction: false, manufacturing: true},
    labs: [],
  },
};

const DEMO_META = {
  'krone_zahn_36_de.stl': {bezeichnung:'Krone Zahn 36',zahnRegion:'36',material:'E.max',praxis:'Demo-Praxis'},
};

const LOCAL_STL_DEFAULTS = {bezeichnung:'Krone Zahn 36',zahnRegion:'36',material:'E.max',praxis:'Demo-Praxis'};

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
let labComment = '';
let caseMetadata = {bezeichnung:'',zahnRegion:'',material:'',praxis:''};
let _pendingBuffer = null;
let _isDemoMesh = true;
let caseId = null;
let displayFilename = null;
let localStlActive = false;
const _threeVars = {scene:null,camera:null,renderer:null,animId:null,mesh:null};
const _orbit = {rotX:0.3,rotY:0.4,zoom:16,dragging:false,lastX:0,lastY:0};

function setLang(l) {
  lang = l;
  document.getElementById('btn-de').classList.toggle('active', l === 'DE');
  document.getElementById('btn-en').classList.toggle('active', l === 'EN');
  const t = T[l];
  document.getElementById('t-brand-tagline').textContent = t.brandTagline;
  document.getElementById('t-intro-line').textContent = t.introLine;
  document.getElementById('t-upload-title').textContent = t.uploadTitle;
  document.getElementById('t-upload-sub').textContent = t.uploadSub;
  document.getElementById('t-upload-privacy').textContent = t.uploadPrivacy;
  document.getElementById('t-demo-notice').textContent = t.demoNotice;
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
  if (lastResultOk !== null) {
    document.getElementById('labs-list').innerHTML =
      '<div style="font-size:.88rem;color:var(--sub);line-height:1.55">' + t.fertigungBody + '</div>';
  }
  document.getElementById('t-audit-label').textContent = t.auditLabel;
  document.getElementById('t-nachweis-subtitle').textContent = t.nachweisSubtitle;
  if (document.getElementById('stl-loaded-banner').style.display !== 'none') {
    document.getElementById('t-stl-loaded-text').textContent = localStlActive ? t.stlLoadedBanner : t.demoLoadedBanner;
  }
  document.getElementById('t-audit-id-lbl').textContent = t.auditIdLbl;
  document.getElementById('t-audit-time-lbl').textContent = t.auditTimeLbl;
  document.getElementById('t-audit-status-lbl').textContent = t.auditStatusLbl;
  document.getElementById('t-nachweis-fall-lbl').textContent = t.nachweisFallLbl;
  document.getElementById('t-nachweis-decision-lbl').textContent = t.nachweisDecisionLbl;
  document.getElementById('t-nachweis-grundlage-lbl').textContent = t.nachweisGrundlageLbl;
  document.getElementById('t-nachweis-body').textContent = t.nachweisBody;
  document.getElementById('t-proof-label').textContent = t.proofLabel;
  document.getElementById('t-reset').textContent = t.reset;
  document.getElementById('t-proc-reset').textContent = t.reset;
  document.getElementById('t-gate-reset').textContent = t.reset;
  document.getElementById('t-back-decision').textContent = t.backToDecision;
  document.getElementById('t-gate-badge').textContent = t.gateBadge;
  document.getElementById('t-gate-title').textContent = t.gateTitle;
  document.getElementById('t-gate-sub').textContent = t.gateSub;
  document.getElementById('choice-proceed').textContent = t.optProceed;
  document.getElementById('choice-proceed_with_risk').textContent = t.optRisk;
  document.getElementById('choice-request_correction').textContent = t.optCorrection;
  document.getElementById('t-visual-badge').textContent = t.visualBadge;
  document.getElementById('t-visual-title').textContent = t.visualTitle;
  document.getElementById('t-visual-sub').textContent = t.visualSub;
  document.getElementById('t-visual-placeholder-lbl').textContent = t.visualPlaceholderLbl;
  document.getElementById('t-visual-placeholder-hint').textContent = t.viewerFallbackHint;
  document.getElementById('t-visual-disclaimer').textContent = t.visualDisclaimer;
  document.getElementById('t-viewer-label').textContent = _isDemoMesh ? t.viewerLabelDemo : t.viewerLabelLocal;
  document.getElementById('t-viewer-hint').textContent = t.viewerHint;
  document.getElementById('t-meta-bezeichnung-lbl').textContent = t.metaBezeichnungLbl;
  document.getElementById('t-meta-zahn-lbl').textContent = t.metaZahnLbl;
  document.getElementById('t-meta-material-lbl').textContent = t.metaMaterialLbl;
  document.getElementById('t-meta-praxis-lbl').textContent = t.metaPraxisLbl;
  document.getElementById('t-meta-privacy').textContent = t.metaPrivacy;
  document.getElementById('t-meta-caseid-lbl').textContent = t.metaCaseidLbl;
  document.getElementById('t-meta-datei-lbl').textContent = t.metaDateiLbl;
  document.getElementById('t-lab-comment-label').textContent = t.labCommentLabel;
  document.getElementById('t-lab-comment-sub').textContent = t.labCommentSub;
  document.getElementById('t-ereignis-lbl').textContent = t.ereignisLbl;
  document.getElementById('visual-next-btn').textContent = t.visualNextBtn;
  document.getElementById('t-visual-reset').textContent = t.reset;
  document.getElementById('t-nachweis-rcid-lbl').textContent = t.nachweisRcIdLbl;
  document.getElementById('t-nachweis-datei-lbl').textContent = t.nachweisDateiLbl;
  document.getElementById('t-nachweis-ereignis-lbl').textContent = t.nachweisEreignisLbl;
  document.getElementById('t-nachweis-person-lbl').textContent = t.nachweisPersonLbl;
  document.getElementById('t-nachweis-visual-lbl').textContent = t.nachweisVisualLbl;
  document.getElementById('t-nachweis-kommentar-lbl').textContent = t.nachweisKommentarLbl;
  document.getElementById('t-nachweis-bezeichnung-lbl').textContent = t.nachweisBezeichnungLbl;
  document.getElementById('t-nachweis-zahn-lbl').textContent = t.nachweisZahnLbl;
  document.getElementById('t-nachweis-material-lbl').textContent = t.nachweisMetaMaterialLbl;
  document.getElementById('t-nachweis-praxis-lbl').textContent = t.nachweisMetaPraxisLbl;
  document.getElementById('t-reason-label').textContent = t.reasonLabel;
  document.getElementById('confirm-btn').textContent = t.confirmBtn;
  document.getElementById('hint-proceed').textContent = t.hintProceed;
  document.getElementById('hint-proceed_with_risk').textContent = t.hintRisk;
  document.getElementById('hint-request_correction').textContent = t.hintCorrection;
  document.getElementById('aha-line').textContent = t.ahaLine;
  const sel = document.getElementById('reason-code');
  sel.options[0].text = t.reasonSelect;
  sel.options[1].text = t.rcIncompleteScan;
  sel.options[2].text = t.rcUnclearMargin;
  sel.options[3].text = t.rcPrepUncertainty;
  sel.options[4].text = t.rcTimePressure;
  sel.options[5].text = t.rcOther;
  if (lastResultOk !== null) {
    const expl = (lastResultOk && selectedDecision === 'proceed_with_risk') ? t.explanationRisk : lastResultOk ? t.explanationOk : t.explanationBlock;
    document.getElementById('result-explanation').textContent = expl;
    const grundKeys = {material: 'grundMaterial', jurisdiction: 'grundJurisdiction', manufacturing: 'grundManufacturing'};
    ['material', 'jurisdiction', 'manufacturing'].forEach(key => {
      const el = document.getElementById('grund-' + key);
      if (el && el.style.display !== 'none') el.textContent = t[grundKeys[key]];
    });
    const nsEl = document.getElementById('result-next-step');
    if (nsEl && nsEl.style.display !== 'none') nsEl.textContent = t.nextStepBlock;
    const auditVerdict2 = (lastResultOk && selectedDecision === 'proceed_with_risk') ? t.verdictRisk : lastResultOk ? t.verdictOk : t.verdictBlock;
    document.getElementById('nachweis-decision').textContent = auditVerdict2;
    const grundlage2 = selectedDecision === 'request_correction' ? t.decisionBlockedExplanation : (lastResultOk && selectedDecision === 'proceed_with_risk') ? t.explanationRisk : lastResultOk ? t.explanationOk : t.explanationBlock;
    document.getElementById('nachweis-grundlage').textContent = grundlage2;
    document.getElementById('nachweis-ereignis').textContent = t.ereignisValue;
    document.getElementById('nachweis-person').textContent = t.nachweisPersonValue;
  }
}

function onFileInput(input) {
  if (!input.files[0]) return;
  const file = input.files[0];
  const name = file.name;
  input.value = '';
  const reader = new FileReader();
  reader.onload = function(e) { _pendingBuffer = e.target.result; startProcessing(name); };
  reader.onerror = function() { _pendingBuffer = null; startProcessing(name); };
  reader.readAsArrayBuffer(file);
}

function loadDemo(filename) {
  _pendingBuffer = null;
  localStlActive = false;
  startProcessing(filename);
}

function generateCaseId() {
  return 'RC-' + new Date().getFullYear() + '-' + String(Math.floor(Math.random() * 99999)).padStart(5, '0');
}

async function startProcessing(filename) {
  caseId = generateCaseId();
  const _lc = filename.toLowerCase();
  const _cd = getCaseData(filename);
  displayFilename = DEMO_META[_lc] ? ('Demo-Fall · ' + _cd.proc) : ('Lokale Datei · ' + filename);
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
    await delay(i === 0 ? 300 : 500);
    document.getElementById('pstep-' + i).classList.add('visible');
  }
  await delay(600);
  showVisualStep(filename);
}

function showVisualStep(filename) {
  currentFilename = filename;
  const c = getCaseData(filename);
  document.getElementById('visual-case-ctx').textContent = c.proc + ' · ' + c.material + ' · ' + c.land;
  document.getElementById('lab-comment').value = '';
  const dm = DEMO_META[filename.toLowerCase()];
  const meta = dm || (_pendingBuffer !== null ? LOCAL_STL_DEFAULTS : {});
  document.getElementById('meta-bezeichnung').value = meta.bezeichnung || '';
  document.getElementById('meta-zahn').value = meta.zahnRegion || '';
  document.getElementById('meta-material').value = meta.material || '';
  document.getElementById('meta-praxis').value = meta.praxis || '';
  document.getElementById('comment-error').style.display = 'none';
  document.getElementById('meta-error').style.display = 'none';
  const _isLocalUpload = _pendingBuffer !== null;
  document.getElementById('t-stl-loaded-text').textContent = _isLocalUpload ? T[lang].stlLoadedBanner : T[lang].demoLoadedBanner;
  document.getElementById('stl-loaded-details').textContent = (displayFilename || filename) + ' · ' + (caseId || '');
  document.getElementById('stl-loaded-banner').style.display = 'block';
  document.getElementById('case-id-display').textContent = caseId || '—';
  document.getElementById('datei-display').textContent = displayFilename || '—';
  document.getElementById('phase-processing').style.display = 'none';
  document.getElementById('phase-visual').style.display = 'block';
  requestAnimationFrame(function() {
    requestAnimationFrame(function() { initViewer(_pendingBuffer, filename); });
  });
}

function proceedToDecision() {
  const comment     = document.getElementById('lab-comment').value.trim();
  const bezeichnung = document.getElementById('meta-bezeichnung').value.trim();
  const zahnRegion  = document.getElementById('meta-zahn').value.trim();
  const material    = document.getElementById('meta-material').value.trim();
  const praxis      = document.getElementById('meta-praxis').value.trim();

  const commentErrEl = document.getElementById('comment-error');
  const metaErrEl    = document.getElementById('meta-error');
  let hasError = false;
  if (!comment) { commentErrEl.style.display = 'block'; hasError = true; }
  else           { commentErrEl.style.display = 'none'; }
  if (!bezeichnung || !zahnRegion || !material || !praxis) { metaErrEl.style.display = 'block'; hasError = true; }
  else                                                     { metaErrEl.style.display = 'none'; }
  if (hasError) return;

  labComment = comment;
  caseMetadata = { bezeichnung, zahnRegion, material, praxis };
  disposeViewer();
  document.getElementById('phase-visual').style.display = 'none';
  showDecisionGate(currentFilename);
}

function showDecisionGate(filename) {
  currentFilename = filename;
  selectedDecision = null;
  setLang(lang);
  const c = getCaseData(filename);

  document.getElementById('gate-case-ctx').textContent = c.proc + ' \u00b7 ' + c.material + ' \u00b7 ' + c.land;
  ['proceed', 'proceed_with_risk', 'request_correction'].forEach(d => {
    document.getElementById('choice-' + d).className = 'choice-btn';
  });
  document.getElementById('reason-row').style.display = 'none';
  document.getElementById('reason-code').value = '';
  document.getElementById('confirm-btn').disabled = true;
  document.getElementById('confirm-hint').textContent = '';
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
  const disabled = needsReason && !hasReason;
  document.getElementById('confirm-btn').disabled = disabled;
  document.getElementById('confirm-hint').textContent = disabled ? T[lang].reasonHint : '';
}

async function confirmDecision() {
  if (!selectedDecision) return;
  // Local STL uploaded and rendered — generate receipt entirely client-side.
  // This check must come before the demo-file lookup so the flag, not the filename, governs.
  if (localStlActive) { confirmLocalDecision(); return; }
  // Demo path — validate the filename against known demo cases.
  const caseObj = currentFilename ? FILE_CASES_API[currentFilename.toLowerCase()] : undefined;
  if (!caseObj) { showGateError(T[lang].demoOnlyError); return; }

  const t = T[lang];
  const btn = document.getElementById('confirm-btn');
  btn.disabled = true;
  btn.textContent = '\u2026';
  document.getElementById('gate-error').style.display = 'none';

  try {
    const casesRes = await fetch('/cases', {method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(caseObj)});
    // 201 Created, 200 Identical, and 409 Conflict (case stored with different content) all mean
    // the case_id exists in the store — safe to proceed to /decisions.
    if (!casesRes.ok && casesRes.status !== 409) {
      const errData = await casesRes.json().catch(() => ({}));
      const msg = errData.error && errData.error.message ? errData.error.message : 'Case intake failed (' + casesRes.status + ')';
      showGateError(msg);
      btn.disabled = false;
      btn.textContent = t.confirmBtn;
      return;
    }

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
      showGateError(err.error && err.error.message ? err.error.message : 'Decision failed (' + decRes.status + ')');
      btn.disabled = false;
      btn.textContent = t.confirmBtn;
      return;
    }

    document.getElementById('phase-decision').style.display = 'none';
    await delay(500);

    if (selectedDecision === 'request_correction') {
      showResultBlocked(currentFilename);
    } else {
      showResult(currentFilename);
      fetchAndRenderProof(currentFilename);
    }
  } catch(e) {
    console.error('[reviewer] confirmDecision:', e);
    showGateError('Network error: ' + (e && e.message ? e.message : String(e)));
    btn.disabled = false;
    btn.textContent = t.confirmBtn;
  }
}

function confirmLocalDecision() {
  const t = T[lang];
  document.getElementById('confirm-btn').disabled = true;
  document.getElementById('gate-error').style.display = 'none';
  document.getElementById('phase-decision').style.display = 'none';

  const isBlock = selectedDecision === 'request_correction';
  const isRisk  = selectedDecision === 'proceed_with_risk';
  lastResultOk  = !isBlock;

  const auditVerdict = isBlock ? t.verdictBlock : isRisk ? t.verdictRisk : t.verdictOk;
  const vClass       = isBlock ? 'verdict-blocked' : isRisk ? 'verdict-risk' : 'verdict-ok';
  const explanation  = isBlock ? t.decisionBlockedExplanation : isRisk ? t.explanationRisk : t.explanationOk;
  const ergebnis     = isBlock ? t.ergebnisBlock : isRisk ? t.ergebnisRisk : t.ergebnisOk;
  const grundlage    = isBlock ? t.decisionBlockedExplanation : isRisk ? t.explanationRisk : t.explanationOk;
  const proc         = caseMetadata.bezeichnung || currentFilename;

  const vEl = document.getElementById('result-verdict');
  vEl.className = 'result-verdict ' + vClass;
  vEl.textContent = auditVerdict;
  document.getElementById('result-sub').textContent = isBlock ? t.decisionBlockedSub : isRisk ? t.subRisk : t.subOk;
  document.getElementById('result-explanation').textContent = explanation;

  document.getElementById('res-proc').textContent = proc;
  document.getElementById('res-material').textContent = caseMetadata.material || '—';
  document.getElementById('res-land').textContent = '—';
  document.getElementById('res-indication').textContent = '—';

  setCheck('chk-material',     !isBlock);
  setCheck('chk-jurisdiction', !isBlock);
  setCheck('chk-manufacturing',!isBlock);
  document.getElementById('check-ergebnis').textContent = ergebnis;
  const nsEl = document.getElementById('result-next-step');
  if (isBlock) { nsEl.textContent = t.nextStepBlock; nsEl.style.display = 'block'; }
  else { nsEl.style.display = 'none'; }

  document.getElementById('labs-section').style.display = 'block';
  document.getElementById('labs-list').innerHTML =
    '<div style="font-size:.88rem;color:var(--sub);line-height:1.55">' + t.fertigungBody + '</div>';

  document.getElementById('audit-id').textContent = 'PC-' + new Date().getFullYear() + '-' + String(Math.floor(Math.random() * 99999)).padStart(5, '0');
  document.getElementById('audit-time').textContent = new Date().toLocaleTimeString('de-DE', {hour:'2-digit', minute:'2-digit', second:'2-digit'});
  document.getElementById('audit-status').textContent = auditVerdict;
  document.getElementById('nachweis-caseid').textContent = na(caseId);
  document.getElementById('nachweis-datei').textContent = na(displayFilename);
  document.getElementById('nachweis-ereignis').textContent = t.ereignisValue;
  document.getElementById('nachweis-bezeichnung').textContent = na(caseMetadata.bezeichnung);
  document.getElementById('nachweis-zahn').textContent = na(caseMetadata.zahnRegion);
  document.getElementById('nachweis-material').textContent = na(caseMetadata.material);
  document.getElementById('nachweis-praxis').textContent = na(caseMetadata.praxis);
  document.getElementById('nachweis-fall').textContent = proc;
  document.getElementById('nachweis-decision').textContent = auditVerdict;
  document.getElementById('nachweis-grundlage').textContent = grundlage;
  document.getElementById('nachweis-visual').textContent = t.viewerLabelLocal + ' · ' + na(caseMetadata.bezeichnung || currentFilename);
  document.getElementById('nachweis-kommentar').textContent = na(labComment);
  document.getElementById('nachweis-person').textContent = t.nachweisPersonValue;

  const reasonVal = document.getElementById('reason-code').value;
  const localReceipt = {
    case_id: caseId,
    datei: displayFilename,
    ereignis: t.ereignisValue,
    fallbezeichnung: caseMetadata.bezeichnung || null,
    zahn_region: caseMetadata.zahnRegion || null,
    material: caseMetadata.material || null,
    praxis: caseMetadata.praxis || null,
    decision: selectedDecision,
    reason_code: reasonVal || null,
    lab_comment: labComment || null,
    timestamp: new Date().toISOString(),
    actor_role: 'reviewer',
    storage: 'Lokale Datei · nicht auf Server gespeichert',
  };
  document.getElementById('proof-receipt-json').textContent = JSON.stringify(localReceipt, null, 2);
  document.getElementById('proof-section').style.display = '';

  document.getElementById('phase-result').style.display = 'block';
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
  const labsSecBlocked = document.getElementById('labs-section');
  labsSecBlocked.style.display = 'block';
  document.getElementById('labs-list').innerHTML =
    '<div style="font-size:.88rem;color:var(--sub);line-height:1.55">' + t.fertigungBody + '</div>';
  const nsElBlocked = document.getElementById('result-next-step');
  nsElBlocked.textContent = t.nextStepBlock;
  nsElBlocked.style.display = 'block';

  document.getElementById('audit-id').textContent = 'PC-2026-' + String(Math.floor(Math.random() * 99999)).padStart(5, '0');
  document.getElementById('audit-time').textContent = new Date().toLocaleTimeString('de-DE', {hour:'2-digit', minute:'2-digit', second:'2-digit'});
  document.getElementById('audit-status').textContent = t.verdictBlock;

  document.getElementById('nachweis-caseid').textContent = na(caseId);
  document.getElementById('nachweis-datei').textContent = na(displayFilename);
  document.getElementById('nachweis-ereignis').textContent = t.ereignisValue;
  document.getElementById('nachweis-bezeichnung').textContent = na(caseMetadata.bezeichnung);
  document.getElementById('nachweis-zahn').textContent = na(caseMetadata.zahnRegion);
  document.getElementById('nachweis-material').textContent = na(caseMetadata.material);
  document.getElementById('nachweis-praxis').textContent = na(caseMetadata.praxis);
  document.getElementById('nachweis-fall').textContent = c.proc;
  document.getElementById('nachweis-decision').textContent = t.verdictBlock;
  document.getElementById('nachweis-grundlage').textContent = t.decisionBlockedExplanation;
  document.getElementById('nachweis-visual').textContent = t.visualClarificationSummary + ' · ' + c.proc;
  document.getElementById('nachweis-kommentar').textContent = na(labComment);
  document.getElementById('nachweis-person').textContent = t.nachweisPersonValue;

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
  if (c.ok && selectedDecision === 'proceed_with_risk') {
    vEl.className = 'result-verdict verdict-risk';
    vEl.textContent = t.verdictRisk;
    document.getElementById('result-sub').textContent = t.subRisk;
    document.getElementById('result-explanation').textContent = t.explanationRisk;
  } else if (c.ok) {
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

  setCheck('chk-material', c.checks.material, t.grundMaterial);
  setCheck('chk-jurisdiction', c.checks.jurisdiction, t.grundJurisdiction);
  setCheck('chk-manufacturing', c.checks.manufacturing, t.grundManufacturing);
  document.getElementById('check-ergebnis').textContent = (c.ok && selectedDecision === 'proceed_with_risk') ? t.ergebnisRisk : c.ok ? t.ergebnisOk : t.ergebnisBlock;
  const nsEl = document.getElementById('result-next-step');
  if (!c.ok) { nsEl.textContent = t.nextStepBlock; nsEl.style.display = 'block'; }
  else { nsEl.style.display = 'none'; }

  document.getElementById('labs-section').style.display = 'block';
  document.getElementById('labs-list').innerHTML =
    '<div style="font-size:.88rem;color:var(--sub);line-height:1.55">' + t.fertigungBody + '</div>';

  document.getElementById('audit-id').textContent = 'PC-2026-' + String(Math.floor(Math.random() * 99999)).padStart(5, '0');
  document.getElementById('audit-time').textContent = new Date().toLocaleTimeString('de-DE', {hour:'2-digit', minute:'2-digit', second:'2-digit'});
  const auditVerdict = !c.ok ? t.verdictBlock : selectedDecision === 'proceed_with_risk' ? t.verdictRisk : t.verdictOk;
  document.getElementById('audit-status').textContent = auditVerdict;

  document.getElementById('nachweis-caseid').textContent = na(caseId);
  document.getElementById('nachweis-datei').textContent = na(displayFilename);
  document.getElementById('nachweis-ereignis').textContent = t.ereignisValue;
  document.getElementById('nachweis-bezeichnung').textContent = na(caseMetadata.bezeichnung);
  document.getElementById('nachweis-zahn').textContent = na(caseMetadata.zahnRegion);
  document.getElementById('nachweis-material').textContent = na(caseMetadata.material);
  document.getElementById('nachweis-praxis').textContent = na(caseMetadata.praxis);
  document.getElementById('nachweis-fall').textContent = c.proc;
  document.getElementById('nachweis-decision').textContent = auditVerdict;
  const nachweisGrundlage = (c.ok && selectedDecision === 'proceed_with_risk') ? t.explanationRisk : c.ok ? t.explanationOk : t.explanationBlock;
  document.getElementById('nachweis-grundlage').textContent = nachweisGrundlage;
  document.getElementById('nachweis-visual').textContent = t.visualClarificationSummary + ' · ' + c.proc;
  document.getElementById('nachweis-kommentar').textContent = na(labComment);
  document.getElementById('nachweis-person').textContent = t.nachweisPersonValue;

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

function setCheck(id, pass, grundText) {
  const el = document.getElementById(id);
  el.textContent = pass ? '\u2713' : '\u2715';
  el.className = pass ? 'chk-ok' : 'chk-fail';
  const grundEl = document.getElementById('grund-' + id.replace('chk-', ''));
  if (grundEl) {
    if (!pass && grundText) {
      grundEl.textContent = grundText;
      grundEl.style.display = 'block';
    } else {
      grundEl.style.display = 'none';
    }
  }
}

function backToDecision() {
  document.getElementById('phase-result').style.display = 'none';
  showDecisionGate(currentFilename);
}

function resetDemo() {
  lastResultOk = null;
  currentFilename = null;
  selectedDecision = null;
  labComment = '';
  caseMetadata = {bezeichnung:'',zahnRegion:'',material:'',praxis:''};
  _pendingBuffer = null;
  caseId = null;
  displayFilename = null;
  localStlActive = false;
  disposeViewer();
  document.getElementById('phase-result').style.display = 'none';
  document.getElementById('phase-processing').style.display = 'none';
  document.getElementById('phase-visual').style.display = 'none';
  document.getElementById('phase-decision').style.display = 'none';
  document.getElementById('phase-upload').style.display = 'block';
  document.getElementById('lab-comment').value = '';
  document.getElementById('meta-bezeichnung').value = '';
  document.getElementById('meta-zahn').value = '';
  document.getElementById('meta-material').value = '';
  document.getElementById('meta-praxis').value = '';
  document.getElementById('proof-section').style.display = 'none';
  document.getElementById('proof-receipt-json').textContent = '';
  document.getElementById('comment-error').style.display = 'none';
  document.getElementById('meta-error').style.display = 'none';
  document.getElementById('stl-loaded-banner').style.display = 'none';
}

function delay(ms) { return new Promise(r => setTimeout(r, ms)); }
function na(v) { return v || 'Nicht angegeben'; }

function initViewer(buffer, filename) {
  disposeViewer();
  const wrap = document.getElementById('stl-viewer-wrap');
  const canvas = document.getElementById('stl-canvas');
  const isUserFile = !!buffer;
  if (!window.THREE) { showViewerFallback(true); return; }
  try {
    const w = wrap.clientWidth > 0 ? wrap.clientWidth : 540;
    const h = 280;
    const renderer = new THREE.WebGLRenderer({canvas: canvas, antialias: true});
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(w, h);
    renderer.setClearColor(0xeceff4, 1);
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(40, w / h, 0.1, 1000);
    scene.add(new THREE.AmbientLight(0xffffff, 0.7));
    const dir = new THREE.DirectionalLight(0xffffff, 0.8);
    dir.position.set(1, 2, 2);
    scene.add(dir);
    _threeVars.scene = scene;
    _threeVars.camera = camera;
    _threeVars.renderer = renderer;
    _threeVars.mesh = null;
    let geometry = null;
    if (buffer) { geometry = loadFileGeometry(buffer); }
    if (geometry) {
      _isDemoMesh = false;
      localStlActive = true;
      document.getElementById('t-viewer-label').textContent = T[lang].viewerLabelLocal;
    } else if (!isUserFile) {
      geometry = loadDemoMesh();
      _isDemoMesh = true;
      localStlActive = false;
      document.getElementById('t-viewer-label').textContent = T[lang].viewerLabelDemo;
    } else {
      _isDemoMesh = false;
      localStlActive = false;
      document.getElementById('t-visual-placeholder-hint').textContent = T[lang].stlParseError;
      showViewerFallback(true);
      return;
    }
    document.getElementById('t-viewer-hint').textContent = T[lang].viewerHint;
    if (geometry) {
      geometry.computeBoundingBox();
      const box = geometry.boundingBox;
      const center = new THREE.Vector3();
      box.getCenter(center);
      geometry.translate(-center.x, -center.y, -center.z);
      const size = new THREE.Vector3();
      box.getSize(size);
      _orbit.zoom = Math.max(size.x, size.y, size.z) * 2.5;
      const mat = new THREE.MeshPhongMaterial({color:0xd4c5b0, specular:0x888888, shininess:40});
      const mesh = new THREE.Mesh(geometry, mat);
      scene.add(mesh);
      _threeVars.mesh = mesh;
    }
    setupViewerControls(canvas);
    startRenderLoop();
    showViewerFallback(false);
  } catch(e) {
    console.warn('[viewer] init failed:', e);
    if (isUserFile) { document.getElementById('t-visual-placeholder-hint').textContent = T[lang].stlParseError; }
    showViewerFallback(true);
  }
}

function loadDemoMesh() {
  if (!window.THREE) return null;
  return new THREE.CylinderGeometry(4.5, 5.5, 9, 32);
}

function loadFileGeometry(buffer) {
  try {
    const data = parseSTL(buffer);
    if (!data) return null;
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(data.positions), 3));
    if (data.normals.length === data.positions.length) {
      geo.setAttribute('normal', new THREE.BufferAttribute(new Float32Array(data.normals), 3));
    } else {
      geo.computeVertexNormals();
    }
    return geo;
  } catch(e) { return null; }
}

function parseSTLBinary(buffer) {
  const view = new DataView(buffer);
  if (buffer.byteLength < 84) return null;
  const triCount = view.getUint32(80, true);
  if (84 + triCount * 50 > buffer.byteLength) return null;
  const positions = [], normals = [];
  let off = 84;
  for (let i = 0; i < triCount; i++) {
    const nx = view.getFloat32(off, true), ny = view.getFloat32(off+4, true), nz = view.getFloat32(off+8, true);
    off += 12;
    for (let v = 0; v < 3; v++) {
      positions.push(view.getFloat32(off, true), view.getFloat32(off+4, true), view.getFloat32(off+8, true));
      normals.push(nx, ny, nz);
      off += 12;
    }
    off += 2;
  }
  return {positions, normals};
}

function parseSTL(buffer) {
  const sample = new TextDecoder('ascii', {fatal: false}).decode(new Uint8Array(buffer, 0, Math.min(1024, buffer.byteLength)));
  if (sample.includes('facet normal')) {
    const result = parseSTLAscii(buffer);
    if (result && result.positions.length > 0) return result;
  }
  return parseSTLBinary(buffer);
}

function parseSTLAscii(buffer) {
  const text = new TextDecoder().decode(buffer);
  const positions = [], normals = [];
  const normRe = /facet\s+normal\s+([\d.eE+\-]+)\s+([\d.eE+\-]+)\s+([\d.eE+\-]+)/g;
  const vertRe = /vertex\s+([\d.eE+\-]+)\s+([\d.eE+\-]+)\s+([\d.eE+\-]+)/g;
  let nMatch;
  while ((nMatch = normRe.exec(text)) !== null) {
    const nx = parseFloat(nMatch[1]), ny = parseFloat(nMatch[2]), nz = parseFloat(nMatch[3]);
    for (let v = 0; v < 3; v++) {
      const vMatch = vertRe.exec(text);
      if (!vMatch) return null;
      positions.push(parseFloat(vMatch[1]), parseFloat(vMatch[2]), parseFloat(vMatch[3]));
      normals.push(nx, ny, nz);
    }
  }
  if (positions.length === 0) return null;
  return {positions, normals};
}

function setupViewerControls(canvas) {
  canvas.addEventListener('mousedown', e => { _orbit.dragging = true; _orbit.lastX = e.clientX; _orbit.lastY = e.clientY; });
  window.addEventListener('mouseup', () => { _orbit.dragging = false; });
  window.addEventListener('mousemove', e => {
    if (!_orbit.dragging) return;
    _orbit.rotY += (e.clientX - _orbit.lastX) * 0.01;
    _orbit.rotX += (e.clientY - _orbit.lastY) * 0.01;
    _orbit.rotX = Math.max(-Math.PI/2, Math.min(Math.PI/2, _orbit.rotX));
    _orbit.lastX = e.clientX; _orbit.lastY = e.clientY;
  });
  canvas.addEventListener('wheel', e => {
    e.preventDefault();
    _orbit.zoom *= 1 + e.deltaY * 0.001;
    _orbit.zoom = Math.max(1, Math.min(500, _orbit.zoom));
  }, {passive:false});
  let lastPinch = 0;
  canvas.addEventListener('touchstart', e => {
    if (e.touches.length === 1) { _orbit.dragging = true; _orbit.lastX = e.touches[0].clientX; _orbit.lastY = e.touches[0].clientY; }
    if (e.touches.length === 2) lastPinch = Math.hypot(e.touches[0].clientX - e.touches[1].clientX, e.touches[0].clientY - e.touches[1].clientY);
  }, {passive:true});
  canvas.addEventListener('touchend', () => { _orbit.dragging = false; }, {passive:true});
  canvas.addEventListener('touchmove', e => {
    if (e.touches.length === 1 && _orbit.dragging) {
      _orbit.rotY += (e.touches[0].clientX - _orbit.lastX) * 0.01;
      _orbit.rotX += (e.touches[0].clientY - _orbit.lastY) * 0.01;
      _orbit.rotX = Math.max(-Math.PI/2, Math.min(Math.PI/2, _orbit.rotX));
      _orbit.lastX = e.touches[0].clientX; _orbit.lastY = e.touches[0].clientY;
    }
    if (e.touches.length === 2) {
      const p = Math.hypot(e.touches[0].clientX - e.touches[1].clientX, e.touches[0].clientY - e.touches[1].clientY);
      _orbit.zoom *= lastPinch / p; _orbit.zoom = Math.max(1, Math.min(500, _orbit.zoom)); lastPinch = p;
    }
  }, {passive:true});
}

function startRenderLoop() {
  function render() {
    _threeVars.animId = requestAnimationFrame(render);
    if (!_threeVars.renderer || !_threeVars.scene || !_threeVars.camera) return;
    const r = _orbit.zoom;
    _threeVars.camera.position.set(
      r * Math.sin(_orbit.rotY) * Math.cos(_orbit.rotX),
      r * Math.sin(_orbit.rotX),
      r * Math.cos(_orbit.rotY) * Math.cos(_orbit.rotX)
    );
    _threeVars.camera.lookAt(0, 0, 0);
    _threeVars.renderer.render(_threeVars.scene, _threeVars.camera);
  }
  render();
}

function disposeViewer() {
  if (_threeVars.animId !== null) { cancelAnimationFrame(_threeVars.animId); _threeVars.animId = null; }
  if (_threeVars.renderer) { _threeVars.renderer.dispose(); _threeVars.renderer = null; }
  if (_threeVars.mesh && _threeVars.mesh.geometry) _threeVars.mesh.geometry.dispose();
  _threeVars.scene = null; _threeVars.camera = null; _threeVars.mesh = null;
  _orbit.dragging = false;
}

function showViewerFallback(show) {
  document.getElementById('stl-viewer-wrap').style.display = show ? 'none' : '';
  document.getElementById('stl-viewer-fallback').style.display = show ? 'block' : 'none';
}

(function() {
  const zone = document.getElementById('upload-zone');
  zone.addEventListener('dragover', e => { e.preventDefault(); zone.classList.add('drag-over'); });
  zone.addEventListener('dragleave', e => { if (!zone.contains(e.relatedTarget)) zone.classList.remove('drag-over'); });
  zone.addEventListener('drop', e => {
    e.preventDefault(); zone.classList.remove('drag-over');
    const file = e.dataTransfer.files[0];
    if (!file) return;
    const name = file.name;
    const reader = new FileReader();
    reader.onload = function(ev) { _pendingBuffer = ev.target.result; startProcessing(name); };
    reader.onerror = function() { _pendingBuffer = null; startProcessing(name); };
    reader.readAsArrayBuffer(file);
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
