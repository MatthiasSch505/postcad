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
<title>PostCAD — From CAD design to manufacturing dispatch</title>
<style>
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
html{scroll-behavior:smooth}

:root{
  --bg:#0e1117;
  --surface:#151c28;
  --surface2:#1a2235;
  --surface3:#1f2840;
  --border:rgba(255,255,255,0.07);
  --border-md:rgba(255,255,255,0.13);
  --border-strong:rgba(255,255,255,0.2);
  --text-1:#eaf0f9;
  --text-2:#7a8fa8;
  --text-3:#3a4d63;
  --green:#2fcf7a;
  --green-bg:rgba(47,207,122,0.08);
  --green-border:rgba(47,207,122,0.22);
  --green-dim:rgba(47,207,122,0.5);
  --amber:#e8a020;
  --blue:#5b8cfc;
  --blue-bg:rgba(91,140,252,0.08);
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
  padding:.85rem 2.5rem;
  display:flex;align-items:center;
  position:sticky;top:0;z-index:10;
  background:rgba(14,17,23,0.94);
  backdrop-filter:blur(14px);
}
.logo{font-size:.95rem;font-weight:700;color:var(--text-1);letter-spacing:-.015em;text-decoration:none}
nav{margin-left:auto;display:flex;align-items:center;gap:1.75rem}
.nav-link{
  font-size:.82rem;color:var(--text-2);text-decoration:none;
  transition:color .12s;
}
.nav-link:hover{color:var(--text-1)}
.btn-nav{
  font-family:inherit;font-size:.84rem;font-weight:700;
  background:var(--green);color:#05140d;
  border:none;border-radius:6px;padding:.5rem 1.2rem;
  cursor:pointer;text-decoration:none;
  display:inline-flex;align-items:center;gap:.35rem;
  transition:opacity .12s;
}
.btn-nav:hover{opacity:.88}

/* ── page ── */
.page{max-width:1040px;margin:0 auto;padding:0 2.5rem}

/* ── hero ── */
.hero{padding:8rem 0 6rem}
.hero-kicker{
  display:inline-flex;align-items:center;gap:.5rem;
  font-size:.68rem;font-weight:700;color:var(--text-3);
  text-transform:uppercase;letter-spacing:.12em;
  margin-bottom:1.75rem;
}
.hero-kicker-dot{
  width:5px;height:5px;border-radius:50%;
  background:var(--green);flex-shrink:0;
  box-shadow:0 0 6px var(--green-dim);
}
.hero-h1{
  font-size:4rem;font-weight:700;
  letter-spacing:-.045em;line-height:1.05;
  color:var(--text-1);margin-bottom:1.6rem;
  max-width:720px;
}
.hero-sub{
  font-size:1.1rem;color:var(--text-2);
  line-height:1.7;max-width:540px;margin-bottom:3rem;
}
.cta-row{display:flex;flex-wrap:wrap;gap:.9rem;align-items:center;margin-bottom:2rem}
.btn-cta-primary{
  font-family:inherit;font-size:.95rem;font-weight:700;
  background:var(--green);color:#05140d;
  border:none;border-radius:8px;padding:.85rem 2rem;
  cursor:pointer;text-decoration:none;
  display:inline-flex;align-items:center;gap:.45rem;
  transition:opacity .12s;
  box-shadow:0 0 24px rgba(47,207,122,0.18);
}
.btn-cta-primary:hover{opacity:.88}
.btn-cta-secondary{
  font-family:inherit;font-size:.9rem;font-weight:600;
  background:transparent;color:var(--text-2);
  border:1px solid var(--border-md);border-radius:8px;
  padding:.82rem 1.65rem;cursor:pointer;text-decoration:none;
  display:inline-flex;align-items:center;gap:.35rem;
  transition:border-color .12s,color .12s;
}
.btn-cta-secondary:hover{border-color:var(--border-strong);color:var(--text-1)}
.live-tag{
  display:inline-flex;align-items:center;gap:.3rem;
  font-size:.65rem;font-weight:700;font-family:var(--mono);
  color:var(--green);
}
.live-tag::before{
  content:'';width:5px;height:5px;border-radius:50%;
  background:var(--green);
  box-shadow:0 0 5px var(--green-dim);
}
.hero-note{
  font-size:.78rem;color:var(--text-3);line-height:1.65;
  border-left:2px solid var(--border-md);
  padding-left:.9rem;max-width:420px;
}

/* ── example run block ── */
.example-run{
  margin-top:3.5rem;
  background:var(--surface);
  border:1px solid var(--border-md);
  border-radius:12px;
  overflow:hidden;
}
.example-run-header{
  padding:.6rem 1.25rem;
  border-bottom:1px solid var(--border);
  display:flex;align-items:center;gap:.65rem;
  background:var(--surface2);
}
.example-run-label{
  font-size:.65rem;font-weight:700;color:var(--text-3);
  text-transform:uppercase;letter-spacing:.09em;font-family:var(--mono);
}
.example-run-tag{
  font-size:.6rem;font-weight:700;font-family:var(--mono);
  color:var(--green);background:var(--green-bg);
  border:1px solid var(--green-border);border-radius:2px;
  padding:.08rem .35rem;text-transform:uppercase;letter-spacing:.05em;
}
.example-run-body{
  display:grid;grid-template-columns:repeat(4,1fr);
}
.er-step{
  padding:1.25rem 1.4rem;
  border-right:1px solid var(--border);
}
.er-step:last-child{border-right:none}
.er-step-name{
  font-size:.6rem;font-weight:700;color:var(--text-3);
  text-transform:uppercase;letter-spacing:.09em;
  margin-bottom:.65rem;font-family:var(--mono);
}
.er-step-value{
  font-size:.8rem;font-weight:600;color:var(--text-1);
  line-height:1.4;margin-bottom:.25rem;
}
.er-step-detail{
  font-size:.7rem;color:var(--text-2);font-family:var(--mono);
  word-break:break-all;line-height:1.4;
}
.er-ok{color:var(--green)}
.er-hash{
  font-size:.65rem;color:var(--text-3);font-family:var(--mono);
  margin-top:.25rem;
}

/* ── section divider ── */
.section-divider{border:none;border-top:1px solid var(--border);margin:0}

/* ── how it works section ── */
.how-section{padding:6rem 0}
.how-header{
  display:grid;grid-template-columns:1fr 1fr;gap:4rem;
  align-items:end;margin-bottom:4rem;
}
.how-title{
  font-size:2rem;font-weight:700;letter-spacing:-.03em;
  color:var(--text-1);line-height:1.15;
}
.how-sub{
  font-size:.88rem;color:var(--text-2);line-height:1.7;
  align-self:end;
}
.how-steps{display:grid;grid-template-columns:repeat(4,1fr);gap:0;position:relative}
.how-connector{
  position:absolute;top:14px;
  left:calc(12.5% + 10px);right:calc(12.5% + 10px);
  height:1px;background:linear-gradient(90deg,var(--border) 0%,var(--green-border) 40%,var(--green-border) 100%);
}
.how-step{padding:0 1rem;position:relative}
.how-step:first-child{padding-left:0}
.how-step:last-child{padding-right:0}
.how-dot{
  width:28px;height:28px;border-radius:50%;
  border:1px solid var(--border-md);background:var(--surface);
  display:inline-flex;align-items:center;justify-content:center;
  font-size:.58rem;font-weight:700;color:var(--text-3);
  font-family:var(--mono);margin-bottom:1.1rem;
  position:relative;z-index:1;
}
.how-step.how-postcad .how-dot{
  background:var(--green-bg);border-color:var(--green-border);color:var(--green);
}
.how-tag{
  font-size:.58rem;font-weight:700;font-family:var(--mono);
  color:var(--green);background:var(--green-bg);
  border:1px solid var(--green-border);border-radius:2px;
  padding:.06rem .28rem;text-transform:uppercase;letter-spacing:.05em;
  margin-bottom:.4rem;display:inline-block;
}
.how-step-label{
  font-size:.88rem;font-weight:600;color:var(--text-1);
  margin-bottom:.3rem;letter-spacing:-.01em;
}
.how-step-desc{font-size:.75rem;color:var(--text-2);line-height:1.55}

/* ── principles ── */
.principles-section{
  padding:6rem 0;border-top:1px solid var(--border);
}
.principles-title{
  font-size:2rem;font-weight:700;letter-spacing:-.03em;
  color:var(--text-1);margin-bottom:3rem;
}
.principles-grid{
  display:grid;grid-template-columns:repeat(3,1fr);gap:1.25rem;
}
.principle-block{
  padding:1.75rem 2rem;
  background:var(--surface);
  border:1px solid var(--border);
  border-radius:10px;
}
.principle-num{
  font-size:.6rem;font-weight:700;color:var(--text-3);
  font-family:var(--mono);text-transform:uppercase;
  letter-spacing:.08em;margin-bottom:.9rem;
}
.principle-title{
  font-size:.98rem;font-weight:700;color:var(--text-1);
  letter-spacing:-.012em;margin-bottom:.45rem;
}
.principle-body{font-size:.8rem;color:var(--text-2);line-height:1.65}

/* ── callout ── */
.callout-section{padding:5rem 0;border-top:1px solid var(--border)}
.callout-inner{
  background:var(--surface);
  border:1px solid var(--border-md);
  border-radius:14px;
  padding:3.25rem 3.75rem;
  display:flex;align-items:center;
  justify-content:space-between;gap:2.5rem;
  flex-wrap:wrap;
}
.callout-left{}
.callout-live{
  font-size:.65rem;font-weight:700;font-family:var(--mono);
  color:var(--green);text-transform:uppercase;letter-spacing:.08em;
  display:flex;align-items:center;gap:.4rem;margin-bottom:.7rem;
}
.callout-live::before{
  content:'';width:5px;height:5px;border-radius:50%;
  background:var(--green);
  box-shadow:0 0 6px var(--green-dim);
}
.callout-title{
  font-size:1.55rem;font-weight:700;letter-spacing:-.025em;
  margin-bottom:.5rem;color:var(--text-1);
}
.callout-sub{
  font-size:.88rem;color:var(--text-2);
  max-width:440px;line-height:1.65;
}

/* ── footer ── */
footer{
  border-top:1px solid var(--border);
  padding:1.75rem 2.5rem;
  display:flex;align-items:center;justify-content:space-between;
  flex-wrap:wrap;gap:.75rem;
}
.footer-brand{font-size:.82rem;font-weight:700;color:var(--text-3);text-decoration:none}
.footer-links{display:flex;gap:1.5rem}
.footer-link{
  font-size:.78rem;color:var(--text-3);text-decoration:none;
  transition:color .12s;
}
.footer-link:hover{color:var(--text-2)}
.footer-note{font-size:.72rem;color:var(--text-3)}

/* ── responsive ── */
@media(max-width:860px){
  .hero{padding:5rem 0 4rem}
  .hero-h1{font-size:2.8rem}
  .example-run-body{grid-template-columns:1fr 1fr}
  .er-step{border-bottom:1px solid var(--border)}
  .er-step:nth-child(2n){border-right:none}
  .er-step:nth-last-child(-n+2){border-bottom:none}
  .how-header{grid-template-columns:1fr}
  .how-steps{grid-template-columns:1fr 1fr;gap:2rem}
  .how-connector{display:none}
  .how-step,.how-step:first-child,.how-step:last-child{padding:0}
  .principles-grid{grid-template-columns:1fr}
  .callout-inner{padding:2.25rem 2rem}
  .page{padding:0 1.5rem}
  header{padding:.75rem 1.5rem}
  nav .nav-link{display:none}
  footer{padding:1.5rem 1.5rem}
}
@media(max-width:540px){
  .hero-h1{font-size:2.2rem}
  .example-run-body{grid-template-columns:1fr}
  .er-step{border-right:none}
  .er-step:last-child{border-bottom:none}
  .how-steps{grid-template-columns:1fr}
  .cta-row{flex-direction:column;align-items:flex-start}
}
</style>
</head>
<body>

<!-- ── header ── -->
<header>
  <a class="logo" href="/">PostCAD</a>
  <nav>
    <a class="nav-link" href="#how-it-works">How it works</a>
    <a class="nav-link" href="#principles">Principles</a>
    <a class="btn-nav" href="/reviewer">Open demo →</a>
  </nav>
</header>

<div class="page">

<!-- ── hero ── -->
<section class="hero">
  <div class="hero-kicker">
    <span class="hero-kicker-dot"></span>
    Post-CAD manufacturing layer
  </div>
  <h1 class="hero-h1">From CAD design<br>to manufacturing<br>dispatch.</h1>
  <p class="hero-sub">
    PostCAD sits between your dental CAD output and production.
    It verifies manufacturer certifications, applies jurisdiction rules,
    routes to the right lab, and produces an immutable audit trail —
    without making clinical decisions.
  </p>
  <div class="cta-row">
    <a class="btn-cta-primary" href="/reviewer">Open live demo →</a>
    <a class="btn-cta-secondary" href="mailto:pilot@routecore.ai">Request pilot access</a>
  </div>
  <p class="hero-note">
    No AI decision-making. No clinical liability.<br>
    Every output carries a reason code and a verifiable receipt hash.
  </p>

  <!-- Real example run -->
  <div class="example-run">
    <div class="example-run-header">
      <span class="example-run-label">Example run</span>
      <span class="example-run-tag">real engine output</span>
    </div>
    <div class="example-run-body">
      <div class="er-step">
        <div class="er-step-name">Input</div>
        <div class="er-step-value">Crown · Zirconia</div>
        <div class="er-step-detail">Germany · case f1000001</div>
      </div>
      <div class="er-step">
        <div class="er-step-name">Routing</div>
        <div class="er-step-value er-ok">3 candidates → 1 selected</div>
        <div class="er-step-detail">pilot-de-001<br>Alpha Dental GmbH</div>
      </div>
      <div class="er-step">
        <div class="er-step-name">Receipt</div>
        <div class="er-step-value er-ok">Issued · verified</div>
        <div class="er-hash">0db54077cff0fbc4…</div>
      </div>
      <div class="er-step">
        <div class="er-step-name">Dispatch</div>
        <div class="er-step-value er-ok">Ready for handoff</div>
        <div class="er-step-detail">Audit record attached</div>
      </div>
    </div>
  </div>
</section>

<hr class="section-divider">

<!-- ── how it works ── -->
<section class="how-section" id="how-it-works">
  <div class="how-header">
    <h2 class="how-title">One deterministic<br>path from design<br>to production</h2>
    <p class="how-sub">
      The same inputs always produce the same manufacturer selection and receipt hash.
      PostCAD controls steps 2–4. Step 1 is your existing CAD tool.
      Step 5 is your manufacturer.
    </p>
  </div>

  <div class="how-steps">
    <div class="how-connector" aria-hidden="true"></div>

    <div class="how-step">
      <div class="how-dot">01</div>
      <div class="how-step-label">CAD Design</div>
      <div class="how-step-desc">
        A structured case exits your design software.
        Material, procedure, and jurisdiction defined.
      </div>
    </div>

    <div class="how-step how-postcad">
      <div class="how-dot">02</div>
      <div class="how-tag">PostCAD</div>
      <div class="how-step-label">Routing Engine</div>
      <div class="how-step-desc">
        Registry checked against certifications,
        jurisdiction rules, and capability constraints.
      </div>
    </div>

    <div class="how-step how-postcad">
      <div class="how-dot">03</div>
      <div class="how-tag">PostCAD</div>
      <div class="how-step-label">Verification</div>
      <div class="how-step-desc">
        Receipt replayed from original inputs.
        Routing decision confirmed reproducible.
      </div>
    </div>

    <div class="how-step how-postcad">
      <div class="how-dot">04</div>
      <div class="how-tag">PostCAD</div>
      <div class="how-step-label">Dispatch</div>
      <div class="how-step-desc">
        Traceable handoff prepared.
        Full audit chain attached before manufacturing.
      </div>
    </div>
  </div>
</section>

<!-- ── principles ── -->
<section class="principles-section" id="principles">
  <h2 class="principles-title">Built for regulated environments</h2>
  <div class="principles-grid">
    <div class="principle-block">
      <div class="principle-num">01 · Determinism</div>
      <div class="principle-title">Same input, same output — always</div>
      <div class="principle-body">
        Identical case data and registry state produce the same manufacturer selection
        and receipt hash on every run. No variance, no approximation.
      </div>
    </div>
    <div class="principle-block">
      <div class="principle-num">02 · Verifiability</div>
      <div class="principle-title">Every decision is replayable</div>
      <div class="principle-body">
        Routing receipts can be independently verified by replaying the kernel
        against original inputs. No black-box outputs, no manual sign-off required.
      </div>
    </div>
    <div class="principle-block">
      <div class="principle-num">03 · Immutable audit trail</div>
      <div class="principle-title">Records that can't be altered</div>
      <div class="principle-body">
        Every step produces a hash-chained entry. The chain cannot be modified
        without detection. Designed to satisfy regulatory inspection requirements.
      </div>
    </div>
  </div>
</section>

<!-- ── callout ── -->
<section class="callout-section">
  <div class="callout-inner">
    <div class="callout-left">
      <div class="callout-live">Engine-connected · real outputs</div>
      <div class="callout-title">See all three scenarios live</div>
      <p class="callout-sub">
        The operator demo runs against the real PostCAD engine.
        Choose from an eligible routing, a jurisdiction refusal, or a capability mismatch —
        and see exactly how the engine responds.
      </p>
    </div>
    <a class="btn-cta-primary" href="/reviewer">Open operator demo →</a>
  </div>
</section>

</div><!-- /page -->

<!-- ── footer ── -->
<footer>
  <a class="footer-brand" href="/">PostCAD</a>
  <div class="footer-links">
    <a class="footer-link" href="/reviewer">Operator Demo</a>
    <a class="footer-link" href="mailto:pilot@routecore.ai">Contact</a>
  </div>
  <span class="footer-note">Post-CAD manufacturing routing layer · routecore.ai</span>
</footer>

</body>
</html>"##;
