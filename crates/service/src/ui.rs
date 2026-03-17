//! Public landing page for PostCAD.
//!
//! Served at `GET /`. Static marketing surface — no backend calls, no state.
//! Links to `GET /reviewer` for the live operator demo.

/// Full single-page landing page, embedded at compile time.
pub const OPERATOR_UI_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PostCAD — Deterministic manufacturing routing</title>
<style>
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
html{scroll-behavior:smooth}

:root{
  --bg:#0f1118;
  --surface:#161b26;
  --surface2:#1a2030;
  --border:rgba(255,255,255,0.07);
  --border-md:rgba(255,255,255,0.12);
  --border-strong:rgba(255,255,255,0.18);
  --text-1:#e8edf5;
  --text-2:#7a8fa8;
  --text-3:#3d4d62;
  --green:#2fcf7a;
  --green-bg:rgba(47,207,122,0.08);
  --green-border:rgba(47,207,122,0.2);
  --blue:#5b8cfc;
  --mono:'ui-monospace','Cascadia Code','Menlo',monospace;
}

body{
  font-family:-apple-system,BlinkMacSystemFont,'Inter','Segoe UI',sans-serif;
  background:var(--bg);color:var(--text-1);min-height:100vh;
  font-size:15px;line-height:1.6;-webkit-font-smoothing:antialiased;
}

/* ── header ── */
header{
  border-bottom:1px solid var(--border);
  padding:.8rem 2.5rem;
  display:flex;align-items:center;
  position:sticky;top:0;z-index:10;
  background:rgba(15,17,24,0.92);
  backdrop-filter:blur(12px);
}
.logo{font-size:.95rem;font-weight:700;color:var(--text-1);letter-spacing:-.015em}
nav{margin-left:auto;display:flex;align-items:center;gap:1.5rem}
.nav-link{
  font-size:.82rem;color:var(--text-2);text-decoration:none;
  transition:color .12s;
}
.nav-link:hover{color:var(--text-1)}
.btn-nav{
  font-family:inherit;font-size:.82rem;font-weight:700;
  background:var(--text-1);color:#0d1018;
  border:none;border-radius:6px;padding:.45rem 1.1rem;
  cursor:pointer;text-decoration:none;
  display:inline-flex;align-items:center;gap:.3rem;
  transition:opacity .12s;
}
.btn-nav:hover{opacity:.88}

/* ── page container ── */
.page{max-width:1060px;margin:0 auto;padding:0 2.5rem}

/* ── hero ── */
.hero{padding:7.5rem 0 5.5rem;max-width:720px}
.hero-eyebrow{
  font-size:.68rem;font-weight:700;color:var(--text-3);
  text-transform:uppercase;letter-spacing:.12em;
  margin-bottom:1.4rem;
  display:flex;align-items:center;gap:.55rem;
}
.hero-eyebrow::before{
  content:'';width:20px;height:1px;
  background:var(--text-3);display:inline-block;
}
.hero-h1{
  font-size:3.4rem;font-weight:700;
  letter-spacing:-.04em;line-height:1.08;
  color:var(--text-1);margin-bottom:1.5rem;
}
.hero-h1 em{font-style:normal;color:var(--text-2)}
.hero-sub{
  font-size:1.08rem;color:var(--text-2);
  line-height:1.7;max-width:560px;margin-bottom:2.5rem;
}
.cta-row{display:flex;flex-wrap:wrap;gap:.75rem;align-items:center}
.btn-cta-primary{
  font-family:inherit;font-size:.9rem;font-weight:700;
  background:var(--text-1);color:#0d1018;
  border:none;border-radius:7px;padding:.78rem 1.85rem;
  cursor:pointer;text-decoration:none;
  display:inline-flex;align-items:center;gap:.4rem;
  transition:opacity .12s;
}
.btn-cta-primary:hover{opacity:.88}
.btn-cta-secondary{
  font-family:inherit;font-size:.88rem;font-weight:600;
  background:transparent;color:var(--text-2);
  border:1px solid var(--border-md);border-radius:7px;
  padding:.74rem 1.55rem;cursor:pointer;text-decoration:none;
  display:inline-flex;align-items:center;gap:.35rem;
  transition:border-color .12s,color .12s;
}
.btn-cta-secondary:hover{border-color:var(--border-strong);color:var(--text-1)}
.hero-note{font-size:.76rem;color:var(--text-3);margin-top:1.6rem;line-height:1.55}

/* ── section divider ── */
.section-divider{border:none;border-top:1px solid var(--border)}

/* ── shared section typography ── */
.section-eyebrow{
  font-size:.63rem;font-weight:700;color:var(--text-3);
  text-transform:uppercase;letter-spacing:.1em;margin-bottom:.9rem;
}
.section-title{
  font-size:1.7rem;font-weight:700;letter-spacing:-.025em;
  color:var(--text-1);margin-bottom:.65rem;
}
.section-sub{
  font-size:.92rem;color:var(--text-2);
  max-width:480px;line-height:1.65;
}

/* ── process section ── */
.process-section{padding:5.5rem 0}
.process-header{margin-bottom:3.5rem}
.process-flow{
  display:grid;grid-template-columns:repeat(4,1fr);
  gap:0;position:relative;
}
.process-connector{
  position:absolute;top:15px;
  left:calc(12.5% + 12px);right:calc(12.5% + 12px);
  height:1px;background:var(--border-md);
}
.process-step{
  padding:0 1.1rem;text-align:center;
  position:relative;
}
.process-step:first-child{padding-left:0;text-align:left}
.process-step:last-child{padding-right:0;text-align:right}
.process-dot{
  width:30px;height:30px;border-radius:50%;
  border:1px solid var(--border-md);background:var(--surface2);
  display:inline-flex;align-items:center;justify-content:center;
  font-size:.6rem;font-weight:700;color:var(--text-3);
  font-family:var(--mono);margin-bottom:1.1rem;
  position:relative;z-index:1;
}
.process-step.process-postcad .process-dot{
  background:var(--green-bg);
  border-color:var(--green-border);
  color:var(--green);
}
.process-step-label{
  font-size:.85rem;font-weight:600;color:var(--text-1);
  margin-bottom:.3rem;letter-spacing:-.01em;
}
.process-step-desc{font-size:.76rem;color:var(--text-2);line-height:1.5}
.process-step-tag{
  display:inline-block;font-size:.58rem;font-weight:700;
  font-family:var(--mono);color:var(--green);
  background:var(--green-bg);border:1px solid var(--green-border);
  border-radius:2px;padding:.08rem .3rem;
  text-transform:uppercase;letter-spacing:.05em;margin-bottom:.4rem;
}

/* ── principles section ── */
.principles-section{
  padding:5.5rem 0;
  border-top:1px solid var(--border);
}
.principles-header{margin-bottom:3rem}
.principles-grid{
  display:grid;grid-template-columns:repeat(3,1fr);gap:1.5rem;
}
.principle-block{
  padding:1.6rem 1.75rem;
  background:var(--surface);
  border:1px solid var(--border);
  border-radius:10px;
}
.principle-index{
  font-size:.6rem;font-weight:700;color:var(--text-3);
  font-family:var(--mono);text-transform:uppercase;
  letter-spacing:.08em;margin-bottom:.85rem;
}
.principle-title{
  font-size:.95rem;font-weight:700;color:var(--text-1);
  letter-spacing:-.01em;margin-bottom:.45rem;
}
.principle-body{font-size:.8rem;color:var(--text-2);line-height:1.65}

/* ── callout / CTA strip ── */
.callout-section{
  padding:5rem 0;
  border-top:1px solid var(--border);
}
.callout-inner{
  background:var(--surface);border:1px solid var(--border-md);
  border-radius:12px;padding:3rem 3.5rem;
  display:flex;align-items:center;justify-content:space-between;
  gap:2rem;flex-wrap:wrap;
}
.callout-tag{
  display:inline-block;font-size:.62rem;font-weight:700;
  font-family:var(--mono);color:var(--green);
  background:var(--green-bg);border:1px solid var(--green-border);
  border-radius:3px;padding:.15rem .45rem;margin-bottom:.7rem;
  text-transform:uppercase;letter-spacing:.06em;
}
.callout-title{
  font-size:1.4rem;font-weight:700;letter-spacing:-.02em;
  margin-bottom:.45rem;
}
.callout-sub{
  font-size:.88rem;color:var(--text-2);
  max-width:440px;line-height:1.65;
}

/* ── footer ── */
footer{
  border-top:1px solid var(--border);
  padding:2rem 2.5rem;
  display:flex;align-items:center;justify-content:space-between;
  flex-wrap:wrap;gap:1rem;
}
.footer-brand{font-size:.82rem;font-weight:700;color:var(--text-3)}
.footer-links{display:flex;gap:1.5rem}
.footer-link{
  font-size:.78rem;color:var(--text-3);text-decoration:none;
  transition:color .12s;
}
.footer-link:hover{color:var(--text-2)}
.footer-note{font-size:.72rem;color:var(--text-3)}

/* ── responsive ── */
@media(max-width:760px){
  .hero{padding:4.5rem 0 3.5rem}
  .hero-h1{font-size:2.2rem}
  .process-flow{grid-template-columns:1fr 1fr;gap:1.75rem}
  .process-connector{display:none}
  .process-step,.process-step:first-child,.process-step:last-child{
    text-align:left;padding:0;
  }
  .principles-grid{grid-template-columns:1fr}
  .callout-inner{padding:2rem 1.75rem}
  header{padding:.75rem 1.25rem}
  nav .nav-link{display:none}
  .page{padding:0 1.25rem}
  footer{padding:1.75rem 1.25rem}
}
</style>
</head>
<body>

<!-- ── header ── -->
<header>
  <span class="logo">PostCAD</span>
  <nav>
    <a class="nav-link" href="#process">How it works</a>
    <a class="nav-link" href="#principles">Principles</a>
    <a class="btn-nav" href="/reviewer">Open Demo ↗</a>
  </nav>
</header>

<div class="page">

<!-- ── hero ── -->
<section class="hero">
  <div class="hero-eyebrow">Post-CAD manufacturing layer</div>
  <h1 class="hero-h1">Deterministic routing<br>from CAD to <em>production</em></h1>
  <p class="hero-sub">
    PostCAD sits between dental CAD design and manufacturing.
    It verifies certifications, applies jurisdiction rules, selects an eligible manufacturer,
    and records an immutable audit trail — without making clinical decisions.
  </p>
  <div class="cta-row">
    <a class="btn-cta-primary" href="/reviewer">Open live demo →</a>
    <a class="btn-cta-secondary" href="mailto:pilot@routecore.ai">Request pilot</a>
  </div>
  <p class="hero-note">
    No AI decision-making. No clinical liability.<br>
    Every routing output carries a reason code and a verifiable receipt hash.
  </p>
</section>

<hr class="section-divider">

<!-- ── process ── -->
<section class="process-section" id="process">
  <div class="process-header">
    <div class="section-eyebrow">Workflow</div>
    <h2 class="section-title">One deterministic path</h2>
    <p class="section-sub">
      The same inputs always produce the same manufacturer selection and receipt hash.
      No variance between runs, no manual reinterpretation.
    </p>
  </div>

  <div class="process-flow">
    <div class="process-connector" aria-hidden="true"></div>

    <div class="process-step">
      <div class="process-dot">01</div>
      <div class="process-step-label">CAD Design</div>
      <div class="process-step-desc">
        A structured case file exits your design software.
        Material, procedure, and jurisdiction are defined.
      </div>
    </div>

    <div class="process-step process-postcad">
      <div class="process-dot">02</div>
      <div class="process-step-tag">PostCAD</div>
      <div class="process-step-label">Routing Engine</div>
      <div class="process-step-desc">
        Manufacturer registry is checked against certifications and country rules.
        A deterministic selection is made.
      </div>
    </div>

    <div class="process-step process-postcad">
      <div class="process-dot">03</div>
      <div class="process-step-tag">PostCAD</div>
      <div class="process-step-label">Verification</div>
      <div class="process-step-desc">
        The receipt is replayed from original inputs to confirm
        the routing decision is reproducible and unmodified.
      </div>
    </div>

    <div class="process-step process-postcad">
      <div class="process-dot">04</div>
      <div class="process-step-tag">PostCAD</div>
      <div class="process-step-label">Dispatch</div>
      <div class="process-step-desc">
        A traceable handoff is prepared. Full audit record attached
        before the case reaches the manufacturer.
      </div>
    </div>
  </div>
</section>

<!-- ── principles ── -->
<section class="principles-section" id="principles">
  <div class="principles-header">
    <div class="section-eyebrow">Principles</div>
    <h2 class="section-title">Built for regulated environments</h2>
  </div>

  <div class="principles-grid">
    <div class="principle-block">
      <div class="principle-index">01 · Determinism</div>
      <div class="principle-title">Same input, same output</div>
      <div class="principle-body">
        Identical case data and registry state produce the same manufacturer selection
        and receipt hash every time. There is no variance between runs.
      </div>
    </div>
    <div class="principle-block">
      <div class="principle-index">02 · Verifiability</div>
      <div class="principle-title">Every decision is replayable</div>
      <div class="principle-body">
        Routing receipts can be independently verified by replaying the kernel
        from the original inputs. No black-box outputs. No manual sign-off required.
      </div>
    </div>
    <div class="principle-block">
      <div class="principle-index">03 · Audit trail</div>
      <div class="principle-title">Immutable record by design</div>
      <div class="principle-body">
        Every step produces a hash-chained audit entry. The record cannot be altered
        without detection. Designed to withstand regulatory inspection.
      </div>
    </div>
  </div>
</section>

<!-- ── callout ── -->
<section class="callout-section">
  <div class="callout-inner">
    <div>
      <div class="callout-tag">Live · engine-connected</div>
      <div class="callout-title">See the routing kernel run</div>
      <p class="callout-sub">
        The operator demo connects to the real PostCAD engine.
        Load a pilot case, run routing, verify the receipt, and create a dispatch record —
        all against live backend endpoints.
      </p>
    </div>
    <a class="btn-cta-primary" href="/reviewer">Open operator demo →</a>
  </div>
</section>

</div><!-- /page -->

<!-- ── footer ── -->
<footer>
  <span class="footer-brand">PostCAD</span>
  <div class="footer-links">
    <a class="footer-link" href="/reviewer">Operator Demo</a>
    <a class="footer-link" href="mailto:pilot@routecore.ai">Contact</a>
  </div>
  <span class="footer-note">Post-CAD manufacturing routing layer</span>
</footer>

</body>
</html>"##;
