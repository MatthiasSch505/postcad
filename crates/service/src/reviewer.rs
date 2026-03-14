//! Reviewer shell — single page, three actions, real kernel output.
//!
//! Served at `GET /reviewer`. Auto-loads pilot fixtures from `examples/pilot/`
//! via `GET /pilot-fixtures`. Calls real endpoints only:
//!   POST /route  — routing kernel execution
//!   POST /verify — deterministic receipt verification
//!
//! No mock data. No fake outputs. No mocked decisions.

pub const REVIEWER_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>PostCAD — Reviewer Shell</title>
<style>
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
body{font-family:ui-monospace,"Cascadia Mono","Menlo",monospace;
     background:#0f1117;color:#c9d1d9;min-height:100vh;font-size:13px}

/* ── header ── */
header{background:#161b22;border-bottom:1px solid #30363d;
       padding:.55rem 1.2rem;display:flex;align-items:center;gap:.75rem}
#status-dot{width:7px;height:7px;border-radius:50%;background:#d29922;flex-shrink:0}
#status-dot.ok{background:#3fb950}
#status-dot.err{background:#f85149}
#hdr-title{font-size:.92rem;font-weight:700;color:#f0f6fc}
#hdr-tag{font-size:.7rem;color:#6e7681;margin-left:.25rem}
#ver{margin-left:auto;font-size:.7rem;color:#6e7681}

/* ── layout ── */
main{max-width:1100px;margin:1.2rem auto;padding:0 1rem 3.5rem;display:grid;gap:1rem}

/* ── hero ── */
.hero{background:#161b22;border:1px solid #30363d;border-radius:8px;
      padding:1rem 1.25rem}
.hero-title{font-size:1.15rem;font-weight:700;color:#f0f6fc;margin-bottom:.2rem}
.hero-sub{font-size:.82rem;color:#8b949e;margin-bottom:.25rem}
.hero-why{font-size:.78rem;color:#6e7681;margin-bottom:.9rem;
          border-left:2px solid #21262d;padding-left:.6rem}

/* ── 4-step flow ── */
.flow{display:flex;align-items:stretch;gap:0;overflow-x:auto}
.flow-step{flex:1;min-width:0;background:#0d1117;border:1px solid #21262d;
           border-radius:0;padding:.6rem .75rem;position:relative}
.flow-step:first-child{border-radius:6px 0 0 6px}
.flow-step:last-child {border-radius:0 6px 6px 0}
.flow-step+.flow-step{border-left:none}
.flow-num{font-size:.6rem;color:#6e7681;font-weight:700;letter-spacing:.08em;
          text-transform:uppercase;margin-bottom:.2rem}
.flow-label{font-size:.75rem;font-weight:700;color:#c9d1d9;margin-bottom:.2rem}
.flow-desc{font-size:.68rem;color:#8b949e;line-height:1.45}
.flow-arrow{align-self:center;color:#6e7681;font-size:.9rem;padding:0 .1rem;
            flex-shrink:0}

/* ── card ── */
.card{background:#161b22;border:1px solid #30363d;border-radius:8px;
      padding:.9rem 1.1rem}
.card-title{font-size:.65rem;font-weight:700;color:#8b949e;letter-spacing:.08em;
            text-transform:uppercase;margin-bottom:.65rem}

/* ── two-col ── */
.two-col{display:grid;grid-template-columns:320px 1fr;gap:1rem}
@media(max-width:800px){.two-col{grid-template-columns:1fr}}

/* ── inputs panel ── */
.input-label{font-size:.65rem;color:#6e7681;text-transform:uppercase;
             letter-spacing:.06em;margin:.55rem 0 .15rem;display:flex;
             align-items:center;gap:.4rem}
.input-label:first-child{margin-top:0}
.input-badge{font-size:.6rem;color:#8b949e;background:#1c2128;
             border:1px solid #30363d;border-radius:2px;padding:.02rem .3rem}
details{margin-bottom:.15rem}
summary{font-size:.72rem;color:#58a6ff;cursor:pointer;padding:.15rem 0;
        user-select:none;list-style:none;display:flex;align-items:center;gap:.3rem}
summary::before{content:"▶";font-size:.55rem;transition:transform .15s;color:#30363d}
details[open] summary::before{transform:rotate(90deg)}
summary:hover{color:#79c0ff}
pre.fixture{background:#0d1117;border:1px solid #21262d;border-radius:5px;
            padding:.45rem .65rem;font-size:.67rem;overflow-x:auto;
            white-space:pre-wrap;word-break:break-all;max-height:160px;
            overflow-y:auto;line-height:1.4;margin-top:.25rem;color:#8b949e}

/* ── buttons ── */
.btn{display:inline-flex;align-items:center;gap:.35rem;padding:.38rem .9rem;
     font-family:inherit;font-size:.78rem;border-radius:5px;
     border:1px solid transparent;cursor:pointer;transition:opacity .12s;font-weight:600}
.btn:hover:not(:disabled){opacity:.82}
.btn:disabled{opacity:.4;cursor:default}
.btn-route      {background:#238636;border-color:#2ea043;color:#fff;width:100%;
                 justify-content:center;margin-top:.5rem}
.btn-route-norm {background:#1a3455;border-color:#388bfd;color:#79c0ff;width:100%;
                 justify-content:center;margin-top:.35rem;font-size:.75rem}
.btn-verify {background:#6e40c9;border-color:#8957e5;color:#fff;width:100%;
             justify-content:center;margin-top:.5rem}
.btn-tamper   {background:#21262d;border-color:#b36200;color:#d29922;width:100%;
               justify-content:center;margin-top:.4rem;font-size:.72rem}
.btn-dispatch {background:#1a3455;border-color:#388bfd;color:#79c0ff;width:100%;
               justify-content:center;margin-top:.5rem}
.btn-approve  {background:#1a3e2c;border-color:#2ea043;color:#3fb950;width:100%;
               justify-content:center;margin-top:.35rem}
.btn-export   {background:#21262d;border-color:#388bfd;color:#58a6ff;width:100%;
               justify-content:center;margin-top:.35rem;font-size:.72rem}

/* ── artifact summary ── */
.artifacts{background:#0d1117;border:1px solid #21262d;border-radius:6px;
           padding:.65rem .85rem;margin-bottom:.65rem}
.artifact-row{display:grid;grid-template-columns:max-content 1fr;gap:.15rem .6rem;
              align-items:baseline;margin-bottom:.2rem}
.artifact-row:last-child{margin-bottom:0}
.art-key{font-size:.65rem;color:#6e7681;text-transform:uppercase;
         letter-spacing:.05em;white-space:nowrap}
.art-val{font-size:.8rem;font-weight:700;color:#f0f6fc;word-break:break-all}
.art-hash{font-size:.71rem;color:#58a6ff;word-break:break-all;font-family:inherit}
.determinism-note{font-size:.67rem;color:#3fb950;margin-top:.5rem;
                  padding-top:.4rem;border-top:1px solid #21262d;
                  display:flex;align-items:center;gap:.35rem}

/* ── pills / badges ── */
.pill{display:inline-block;padding:.06rem .4rem;border-radius:3px;
      font-size:.68rem;font-weight:700}
.pill-ok   {background:#1a3e2c;color:#3fb950}
.pill-err  {background:#3d1f1f;color:#f85149}
.pill-warn {background:#2d2009;color:#d29922}
.pill-info {background:#1e2d45;color:#58a6ff}

/* ── receipt JSON ── */
.section-title{font-size:.65rem;font-weight:700;color:#8b949e;text-transform:uppercase;
               letter-spacing:.07em;margin:.8rem 0 .3rem;
               display:flex;align-items:center;gap:.4rem}
pre.result{background:#0d1117;border:1px solid #21262d;border-radius:5px;
           padding:.55rem .75rem;font-size:.68rem;overflow-x:auto;
           white-space:pre-wrap;word-break:break-all;max-height:380px;
           overflow-y:auto;line-height:1.45}
pre.result-ok  {border-left:3px solid #3fb950}
pre.result-err {border-left:3px solid #f85149}
pre.result-info{border-left:3px solid #388bfd}

/* ── verify banner ── */
.verify-section{margin-top:.9rem}
.verify-banner{border-radius:6px;padding:.7rem 1rem;font-size:.88rem;
               font-weight:700;text-align:center;margin-bottom:.5rem}
.banner-ok  {background:#1a3e2c;color:#3fb950;border:1px solid #2ea043}
.banner-err {background:#3d1f1f;color:#f85149;border:1px solid #f85149}
.verify-sub{font-size:.72rem;color:#8b949e;margin-bottom:.3rem;
            font-weight:400;display:block}

/* ── tamper section ── */
.tamper-section{margin-top:.9rem;padding-top:.8rem;border-top:1px dashed #30363d}
.tamper-label{font-size:.65rem;color:#6e7681;text-transform:uppercase;
              letter-spacing:.07em;margin-bottom:.4rem}
.tamper-desc{font-size:.72rem;color:#8b949e;margin-bottom:.5rem;line-height:1.5}

/* ── footer ── */
footer{position:fixed;bottom:0;left:0;right:0;background:#0d1117;
       border-top:1px solid #21262d;padding:.45rem 1.2rem;
       display:flex;align-items:center;gap:1.4rem;font-size:.67rem;
       color:#6e7681;z-index:10;overflow-x:auto;white-space:nowrap}
.ft-label{color:#6e7681;font-size:.62rem;text-transform:uppercase;
          letter-spacing:.07em;margin-right:.3rem}
.ft-ep{color:#8b949e}
.ft-ep .method{color:#d29922}
.ft-ep .path{color:#58a6ff}
.ft-arch{margin-left:auto;color:#6e7681;font-size:.65rem}

/* ── misc ── */
.dimmed{color:#6e7681;font-size:.75rem}
.hidden{display:none!important}
.error-note  {font-size:.75rem;color:#f85149;margin-top:.4rem;line-height:1.5}
.warn-note   {font-size:.75rem;color:#d29922;margin-top:.4rem;line-height:1.5}
.success-note{font-size:.75rem;color:#3fb950;margin-top:.4rem;line-height:1.5}
.loading-note{font-size:.75rem;color:#6e7681;margin-top:.4rem;line-height:1.5}
.norm-preview{background:#0d1117;border:1px solid #21262d;border-radius:5px;
              padding:.4rem .65rem;margin-top:.35rem;font-size:.67rem;line-height:1.6}
.norm-preview-row{display:grid;grid-template-columns:max-content 1fr;gap:.1rem .6rem}
.norm-preview-key{color:#6e7681;text-transform:uppercase;font-size:.6rem;
                  letter-spacing:.05em;white-space:nowrap}
.norm-preview-val{color:#c9d1d9;word-break:break-all}
.norm-field-invalid{border-color:#f85149!important}
.copy-btn{background:none;border:1px solid #30363d;border-radius:3px;color:#58a6ff;
          cursor:pointer;font-family:inherit;font-size:.6rem;padding:.05rem .3rem;
          margin-left:.35rem;transition:color .1s}
.copy-btn:hover{color:#79c0ff}
.btn-dl{display:inline-flex;align-items:center;gap:.3rem;background:#1a3455;
        border:1px solid #388bfd;border-radius:4px;color:#79c0ff;cursor:pointer;
        font-family:inherit;font-size:.72rem;font-weight:600;padding:.28rem .7rem;
        margin-top:.45rem;transition:opacity .12s}
.btn-dl:hover{opacity:.82}

/* ── norm form inputs ── */
.norm-field-wrap{margin:.42rem 0 .1rem}
.norm-field-label{font-size:.62rem;color:#6e7681;text-transform:uppercase;
                  letter-spacing:.06em;margin-bottom:.15rem;display:flex;
                  align-items:center;gap:.3rem}
.norm-req{color:#f85149;font-size:.65rem;line-height:1}
.norm-input{width:100%;background:#0d1117;border:1px solid #30363d;border-radius:4px;
            color:#c9d1d9;font-family:inherit;font-size:.8rem;padding:.3rem .5rem;
            outline:none;transition:border-color .12s}
.norm-input:focus{border-color:#388bfd}
.norm-input.norm-field-invalid{border-color:#f85149!important}
/* ── step framing inside norm section ── */
.norm-step{display:flex;align-items:center;gap:.4rem;
           margin:.65rem 0 .3rem;padding-top:.55rem;border-top:1px solid #21262d}
.norm-step-num{background:#1e2d45;color:#58a6ff;border-radius:50%;
               width:17px;height:17px;display:inline-flex;align-items:center;
               justify-content:center;font-size:.58rem;font-weight:700;flex-shrink:0}
.norm-step-num.done{background:#1a3e2c;color:#3fb950}
.norm-step-lbl{font-size:.7rem;font-weight:700;color:#8b949e;text-transform:uppercase;
               letter-spacing:.06em}
/* ── success panel (steps 3–4) ── */
.norm-success-panel{background:#0d1117;border:1px solid #2ea04355;border-radius:5px;
                    padding:.55rem .75rem;margin-top:.1rem}
.norm-success-title{font-size:.65rem;font-weight:700;color:#3fb950;
                    text-transform:uppercase;letter-spacing:.07em;margin-bottom:.35rem;
                    display:flex;align-items:center;gap:.35rem}
.norm-success-actions{display:flex;gap:.45rem;flex-wrap:wrap;margin-top:.45rem;
                      padding-top:.35rem;border-top:1px solid #21262d}
/* ── error guidance panel ── */
.norm-error-panel{background:#0d1117;border:1px solid #f8514944;border-radius:5px;
                  padding:.55rem .75rem;margin-top:.1rem}
.norm-error-code{font-size:.63rem;font-weight:700;color:#f85149;
                 text-transform:uppercase;letter-spacing:.06em;margin-bottom:.18rem}
.norm-error-hint{font-size:.7rem;color:#8b949e;border-left:2px solid #f8514966;
                 padding-left:.45rem;margin-top:.2rem;line-height:1.5}

/* ── source-of-truth badge ── */
.sot-badge{font-size:.55rem;color:#3fb950;background:#1a3e2c;border-radius:2px;
           padding:.02rem .28rem;font-weight:700;letter-spacing:0;text-transform:none;
           margin-left:.3rem;vertical-align:middle}
/* ── artifact guide ── */
.artifact-guide{background:#0d1117;border:1px solid #21262d;border-radius:5px;
               padding:.5rem .75rem;margin-bottom:.55rem}
.ag-row{display:grid;grid-template-columns:110px 1fr;gap:.1rem .55rem;
        font-size:.7rem;align-items:baseline;margin-bottom:.25rem}
.ag-row:last-child{margin-bottom:0}
.ag-key{color:#6e7681;font-weight:700;font-size:.63rem;text-transform:uppercase;
        letter-spacing:.04em;white-space:nowrap}
.ag-val{color:#8b949e;line-height:1.5}
/* ── operator state block ── */
.op-state-block{background:#0d1117;border:1px solid #21262d;border-radius:6px;
               padding:.5rem .75rem;margin-bottom:.65rem}
.op-state-grid{display:grid;grid-template-columns:repeat(4,1fr);gap:.3rem .5rem}
.op-state-item{display:flex;flex-direction:column;gap:.1rem}
.op-state-key{font-size:.55rem;color:#6e7681;text-transform:uppercase;letter-spacing:.06em}
.op-not-run  {font-size:.7rem;font-weight:700;color:#484f58}
.op-available{font-size:.7rem;font-weight:700;color:#d29922}
.op-verified {font-size:.7rem;font-weight:700;color:#3fb950}
.op-failed   {font-size:.7rem;font-weight:700;color:#f85149}
.op-missing  {font-size:.7rem;font-weight:700;color:#f85149}
/* guidance notes */
.guidance-note{font-size:.71rem;color:#d29922;background:#2d200944;
               border:1px solid #d2992233;border-radius:4px;
               padding:.35rem .6rem;margin-top:.35rem;line-height:1.5}
.guidance-note-err{font-size:.71rem;color:#f85149;background:#3d1f1f44;
                   border:1px solid #f8514933;border-radius:4px;
                   padding:.35rem .6rem;margin-top:.35rem;line-height:1.5}
/* ── dispatch readiness panel ── */
.dr-panel{background:#0d1117;border:1px solid #21262d;border-radius:6px;
          padding:.5rem .75rem;margin-bottom:.5rem}
.dr-ready    {font-size:.78rem;font-weight:700;color:#3fb950}
.dr-not-ready{font-size:.78rem;font-weight:700;color:#d29922}
.dr-completed{font-size:.78rem;font-weight:700;color:#58a6ff}
.dr-reason   {font-size:.68rem;color:#8b949e;margin-top:.15rem;line-height:1.5}
.checklist   {display:grid;gap:.2rem;margin-top:.4rem;padding-top:.35rem;
              border-top:1px solid #21262d33}
.cl-item     {font-size:.68rem;display:flex;align-items:center;gap:.3rem;line-height:1.4}
.cl-ok       {color:#3fb950}
.cl-pending  {color:#484f58}
/* ── panel subtitle ── */
.panel-subtitle{font-size:.72rem;color:#6e7681;margin:.05rem 0 .65rem;line-height:1.5}
/* ── integrity badges ── */
.integrity-badge{display:inline-flex;align-items:center;font-size:.52rem;font-weight:700;
                 border-radius:2px;padding:.03rem .28rem;text-transform:uppercase;
                 letter-spacing:.05em;margin-left:.4rem;vertical-align:middle}
.ib-unverified{background:#1c2128;color:#6e7681;border:1px solid #30363d}
.ib-verified  {background:#1a3e2c;color:#3fb950;border:1px solid #2ea04355}
.ib-failed    {background:#3d1f1f;color:#f85149;border:1px solid #f8514955}
/* ── consistency sentinel ── */
.ccs{background:#0d1117;border:1px solid #21262d;border-radius:6px;
     padding:.45rem .75rem;margin-bottom:.5rem}
.ccs-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
           letter-spacing:.08em;margin-bottom:.2rem}
.ccs-headline{font-size:.72rem;font-weight:700;margin-bottom:.1rem}
.ccs-detail{font-size:.65rem;line-height:1.4}
.ccs-mismatch{color:#d29922;padding:.04rem 0}
.ccs-consistent{border-left:3px solid #2ea04355}
.ccs-consistent .ccs-headline{color:#3fb950}
.ccs-attention {border-left:3px solid #d29922}
.ccs-attention  .ccs-headline{color:#d29922}
/* ── handoff summary card ── */
.hsc{background:#0d1117;border:1px solid #21262d;border-radius:6px;
     padding:.55rem .8rem;margin-bottom:.5rem}
.hsc-title{font-size:.7rem;font-weight:700;color:#c9d1d9;margin-bottom:.28rem}
.hsc-verdict{font-size:.82rem;font-weight:700;margin-bottom:.22rem}
.hsc-verdict-not-ready{color:#484f58}
.hsc-verdict-ready    {color:#d29922}
.hsc-verdict-complete {color:#3fb950}
.hsc-section{margin-top:.28rem}
.hsc-section-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
                   letter-spacing:.07em;margin-bottom:.12rem}
.hsc-row{font-size:.67rem;display:flex;gap:.35rem;line-height:1.4;padding:.04rem 0}
.hsc-row-ok{color:#3fb950}
.hsc-row-no{color:#484f58}
.hsc-summary-line{font-size:.7rem;color:#c9d1d9;margin-top:.3rem;padding-top:.28rem;
                  border-top:1px solid #21262d;line-height:1.45;font-style:italic}
@media print{
  header,footer,#op-cheatsheet,.ase-bar,#nar-rail,#run-timeline,#oab,#orb,
  #crc,#pfc,.op-state-block,#active-run-context,#run-history-panel,
  #results-placeholder,#results-loading,#route-error,#route-result,
  .hero,details,.two-col>div:first-child{display:none!important}
  .two-col{display:block!important}
  .card{border:none!important;padding:0!important}
  #hsc{border:1px solid #ccc;background:#fff;color:#000;border-radius:0}
  #hsc .hsc-title{color:#000}
  #hsc .hsc-verdict-not-ready,#hsc .hsc-verdict-ready,#hsc .hsc-verdict-complete{color:#000}
  #hsc .hsc-section-label{color:#555}
  #hsc .hsc-row-ok,#hsc .hsc-row-no{color:#000}
  #hsc .hsc-summary-line{color:#000;border-top-color:#ccc}
}
/* ── audit snapshot export ── */
.ase-bar{display:flex;align-items:center;gap:.5rem;margin-bottom:.5rem;
         padding:.35rem .65rem;background:#0d111766;border:1px solid #21262d;
         border-radius:4px}
.ase-label{font-size:.62rem;color:#6e7681;flex:1}
.ase-btn{background:none;border:1px solid #30363d;border-radius:3px;color:#58a6ff;
         cursor:pointer;font-family:inherit;font-size:.65rem;font-weight:700;
         padding:.18rem .5rem;white-space:nowrap;transition:color .1s,border-color .1s}
.ase-btn:hover{color:#79c0ff;border-color:#388bfd}
/* ── preflight summary card ── */
.pfc{background:#0d1117;border:1px solid #21262d;border-radius:6px;
     padding:.5rem .75rem;margin-bottom:.5rem}
.pfc-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
           letter-spacing:.08em;margin-bottom:.25rem}
.pfc-headline{font-size:.82rem;font-weight:700;margin-bottom:.1rem}
.pfc-detail{font-size:.67rem;color:#8b949e;line-height:1.45;margin-bottom:.2rem}
.pfc-rows{display:grid;gap:.1rem;margin:.2rem 0 .15rem}
.pfc-row{font-size:.67rem;display:flex;align-items:baseline;gap:.35rem;line-height:1.4}
.pfc-ok {color:#3fb950}
.pfc-dim{color:#484f58}
.pfc-link{font-size:.62rem;color:#58a6ff;background:none;border:none;
          font-family:inherit;padding:0;cursor:pointer;
          text-decoration:underline;text-underline-offset:2px;margin-top:.15rem;display:inline-block}
.pfc-link:hover{color:#79c0ff}
.pfc-not-ready{border-left:3px solid #30363d}
.pfc-ready    {border-left:3px solid #d29922}
.pfc-complete {border-left:3px solid #2ea043}
.pfc-not-ready .pfc-headline{color:#484f58}
.pfc-ready     .pfc-headline{color:#d29922}
.pfc-complete  .pfc-headline{color:#3fb950}
/* ── active section emphasis ── */
.as-chip{display:inline-block;padding:.04rem .3rem;border-radius:2px;font-size:.55rem;
         font-weight:700;vertical-align:middle;margin-left:.35rem;letter-spacing:.03em;
         background:#1e2d45;color:#388bfd;text-transform:lowercase}
.as-active{border-color:#388bfd44!important;box-shadow:inset 0 0 0 1px #388bfd1a}
/* ── current-run checklist card ── */
.crc{background:#0d1117;border:1px solid #21262d;border-radius:6px;
     padding:.5rem .75rem;margin-bottom:.5rem}
.crc-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
           letter-spacing:.08em;margin-bottom:.3rem}
.crc-row{font-size:.7rem;display:flex;align-items:baseline;gap:.4rem;
         padding:.1rem 0;line-height:1.45}
.crc-icon-done   {color:#3fb950;flex-shrink:0;width:.85rem}
.crc-icon-pending{color:#484f58;flex-shrink:0;width:.85rem}
.crc-icon-blocked{color:#d29922;flex-shrink:0;width:.85rem}
.crc-text-done   {color:#c9d1d9}
.crc-text-pending{color:#484f58}
.crc-text-blocked{color:#6e7681}
.crc-anchor{font-size:.62rem;color:#58a6ff;background:none;border:none;
            font-family:inherit;padding:0;cursor:pointer;
            text-decoration:underline;text-underline-offset:2px;margin-left:.35rem}
.crc-anchor:hover{color:#79c0ff}
.crc-footer{font-size:.67rem;margin-top:.28rem;padding-top:.28rem;
            border-top:1px solid #21262d;line-height:1.4}
.crc-footer-incomplete{color:#484f58}
.crc-footer-ready     {color:#d29922}
.crc-footer-complete  {color:#3fb950}
/* ── dispatch blocker list ── */
.dbl{background:#0d1117;border:1px solid #21262d;border-radius:6px;
     padding:.45rem .75rem;margin-top:.4rem;margin-bottom:.5rem}
.dbl-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
           letter-spacing:.08em;margin-bottom:.22rem}
.dbl-item{font-size:.68rem;display:flex;align-items:baseline;gap:.4rem;
          padding:.1rem 0;line-height:1.45}
.dbl-item-blocked{color:#d29922}
.dbl-item-bullet{color:#d29922;flex-shrink:0}
.dbl-clear{font-size:.68rem;color:#3fb950;line-height:1.45}
.dbl-done {font-size:.68rem;color:#58a6ff;line-height:1.45}
.dbl-anchor{font-size:.62rem;color:#58a6ff;background:none;border:none;
            font-family:inherit;padding:0;cursor:pointer;
            text-decoration:underline;text-underline-offset:2px;margin-left:.3rem}
.dbl-anchor:hover{color:#79c0ff}
/* ── artifact freshness markers ── */
.fm{display:inline-block;padding:.04rem .32rem;border-radius:2px;font-size:.57rem;
    font-weight:700;vertical-align:middle;margin-left:.2rem;letter-spacing:.03em;
    text-transform:lowercase}
.fm-fresh  {background:#1e2d45;color:#388bfd}
.fm-pending{background:#1c2128;color:#484f58}
/* ── panel microbadges ── */
.mb{display:inline-block;padding:.04rem .32rem;border-radius:2px;font-size:.57rem;
    font-weight:700;vertical-align:middle;margin-left:.35rem;letter-spacing:.03em;
    text-transform:lowercase}
.mb-on {background:#1a3e2c;color:#3fb950}
.mb-dim{background:#1c2128;color:#484f58}
.mb-err{background:#3d1f1f;color:#f85149}
/* ── next-action rail ── */
.nar-rail{background:#0d1117;border:1px solid #21262d;border-radius:5px;
          padding:.4rem .65rem;margin-bottom:.5rem}
.nar-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
           letter-spacing:.08em;margin-bottom:.1rem}
.nar-action{font-size:.75rem;font-weight:700;display:block}
.nar-action-idle{color:#484f58}
.nar-action-next{color:#d29922}
.nar-action-done{color:#3fb950}
.nar-reason{font-size:.65rem;color:#6e7681;margin-top:.08rem;line-height:1.4;display:block}
/* ── active run context ── */
.arc-block{background:#0d1117;border:1px solid #21262d;border-radius:6px;
           padding:.5rem .75rem;margin-bottom:.5rem}
.arc-row{display:grid;grid-template-columns:90px 1fr;gap:.1rem .5rem;
         font-size:.68rem;align-items:baseline;margin-bottom:.2rem}
.arc-row:last-child{margin-bottom:0}
.arc-key{color:#6e7681;text-transform:uppercase;font-size:.6rem;
         letter-spacing:.05em;white-space:nowrap}
.arc-val{color:#c9d1d9;word-break:break-all}
.arc-val-pending{color:#484f58;font-style:italic;word-break:break-all}
.arc-val-ok {color:#3fb950;word-break:break-all}
.arc-val-err{color:#f85149;word-break:break-all}
/* ── handoff note ── */
.handoff-note{background:#0d1117;border:1px solid #21262d;border-radius:5px;
              padding:.45rem .7rem;margin-top:.5rem}
.handoff-note-active{border-color:#388bfd44}
.hn-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
          letter-spacing:.08em;margin-bottom:.22rem}
.hn-row{font-size:.68rem;color:#8b949e;display:flex;align-items:baseline;
        gap:.35rem;line-height:1.45;padding:.06rem 0}
.hn-row-check{color:#3fb950}
.hn-object{font-size:.67rem;color:#6e7681;margin-top:.2rem;padding-top:.2rem;
           border-top:1px solid #21262d33;line-height:1.4}
/* ── run history ── */
.run-history{background:#0d1117;border:1px solid #21262d;border-radius:5px;
             padding:.45rem .7rem;margin-bottom:.3rem}
.rh-entry{font-size:.68rem;display:flex;align-items:baseline;gap:.5rem;
          padding:.15rem 0;border-bottom:1px solid #21262d33;line-height:1.4}
.rh-entry:last-child{border-bottom:none}
.rh-ts{color:#484f58;font-size:.62rem;white-space:nowrap;flex-shrink:0}
.rh-ok{color:#3fb950}
.rh-err{color:#f85149}
/* ── artifact size guard ── */
pre.result.collapsed{max-height:120px;overflow:hidden}
.expand-btn{background:none;border:1px solid #30363d;border-radius:3px;
            color:#58a6ff;cursor:pointer;font-family:inherit;font-size:.63rem;
            padding:.1rem .38rem;margin-top:.2rem;transition:color .1s}
.expand-btn:hover{color:#79c0ff}
/* ── run timeline ── */
.run-timeline{background:#0d1117;border:1px solid #21262d;border-radius:6px;
              padding:.5rem .75rem;margin-bottom:.5rem}
.rt-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
          letter-spacing:.08em;margin-bottom:.35rem}
.rt-steps{display:flex;align-items:center;gap:0;margin-bottom:.28rem}
.rt-step{display:flex;flex-direction:column;align-items:center;gap:.12rem;flex:1;min-width:0}
.rt-dot{width:8px;height:8px;border-radius:50%;background:#1c2128;border:1px solid #30363d}
.rt-name{font-size:.6rem;font-weight:700;letter-spacing:.04em;text-transform:uppercase;
         color:#484f58;white-space:nowrap}
.rt-conn{flex:1;height:1px;background:#21262d;min-width:.4rem}
.rt-idle    .rt-dot{background:#1c2128;border-color:#30363d}
.rt-idle    .rt-name{color:#484f58}
.rt-ready   .rt-dot{background:#1e2d45;border-color:#388bfd}
.rt-ready   .rt-name{color:#58a6ff}
.rt-done    .rt-dot{background:#1a3e2c;border-color:#2ea043}
.rt-done    .rt-name{color:#3fb950}
.rt-blocked .rt-dot{background:#2d2009;border-color:#d29922}
.rt-blocked .rt-name{color:#6e7681}
.rt-summary{font-size:.69rem;color:#6e7681;line-height:1.4}
/* ── operator action bar ── */
.oab{background:#0d1117;border:1px solid #21262d;border-radius:6px;
     padding:.45rem .75rem;margin-bottom:.5rem}
.oab-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
           letter-spacing:.08em;margin-bottom:.2rem}
.oab-body{display:flex;align-items:center;gap:.5rem}
.oab-action{font-size:.78rem;font-weight:700;flex:1}
.oab-action-idle    {color:#484f58}
.oab-action-active  {color:#d29922}
.oab-action-complete{color:#3fb950}
.oab-btn{background:none;border:1px solid #30363d;border-radius:3px;color:#58a6ff;
         cursor:pointer;font-family:inherit;font-size:.68rem;font-weight:700;
         padding:.15rem .5rem;white-space:nowrap;flex-shrink:0;
         transition:color .1s,border-color .1s}
.oab-btn:hover{color:#79c0ff;border-color:#388bfd}
.oab-btn-complete{color:#6e7681;border-color:#21262d;cursor:default}
.oab-reason{font-size:.65rem;color:#6e7681;margin-top:.18rem;line-height:1.4}
/* ── outcome banner ── */
.orb{background:#0d1117;border-left:3px solid #30363d;border:1px solid #21262d;
     border-radius:6px;padding:.5rem .75rem;margin-bottom:.5rem}
.orb-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
           letter-spacing:.08em;margin-bottom:.22rem}
.orb-headline{font-size:.8rem;font-weight:700;margin-bottom:.1rem}
.orb-detail{font-size:.67rem;color:#8b949e;line-height:1.45;margin-bottom:.15rem}
.orb-link{font-size:.67rem;color:#58a6ff;cursor:pointer;background:none;border:none;
          font-family:inherit;padding:0;text-decoration:underline;text-underline-offset:2px}
.orb-link:hover{color:#79c0ff}
.orb-neutral .orb-headline{color:#484f58}
.orb-success .orb-headline{color:#3fb950}
.orb-warning .orb-headline{color:#d29922}
.orb-blocked .orb-headline{color:#f85149}
.orb-neutral{border-left-color:#30363d}
.orb-success{border-left-color:#2ea043}
.orb-warning{border-left-color:#d29922}
.orb-blocked{border-left-color:#f85149}
/* ── session activity log ── */
.sal{background:#0d1117;border:1px solid #21262d;border-radius:6px;
     padding:.45rem .75rem;margin-bottom:.5rem}
.sal-header{display:flex;align-items:center;margin-bottom:.22rem}
.sal-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
           letter-spacing:.08em;flex:1}
.sal-clear{background:none;border:none;color:#484f58;font-family:inherit;font-size:.6rem;
           cursor:pointer;padding:0;text-decoration:underline;text-underline-offset:2px}
.sal-clear:hover{color:#8b949e}
.sal-empty{font-size:.65rem;color:#3d4349;font-style:italic}
.sal-list{display:flex;flex-direction:column;gap:.06rem}
.sal-entry{font-size:.67rem;display:flex;align-items:baseline;gap:.4rem;
           padding:.06rem 0;line-height:1.4}
.sal-entry-latest{color:#c9d1d9;font-weight:600}
.sal-entry-older{color:#484f58}
.sal-idx{color:#3d4349;font-size:.6rem;min-width:.9rem;flex-shrink:0;text-align:right}
.sal-msg{color:#6e7681;font-size:.62rem}
/* ── run identity block ── */
.rib{background:#0d1117;border:1px solid #21262d;border-radius:6px;
     padding:.45rem .75rem;margin-bottom:.5rem}
.rib-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
           letter-spacing:.08em;margin-bottom:.25rem}
.rib-row{display:grid;grid-template-columns:90px 1fr;gap:.06rem .5rem;
         font-size:.68rem;align-items:baseline}
.rib-key{color:#6e7681;font-size:.63rem;text-transform:uppercase;letter-spacing:.04em}
.rib-val-current{color:#3fb950}.rib-val-prev{color:#d29922}.rib-val-idle{color:#484f58}
.rib-val-err{color:#f85149}
.rib-hint{font-size:.6rem;color:#6e7681;grid-column:2;margin-top:.02rem;font-style:italic}
/* ── artifact lineage badge ── */
.lin{display:inline-block;padding:.04rem .32rem;border-radius:2px;font-size:.57rem;
     font-weight:700;vertical-align:middle;margin-left:.2rem;letter-spacing:.03em;
     text-transform:lowercase}
.lin-current{background:#1a3e2c;color:#3fb950}
.lin-prev   {background:#2d2009;color:#d29922}
.lin-idle   {background:#1c2128;color:#484f58}
/* ── lineage mismatch note ── */
.lin-note{font-size:.67rem;line-height:1.5;color:#d29922;
          background:#1c180055;border:1px solid #d2992244;border-radius:4px;
          padding:.3rem .55rem;margin-top:.35rem;margin-bottom:.3rem}
.lin-note-hint{font-size:.62rem;color:#8b949e;margin-top:.1rem}
/* ── dispatch handoff dossier ── */
.dhd{background:#0d1117;border:1px solid #21262d;border-radius:6px;
     padding:.55rem .8rem;margin-top:.5rem}
.dhd-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
           letter-spacing:.08em;margin-bottom:.28rem}
.dhd-verdict{font-size:.85rem;font-weight:700;margin-bottom:.18rem}
.dhd-verdict-none       {color:#484f58}
.dhd-verdict-not-ready  {color:#484f58}
.dhd-verdict-ready      {color:#d29922}
.dhd-verdict-exported   {color:#3fb950}
.dhd-verdict-attention  {color:#f85149}
.dhd-meaning{font-size:.67rem;color:#8b949e;line-height:1.5;margin-bottom:.28rem;
             padding-bottom:.25rem;border-bottom:1px solid #21262d}
.dhd-checklist{display:grid;gap:.08rem;margin-bottom:.25rem}
.dhd-row{font-size:.68rem;display:flex;align-items:baseline;gap:.38rem;
         line-height:1.4;padding:.04rem 0}
.dhd-ok  {color:#3fb950;flex-shrink:0;width:.85rem}
.dhd-no  {color:#484f58;flex-shrink:0;width:.85rem}
.dhd-warn{color:#d29922;flex-shrink:0;width:.85rem}
.dhd-next{font-size:.67rem;color:#c9d1d9;background:#21262d44;border:1px solid #30363d;
          border-radius:4px;padding:.28rem .5rem;margin-top:.22rem;line-height:1.45}
.dhd-next-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
                letter-spacing:.07em;display:block;margin-bottom:.06rem}
/* ── dispatch packet inspection ── */
.dpi{background:#0d1117;border:1px solid #21262d;border-radius:6px;
     padding:.5rem .75rem;margin-top:.5rem}
.dpi-label{font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;
           letter-spacing:.08em;margin-bottom:.28rem}
.dpi-meta{display:flex;align-items:center;gap:.65rem;margin-bottom:.3rem;flex-wrap:wrap}
.dpi-meta-item{display:flex;align-items:baseline;gap:.3rem;font-size:.65rem}
.dpi-meta-key{color:#6e7681;text-transform:uppercase;font-size:.6rem;letter-spacing:.04em}
.dpi-origin-current{color:#3fb950;font-weight:700}
.dpi-origin-prev   {color:#d29922;font-weight:700}
.dpi-origin-none   {color:#484f58;font-weight:700}
.dpi-integrity-ok  {color:#3fb950;font-weight:700}
.dpi-integrity-fail{color:#f85149;font-weight:700}
.dpi-integrity-none{color:#484f58;font-weight:700}
.dpi-empty{font-size:.67rem;color:#3d4349;font-style:italic;line-height:1.5}
.dpi-empty-hint{font-size:.63rem;color:#3d4349;margin-top:.1rem}
.dpi-viewer{background:#0d111788;border:1px solid #21262d;border-radius:4px;
            font-size:.67rem;line-height:1.45;padding:.4rem .6rem;
            white-space:pre-wrap;word-break:break-all;max-height:220px;
            overflow-y:auto;color:#8b949e;margin-top:.22rem}
</style>
</head>
<body>

<header>
  <span id="status-dot"></span>
  <span id="hdr-title">PostCAD</span>
  <span id="hdr-tag">reviewer shell</span>
  <span id="ver">loading…</span>
</header>

<main>

  <!-- A. Hero / info architecture -->
  <div class="hero">
    <div class="hero-title">PostCAD — Human Review Surface</div>
    <div class="hero-sub">Deterministic manufacturing routing · verifiable receipts · operator-gated dispatch</div>
    <div class="hero-why"><strong style="color:#c9d1d9">What is this page?</strong> This is the PostCAD reviewer shell — a human review layer on top of a deterministic routing kernel. The operator reviews the routed result, supporting evidence, and dispatch readiness before a case is handed off to manufacturing. The protocol is auditable: every decision carries a machine-readable reason code and a cryptographic receipt. This UI does not make routing decisions; it presents the kernel output for human review.</div>

    <div style="font-size:.6rem;font-weight:700;color:#6e7681;text-transform:uppercase;letter-spacing:.08em;margin:.8rem 0 .3rem">Operator flow — 5 steps</div>
    <div class="flow" style="margin-bottom:.8rem">
      <div class="flow-step">
        <div class="flow-num">step 1</div>
        <div class="flow-label">Open reviewer</div>
        <div class="flow-desc">Fixtures load automatically. Confirm all four pilot fields are present before proceeding.</div>
      </div>
      <div class="flow-arrow">›</div>
      <div class="flow-step">
        <div class="flow-num">step 2</div>
        <div class="flow-label">Run route</div>
        <div class="flow-desc">Submit for review. The kernel evaluates eligibility and issues a cryptographic receipt. Routing status changes to <strong>available</strong>.</div>
      </div>
      <div class="flow-arrow">›</div>
      <div class="flow-step">
        <div class="flow-num">step 3</div>
        <div class="flow-label">Inspect receipt</div>
        <div class="flow-desc">Review the receipt hash, selected manufacturer, and jurisdiction. Confirm the decision is correct before verifying.</div>
      </div>
      <div class="flow-arrow">›</div>
      <div class="flow-step">
        <div class="flow-num">step 4</div>
        <div class="flow-label">Verify replay</div>
        <div class="flow-desc">Run replay verification. The kernel re-derives the receipt from original inputs. Verification status changes to <strong>verified</strong>.</div>
      </div>
      <div class="flow-arrow">›</div>
      <div class="flow-step">
        <div class="flow-num">step 5</div>
        <div class="flow-label">Dispatch</div>
        <div class="flow-desc">If verified, create and approve the dispatch commitment. Stop here if evidence is insufficient. Dispatch is irreversible once approved.</div>
      </div>
    </div>

    <div style="font-size:.6rem;font-weight:700;color:#6e7681;text-transform:uppercase;letter-spacing:.08em;margin-bottom:.3rem">Protocol internals</div>
    <div class="flow">
      <div class="flow-step">
        <div class="flow-num">01 · inputs</div>
        <div class="flow-label">Inputs</div>
        <div class="flow-desc">Dental CAD case + manufacturer registry + routing policy</div>
      </div>
      <div class="flow-arrow">›</div>
      <div class="flow-step">
        <div class="flow-num">02 · kernel</div>
        <div class="flow-label">Kernel</div>
        <div class="flow-desc">Deterministic routing engine evaluates eligibility and selects a manufacturer</div>
      </div>
      <div class="flow-arrow">›</div>
      <div class="flow-step">
        <div class="flow-num">03 · output</div>
        <div class="flow-label">Output</div>
        <div class="flow-desc">Cryptographically committed receipt with hash-chained audit entry</div>
      </div>
      <div class="flow-arrow">›</div>
      <div class="flow-step">
        <div class="flow-num">04 · verify</div>
        <div class="flow-label">Verification</div>
        <div class="flow-desc">Independent replay confirms the decision — same hash, every time</div>
      </div>
    </div>
  </div>

  <!-- CLI quick reference -->
  <details class="card" id="cli-quickref">
    <summary style="color:#8b949e;font-weight:700;letter-spacing:.06em;text-transform:uppercase;font-size:.65rem">CLI helper commands &amp; quick reference</summary>
    <div style="margin-top:.6rem">
      <div style="font-size:.65rem;font-weight:700;color:#6e7681;text-transform:uppercase;letter-spacing:.07em;margin-bottom:.35rem">Golden path</div>
      <div style="font-size:.73rem;color:#8b949e;margin-bottom:.75rem;border-left:2px solid #21262d;padding-left:.6rem;line-height:1.6">Open reviewer &rarr; Run route &rarr; Inspect receipt &rarr; Verify replay &rarr; Dispatch</div>
      <div style="font-size:.65rem;font-weight:700;color:#6e7681;text-transform:uppercase;letter-spacing:.07em;margin-bottom:.35rem">CLI companion scripts (no service required)</div>
      <div style="display:grid;gap:.35rem;margin-bottom:.75rem">
        <div style="display:grid;grid-template-columns:240px 1fr;gap:0 .75rem;font-size:.72rem;align-items:baseline">
          <code style="color:#58a6ff">./examples/pilot/run_pilot.sh</code>
          <span style="color:#8b949e">Route the pilot case + self-verify. Writes <code style="color:#8b949e">examples/pilot/receipt.json</code>.</span>
        </div>
        <div style="display:grid;grid-template-columns:240px 1fr;gap:0 .75rem;font-size:.72rem;align-items:baseline">
          <code style="color:#58a6ff">./examples/pilot/verify.sh</code>
          <span style="color:#8b949e">Replay-verify the receipt against the original inputs. No stored state trusted.</span>
        </div>
      </div>
      <div style="font-size:.65rem;font-weight:700;color:#6e7681;text-transform:uppercase;letter-spacing:.07em;margin-bottom:.35rem">This page uses the same protocol over HTTP</div>
      <div style="font-size:.72rem;color:#8b949e;line-height:1.6">This reviewer calls <code style="color:#8b949e">POST /pilot/route-normalized</code> and <code style="color:#8b949e">POST /verify</code> — the same kernel and verifier as the CLI scripts. Pilot fixtures are loaded automatically from <code style="color:#8b949e">examples/pilot/</code>. Use this page for human review and dispatch; use the CLI scripts for headless CI or independent verification.</div>
    </div>
  </details>

  <!-- Glossary -->
  <details class="card">
    <summary style="color:#8b949e;font-weight:700;letter-spacing:.06em;text-transform:uppercase;font-size:.65rem">Terms &amp; glossary</summary>
    <div style="margin-top:.6rem;display:grid;gap:.3rem">
      <div style="display:grid;grid-template-columns:145px 1fr;gap:0 .6rem;font-size:.72rem;align-items:baseline"><span style="color:#8b949e;font-weight:700">Candidate</span><span style="color:#c9d1d9;line-height:1.5">A manufacturer that meets the capability and compliance requirements for this case.</span></div>
      <div style="display:grid;grid-template-columns:145px 1fr;gap:0 .6rem;font-size:.72rem;align-items:baseline"><span style="color:#8b949e;font-weight:700">Route</span><span style="color:#c9d1d9;line-height:1.5">Running the routing kernel: compliance rules are evaluated, candidates are filtered, one manufacturer is selected deterministically.</span></div>
      <div style="display:grid;grid-template-columns:145px 1fr;gap:0 .6rem;font-size:.72rem;align-items:baseline"><span style="color:#8b949e;font-weight:700">Receipt</span><span style="color:#c9d1d9;line-height:1.5">The auditable record of a routing decision — a JSON object with hash-committed fields that can be independently verified from the original inputs.</span></div>
      <div style="display:grid;grid-template-columns:145px 1fr;gap:0 .6rem;font-size:.72rem;align-items:baseline"><span style="color:#8b949e;font-weight:700">Dispatch</span><span style="color:#c9d1d9;line-height:1.5">A commitment to hand the case off to the selected manufacturer. Requires a verified receipt; irreversible once approved.</span></div>
      <div style="display:grid;grid-template-columns:145px 1fr;gap:0 .6rem;font-size:.72rem;align-items:baseline"><span style="color:#8b949e;font-weight:700">Refusal</span><span style="color:#c9d1d9;line-height:1.5">The kernel's explicit rejection of a case, always with a machine-readable reason code. Never a silent failure.</span></div>
      <div style="display:grid;grid-template-columns:145px 1fr;gap:0 .6rem;font-size:.72rem;align-items:baseline"><span style="color:#8b949e;font-weight:700">Replay verification</span><span style="color:#c9d1d9;line-height:1.5">Re-runs routing from the original inputs and recomputes every hash in the receipt. Does not trust stored state.</span></div>
    </div>
  </details>

  <!-- Two-column: inputs + results -->
  <div class="two-col">

    <!-- LEFT: Inputs -->
    <div id="as-route-section" class="card as-active">
      <div class="card-title">Case inputs <span style="font-weight:400;color:#6e7681">— pilot fixtures loaded from examples/pilot/</span><span id="as-chip-route" class="as-chip">active step</span></div>
      <div class="panel-subtitle">Run the deterministic pilot route — pilot fixtures auto-load from <code style="color:#6e7681">examples/pilot/</code>.</div>

      <p id="fixtures-loading" class="dimmed">Loading fixtures…</p>
      <div id="fixtures-panel" class="hidden">

        <div class="input-label">case <span class="input-badge">case.json</span></div>
        <details open>
          <summary>view JSON</summary>
          <pre class="fixture" id="fix-case"></pre>
        </details>

        <div class="input-label">registry <span class="input-badge">registry_snapshot.json</span></div>
        <details>
          <summary>view JSON</summary>
          <pre class="fixture" id="fix-registry"></pre>
        </details>

        <div class="input-label">policy / config <span class="input-badge">config.json</span></div>
        <details>
          <summary>view JSON</summary>
          <pre class="fixture" id="fix-config"></pre>
        </details>
      </div>
      <div id="fixtures-error" class="hidden error-note"></div>

      <button class="btn btn-route" id="btn-route" onclick="routeCase(this)" disabled>
        ▶ Execute Routing Kernel
      </button>

      <div id="norm-input-section" style="margin-top:.75rem;border-top:1px solid #21262d;padding-top:.65rem">

        <!-- Step 1: Enter input -->
        <div class="norm-step" style="border-top:none;padding-top:0;margin-top:0">
          <span class="norm-step-num">1</span>
          <span class="norm-step-lbl">Enter normalized pilot input</span>
        </div>
        <p style="font-size:.7rem;color:#6e7681;margin:.2rem 0 .5rem;line-height:1.5">
          All four fields are required. Use <strong style="color:#c9d1d9">⊕ Load sample</strong> to pre-fill the canonical pilot case.
        </p>
        <div class="norm-field-wrap">
          <div class="norm-field-label">case_id <span class="norm-req">*</span></div>
          <input class="norm-input" id="norm-case-id"
                 value="f1000001-0000-0000-0000-000000000001"
                 placeholder="UUID" autocomplete="off" spellcheck="false">
        </div>
        <div class="norm-field-wrap">
          <div class="norm-field-label">restoration_type <span class="norm-req">*</span></div>
          <input class="norm-input" id="norm-restoration-type"
                 value="crown"
                 placeholder="crown / bridge / …" autocomplete="off" spellcheck="false">
        </div>
        <div class="norm-field-wrap">
          <div class="norm-field-label">material <span class="norm-req">*</span></div>
          <input class="norm-input" id="norm-material"
                 value="zirconia"
                 placeholder="zirconia / pmma / …" autocomplete="off" spellcheck="false">
        </div>
        <div class="norm-field-wrap">
          <div class="norm-field-label">jurisdiction <span class="norm-req">*</span></div>
          <input class="norm-input" id="norm-jurisdiction"
                 value="DE"
                 placeholder="DE / US / JP / …" autocomplete="off" spellcheck="false">
        </div>
        <div style="margin-top:.4rem;display:flex;gap:.5rem;align-items:center">
          <button class="copy-btn" onclick="clearNormForm()">↺ Clear form</button>
          <button class="copy-btn" onclick="loadNormSample()">⊕ Load sample</button>
        </div>

        <!-- Step 2: Submit -->
        <div class="norm-step">
          <span class="norm-step-num">2</span>
          <span class="norm-step-lbl">Submit for review</span>
        </div>
        <p style="font-size:.7rem;color:#6e7681;margin:.2rem 0 .35rem;line-height:1.5">
          Keyboard shortcut: <strong style="color:#c9d1d9">Ctrl+Enter</strong> (or ⌘+Enter on Mac). The button disables during routing to prevent double-submission.
        </p>
        <button class="btn btn-route-norm" id="btn-route-norm" onclick="routeNormalized(this)" disabled>
          ▶ Submit for Review
        </button>
        <div id="route-norm-inline" class="hidden"></div>

        <!-- Steps 3 + 4 rendered here by JS on success -->
        <div id="route-norm-preview" class="hidden"></div>
      </div>
    </div>

    <!-- RIGHT: Results -->
    <div class="card">
      <!-- Operator cheat sheet — always visible -->
      <div id="op-cheatsheet" style="font-size:.67rem;color:#484f58;margin-bottom:.5rem;padding:.3rem .6rem;background:#0d111766;border:1px solid #21262d;border-radius:4px;line-height:1.7">
        <span style="color:#6e7681;font-weight:700">Quick path: </span>Run route &rarr; Inspect artifacts &rarr; Verify replay &rarr; Dispatch after verification succeeds
      </div>

      <!-- Audit snapshot export bar — always visible -->
      <div class="ase-bar">
        <span class="ase-label">Audit snapshot — current-run artifacts only</span>
        <button class="ase-btn" id="btn-copy-snapshot" onclick="copyAuditSnapshot(this)">Copy snapshot</button>
        <button class="ase-btn" onclick="downloadAuditSnapshot()">↓ Download .txt</button>
        <button class="ase-btn" id="btn-print-handoff" onclick="window.print()">⎙ Print summary</button>
      </div>

      <!-- Next-action rail — always visible, one action at a time -->
      <div id="nar-rail" class="nar-rail">
        <div class="nar-label">Next action</div>
        <span id="nar-action" class="nar-action nar-action-idle">Next: run route</span>
        <span id="nar-reason" class="nar-reason">No current receipt loaded.</span>
      </div>

      <!-- Current-run timeline strip — always visible -->
      <div id="run-timeline" class="run-timeline">
        <div class="rt-label">Current run</div>
        <div class="rt-steps">
          <div id="rt-route" class="rt-step rt-idle">
            <div class="rt-dot"></div>
            <div class="rt-name">Route</div>
          </div>
          <div class="rt-conn"></div>
          <div id="rt-receipt" class="rt-step rt-idle">
            <div class="rt-dot"></div>
            <div class="rt-name">Receipt</div>
          </div>
          <div class="rt-conn"></div>
          <div id="rt-verify" class="rt-step rt-idle">
            <div class="rt-dot"></div>
            <div class="rt-name">Verify</div>
          </div>
          <div class="rt-conn"></div>
          <div id="rt-dispatch" class="rt-step rt-idle">
            <div class="rt-dot"></div>
            <div class="rt-name">Dispatch</div>
          </div>
        </div>
        <div id="rt-summary" class="rt-summary">Current run not started</div>
      </div>

      <!-- Current run identity — lineage-aware status summary -->
      <div id="rib" class="rib">
        <div class="rib-label">Current run</div>
        <div class="rib-row"><span class="rib-key">Route</span><span id="rib-route" class="rib-val-idle">no run yet</span></div>
        <div class="rib-row"><span class="rib-key">Receipt</span><span id="rib-receipt" class="rib-val-idle">not generated</span></div>
        <div class="rib-row"><span class="rib-key">Verification</span><span id="rib-verify" class="rib-val-idle">not executed</span></div>
        <div class="rib-row"><span class="rib-key">Dispatch</span><span id="rib-dispatch" class="rib-val-idle">not exported</span></div>
      </div>

      <!-- Operator action bar — always visible, exactly one next step -->
      <div id="oab" class="oab">
        <div class="oab-label">Next action</div>
        <div class="oab-body">
          <span id="oab-action" class="oab-action oab-action-idle">Start a route for the current case</span>
          <button id="oab-btn" class="oab-btn" onclick="oabNavigate()">→ Go to route</button>
        </div>
        <div id="oab-reason" class="oab-reason">No current route artifacts exist yet.</div>
      </div>

      <!-- Current-run outcome banner — always visible -->
      <div id="orb" class="orb orb-neutral">
        <div class="orb-label">Current run status</div>
        <div id="orb-headline" class="orb-headline">No current run started</div>
        <div id="orb-detail" class="orb-detail">Start routing to generate current-run artifacts.</div>
        <button id="orb-link" class="orb-link hidden" onclick="orbNavigate()"></button>
      </div>

      <!-- Current-run completion checklist — always visible -->
      <div id="crc" class="crc">
        <div class="crc-label">Current run checklist</div>
        <div id="crc-rows"></div>
        <div id="crc-footer" class="crc-footer crc-footer-incomplete">Current run incomplete</div>
      </div>

      <!-- Current-run preflight summary — always visible -->
      <div id="pfc" class="pfc pfc-not-ready">
        <div class="pfc-label">Current run preflight</div>
        <div id="pfc-headline" class="pfc-headline">Current run not ready</div>
        <div id="pfc-detail" class="pfc-detail">Complete remaining workflow steps before dispatch export.</div>
        <div id="pfc-rows" class="pfc-rows"></div>
        <button id="pfc-link" class="pfc-link hidden" onclick="pfcNavigate()"></button>
      </div>

      <!-- Handoff summary card — visible in shell and printable -->
      <div id="hsc" class="hsc">
        <div class="hsc-title">PostCAD current-run handoff summary</div>
        <div id="hsc-verdict" class="hsc-verdict hsc-verdict-not-ready">Not ready</div>
        <div class="hsc-section">
          <div class="hsc-section-label">Workflow status</div>
          <div id="hsc-rows"></div>
        </div>
        <div class="hsc-section">
          <div class="hsc-section-label">Dispatch readiness</div>
          <div id="hsc-readiness"></div>
        </div>
        <div class="hsc-section">
          <div class="hsc-section-label">Artifact availability</div>
          <div id="hsc-artifacts"></div>
        </div>
        <div id="hsc-summary" class="hsc-summary-line">Current run requires routing before dispatch.</div>
      </div>

      <!-- Current-run consistency sentinel — always visible -->
      <div id="ccs" class="ccs ccs-consistent">
        <div class="ccs-label">Current run consistency</div>
        <div id="ccs-headline" class="ccs-headline">Current run shell state is consistent</div>
        <div id="ccs-detail" class="ccs-detail"><span style="color:#484f58;font-size:.65rem">Visible workflow indicators agree for the current run.</span></div>
      </div>

      <!-- Operator workflow status — always visible -->
      <div class="op-state-block">
        <div style="font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;letter-spacing:.08em;margin-bottom:.35rem">Workflow status</div>
        <div class="op-state-grid">
          <div class="op-state-item">
            <span class="op-state-key">Routing</span>
            <span id="ops-routing" class="op-not-run">not-run</span>
          </div>
          <div class="op-state-item">
            <span class="op-state-key">Receipt</span>
            <span id="ops-receipt" class="op-not-run">not-run</span>
          </div>
          <div class="op-state-item">
            <span class="op-state-key">Verification</span>
            <span id="ops-verify" class="op-not-run">not-run</span>
          </div>
          <div class="op-state-item">
            <span class="op-state-key">Dispatch</span>
            <span id="ops-dispatch" class="op-not-run">not-run</span>
          </div>
        </div>
      </div>

      <!-- Active run context -->
      <div id="active-run-context" class="arc-block hidden">
        <div style="font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;letter-spacing:.08em;margin-bottom:.3rem">Active run context</div>
        <div class="arc-row">
          <span class="arc-key">Manufacturer</span>
          <span class="arc-val" id="arc-manufacturer">—</span>
        </div>
        <div class="arc-row">
          <span class="arc-key">Receipt hash</span>
          <span class="arc-val" id="arc-receipt-hash">—</span>
        </div>
        <div class="arc-row">
          <span class="arc-key">Verification</span>
          <span id="arc-verify-status" class="arc-val-pending">No verification result for current route</span>
        </div>
        <div class="arc-row">
          <span class="arc-key">Dispatch</span>
          <span id="arc-dispatch-status" class="arc-val-pending">No dispatch export for current route</span>
        </div>
      </div>

      <!-- Pilot run history -->
      <div id="run-history-panel" class="hidden" style="margin-bottom:.5rem">
        <div style="font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;letter-spacing:.08em;margin-bottom:.28rem">Pilot run history</div>
        <div id="run-history-list" class="run-history"></div>
      </div>

      <!-- Session activity log — current page session only, not persisted -->
      <div id="sal" class="sal">
        <div class="sal-header">
          <span class="sal-label">Current session activity</span>
          <button class="sal-clear" onclick="clearSessionLog()">Clear log</button>
        </div>
        <div id="sal-empty" class="sal-empty">No activity yet. Start routing to begin.</div>
        <div id="sal-list" class="sal-list hidden"></div>
      </div>

      <div id="results-placeholder" style="padding:1.4rem .75rem">
        <div style="font-size:.75rem;font-weight:700;color:#484f58;margin-bottom:.3rem">No case submitted yet</div>
        <div style="font-size:.71rem;color:#3d4349;line-height:1.6;margin-bottom:.45rem">Artifact not yet generated. Run route to continue.</div>
        <div style="font-size:.68rem;color:#3a3f44;line-height:1.55;background:#0d111766;border:1px solid #21262d;border-radius:4px;padding:.4rem .6rem">
          Enter case details on the left and click <strong style="color:#6e7681">Submit for Review</strong>. Dispatch actions are not available until a routed receipt exists.
        </div>
      </div>
      <div id="results-loading" class="hidden" style="padding:1.5rem .5rem;text-align:center">
        <div class="loading-note" style="font-size:.82rem;font-weight:600">Routing in progress…</div>
        <div style="font-size:.7rem;color:#484f58;margin-top:.3rem;line-height:1.5">Kernel is evaluating eligibility and selecting a manufacturer.</div>
      </div>

      <!-- Receipt empty-state: shown when no receipt for current run -->
      <div id="receipt-empty-state" style="font-size:.71rem;color:#6e7681;background:#0d111766;border:1px solid #21262d;border-radius:4px;padding:.35rem .6rem;margin-bottom:.3rem;line-height:1.5">no receipt for current route</div>

      <!-- B. Artifact summary (shown after route) -->
      <div id="route-result" class="hidden">
        <div class="card-title">Routing decision — audit record<span id="route-result-badge" class="integrity-badge hidden"></span></div>
        <div class="panel-subtitle">Inspect generated audit artifacts. Verify before dispatching.</div>

        <div class="artifacts">
          <div class="artifact-row">
            <span class="art-key">Outcome</span>
            <span class="art-val" id="art-outcome"></span>
          </div>
          <div class="artifact-row">
            <span class="art-key">Selected Manufacturer</span>
            <span class="art-val" id="art-selected"></span>
          </div>
          <div class="artifact-row">
            <span class="art-key">Receipt Hash <span class="sot-badge">source of truth</span></span>
            <span><span class="art-hash" id="art-hash"></span><button class="copy-btn hidden" id="art-hash-copy" onclick="copyArtHashVal(this)">Copy</button></span>
          </div>
          <div class="artifact-row">
            <span class="art-key">Kernel Version</span>
            <span class="art-val" id="art-kver"></span>
          </div>
          <div class="determinism-note">
            <span>◆</span>
            <span>Deterministic result — same inputs produce the same receipt hash on every run</span>
          </div>
        </div>

        <!-- Artifact guide -->
        <details style="margin-bottom:.5rem">
          <summary id="artifact-guide-summary" style="color:#6e7681;font-size:.63rem;font-weight:700;text-transform:uppercase;letter-spacing:.07em">Artifacts in this flow</summary>
          <div class="artifact-guide" id="artifact-guide">
            <div class="ag-row">
              <span class="ag-key">Receipt</span>
              <span class="ag-val">Routing decision audit record — <strong style="color:#c9d1d9">inspect this first</strong> before verifying or dispatching.</span>
            </div>
            <div class="ag-row">
              <span class="ag-key">Receipt Hash</span>
              <span class="ag-val">Cryptographic commitment to all receipt fields. <strong style="color:#c9d1d9">Verification source of truth</strong> — the verifier recomputes this hash from the original inputs.</span>
            </div>
            <div class="ag-row">
              <span class="ag-key">Verification</span>
              <span class="ag-val">Confirms the receipt hash is authentic by replaying routing from scratch. No stored state trusted. Required before dispatch.</span>
            </div>
            <div class="ag-row">
              <span class="ag-key">Dispatch packet</span>
              <span class="ag-val">Manufacturer handoff commitment bound to the receipt hash. Irreversible once approved.</span>
            </div>
          </div>
        </details>

        <div class="section-title">
          Receipt JSON
          <span style="font-weight:400;color:#6e7681;font-size:.63rem;text-transform:none">— verification source of truth · inspect before dispatch</span>
          <span id="mb-receipt" class="mb mb-on">available</span>
          <span id="fm-receipt" class="fm fm-pending">not yet produced for current run</span>
          <span id="receipt-json-badge" class="integrity-badge hidden"></span>
        </div>
        <pre class="result result-ok" id="route-receipt-json"></pre>
        <div id="receipt-json-actions" class="hidden" style="display:flex;gap:.45rem;flex-wrap:wrap;margin-top:.3rem;margin-bottom:.35rem">
          <button class="copy-btn" style="font-size:.68rem;padding:.18rem .5rem" onclick="copyReceiptJson(this)">Copy artifact</button>
        </div>
        <button class="expand-btn hidden" id="receipt-expand-btn" onclick="expandArtifact('route-receipt-json','receipt-expand-btn')">Expand artifact</button>

        <!-- D. Verify section -->
        <div id="as-verify-section" style="border-radius:6px;border:1px solid transparent;padding:.1rem 0">
        <div class="section-title" style="margin-top:.8rem">Verify before dispatch
          <span style="font-weight:400;color:#6e7681;font-size:.63rem;text-transform:none">— replay re-derives the receipt from original inputs</span>
          <span id="mb-verify" class="mb mb-dim">not available</span>
          <span id="fm-verify" class="fm fm-pending">not yet executed for current run</span>
          <span id="as-chip-verify" class="as-chip hidden">active step</span>
        </div>
        <div class="panel-subtitle">The kernel recomputes every hash from scratch. No stored state is trusted.</div>
        <div id="verify-artifact-note" class="hidden" style="font-size:.71rem;color:#6e7681;background:#0d111766;border:1px solid #21262d;border-radius:4px;padding:.35rem .6rem;margin-bottom:.3rem;line-height:1.5">verification not yet executed for current route</div>
        <button class="btn btn-verify" id="btn-verify" onclick="verifyReceipt(this)" disabled>
          ↩ Replay Verification
        </button>
        </div>

        <!-- E. Tamper demo -->
        <div class="tamper-section">
          <div class="tamper-label">Tamper detection demo</div>
          <div class="tamper-desc">
            Modifies <code>selected_candidate_id</code> in the receipt client-side,
            then submits to the real <code>POST /verify</code> endpoint.
            The verifier catches the mismatch — no backend changes. This demonstrates
            that the receipt is cryptographically bound to its content.
          </div>
          <button class="btn btn-tamper" id="btn-tamper" onclick="tamperVerify(this)" disabled>
            ⚠ Tamper + Verify
          </button>
        </div>

        <!-- F. Verify result (deterministic order: route → receipt → verification → dispatch) -->
        <div id="verify-result" class="hidden">
          <div class="verify-section">
            <div class="section-title">Verification result <span id="verify-kind-label"></span><span style="font-weight:400;color:#6e7681;font-size:.63rem;text-transform:none"> — confirms receipt hash is authentic</span><span id="lin-verify" class="lin lin-idle">not executed</span><span id="verify-result-badge" class="integrity-badge hidden"></span></div>
            <div id="lin-verify-note" class="lin-note hidden">Verification belongs to previous run.<div class="lin-note-hint">Run verification again for current route.</div></div>
            <div id="verify-banner"></div>
            <pre class="result" id="verify-json"></pre>
            <div id="verify-json-actions" class="hidden" style="display:flex;gap:.45rem;flex-wrap:wrap;margin-top:.3rem">
              <button class="copy-btn" style="font-size:.68rem;padding:.18rem .5rem" onclick="copyVerifyJson(this)">Copy artifact</button>
            </div>
            <button class="expand-btn hidden" id="verify-expand-btn" onclick="expandArtifact('verify-json','verify-expand-btn')">Expand artifact</button>
          </div>
        </div>

        <!-- G. Dispatch commitment -->
        <div class="tamper-section" id="dispatch-section">
          <div class="section-title">Dispatch commitment
            <span style="font-weight:400;color:#6e7681;font-size:.63rem;text-transform:none">— handoff to manufacturing · irreversible once approved</span>
            <span id="mb-dispatch" class="mb mb-dim">not available</span>
            <span id="fm-dispatch" class="fm fm-pending">not yet exported for current run</span>
            <span id="as-chip-dispatch" class="as-chip hidden">active step</span>
          </div>
          <div class="panel-subtitle">Dispatch after verification succeeds. The server re-verifies the receipt before creating the record.</div>
          <div class="tamper-desc">
            Calls <code>POST /dispatch/create</code> — approve makes the commitment
            immutable; export produces the deterministic dispatch packet.
          </div>
          <!-- Dispatch readiness panel -->
          <div class="dr-panel" id="dispatch-readiness-panel">
            <div style="font-size:.55rem;font-weight:700;color:#6e7681;text-transform:uppercase;letter-spacing:.08em;margin-bottom:.3rem">Dispatch readiness</div>
            <div id="dr-status" class="dr-not-ready">Not ready for dispatch</div>
            <div id="dr-reason" class="dr-reason">Required artifact not yet generated.</div>
            <div class="checklist">
              <div id="cl-receipt"  class="cl-item cl-pending">◻ Receipt reviewed</div>
              <div id="cl-verify"   class="cl-item cl-pending">◻ Verification succeeded</div>
              <div id="cl-dispatch" class="cl-item cl-pending">◻ Dispatch action confirmed</div>
            </div>
          </div>

          <!-- Dispatch blocker list -->
          <div id="dbl" class="dbl">
            <div class="dbl-label">Dispatch blockers</div>
            <div id="dbl-body">
              <div class="dbl-item dbl-item-blocked"><span class="dbl-item-bullet">▸</span><span>No current route result — run routing first.</span></div>
            </div>
          </div>

          <div style="font-size:.7rem;color:#d29922;background:#2d200966;border:1px solid #d2992233;border-radius:4px;padding:.4rem .6rem;margin-bottom:.2rem;line-height:1.55">
            <strong style="color:#f0f6fc">Stop here if:</strong> evidence is insufficient · jurisdiction or compliance fit is unclear · manufacturer handoff should not proceed. Dispatch is irreversible once approved.
          </div>
          <button class="btn btn-dispatch" id="btn-dispatch-create" onclick="createDispatch(this)" disabled>
            ⬦ Create Dispatch
          </button>
          <div id="verify-pending-note" class="guidance-note hidden">Verification pending. Run verify before dispatch.</div>
          <div id="dispatch-blocked-note" class="guidance-note-err hidden">Dispatch blocked until verification succeeds.</div>
          <div id="dispatch-stale-note" class="hidden" style="font-size:.71rem;color:#6e7681;background:#0d111766;border:1px solid #21262d;border-radius:4px;padding:.35rem .6rem;margin-top:.35rem;line-height:1.5">no dispatch export for current route</div>

          <div id="dispatch-created" class="hidden" style="margin-top:.55rem">
            <div class="artifacts">
              <div class="artifact-row">
                <span class="art-key">Dispatch ID</span>
                <span><span class="art-hash" id="art-dispatch-id"></span><button class="copy-btn hidden" id="art-dispatch-id-copy" onclick="copyDispatchId(this)">Copy</button></span>
              </div>
              <div class="artifact-row">
                <span class="art-key">Status</span>
                <span class="art-val" id="art-dispatch-status"></span>
              </div>
            </div>
            <button class="btn btn-approve" id="btn-dispatch-approve" onclick="approveDispatch(this)" disabled>
              ✓ Approve Dispatch
            </button>
            <button class="btn btn-export" id="btn-dispatch-export" onclick="exportDispatch(this)" disabled>
              ↓ Export Dispatch Packet
            </button>
          </div>

          <div id="dispatch-export-result" class="hidden" style="margin-top:.55rem">
            <div class="section-title">Export packet — deterministic dispatch record
              <span style="font-weight:400;color:#6e7681;font-size:.63rem;text-transform:none"> · ready for handoff</span>
              <span id="as-chip-export" class="as-chip hidden">active step</span>
              <span id="lin-dispatch-export" class="lin lin-idle">not exported</span>
              <span id="dispatch-result-badge" class="integrity-badge hidden"></span>
            </div>
            <div id="lin-dispatch-note" class="lin-note hidden">Dispatch export belongs to previous run.<div class="lin-note-hint">Export dispatch packet again for current route.</div></div>
            <pre class="result result-info" id="dispatch-export-json"></pre>
            <div id="dispatch-export-actions" class="hidden" style="display:flex;gap:.45rem;flex-wrap:wrap;margin-top:.4rem;padding-top:.35rem;border-top:1px solid #21262d">
              <button class="btn-dl" onclick="downloadExportPacket()">↓ Download export_packet.json</button>
              <button class="copy-btn" style="font-size:.68rem;padding:.18rem .5rem" onclick="copyExportJson(this)">Copy JSON</button>
            </div>
            <button class="expand-btn hidden" id="dispatch-expand-btn" onclick="expandArtifact('dispatch-export-json','dispatch-expand-btn')">Expand artifact</button>
          </div>

          <!-- Dispatch packet inspection panel — always visible -->
          <div id="dpi" class="dpi">
            <div class="dpi-label">Dispatch packet inspection</div>
            <div id="dpi-meta" class="dpi-meta hidden">
              <div class="dpi-meta-item">
                <span class="dpi-meta-key">Packet origin</span>
                <span id="dpi-origin" class="dpi-origin-none">—</span>
              </div>
              <div class="dpi-meta-item">
                <span class="dpi-meta-key">Packet integrity</span>
                <span id="dpi-integrity" class="dpi-integrity-none">—</span>
              </div>
            </div>
            <div id="dpi-empty">
              <div class="dpi-empty">No dispatch packet generated for the current run.</div>
              <div class="dpi-empty-hint">Run dispatch export to generate a packet for inspection.</div>
            </div>
            <pre id="dpi-viewer" class="dpi-viewer hidden"></pre>
          </div>

          <!-- Operator handoff note — neutral until export exists -->
          <div id="handoff-note" class="handoff-note">
            <div class="hn-label">Handoff</div>
            <div id="hn-body"><span style="color:#484f58;font-style:italic">No export for current route. Handoff not yet applicable.</span></div>
          </div>

          <!-- Dispatch handoff dossier — final operator checkpoint for active run -->
          <div id="dhd" class="dhd">
            <div class="dhd-label">Dispatch handoff dossier</div>
            <div id="dhd-verdict" class="dhd-verdict dhd-verdict-none">No current dispatch packet</div>
            <div id="dhd-meaning" class="dhd-meaning">No route has been generated yet for the current session.</div>
            <div id="dhd-checklist" class="dhd-checklist"></div>
            <div class="dhd-next">
              <span class="dhd-next-label">Next step</span>
              <span id="dhd-next-text">Generate a route first.</span>
            </div>
          </div>

          <div id="dispatch-success" class="hidden success-note" style="margin-top:.4rem"></div>
          <div id="dispatch-error" class="hidden error-note" style="margin-top:.4rem"></div>
        </div>
      </div>

      <!-- Route error -->
      <div id="route-error" class="hidden">
        <div class="card-title">Routing failed — case refused or invalid input</div>
        <div id="route-error-banner" class="hidden error-note" style="margin-bottom:.35rem"></div>
        <div style="font-size:.7rem;color:#6e7681;margin-bottom:.5rem">
          The kernel returned an explicit refusal with a reason code. Review the error below, correct the inputs, and resubmit. If the case genuinely does not meet routing criteria, refusal is the correct and expected outcome — not an error.
        </div>
        <pre class="result result-err" id="route-error-json"></pre>
        <div id="route-error-json-actions" style="display:flex;gap:.45rem;flex-wrap:wrap;margin-top:.3rem">
          <button class="copy-btn" style="font-size:.68rem;padding:.18rem .5rem" onclick="copyRouteErrorJson(this)">Copy artifact</button>
        </div>
      </div>


    </div>
  </div>

</main>

<!-- C. Architecture / endpoint footer -->
<footer>
  <span>
    <span class="ft-label">endpoints</span>
    <span class="ft-ep"><span class="method">GET</span> <span class="path">/pilot-fixtures</span></span>
    <span style="color:#30363d;margin:0 .3rem">·</span>
    <span class="ft-ep"><span class="method">POST</span> <span class="path">/route</span></span>
    <span style="color:#30363d;margin:0 .3rem">·</span>
    <span class="ft-ep"><span class="method">POST</span> <span class="path">/pilot/route-normalized</span></span>
    <span style="color:#30363d;margin:0 .3rem">·</span>
    <span class="ft-ep"><span class="method">POST</span> <span class="path">/verify</span></span>
    <span style="color:#30363d;margin:0 .3rem">·</span>
    <span class="ft-ep"><span class="method">POST</span> <span class="path">/dispatch/create</span></span>
    <span style="color:#30363d;margin:0 .3rem">·</span>
    <span class="ft-ep"><span class="method">POST</span> <span class="path">/dispatch/:id/approve</span></span>
    <span style="color:#30363d;margin:0 .3rem">·</span>
    <span class="ft-ep"><span class="method">GET</span> <span class="path">/dispatch/:id/export</span></span>
  </span>
  <span class="ft-arch">Reviewer UI → HTTP API → PostCAD Service → Routing Kernel → Receipt / Verification / Dispatch</span>
</footer>

<script>
// ── state ──────────────────────────────────────────────────────────────────
let fixtures       = null;   // {case, registry_snapshot, routing_config}
let lastReceipt    = null;
let lastPolicy     = null;
let lastDispatchId = null;
let lastExportPacket = null;
let opRouting  = 'not-run';  // not-run | available | failed
let opReceipt  = 'not-run';  // not-run | available | missing
let opVerify   = 'not-run';  // not-run | verified  | failed
let opDispatch = 'not-run';  // not-run | available | failed
const runHistory = [];       // chronological pilot run actions
let runSerial      = 0;   // monotonic counter, increments on each new route
let verifySerial   = 0;   // set to runSerial when verification completes
let dispatchSerial = 0;   // set to runSerial when dispatch is exported

// ── boot ───────────────────────────────────────────────────────────────────
(async function boot() {
  try {
    const r = await fetch('/version');
    const v = await r.json();
    document.getElementById('status-dot').className = r.ok ? 'ok' : 'err';
    document.getElementById('ver').textContent =
      v.protocol_version
        ? v.protocol_version + ' · ' + v.routing_kernel_version
        : JSON.stringify(v);
  } catch(e) {
    document.getElementById('status-dot').className = 'err';
    document.getElementById('ver').textContent = 'service unreachable';
  }

  try {
    const r = await fetch('/pilot-fixtures');
    if (!r.ok) throw new Error('HTTP ' + r.status + ': ' + await r.text());
    fixtures = await r.json();
    document.getElementById('fix-case').textContent     = fmt(fixtures.case);
    document.getElementById('fix-registry').textContent = fmt(fixtures.registry_snapshot);
    document.getElementById('fix-config').textContent   = fmt(fixtures.routing_config);
    document.getElementById('fixtures-loading').classList.add('hidden');
    document.getElementById('fixtures-panel').classList.remove('hidden');
    document.getElementById('btn-route').disabled      = false;
    document.getElementById('btn-route-norm').disabled  = false;
  } catch(e) {
    document.getElementById('fixtures-loading').classList.add('hidden');
    const errEl = document.getElementById('fixtures-error');
    errEl.innerHTML =
      '<div style="background:#3d1f1f44;border:1px solid #f8514933;border-radius:5px;padding:.55rem .75rem;margin-bottom:.4rem">'
      + '<div style="font-size:.65rem;font-weight:700;color:#f85149;text-transform:uppercase;letter-spacing:.06em;margin-bottom:.25rem">Cannot review — no valid case context loaded</div>'
      + '<div style="font-size:.71rem;color:#c9d1d9;line-height:1.5;margin-bottom:.3rem">Pilot fixtures could not be loaded. This reviewer requires a valid routed case context before any review or dispatch action is possible. This page is <strong>not dispatchable</strong> in this state.</div>'
      + '<div style="font-size:.67rem;color:#8b949e;border-left:2px solid #f8514966;padding-left:.4rem;line-height:1.45">' + esc(e.message) + '</div>'
      + '</div>'
      + '<div style="font-size:.69rem;color:#d29922;line-height:1.55">'
      + '<strong style="color:#f0f6fc">Safe action:</strong> Start the service from the repo root '
      + '(<code style="color:#8b949e">cargo run -p postcad-service</code>) so <code style="color:#8b949e">examples/pilot/</code> '
      + 'is reachable, then reload this page. Do not attempt to dispatch without a valid loaded case.'
      + '</div>';
    errEl.classList.remove('hidden');
  }
})();

// ── Execute Routing Kernel ─────────────────────────────────────────────────
async function routeCase(btn) {
  if (!fixtures) {
    hide('results-placeholder');
    document.getElementById('route-error-json').textContent =
      'No case fixtures loaded. The reviewer cannot route without valid pilot fixtures.\n\n'
      + 'Reload the page after starting the service from the repo root (cargo run -p postcad-service).';
    show('route-error');
    return;
  }
  setBtn(btn, 'Running kernel…', true);
  document.getElementById('btn-route-norm').disabled = true;
  if (lastReceipt) salLog('Current run reset', 'Previous run cleared for new route.');
  salLog('Route requested', 'Routing kernel execution started.');
  runSerial++;

  hide('results-placeholder');
  hide('route-norm-inline'); hide('route-norm-preview');
  hide('route-result'); hide('route-error'); hide('verify-result');
  hide('dispatch-created'); hide('dispatch-export-result');
  hide('dispatch-success'); hide('dispatch-error');
  show('results-loading');
  lastReceipt = null; lastPolicy = null; lastDispatchId = null; lastExportPacket = null;
  updateOpState('not-run', 'not-run', 'not-run', 'not-run');
  const _chb = document.getElementById('art-hash-copy');
  if (_chb) _chb.classList.add('hidden');
  const _dic = document.getElementById('art-dispatch-id-copy');
  if (_dic) _dic.classList.add('hidden');
  hide('receipt-json-actions'); hide('verify-json-actions'); hide('verify-artifact-note');
  hide('receipt-empty-state');
  hide('dispatch-export-actions');
  hide('receipt-expand-btn'); hide('verify-expand-btn'); hide('dispatch-expand-btn');
  ['route-receipt-json','verify-json','dispatch-export-json'].forEach(id => {
    const el = document.getElementById(id); if (el) el.classList.remove('collapsed');
  });
  clearRunHistory();
  document.getElementById('btn-dispatch-create').disabled  = true;
  document.getElementById('btn-dispatch-approve').disabled = true;
  document.getElementById('btn-dispatch-export').disabled  = true;

  try {
    const r = await fetch('/route', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({
        case:             fixtures.case,
        registry_snapshot: fixtures.registry_snapshot,
        routing_config:   fixtures.routing_config,
      }),
    });
    const data = await r.json();

    if (r.ok && data.receipt) {
      lastReceipt = data.receipt;
      lastPolicy  = data.derived_policy;
      const rc = data.receipt;

      const outcome  = rc.outcome || '—';
      const selected = rc.selected_candidate_id || '(none — refused)';
      const rhash    = rc.receipt_hash || '—';
      const kver     = rc.routing_kernel_version || '—';

      document.getElementById('art-outcome').innerHTML =
        `<span class="pill ${outcome === 'routed' ? 'pill-ok' : 'pill-warn'}">${esc(outcome)}</span>`;
      document.getElementById('art-selected').textContent = selected;
      document.getElementById('art-hash').textContent     = rhash;
      document.getElementById('art-kver').textContent     = kver;
      document.getElementById('route-receipt-json').textContent = fmt(rc);
      collapseIfLarge('route-receipt-json', 'receipt-expand-btn');
      const copyHashBtn = document.getElementById('art-hash-copy');
      if (copyHashBtn && rhash && rhash !== '—') copyHashBtn.classList.remove('hidden');
      show('receipt-json-actions');
      show('verify-artifact-note');

      show('route-result');
      document.getElementById('btn-verify').disabled          = false;
      document.getElementById('btn-tamper').disabled          = false;
      document.getElementById('btn-dispatch-create').disabled = false;
      updateOpState('available', 'available', 'not-run', 'available');
      salLog('Route result received', 'Route receipt generated.');
      appendRunHistory('Route executed', true);
    } else {
      hide('route-error-banner');
      document.getElementById('route-error-json').textContent = fmt(data);
      show('route-error');
      show('receipt-empty-state');
      updateOpState('failed', 'missing', null, null);
      salLog('Route result received', 'Route execution returned an error.');
      appendRunHistory('Route executed', false);
    }
  } catch(e) {
    hide('route-error-banner');
    document.getElementById('route-error-json').textContent = String(e);
    show('route-error');
    show('receipt-empty-state');
    updateOpState('failed', 'missing', null, null);
    salLog('Route result received', 'Route execution returned an error.');
    appendRunHistory('Route executed', false);
  } finally {
    hide('results-loading');
    setBtn(btn, '▶ Execute Routing Kernel', false);
    document.getElementById('btn-route-norm').disabled = false;
  }
}

// ── Route Normalized Pilot Case ────────────────────────────────────────────
async function routeNormalized(btn) {
  if (!fixtures) return;

  const pilotCase = readNormInputs();
  const ni = document.getElementById('route-norm-inline');

  // ── client-side validation ──────────────────────────────────────────────
  const missing = validateNormInput(pilotCase);
  if (missing.length) {
    markNormInvalid(missing);
    ni.textContent = 'Required fields missing: ' + missing.join(', ');
    ni.className = 'error-note';
    ni.classList.remove('hidden');
    return;   // button stays enabled; clearNormForm() / loadNormSample() clear this error
  }
  clearNormInvalid();

  setBtn(btn, 'Running kernel…', true);
  document.getElementById('btn-route').disabled = true;
  if (lastReceipt) salLog('Current run reset', 'Previous run cleared for new route.');
  salLog('Route requested', 'Routing kernel execution started.');
  runSerial++;

  hide('results-placeholder');
  hide('route-result'); hide('route-error'); hide('verify-result');
  hide('dispatch-created'); hide('dispatch-export-result');
  hide('dispatch-success'); hide('dispatch-error');
  hide('route-norm-preview');
  show('results-loading');
  // Transition to submitting state immediately — button stays disabled until finally.
  ni.textContent = 'Submitting…';
  ni.className = 'loading-note';
  ni.classList.remove('hidden');
  lastReceipt = null; lastPolicy = null; lastDispatchId = null; lastExportPacket = null;
  updateOpState('not-run', 'not-run', 'not-run', 'not-run');
  const _chb = document.getElementById('art-hash-copy');
  if (_chb) _chb.classList.add('hidden');
  const _dic = document.getElementById('art-dispatch-id-copy');
  if (_dic) _dic.classList.add('hidden');
  hide('receipt-json-actions'); hide('verify-json-actions'); hide('verify-artifact-note');
  hide('receipt-empty-state');
  hide('dispatch-export-actions');
  hide('receipt-expand-btn'); hide('verify-expand-btn'); hide('dispatch-expand-btn');
  ['route-receipt-json','verify-json','dispatch-export-json'].forEach(id => {
    const el = document.getElementById(id); if (el) el.classList.remove('collapsed');
  });
  clearRunHistory();
  document.getElementById('btn-dispatch-create').disabled  = true;
  document.getElementById('btn-dispatch-approve').disabled = true;
  document.getElementById('btn-dispatch-export').disabled  = true;

  try {
    // ── fetch (network failure → inline error) ──────────────────────────────
    let r;
    try {
      r = await fetch('/pilot/route-normalized', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({
          pilot_case:        pilotCase,
          registry_snapshot: fixtures.registry_snapshot,
          routing_config:    fixtures.routing_config,
        }),
      });
    } catch(netErr) {
      ni.innerHTML = '<div class="norm-error-panel">'
        + '<div class="norm-error-code">Network failure</div>'
        + '<div class="norm-error-hint">' + esc(String(netErr)) + '</div>'
        + '<div class="norm-error-hint" style="margin-top:.2rem">Check your network connection and confirm the PostCAD service is running.</div>'
        + '</div>';
      ni.className = '';
      ni.classList.remove('hidden');
      hide('route-error-banner');
      document.getElementById('route-error-json').textContent = String(netErr);
      show('route-error');
      show('receipt-empty-state');
      updateOpState('failed', 'missing', null, null);
      appendRunHistory('Route executed', false);
      return;
    }

    // ── parse (invalid JSON → inline error) ────────────────────────────────
    let data;
    try {
      data = await r.json();
    } catch(parseErr) {
      ni.innerHTML = '<div class="norm-error-panel">'
        + '<div class="norm-error-code">Invalid JSON response</div>'
        + '<div class="norm-error-hint">HTTP ' + r.status + ' — ' + esc(String(parseErr)) + '</div>'
        + '<div class="norm-error-hint" style="margin-top:.2rem">The service returned an unexpected response. Check the server logs for details.</div>'
        + '</div>';
      ni.className = '';
      ni.classList.remove('hidden');
      hide('route-error-banner');
      document.getElementById('route-error-json').textContent =
        'HTTP ' + r.status + ' — response is not valid JSON: ' + String(parseErr);
      show('route-error');
      show('receipt-empty-state');
      updateOpState('failed', 'missing', null, null);
      appendRunHistory('Route executed', false);
      return;
    }

    if (r.ok && data.receipt) {
      lastReceipt = data.receipt;
      lastPolicy  = data.derived_policy;
      const rc = data.receipt;

      const outcome  = rc.outcome || '—';
      const selected = rc.selected_candidate_id || '(none — refused)';
      const rhash    = rc.receipt_hash || '—';
      const kver     = rc.routing_kernel_version || '—';

      document.getElementById('art-outcome').innerHTML =
        `<span class="pill ${outcome === 'routed' ? 'pill-ok' : 'pill-warn'}">${esc(outcome)}</span>`;
      document.getElementById('art-selected').textContent = selected;
      document.getElementById('art-hash').textContent     = rhash;
      document.getElementById('art-kver').textContent     = kver;
      document.getElementById('route-receipt-json').textContent = fmt(rc);
      collapseIfLarge('route-receipt-json', 'receipt-expand-btn');
      const copyHashBtn = document.getElementById('art-hash-copy');
      if (copyHashBtn && rhash && rhash !== '—') copyHashBtn.classList.remove('hidden');
      show('receipt-json-actions');
      show('verify-artifact-note');

      show('route-result');
      document.getElementById('btn-verify').disabled          = false;
      document.getElementById('btn-tamper').disabled          = false;
      document.getElementById('btn-dispatch-create').disabled = false;
      updateOpState('available', 'available', 'not-run', 'available');
      salLog('Route result received', 'Route receipt generated.');
      appendRunHistory('Route executed', true);
      ni.textContent = '✓ Routing complete — receipt ' + rhash.slice(0, 12) + '…';
      ni.className = 'success-note';
      const prev = document.getElementById('route-norm-preview');
      const hashRow = rhash !== '—'
        ? '<div class="norm-preview-row">'
            + '<span class="norm-preview-key">Receipt Hash</span>'
            + '<span class="norm-preview-val">' + esc(rhash)
            + ' <button class="copy-btn" onclick="copyReceiptHash(this,'
            + JSON.stringify(rhash) + ')">Copy</button></span></div>'
        : previewRow('Receipt Hash', rhash);
      const mfrRow = rc.selected_candidate_id
        ? '<div class="norm-preview-row">'
            + '<span class="norm-preview-key">Manufacturer</span>'
            + '<span class="norm-preview-val">' + esc(selected)
            + ' <button class="copy-btn" onclick="copyReceiptHash(this,'
            + JSON.stringify(rc.selected_candidate_id) + ')">Copy</button></span></div>'
        : previewRow('Manufacturer', selected);
      prev.innerHTML =
        '<div class="norm-step" style="border-top:none;padding-top:0;margin-top:.55rem">'
          + '<span class="norm-step-num done">3</span>'
          + '<span class="norm-step-lbl">Inspect receipt summary</span>'
        + '</div>'
        + '<div class="norm-success-panel">'
          + '<div class="norm-success-title">&#x2713; Routing complete</div>'
          + '<div class="norm-preview">'
          + hashRow + mfrRow
          + previewRow('Jurisdiction', rc.routing_input?.jurisdiction || '—')
          + previewRow('Material',     rc.routing_input?.material     || '—')
          + previewRow('Created At',   rc.created_at                  || '—')
          + '</div>'
        + '</div>'
        + '<div class="norm-step">'
          + '<span class="norm-step-num done">4</span>'
          + '<span class="norm-step-lbl">Copy or download receipt</span>'
        + '</div>'
        + '<div class="norm-success-actions">'
          + '<button class="btn-dl" onclick="downloadReceiptJson()">↓ Download receipt.json</button>'
          + '<button class="btn-route-norm" style="font-size:.72rem"'
          + ' id="btn-toggle-receipt" onclick="toggleNormReceiptJson()">Show receipt JSON</button>'
        + '</div>'
        + '<pre class="fixture hidden" id="norm-receipt-json-block"'
        + ' style="margin-top:.3rem;max-height:300px"></pre>';
      document.getElementById('norm-receipt-json-block').textContent = fmt(rc);
      prev.classList.remove('hidden');
    } else {
      // non-2xx HTTP response → inline error + details panel
      const code = data?.error?.code || data?.result || 'error';
      const msg  = data?.error?.message || '';
      const hint = errorHint(code);
      ni.innerHTML = '<div class="norm-error-panel">'
        + '<div class="norm-error-code">' + esc(code) + '</div>'
        + '<div class="norm-error-hint">' + esc(msg || 'The routing request could not be processed.') + '</div>'
        + (hint ? '<div class="norm-error-hint" style="margin-top:.2rem">' + hint + '</div>' : '')
        + '</div>';
      ni.className = '';
      ni.classList.remove('hidden');
      const banner = document.getElementById('route-error-banner');
      banner.innerHTML = '<strong>[' + esc(code) + ']</strong>'
        + (msg ? ' ' + esc(msg) : '')
        + (hint ? '<br><span style="font-weight:400;font-size:.7rem;color:#8b949e;display:block;margin-top:.2rem">' + hint + '</span>' : '');
      banner.classList.remove('hidden');
      document.getElementById('route-error-json').textContent = fmt(data);
      show('route-error');
      show('receipt-empty-state');
      updateOpState('failed', 'missing', null, null);
      salLog('Route result received', 'Route execution returned an error.');
      appendRunHistory('Route executed', false);
    }
  } catch(e) {
    ni.innerHTML = '<div class="norm-error-panel">'
      + '<div class="norm-error-code">Unexpected error</div>'
      + '<div class="norm-error-hint">' + esc(String(e)) + '</div>'
      + '<div class="norm-error-hint" style="margin-top:.2rem">Try again, or check the browser console for details.</div>'
      + '</div>';
    ni.className = '';
    ni.classList.remove('hidden');
    hide('route-error-banner');
    document.getElementById('route-error-json').textContent = String(e);
    show('route-error');
    show('receipt-empty-state');
    updateOpState('failed', 'missing', null, null);
    salLog('Route result received', 'Route execution returned an error.');
    appendRunHistory('Route executed', false);
  } finally {
    hide('results-loading');
    setBtn(btn, '▶ Submit for Review', false);
    document.getElementById('btn-route').disabled = false;
  }
}

// ── Replay Verification ────────────────────────────────────────────────────
async function verifyReceipt(btn) {
  if (!lastReceipt || !lastPolicy) {
    showVerifyResult(false, {error: {code: 'no_review_context', message: 'No receipt available. Submit a case for review first, then use Replay Verification.'}}, 'No review context');
    return;
  }
  setBtn(btn, 'Replaying…', true);
  hide('verify-result');
  salLog('Verification executed', 'Replay verification started against current receipt.');

  try {
    const r = await fetch('/verify', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({
        receipt: lastReceipt,
        case:    fixtures.case,
        policy:  lastPolicy,
      }),
    });
    const data = await r.json();
    showVerifyResult(r.ok && data.result === 'VERIFIED', data, 'Replay Verification');
  } catch(e) {
    showVerifyResult(false, {error: {code: 'client_error', message: String(e)}}, 'Replay Verification');
  } finally {
    setBtn(btn, '↩ Replay Verification', false);
  }
}

// ── E. Tamper + Verify ─────────────────────────────────────────────────────
async function tamperVerify(btn) {
  if (!lastReceipt || !lastPolicy) {
    showVerifyResult(false, {error: {code: 'no_review_context', message: 'No receipt available. Submit a case for review before attempting the tamper demo.'}}, 'No review context');
    return;
  }
  setBtn(btn, 'Tampering…', true);
  hide('verify-result');

  // Deep-copy receipt and change selected_candidate_id client-side only.
  // No backend changes. The real POST /verify will catch the mismatch.
  const tampered = JSON.parse(JSON.stringify(lastReceipt));
  const original = tampered.selected_candidate_id || '(none)';
  tampered.selected_candidate_id = 'tampered-mfr-reviewer-demo';

  try {
    const r = await fetch('/verify', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({
        receipt: tampered,
        case:    fixtures.case,
        policy:  lastPolicy,
      }),
    });
    const data = await r.json();
    // Annotate response with what was changed so the reviewer sees it clearly
    const annotated = {
      _tamper_note: `selected_candidate_id changed client-side: "${original}" → "tampered-mfr-reviewer-demo"`,
      _submitted_to: 'POST /verify (real endpoint, no backend changes)',
      ...data,
    };
    showVerifyResult(false, annotated, 'Tamper Demo');
  } catch(e) {
    showVerifyResult(false, {error: {code: 'client_error', message: String(e)}}, 'Tamper Demo');
  } finally {
    setBtn(btn, '⚠ Tamper + Verify', false);
  }
}

// ── F. Verification result display ─────────────────────────────────────────
function showVerifyResult(isVerified, data, kind) {
  document.getElementById('verify-kind-label').innerHTML =
    `<span class="pill ${isVerified ? 'pill-ok' : 'pill-err'}">${esc(kind)}</span>`;

  const banner = document.getElementById('verify-banner');
  if (isVerified) {
    banner.className = 'verify-banner banner-ok';
    banner.innerHTML = '✓ VERIFIED — receipt replay matched'
      + '<span class="verify-sub">The kernel reproduced the same receipt hash from the original inputs.</span>';
  } else {
    const code    = data?.error?.code || data?.result || 'FAILED';
    const msg     = data?.error?.message || '';
    const heading = kind === 'Tamper Demo' ? '✗ TAMPER DETECTED' : '✗ VERIFICATION FAILED';
    banner.className = 'verify-banner banner-err';
    banner.innerHTML = heading
      + `<span class="verify-sub">Error code: <strong>${esc(code)}</strong>${msg ? ' — ' + esc(msg) : ''}</span>`;
  }

  if (kind === 'Replay Verification') {
    verifySerial = runSerial;
    updateOpState(null, null, isVerified ? 'verified' : 'failed', null);
    hide('verify-artifact-note');
    appendRunHistory('Verification executed', isVerified);
    salLog('Verification completed', isVerified ? 'Receipt replay matched.' : 'Verification failed.');
  }
  const pre = document.getElementById('verify-json');
  pre.className = 'result ' + (isVerified ? 'result-ok' : 'result-err');
  pre.textContent = fmt(data);
  collapseIfLarge('verify-json', 'verify-expand-btn');
  show('verify-result');
  show('verify-json-actions');
  document.getElementById('verify-result').scrollIntoView({behavior:'smooth', block:'nearest'});
}

// ── G. Dispatch Commitment ─────────────────────────────────────────────────
async function createDispatch(btn) {
  if (!lastReceipt || !lastPolicy) {
    showDispatchMsg('error', 'Cannot create dispatch — no valid receipt. A routed case with a receipt is required. Submit a case for review first.');
    return;
  }
  setBtn(btn, 'Creating…', true);
  // Only clear the error/success displays — preserve any existing dispatch panel
  // so that a 409 (already created) doesn't wipe the visible dispatch_id.
  hide('dispatch-success'); hide('dispatch-error');

  try {
    const r = await fetch('/dispatch/create', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({receipt: lastReceipt, case: fixtures.case, policy: lastPolicy}),
    });
    const data = await r.json();

    if (r.ok && data.dispatch_id) {
      // Fresh creation — update full dispatch panel.
      lastDispatchId = data.dispatch_id;
      document.getElementById('art-dispatch-id').textContent = data.dispatch_id;
      document.getElementById('art-dispatch-status').innerHTML =
        `<span class="pill pill-info">${esc(data.status)}</span>`;
      const _dic2 = document.getElementById('art-dispatch-id-copy');
      if (_dic2) _dic2.classList.remove('hidden');
      hide('dispatch-export-result');
      show('dispatch-created');
      document.getElementById('btn-dispatch-approve').disabled = false;
      document.getElementById('btn-dispatch-export').disabled  = true;
    } else if (r.status === 409) {
      // Already created for this receipt — show as a warning, not an error.
      // Keep any existing dispatch panel visible so the operator can continue.
      showDispatchMsg('warn',
        '[' + (data?.error?.code || 'receipt_already_dispatched') + '] ' +
        (data?.error?.message || 'Dispatch already exists for this receipt.'));
    } else {
      showDispatchMsg('error',
        '[' + (data?.error?.code || 'error') + '] ' + (data?.error?.message || JSON.stringify(data)));
    }
  } catch(e) {
    showDispatchMsg('error', String(e));
  } finally {
    setBtn(btn, '⬦ Create Dispatch', false);
  }
}

async function approveDispatch(btn) {
  if (!lastDispatchId) {
    showDispatchMsg('error', 'Cannot approve — no dispatch record found. Create a dispatch commitment first.');
    return;
  }
  setBtn(btn, 'Approving…', true);
  hide('dispatch-success'); hide('dispatch-error');
  // Track whether this button should stay disabled after the call.
  // true = terminal state (success or already-approved 409).
  let terminal = false;

  try {
    const r = await fetch('/dispatch/' + lastDispatchId + '/approve', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({approved_by: 'reviewer'}),
    });
    const data = await r.json();

    if (r.ok && data.status === 'approved') {
      document.getElementById('art-dispatch-status').innerHTML =
        `<span class="pill pill-ok">${esc(data.status)}</span>`;
      terminal = true;   // immutable — disable approve permanently
      document.getElementById('btn-dispatch-export').disabled = false;
      const s = document.getElementById('dispatch-success');
      s.textContent = 'Dispatch approved.';
      s.classList.remove('hidden');
    } else if (r.status === 409) {
      // Already approved server-side — enable export so the operator can continue.
      terminal = true;
      document.getElementById('btn-dispatch-export').disabled = false;
      showDispatchMsg('warn',
        '[' + (data?.error?.code || 'dispatch_not_draft') + '] ' +
        (data?.error?.message || 'Dispatch is already approved.') +
        ' Export is now available.');
    } else {
      showDispatchMsg('error',
        '[' + (data?.error?.code || 'error') + '] ' + (data?.error?.message || JSON.stringify(data)));
    }
  } catch(e) {
    showDispatchMsg('error', String(e));
  } finally {
    // Re-enable only if not in a terminal state.
    setBtn(btn, '✓ Approve Dispatch', terminal);
  }
}

async function exportDispatch(btn) {
  if (!lastDispatchId) {
    showDispatchMsg('error', 'Cannot export — no dispatch record found. Create and approve a dispatch commitment first.');
    return;
  }
  setBtn(btn, 'Exporting…', true);
  hide('dispatch-export-result'); hide('dispatch-success'); hide('dispatch-error');

  try {
    const r = await fetch('/dispatch/' + lastDispatchId + '/export');
    const data = await r.json();
    if (r.ok) {
      lastExportPacket = data;
      dispatchSerial = runSerial;
      updateIntegrityBadges();
      updateDispatchReadiness();
      updateDispatchBlockers();
      updateMicrobadges();
      updateFreshnessMarkers();
      updateRunTimeline();
      updateOab();
      updateOutcomeBanner();
      updateCompletionChecklist();
      updatePreflightCard();
      updateHandoffSummary();
      updateConsistencySentinel();
      updateActiveSectionEmphasis();
      updateRunIdentityBlock();
      updateLineageBadges();
      updateLineageNotes();
      updateDpi();
      updateDossier();
      const _dsn = document.getElementById('dispatch-stale-note');
      if (_dsn) _dsn.classList.add('hidden');
      updateActiveRunContext();
      updateNextActionRail();
      document.getElementById('art-dispatch-status').innerHTML =
        `<span class="pill pill-ok">${esc(data.status)}</span>`;
      document.getElementById('dispatch-export-json').textContent = fmt(data);
      collapseIfLarge('dispatch-export-json', 'dispatch-expand-btn');
      show('dispatch-export-result');
      show('dispatch-export-actions');
      appendRunHistory('Dispatch executed', true);
      salLog('Dispatch export generated', 'Dispatch packet exported for current run.');
      const s = document.getElementById('dispatch-success');
      s.textContent = 'Export complete — dispatch packet ready for handoff.';
      s.classList.remove('hidden');
    } else {
      showDispatchMsg('error',
        '[' + (data?.error?.code || 'error') + '] ' + (data?.error?.message || JSON.stringify(data)));
    }
  } catch(e) {
    showDispatchMsg('error', String(e));
  } finally {
    setBtn(btn, '↓ Export Dispatch Packet', false);
  }
}

// ── Artifact export / dispatch-ID copy ────────────────────────────────────
function downloadExportPacket() {
  if (!lastExportPacket) return;
  const id = lastDispatchId ? lastDispatchId.slice(0, 8) : 'dispatch';
  const blob = new Blob([fmt(lastExportPacket)], {type: 'application/json'});
  const url  = URL.createObjectURL(blob);
  const a    = document.createElement('a');
  a.href = url; a.download = 'export_packet_' + id + '.json'; a.click();
  URL.revokeObjectURL(url);
}
function copyExportJson(btn) {
  if (!lastExportPacket) return;
  navigator.clipboard.writeText(fmt(lastExportPacket)).then(() => {
    btn.textContent = 'Copied'; btn.style.color = '#3fb950';
  }).catch(() => {
    btn.textContent = 'Failed'; btn.style.color = '#f85149';
  });
  setTimeout(() => { btn.textContent = 'Copy JSON'; btn.style.color = ''; }, 1500);
}
function copyDispatchId(btn) {
  const id = document.getElementById('art-dispatch-id').textContent.trim();
  if (!id) return;
  navigator.clipboard.writeText(id).then(() => {
    btn.textContent = 'Copied'; btn.style.color = '#3fb950';
  }).catch(() => {
    btn.textContent = 'Failed'; btn.style.color = '#f85149';
  });
  setTimeout(() => { btn.textContent = 'Copy'; btn.style.color = ''; }, 1500);
}

// ── Artifact receipt-hash copy ─────────────────────────────────────────────
function copyArtHashVal(btn) {
  const hash = document.getElementById('art-hash').textContent.trim();
  if (!hash || hash === '—') return;
  navigator.clipboard.writeText(hash).then(() => {
    btn.textContent = 'Copied'; btn.style.color = '#3fb950';
  }).catch(() => {
    btn.textContent = 'Failed'; btn.style.color = '#f85149';
  });
  setTimeout(() => { btn.textContent = 'Copy'; btn.style.color = ''; }, 1500);
}

// ── Integrity badges ───────────────────────────────────────────────────────
function setBadge(id, state) {
  const el = document.getElementById(id);
  if (!el) return;
  if (!state) { el.classList.add('hidden'); return; }
  el.classList.remove('hidden', 'ib-unverified', 'ib-verified', 'ib-failed');
  if      (state === 'verified')   { el.textContent = 'VERIFIED';   el.classList.add('ib-verified'); }
  else if (state === 'failed')     { el.textContent = 'FAILED';     el.classList.add('ib-failed'); }
  else                             { el.textContent = 'UNVERIFIED'; el.classList.add('ib-unverified'); }
}
function updateIntegrityBadges() {
  // Receipt panels: hidden before routing; reflects verify state once routed
  const receiptState = opRouting !== 'available' ? null
    : opVerify === 'verified' ? 'verified'
    : opVerify === 'failed'   ? 'failed'
    : 'unverified';
  setBadge('route-result-badge', receiptState);
  setBadge('receipt-json-badge', receiptState);
  // Verification result: shown only after verify has run
  const verifyState = opVerify === 'verified' ? 'verified'
    : opVerify === 'failed' ? 'failed' : null;
  setBadge('verify-result-badge', verifyState);
  // Dispatch export: verified — server re-verifies before creating the record
  setBadge('dispatch-result-badge', lastExportPacket ? 'verified' : null);
}

// ── Artifact panel copy ────────────────────────────────────────────────────
function copyReceiptJson(btn) {
  if (!lastReceipt) return;
  navigator.clipboard.writeText(fmt(lastReceipt)).then(() => {
    btn.textContent = 'Copied'; btn.style.color = '#3fb950';
  }).catch(() => {
    btn.textContent = 'Failed'; btn.style.color = '#f85149';
  });
  setTimeout(() => { btn.textContent = 'Copy artifact'; btn.style.color = ''; }, 1500);
}
function copyVerifyJson(btn) {
  const pre = document.getElementById('verify-json');
  if (!pre) return;
  navigator.clipboard.writeText(pre.textContent).then(() => {
    btn.textContent = 'Copied'; btn.style.color = '#3fb950';
  }).catch(() => {
    btn.textContent = 'Failed'; btn.style.color = '#f85149';
  });
  setTimeout(() => { btn.textContent = 'Copy artifact'; btn.style.color = ''; }, 1500);
}
function copyRouteErrorJson(btn) {
  const pre = document.getElementById('route-error-json');
  if (!pre) return;
  navigator.clipboard.writeText(pre.textContent).then(() => {
    btn.textContent = 'Copied'; btn.style.color = '#3fb950';
  }).catch(() => {
    btn.textContent = 'Failed'; btn.style.color = '#f85149';
  });
  setTimeout(() => { btn.textContent = 'Copy artifact'; btn.style.color = ''; }, 1500);
}

// ── Dispatch readiness panel ───────────────────────────────────────────────
const DR_LABELS = {
  'cl-receipt':  'Receipt reviewed',
  'cl-verify':   'Verification succeeded',
  'cl-dispatch': 'Dispatch action confirmed',
};
function setCheck(id, ok) {
  const el = document.getElementById(id);
  if (!el) return;
  el.className = 'cl-item ' + (ok ? 'cl-ok' : 'cl-pending');
  el.textContent = (ok ? '✓ ' : '◻ ') + (DR_LABELS[id] || '');
}
function updateDispatchReadiness() {
  const status = document.getElementById('dr-status');
  const reason = document.getElementById('dr-reason');
  if (!status) return;
  const receiptOk = opRouting === 'available';
  const verifyOk  = opVerify  === 'verified';
  const completed = !!lastExportPacket;
  setCheck('cl-receipt',  receiptOk);
  setCheck('cl-verify',   verifyOk);
  setCheck('cl-dispatch', completed);
  if (completed) {
    status.textContent = 'Dispatch completed';
    status.className   = 'dr-completed';
    reason.textContent = 'Export packet produced. Current run is complete.';
  } else if (verifyOk) {
    status.textContent = 'Ready for dispatch';
    status.className   = 'dr-ready';
    reason.textContent = 'Verification succeeded. Create and approve the dispatch commitment.';
  } else if (opVerify === 'failed') {
    status.textContent = 'Not ready for dispatch';
    status.className   = 'dr-not-ready';
    reason.textContent = 'Verification failed. Resolve before dispatching.';
  } else if (receiptOk) {
    status.textContent = 'Not ready for dispatch';
    status.className   = 'dr-not-ready';
    reason.textContent = 'Verification pending. Run verify before dispatch.';
  } else {
    status.textContent = 'Not ready for dispatch';
    status.className   = 'dr-not-ready';
    reason.textContent = 'Required artifact not yet generated.';
  }
}

// ── Dispatch blocker list ─────────────────────────────────────────────────
function dispatchBlockers() {
  const items = [];
  if (opRouting !== 'available') {
    items.push({text:'No current route result — run routing first.',
                anchor:{label:'Go to routing', target:'norm-input-section'}});
  } else {
    if (opVerify === 'not-run') {
      items.push({text:'Verification not yet executed for current run.',
                  anchor:{label:'Go to verification', target:'btn-verify'}});
    }
    if (opVerify === 'failed') {
      items.push({text:'Verification result does not satisfy dispatch readiness.',
                  anchor:{label:'Review readiness', target:'dispatch-readiness-panel'}});
    }
  }
  return items;
}
function updateDispatchBlockers() {
  const body = document.getElementById('dbl-body');
  if (!body) return;
  if (lastExportPacket) {
    body.innerHTML = '<div class="dbl-done">Dispatch already exported for current run.</div>';
    return;
  }
  const items = dispatchBlockers();
  if (items.length === 0) {
    body.innerHTML = '<div class="dbl-clear">No current blockers — dispatch export is available.</div>';
    return;
  }
  body.innerHTML = items.map(b =>
    '<div class="dbl-item dbl-item-blocked">'
    + '<span class="dbl-item-bullet">▸</span>'
    + '<span>' + esc(b.text)
    + (b.anchor
        ? ' <button class="dbl-anchor" onclick="dblNavigate('
          + JSON.stringify(b.anchor.target) + ')">'
          + esc(b.anchor.label) + '</button>'
        : '')
    + '</span></div>'
  ).join('');
}
function dblNavigate(target) {
  const el = document.getElementById(target);
  if (!el) return;
  el.scrollIntoView({behavior:'smooth', block:'nearest'});
  if (typeof el.focus === 'function') el.focus({preventScroll:true});
}

// ── Operator state block ───────────────────────────────────────────────────
// ── Dispatch packet inspection ────────────────────────────────────────────
function updateDpi() {
  const meta      = document.getElementById('dpi-meta');
  const empty     = document.getElementById('dpi-empty');
  const viewer    = document.getElementById('dpi-viewer');
  const origin    = document.getElementById('dpi-origin');
  const integrity = document.getElementById('dpi-integrity');
  if (!meta) return;
  if (!lastExportPacket) {
    meta.classList.add('hidden');
    empty.classList.remove('hidden');
    viewer.classList.add('hidden');
    viewer.textContent = '';
    return;
  }
  empty.classList.add('hidden');
  meta.classList.remove('hidden');
  viewer.classList.remove('hidden');
  viewer.textContent = fmt(lastExportPacket);
  const dlin = dispatchLineage();
  if (dlin === 'current')     { origin.className = 'dpi-origin-current'; origin.textContent = 'current run'; }
  else if (dlin === 'prev')   { origin.className = 'dpi-origin-prev';    origin.textContent = 'previous run'; }
  else                        { origin.className = 'dpi-origin-none';    origin.textContent = '—'; }
  const vlin = verifyLineage();
  if (vlin === 'current' && opVerify === 'verified') {
    integrity.className = 'dpi-integrity-ok';   integrity.textContent = 'verified packet';
  } else if (vlin === 'current' && opVerify === 'failed') {
    integrity.className = 'dpi-integrity-fail'; integrity.textContent = 'verification failed';
  } else {
    integrity.className = 'dpi-integrity-none'; integrity.textContent = 'verification not executed';
  }
}

// ── Dispatch handoff dossier ───────────────────────────────────────────────
function dhdVerdictKey() {
  if (runSerial === 0)               return 'none';
  const dlin = dispatchLineage();
  if (dlin === 'current')            return 'exported';
  if (dlin === 'prev')               return 'attention';
  const vlin = verifyLineage();
  if (vlin === 'current' && opVerify === 'verified') return 'ready';
  return 'not-ready';
}
const DHD_VERDICTS = {
  'none':      {cls:'dhd-verdict-none',      text:'No current dispatch packet'},
  'not-ready': {cls:'dhd-verdict-not-ready', text:'Current route not ready for dispatch'},
  'ready':     {cls:'dhd-verdict-ready',     text:'Current route ready for dispatch export'},
  'exported':  {cls:'dhd-verdict-exported',  text:'Current dispatch packet exported'},
  'attention': {cls:'dhd-verdict-attention', text:'Current dispatch packet requires attention'},
};
const DHD_MEANINGS = {
  'none':      'No route has been generated yet for the current session.',
  'not-ready': 'A route exists for the current run but prerequisites are not complete. Verification must pass before dispatch can be exported.',
  'ready':     'The current route is verified. Exporting the dispatch packet creates the handoff record bound to this route and run. Rerouting will require a new export.',
  'exported':  'The dispatch export packet exists for the current route. This run is complete. Rerouting will invalidate this export — a new export will be required for the new route.',
  'attention': 'A dispatch export exists but it belongs to a previous route. The current run has been rerouted. A new export is required to represent the active route.',
};
function dhdNextStep(key) {
  if (key === 'none')      return 'Generate a route first.';
  if (key === 'exported')  return 'Dispatch packet already exported for the current route.';
  if (key === 'attention') return 'Reroute detected — re-export required for the current route.';
  if (key === 'ready')     return 'Export the dispatch packet for the current route.';
  if (opRouting !== 'available') return 'Run routing to begin the current run.';
  if (verifyLineage() === 'idle') return 'Run verification for the current route.';
  if (opVerify === 'failed')      return 'Resolve failed verification before handoff.';
  return 'Run verification for the current route.';
}
function dhdRow(icon, cls, label) {
  return '<div class="dhd-row"><span class="' + cls + '">' + icon + '</span><span>' + esc(label) + '</span></div>';
}
function updateDossier() {
  const vkey = dhdVerdictKey();
  const conf = DHD_VERDICTS[vkey];
  const verdict   = document.getElementById('dhd-verdict');
  const meaning   = document.getElementById('dhd-meaning');
  const checklist = document.getElementById('dhd-checklist');
  const nextText  = document.getElementById('dhd-next-text');
  if (!verdict) return;
  verdict.className   = 'dhd-verdict ' + conf.cls;
  verdict.textContent = conf.text;
  if (meaning)  meaning.textContent  = DHD_MEANINGS[vkey];
  if (nextText) nextText.textContent = dhdNextStep(vkey);
  if (checklist) {
    const vlin = verifyLineage();
    const dlin = dispatchLineage();
    const routeOk    = opRouting === 'available';
    const receiptOk  = opReceipt === 'available';
    const verifyExec = vlin !== 'idle';
    const verifyPass = vlin === 'current' && opVerify === 'verified';
    const dispatchOk = dlin === 'current';
    let h = '';
    h += dhdRow(routeOk   ? '✓' : '◻', routeOk   ? 'dhd-ok' : 'dhd-no', 'Route available');
    h += dhdRow(receiptOk ? '✓' : '◻', receiptOk ? 'dhd-ok' : 'dhd-no', 'Receipt available');
    if (vlin === 'prev') {
      h += dhdRow('⚠', 'dhd-warn', 'Verification executed — previous run only');
    } else {
      h += dhdRow(verifyExec ? '✓' : '◻', verifyExec ? 'dhd-ok' : 'dhd-no',
                  'Verification executed for current run');
    }
    if (vlin === 'current' && opVerify === 'failed') {
      h += dhdRow('⚠', 'dhd-warn', 'Verification failed — not passed');
    } else {
      h += dhdRow(verifyPass ? '✓' : '◻', verifyPass ? 'dhd-ok' : 'dhd-no', 'Verification passed');
    }
    if (dlin === 'prev') {
      h += dhdRow('⚠', 'dhd-warn', 'Dispatch exported — previous run only');
    } else {
      h += dhdRow(dispatchOk ? '✓' : '◻', dispatchOk ? 'dhd-ok' : 'dhd-no',
                  'Dispatch packet exported for current run');
    }
    checklist.innerHTML = h;
  }
}

// ── Run identity + artifact lineage ───────────────────────────────────────
function verifyLineage() {
  if (runSerial === 0)                              return 'idle';
  if (verifySerial === runSerial)                   return 'current';
  if (verifySerial > 0 && verifySerial < runSerial) return 'prev';
  return 'idle';
}
function dispatchLineage() {
  if (runSerial === 0)                                    return 'idle';
  if (dispatchSerial === runSerial)                       return 'current';
  if (dispatchSerial > 0 && dispatchSerial < runSerial)   return 'prev';
  return 'idle';
}
function setRibVal(id, state, text) {
  const el = document.getElementById(id);
  if (!el) return;
  el.className = 'rib-val-' + state;
  el.textContent = text;
}
function updateRunIdentityBlock() {
  if (runSerial === 0) {
    setRibVal('rib-route',    'idle', 'no run yet');
    setRibVal('rib-receipt',  'idle', 'not generated');
    setRibVal('rib-verify',   'idle', 'not executed');
    setRibVal('rib-dispatch', 'idle', 'not exported');
    return;
  }
  if (opRouting === 'available') setRibVal('rib-route', 'current', 'current run');
  else if (opRouting === 'failed') setRibVal('rib-route', 'err', 'failed');
  else setRibVal('rib-route', 'idle', 'no run');
  setRibVal('rib-receipt', opReceipt === 'available' ? 'current' : 'idle',
                           opReceipt === 'available' ? 'current run' : 'not generated');
  const vlin = verifyLineage();
  setRibVal('rib-verify',
    vlin === 'current' ? 'current' : (vlin === 'prev' ? 'prev' : 'idle'),
    vlin === 'current' ? 'current run' : (vlin === 'prev' ? 'previous run' : 'not executed'));
  const dlin = dispatchLineage();
  setRibVal('rib-dispatch',
    dlin === 'current' ? 'current' : (dlin === 'prev' ? 'prev' : 'idle'),
    dlin === 'current' ? 'current run' : (dlin === 'prev' ? 'previous run' : 'not exported'));
}
function setLinBadge(id, lineage, idleLabel) {
  const el = document.getElementById(id);
  if (!el) return;
  if (lineage === 'current')      { el.className = 'lin lin-current'; el.textContent = 'current run'; }
  else if (lineage === 'prev')    { el.className = 'lin lin-prev';    el.textContent = 'previous run'; }
  else { el.className = 'lin lin-idle'; el.textContent = idleLabel || 'not executed'; }
}
function updateLineageBadges() {
  setLinBadge('lin-verify',          verifyLineage(),   'not executed');
  setLinBadge('lin-dispatch-export', dispatchLineage(), 'not exported');
}
function updateLineageNotes() {
  const vn = document.getElementById('lin-verify-note');
  const dn = document.getElementById('lin-dispatch-note');
  if (vn) vn.classList.toggle('hidden', verifyLineage()   !== 'prev');
  if (dn) dn.classList.toggle('hidden', dispatchLineage() !== 'prev');
}

function updateOpState(routing, receipt, verify, dispatch) {
  const MAP = {
    'not-run': 'op-not-run', 'available': 'op-available',
    'verified': 'op-verified', 'failed': 'op-failed', 'missing': 'op-missing',
  };
  if (routing  != null) opRouting  = routing;
  if (receipt  != null) opReceipt  = receipt;
  if (verify   != null) opVerify   = verify;
  if (dispatch != null) opDispatch = dispatch;
  [['ops-routing', opRouting], ['ops-receipt', opReceipt],
   ['ops-verify',  opVerify],  ['ops-dispatch', opDispatch]].forEach(([id, st]) => {
    const el = document.getElementById(id);
    if (!el) return;
    el.textContent = st;
    el.className = MAP[st] || 'op-not-run';
  });
  // guidance notes in dispatch section
  const vpn = document.getElementById('verify-pending-note');
  if (vpn) vpn.classList.toggle('hidden',
    !(opRouting === 'available' && opVerify === 'not-run'));
  const dbn = document.getElementById('dispatch-blocked-note');
  if (dbn) dbn.classList.toggle('hidden', opVerify !== 'failed');
  // dispatch stale note: visible when routed but no export yet
  const dsn = document.getElementById('dispatch-stale-note');
  if (dsn) dsn.classList.toggle('hidden', !(opRouting === 'available' && !lastExportPacket));
  updateIntegrityBadges();
  updateDispatchReadiness();
  updateDispatchBlockers();
  updateActiveRunContext();
  updateNextActionRail();
  updateHandoffNote();
  updateMicrobadges();
  updateFreshnessMarkers();
  updateRunTimeline();
  updateOab();
  updateOutcomeBanner();
  updateCompletionChecklist();
  updatePreflightCard();
  updateHandoffSummary();
  updateConsistencySentinel();
  updateActiveSectionEmphasis();
  updateRunIdentityBlock();
  updateLineageBadges();
  updateLineageNotes();
  updateDpi();
  updateDossier();
}

// ── Active run context ────────────────────────────────────────────────────
function updateActiveRunContext() {
  const block = document.getElementById('active-run-context');
  if (!block) return;
  if (opRouting !== 'available' || !lastReceipt) {
    block.classList.add('hidden');
    return;
  }
  block.classList.remove('hidden');
  document.getElementById('arc-manufacturer').textContent =
    lastReceipt.selected_candidate_id || '(none — refused)';
  const hash = lastReceipt.receipt_hash || '—';
  document.getElementById('arc-receipt-hash').textContent =
    hash !== '—' ? hash.slice(0, 16) + '…' : '—';
  const verEl = document.getElementById('arc-verify-status');
  if      (opVerify === 'verified') { verEl.textContent = 'Verified'; verEl.className = 'arc-val-ok'; }
  else if (opVerify === 'failed')   { verEl.textContent = 'Failed';   verEl.className = 'arc-val-err'; }
  else { verEl.textContent = 'No verification result for current route'; verEl.className = 'arc-val-pending'; }
  const dispEl = document.getElementById('arc-dispatch-status');
  if (lastExportPacket) { dispEl.textContent = 'Exported'; dispEl.className = 'arc-val-ok'; }
  else { dispEl.textContent = 'No dispatch export for current route'; dispEl.className = 'arc-val-pending'; }
}

// ── Next-action rail ──────────────────────────────────────────────────────
function updateNextActionRail() {
  const actionEl = document.getElementById('nar-action');
  const reasonEl = document.getElementById('nar-reason');
  if (!actionEl || !reasonEl) return;
  let action, reason, cls;
  if (lastExportPacket) {
    action = 'Workflow complete';
    reason = 'Dispatch packet exported for current route.';
    cls    = 'nar-action-done';
  } else if (opVerify === 'verified') {
    action = 'Next: export dispatch';
    reason = 'Verification complete. Dispatch not yet exported.';
    cls    = 'nar-action-next';
  } else if (opRouting === 'available') {
    action = 'Next: verify current route';
    reason = 'Receipt exists but verification not yet executed.';
    cls    = 'nar-action-next';
  } else {
    action = 'Next: run route';
    reason = 'No current receipt loaded.';
    cls    = 'nar-action-idle';
  }
  actionEl.textContent = action;
  actionEl.className   = 'nar-action ' + cls;
  reasonEl.textContent = reason;
}

// ── Operator handoff note ─────────────────────────────────────────────────
function updateHandoffNote() {
  const body  = document.getElementById('hn-body');
  const block = document.getElementById('handoff-note');
  if (!body || !block) return;
  if (!lastExportPacket) {
    block.classList.remove('handoff-note-active');
    body.innerHTML = '<span style="color:#484f58;font-style:italic">No export for current route. Handoff not yet applicable.</span>';
    return;
  }
  block.classList.add('handoff-note-active');
  const did = lastDispatchId ? lastDispatchId.slice(0, 8) + '…' : '—';
  body.innerHTML =
    '<div class="hn-row hn-row-check">&#x2713; Route processed — manufacturer selected</div>'
    + '<div class="hn-row hn-row-check">&#x2713; Verification completed</div>'
    + '<div class="hn-row hn-row-check">&#x2713; Dispatch artifact exported</div>'
    + '<div class="hn-object">Handoff object: export packet &middot; dispatch ID <span style="color:#c9d1d9">'
    + esc(did) + '</span> &middot; copy or transfer using the panel above</div>';
}

// ── Panel microbadges ─────────────────────────────────────────────────────
const MB_LABELS  = {'available':'available','not-available':'not available',
                    'verified':'verified','exported':'exported','failed':'failed'};
const MB_CLASSES = {'available':'mb mb-on','not-available':'mb mb-dim',
                    'verified':'mb mb-on','exported':'mb mb-on','failed':'mb mb-err'};
function setMicrobadge(id, state) {
  const el = document.getElementById(id);
  if (!el) return;
  el.className   = MB_CLASSES[state] || 'mb mb-dim';
  el.textContent = MB_LABELS[state]  || state;
}
function updateMicrobadges() {
  setMicrobadge('mb-receipt',
    opRouting === 'available' ? 'available' : 'not-available');
  setMicrobadge('mb-verify',
    opVerify === 'verified' ? 'verified' :
    opVerify === 'failed'   ? 'failed'   : 'not-available');
  setMicrobadge('mb-dispatch', lastExportPacket ? 'exported' : 'not-available');
}

// ── Artifact freshness markers ────────────────────────────────────────────
function setFreshness(id, fresh, label) {
  const el = document.getElementById(id);
  if (!el) return;
  el.className   = 'fm ' + (fresh ? 'fm-fresh' : 'fm-pending');
  el.textContent = label;
}
function updateFreshnessMarkers() {
  setFreshness('fm-receipt',
    opRouting === 'available',
    opRouting === 'available'
      ? 'current run artifact'
      : 'not yet produced for current run');
  const verifyDone = opVerify === 'verified' || opVerify === 'failed';
  setFreshness('fm-verify',
    verifyDone,
    verifyDone ? 'current run artifact' : 'not yet executed for current run');
  setFreshness('fm-dispatch',
    !!lastExportPacket,
    lastExportPacket ? 'current run artifact' : 'not yet exported for current run');
}

// ── Run timeline ──────────────────────────────────────────────────────────
function timelineStepState(step) {
  if (step === 'route')   return opRouting === 'available' ? 'rt-done' : 'rt-idle';
  if (step === 'receipt') return opReceipt === 'available' ? 'rt-done' : 'rt-idle';
  if (step === 'verify') {
    if (opVerify === 'verified') return 'rt-done';
    if (opRouting === 'available') return 'rt-ready';
    return 'rt-idle';
  }
  if (step === 'dispatch') {
    if (lastExportPacket)       return 'rt-done';
    if (opVerify === 'failed')  return 'rt-blocked';
    if (opVerify === 'verified') return 'rt-ready';
    return 'rt-idle';
  }
  return 'rt-idle';
}
function timelineSummary() {
  if (lastExportPacket)            return 'Dispatch exported for current run';
  if (opVerify === 'verified')     return 'Verification completed — dispatch ready';
  if (opVerify === 'failed')       return 'Verification failed — review inputs before dispatch';
  if (opRouting === 'available')   return 'Route produced — verification pending';
  return 'Current run not started';
}
function updateRunTimeline() {
  ['route', 'receipt', 'verify', 'dispatch'].forEach(step => {
    const el = document.getElementById('rt-' + step);
    if (!el) return;
    el.className = 'rt-step ' + timelineStepState(step);
  });
  const sumEl = document.getElementById('rt-summary');
  if (sumEl) sumEl.textContent = timelineSummary();
}

// ── Operator action bar ───────────────────────────────────────────────────
const OAB_STATES = {
  route:    {action:'Start a route for the current case',
             reason:'No current route artifacts exist yet.',
             btnLabel:'→ Go to route',     target:'norm-input-section', cls:'oab-action-idle'},
  verify:   {action:'Run verification for the current route',
             reason:'Verification has not been executed for the current route.',
             btnLabel:'→ Go to verify',    target:'btn-verify',          cls:'oab-action-active'},
  export:   {action:'Export dispatch packet',
             reason:'Dispatch is ready and no export exists for the current route.',
             btnLabel:'→ Go to dispatch',  target:'btn-dispatch-export', cls:'oab-action-active'},
  resolve:  {action:'Resolve readiness items before dispatch',
             reason:'Verification failed. Resolve before dispatching.',
             btnLabel:'→ View readiness',  target:'dispatch-readiness-panel', cls:'oab-action-active'},
  complete: {action:'Current run complete',
             reason:'Current run already has a dispatch export.',
             btnLabel:'✓ Done',            target:'dispatch-export-result',   cls:'oab-action-complete'},
};
function oabStateKey() {
  if (lastExportPacket)             return 'complete';
  if (opVerify === 'verified')      return 'export';
  if (opVerify === 'failed')        return 'resolve';
  if (opRouting === 'available')    return 'verify';
  return 'route';
}
function updateOab() {
  const s = OAB_STATES[oabStateKey()];
  const actionEl = document.getElementById('oab-action');
  const reasonEl = document.getElementById('oab-reason');
  const btnEl    = document.getElementById('oab-btn');
  if (!actionEl || !reasonEl || !btnEl) return;
  actionEl.textContent = s.action;
  actionEl.className   = 'oab-action ' + s.cls;
  reasonEl.textContent = s.reason;
  btnEl.textContent    = s.btnLabel;
  btnEl.className      = 'oab-btn' + (oabStateKey() === 'complete' ? ' oab-btn-complete' : '');
}
function oabNavigate() {
  const target = OAB_STATES[oabStateKey()]?.target;
  if (!target) return;
  const el = document.getElementById(target);
  if (!el) return;
  el.scrollIntoView({behavior:'smooth', block:'nearest'});
  if (typeof el.focus === 'function') el.focus({preventScroll:true});
}

// ── Outcome banner ────────────────────────────────────────────────────────
const ORB_STATES = {
  empty:    {type:'neutral', headline:'No current run started',
             detail:'Start routing to generate current-run artifacts.',
             link:null},
  routed:   {type:'warning', headline:'Route generated — verification pending',
             detail:'Receipt is available. Verification is the next audit step.',
             link:{label:'→ Go to verify', target:'btn-verify'}},
  verified: {type:'success', headline:'Verification completed',
             detail:'Dispatch can be exported for the current run.',
             link:{label:'→ Go to dispatch', target:'btn-dispatch-export'}},
  blocked:  {type:'blocked', headline:'Verification not completed',
             detail:'Verification failed. Review the result before dispatching.',
             link:{label:'→ Review verification', target:'verify-result'}},
  complete: {type:'success', headline:'Dispatch exported for current run',
             detail:'Current run artifacts are complete.',
             link:{label:'→ View export', target:'dispatch-export-result'}},
};
function orbStateKey() {
  if (lastExportPacket)          return 'complete';
  if (opVerify === 'verified')   return 'verified';
  if (opVerify === 'failed')     return 'blocked';
  if (opRouting === 'available') return 'routed';
  return 'empty';
}
function updateOutcomeBanner() {
  const s        = ORB_STATES[orbStateKey()];
  const orb      = document.getElementById('orb');
  const headline = document.getElementById('orb-headline');
  const detail   = document.getElementById('orb-detail');
  const link     = document.getElementById('orb-link');
  if (!orb || !headline || !detail || !link) return;
  orb.className        = 'orb orb-' + s.type;
  headline.textContent = s.headline;
  detail.textContent   = s.detail;
  if (s.link) {
    link.textContent = s.link.label;
    link.classList.remove('hidden');
  } else {
    link.classList.add('hidden');
  }
}
function orbNavigate() {
  const target = ORB_STATES[orbStateKey()]?.link?.target;
  if (!target) return;
  const el = document.getElementById(target);
  if (!el) return;
  el.scrollIntoView({behavior:'smooth', block:'nearest'});
  if (typeof el.focus === 'function') el.focus({preventScroll:true});
}

// ── Current-run completion checklist ─────────────────────────────────────
const CRC_ITEMS = [
  {id:'crc-route',    label:'Route generated',       doneFn: () => opRouting === 'available',
   pendingAnchor:{label:'Go to routing', target:'norm-input-section'}},
  {id:'crc-receipt',  label:'Receipt available',     doneFn: () => opReceipt === 'available',
   pendingAnchor:{label:'Go to routing', target:'norm-input-section'}},
  {id:'crc-verify',   label:'Verification completed',doneFn: () => opVerify === 'verified',
   pendingAnchor:{label:'Go to verification', target:'btn-verify'}},
  {id:'crc-dispatch', label:'Dispatch exported',     doneFn: () => !!lastExportPacket,
   pendingAnchor:{label:'Go to dispatch', target:'btn-dispatch-export'}},
];
function crcRowState(item, idx) {
  if (item.doneFn()) return 'done';
  // blocked if any prior item is not done
  for (let i = 0; i < idx; i++) {
    if (!CRC_ITEMS[i].doneFn()) return 'blocked';
  }
  return 'pending';
}
function updateCompletionChecklist() {
  const rows   = document.getElementById('crc-rows');
  const footer = document.getElementById('crc-footer');
  if (!rows || !footer) return;
  rows.innerHTML = CRC_ITEMS.map((item, idx) => {
    const state = crcRowState(item, idx);
    const icon  = state === 'done' ? '✓' : state === 'blocked' ? '◈' : '◻';
    const anchor = (state === 'pending' || state === 'blocked') && item.pendingAnchor
      ? ' <button class="crc-anchor" onclick="crcNavigate(' + JSON.stringify(item.pendingAnchor.target) + ')">'
        + esc(item.pendingAnchor.label) + '</button>'
      : '';
    return '<div class="crc-row">'
      + '<span class="crc-icon-' + state + '">' + icon + '</span>'
      + '<span class="crc-text-' + state + '">' + esc(item.label) + anchor + '</span>'
      + '</div>';
  }).join('');
  const allDone = CRC_ITEMS.every(item => item.doneFn());
  const readyForExport = opVerify === 'verified' && !lastExportPacket;
  if (allDone) {
    footer.textContent = 'Current run complete';
    footer.className   = 'crc-footer crc-footer-complete';
  } else if (readyForExport) {
    footer.textContent = 'Current run ready for dispatch export';
    footer.className   = 'crc-footer crc-footer-ready';
  } else {
    footer.textContent = 'Current run incomplete';
    footer.className   = 'crc-footer crc-footer-incomplete';
  }
}
function crcNavigate(target) {
  const el = document.getElementById(target);
  if (!el) return;
  el.scrollIntoView({behavior:'smooth', block:'nearest'});
  if (typeof el.focus === 'function') el.focus({preventScroll:true});
}

// ── Preflight summary card ────────────────────────────────────────────────
const PFC_VERDICTS = {
  'not-ready': {type:'not-ready',
    headline:'Current run not ready',
    detail:'Complete remaining workflow steps before dispatch export.',
    link:{label:'→ View next step', target:'nar-rail'}},
  'ready':     {type:'ready',
    headline:'Current run ready for dispatch',
    detail:'All current-run prerequisites are satisfied and no dispatch export exists yet.',
    link:{label:'→ Go to dispatch', target:'btn-dispatch-export'}},
  'complete':  {type:'complete',
    headline:'Current run complete',
    detail:'Dispatch export exists for the current run.',
    link:{label:'→ View export', target:'dispatch-export-result'}},
};
function pfcVerdictKey() {
  if (lastExportPacket)        return 'complete';
  if (opVerify === 'verified') return 'ready';
  return 'not-ready';
}
function updatePreflightCard() {
  const v        = PFC_VERDICTS[pfcVerdictKey()];
  const card     = document.getElementById('pfc');
  const headline = document.getElementById('pfc-headline');
  const detail   = document.getElementById('pfc-detail');
  const rows     = document.getElementById('pfc-rows');
  const link     = document.getElementById('pfc-link');
  if (!card || !headline || !detail || !rows || !link) return;
  card.className       = 'pfc pfc-' + v.type;
  headline.textContent = v.headline;
  detail.textContent   = v.detail;
  const routeOk   = opRouting === 'available';
  const receiptOk = opReceipt === 'available';
  const verifyOk  = opVerify  === 'verified';
  const exported  = !!lastExportPacket;
  function pfcRow(ok, label) {
    return '<div class="pfc-row"><span class="' + (ok ? 'pfc-ok' : 'pfc-dim') + '">'
      + (ok ? '✓' : '◻') + '</span><span class="' + (ok ? 'pfc-ok' : 'pfc-dim') + '">'
      + esc(label) + '</span></div>';
  }
  rows.innerHTML =
    pfcRow(routeOk,   'Route available')
    + pfcRow(receiptOk, 'Receipt available')
    + pfcRow(verifyOk,  'Verification complete')
    + (exported ? pfcRow(true,  'Dispatch exported')
                : pfcRow(false, 'Dispatch not yet exported'));
  if (v.link) {
    link.textContent = v.link.label;
    link.classList.remove('hidden');
  } else {
    link.classList.add('hidden');
  }
}
function pfcNavigate() {
  const target = PFC_VERDICTS[pfcVerdictKey()]?.link?.target;
  if (!target) return;
  const el = document.getElementById(target);
  if (!el) return;
  el.scrollIntoView({behavior:'smooth', block:'nearest'});
  if (typeof el.focus === 'function') el.focus({preventScroll:true});
}

// ── Audit snapshot export ─────────────────────────────────────────────────
function buildAuditSnapshot() {
  const verdict = PFC_VERDICTS[pfcVerdictKey()].headline;
  const routeSummary = lastReceipt
    ? 'Outcome: ' + (lastReceipt.outcome || '—')
      + '\nManufacturer: ' + (lastReceipt.selected_candidate_id || '(none)')
      + '\nReceipt hash: ' + (lastReceipt.receipt_hash || '—')
      + '\nKernel version: ' + (lastReceipt.routing_kernel_version || '—')
    : 'not present';
  const receiptJson = lastReceipt ? fmt(lastReceipt) : 'not present';
  const verifyEl    = document.getElementById('verify-json');
  const verifyText  = (verifyEl && verifyEl.textContent.trim())
    ? verifyEl.textContent.trim() : 'not executed';
  const dispatchText = lastExportPacket ? fmt(lastExportPacket) : 'not exported';
  const drStatusEl = document.getElementById('dr-status');
  const drReasonEl = document.getElementById('dr-reason');
  const readinessText = drStatusEl
    ? (drStatusEl.textContent.trim()
       + (drReasonEl ? ' — ' + drReasonEl.textContent.trim() : ''))
    : 'not available';
  return [
    'POSTCAD REVIEWER AUDIT SNAPSHOT',
    '================================',
    'Current run only. Does not include historical runs.',
    '',
    'Current run status',
    '------------------',
    verdict,
    '',
    'Route',
    '-----',
    routeSummary,
    '',
    'Receipt',
    '-------',
    receiptJson,
    '',
    'Verification',
    '------------',
    verifyText,
    '',
    'Dispatch',
    '--------',
    dispatchText,
    '',
    'Dispatch readiness',
    '------------------',
    readinessText,
  ].join('\n');
}
function copyAuditSnapshot(btn) {
  const snapshot = buildAuditSnapshot();
  navigator.clipboard.writeText(snapshot).then(() => {
    btn.textContent = 'Copied'; btn.style.color = '#3fb950';
  }).catch(() => {
    btn.textContent = 'Failed'; btn.style.color = '#f85149';
  });
  setTimeout(() => { btn.textContent = 'Copy snapshot'; btn.style.color = ''; }, 1500);
}
function downloadAuditSnapshot() {
  const snapshot = buildAuditSnapshot();
  const blob = new Blob([snapshot], {type: 'text/plain'});
  const url  = URL.createObjectURL(blob);
  const a    = document.createElement('a');
  a.href = url; a.download = 'postcad_audit_snapshot.txt'; a.click();
  URL.revokeObjectURL(url);
}

// ── Handoff summary card ──────────────────────────────────────────────────
function updateHandoffSummary() {
  const verdictEl = document.getElementById('hsc-verdict');
  const rowsEl    = document.getElementById('hsc-rows');
  const readyEl   = document.getElementById('hsc-readiness');
  const artsEl    = document.getElementById('hsc-artifacts');
  const sumEl     = document.getElementById('hsc-summary');
  if (!verdictEl || !rowsEl || !readyEl || !artsEl || !sumEl) return;
  const routeOk   = opRouting === 'available';
  const receiptOk = opReceipt === 'available';
  const verifyOk  = opVerify  === 'verified';
  const exported  = !!lastExportPacket;
  const vk = pfcVerdictKey();
  const verdictText = {
    'not-ready':'Not ready',
    'ready':    'Ready for dispatch export',
    'complete': 'Complete',
  }[vk] || 'Not ready';
  verdictEl.textContent = verdictText;
  verdictEl.className   = 'hsc-verdict hsc-verdict-' + vk;
  function hscRow(ok, label) {
    return '<div class="hsc-row ' + (ok ? 'hsc-row-ok' : 'hsc-row-no') + '">'
      + (ok ? '✓' : '✗') + ' ' + esc(label) + ': ' + (ok ? 'yes' : 'no') + '</div>';
  }
  rowsEl.innerHTML =
    hscRow(routeOk,   'Route generated')
    + hscRow(receiptOk, 'Receipt available')
    + hscRow(verifyOk,  'Verification completed')
    + hscRow(exported,  'Dispatch exported');
  const drStatusEl = document.getElementById('dr-status');
  const drReasonEl = document.getElementById('dr-reason');
  const drStatus = drStatusEl ? drStatusEl.textContent.trim() : 'Not evaluated';
  const drReason = drReasonEl ? drReasonEl.textContent.trim() : '';
  const drOk = verifyOk || exported;
  readyEl.innerHTML = '<div class="hsc-row ' + (drOk ? 'hsc-row-ok' : 'hsc-row-no') + '">'
    + esc(drStatus) + (drReason ? ' — ' + esc(drReason) : '') + '</div>';
  function artRow(ok, label, noLabel) {
    return '<div class="hsc-row ' + (ok ? 'hsc-row-ok' : 'hsc-row-no') + '">'
      + esc(label) + ': ' + esc(ok ? 'present' : noLabel) + '</div>';
  }
  artsEl.innerHTML =
    artRow(routeOk,   'Route',        'not present')
    + artRow(receiptOk, 'Receipt',      'not present')
    + artRow(verifyOk,  'Verification', 'not executed')
    + artRow(exported,  'Dispatch',     'not exported');
  const summaryLines = {
    'not-ready':'Current run requires additional workflow steps before dispatch.',
    'ready':    'Current run is ready for dispatch export.',
    'complete': 'Current run handoff is complete.',
  };
  sumEl.textContent = summaryLines[vk] || summaryLines['not-ready'];
}

// ── Consistency sentinel ──────────────────────────────────────────────────
function gatherConsistencyMismatches() {
  const mismatches = [];
  const verifyDone = opVerify !== 'not-run';
  const exported   = !!lastExportPacket;
  const routeOk    = opRouting === 'available';
  const receiptOk  = opReceipt === 'available';
  const verifyOk   = opVerify  === 'verified';
  const vk         = pfcVerdictKey();
  // Rule 1: verification present → route must be present
  if (verifyDone && !routeOk)
    mismatches.push('Verification marked present but route artifact missing.');
  // Rule 2: dispatch present → verification must be present
  if (exported && !verifyOk)
    mismatches.push('Dispatch artifact shown without current-run verification.');
  // Rule 3: dispatch present → receipt must be present
  if (exported && !receiptOk)
    mismatches.push('Dispatch artifact shown without current-run receipt.');
  // Rule 4: complete verdict → dispatch must be present
  if (vk === 'complete' && !exported)
    mismatches.push('Complete verdict shown without current-run dispatch export.');
  // Rule 5: ready verdict → verification must be present
  if (vk === 'ready' && !verifyOk)
    mismatches.push('Ready verdict shown but verification not present.');
  // Rule 6: ready verdict → dispatch must not be present
  if (vk === 'ready' && exported)
    mismatches.push('Ready verdict shown but dispatch export already exists.');
  return mismatches;
}
function updateConsistencySentinel() {
  const card     = document.getElementById('ccs');
  const headline = document.getElementById('ccs-headline');
  const detail   = document.getElementById('ccs-detail');
  if (!card || !headline || !detail) return;
  const mismatches = gatherConsistencyMismatches();
  if (mismatches.length === 0) {
    card.className       = 'ccs ccs-consistent';
    headline.textContent = 'Current run shell state is consistent';
    detail.innerHTML     = '<span style="color:#484f58;font-size:.65rem">'
      + 'Visible workflow indicators agree for the current run.</span>';
  } else {
    card.className       = 'ccs ccs-attention';
    headline.textContent = 'Current run shell state needs attention';
    detail.innerHTML     = mismatches.map(m =>
      '<div class="ccs-mismatch">▸ ' + esc(m) + '</div>'
    ).join('');
  }
}

// ── Active section emphasis ───────────────────────────────────────────────
const AS_CONTAINERS = ['as-route-section','as-verify-section','dispatch-section','dispatch-export-result'];
const AS_CHIPS      = ['as-chip-route','as-chip-verify','as-chip-dispatch','as-chip-export'];
function activeSectionIndex() {
  if (lastExportPacket)                                    return 3;
  if (opVerify === 'verified' || opVerify === 'failed')    return 2;
  if (opRouting === 'available')                           return 1;
  return 0;
}
function updateActiveSectionEmphasis() {
  const active = activeSectionIndex();
  AS_CONTAINERS.forEach((id, i) => {
    const el = document.getElementById(id);
    if (!el) return;
    el.classList.toggle('as-active', i === active);
  });
  AS_CHIPS.forEach((id, i) => {
    const el = document.getElementById(id);
    if (!el) return;
    el.classList.toggle('hidden', i !== active);
  });
}

// ── Pilot run history ─────────────────────────────────────────────────────
function appendRunHistory(label, ok) {
  const ts = new Date().toLocaleTimeString([], {hour:'2-digit',minute:'2-digit',second:'2-digit'});
  runHistory.push({ts, label, ok});
  const list = document.getElementById('run-history-list');
  if (!list) return;
  const entry = document.createElement('div');
  entry.className = 'rh-entry';
  entry.innerHTML = '<span class="rh-ts">' + esc(ts) + '</span>'
    + '<span class="rh-label ' + (ok ? 'rh-ok' : 'rh-err') + '">' + esc(label) + '</span>';
  list.appendChild(entry);
  show('run-history-panel');
}
function clearRunHistory() {
  runHistory.length = 0;
  const list = document.getElementById('run-history-list');
  if (list) list.innerHTML = '';
  hide('run-history-panel');
}

// ── Session activity log ───────────────────────────────────────────────────
const SAL_MAX = 20;
let sessionLog = [];
function salLog(label, msg) {
  sessionLog.unshift({label, msg});
  if (sessionLog.length > SAL_MAX) sessionLog.length = SAL_MAX;
  renderSessionLog();
}
function renderSessionLog() {
  const empty = document.getElementById('sal-empty');
  const list  = document.getElementById('sal-list');
  if (!empty || !list) return;
  if (sessionLog.length === 0) {
    empty.classList.remove('hidden');
    list.classList.add('hidden');
    list.innerHTML = '';
    return;
  }
  empty.classList.add('hidden');
  list.classList.remove('hidden');
  list.innerHTML = sessionLog.map((e, i) =>
    '<div class="sal-entry ' + (i === 0 ? 'sal-entry-latest' : 'sal-entry-older') + '">'
    + '<span class="sal-idx">' + (i + 1) + '</span>'
    + '<span>' + esc(e.label) + '</span>'
    + '<span class="sal-msg">' + esc(e.msg) + '</span>'
    + '</div>'
  ).join('');
}
function clearSessionLog() {
  sessionLog = [];
  renderSessionLog();
}

// ── Artifact size guard ────────────────────────────────────────────────────
const ARTIFACT_COLLAPSE_LINES = 40;
function collapseIfLarge(preId, btnId) {
  const pre = document.getElementById(preId);
  const btn = document.getElementById(btnId);
  if (!pre || !btn) return;
  const lines = (pre.textContent.match(/\n/g) || []).length + 1;
  if (lines > ARTIFACT_COLLAPSE_LINES) {
    pre.classList.add('collapsed');
    btn.classList.remove('hidden');
  } else {
    pre.classList.remove('collapsed');
    btn.classList.add('hidden');
  }
}
function expandArtifact(preId, btnId) {
  const pre = document.getElementById(preId);
  const btn = document.getElementById(btnId);
  if (pre) pre.classList.remove('collapsed');
  if (btn) btn.classList.add('hidden');
}

// ── helpers ────────────────────────────────────────────────────────────────
function fmt(o)        { return JSON.stringify(o, null, 2); }
function esc(s)        { return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;'); }
function previewRow(k, v) {
  return '<div class="norm-preview-row">'
    + '<span class="norm-preview-key">' + esc(k) + '</span>'
    + '<span class="norm-preview-val">'  + esc(String(v)) + '</span>'
    + '</div>';
}
function validateNormInput(c) {
  return ['case_id', 'restoration_type', 'material', 'jurisdiction']
    .filter(k => !c[k] || !String(c[k]).trim());
}
const NORM_INPUT_IDS = {
  case_id:          'norm-case-id',
  restoration_type: 'norm-restoration-type',
  material:         'norm-material',
  jurisdiction:     'norm-jurisdiction',
};
function readNormInputs() {
  return Object.fromEntries(
    Object.entries(NORM_INPUT_IDS).map(([k, id]) => [k, document.getElementById(id).value.trim()])
  );
}
function markNormInvalid(missing) {
  Object.entries(NORM_INPUT_IDS).forEach(([k, id]) => {
    const el = document.getElementById(id);
    if (missing.includes(k)) el.classList.add('norm-field-invalid');
    else                     el.classList.remove('norm-field-invalid');
  });
}
function clearNormInvalid() {
  Object.values(NORM_INPUT_IDS).forEach(id =>
    document.getElementById(id).classList.remove('norm-field-invalid'));
}
function loadNormSample() {
  document.getElementById('norm-case-id').value          = 'f1000001-0000-0000-0000-000000000001';
  document.getElementById('norm-restoration-type').value = 'crown';
  document.getElementById('norm-material').value         = 'zirconia';
  document.getElementById('norm-jurisdiction').value     = 'DE';
  const ni = document.getElementById('route-norm-inline');
  ni.textContent = '';
  ni.className = 'hidden';
  document.getElementById('route-norm-preview').innerHTML = '';
  document.getElementById('route-norm-preview').classList.add('hidden');
  clearNormInvalid();
  if (fixtures) document.getElementById('btn-route-norm').disabled = false;
}
function clearNormForm() {
  document.getElementById('norm-case-id').value          = '';
  document.getElementById('norm-restoration-type').value = '';
  document.getElementById('norm-material').value         = '';
  document.getElementById('norm-jurisdiction').value     = '';
  const ni = document.getElementById('route-norm-inline');
  ni.textContent = '';
  ni.className = 'hidden';
  document.getElementById('route-norm-preview').innerHTML = '';
  document.getElementById('route-norm-preview').classList.add('hidden');
  clearNormInvalid();
  if (fixtures) document.getElementById('btn-route-norm').disabled = false;
}
function toggleNormReceiptJson() {
  const pre = document.getElementById('norm-receipt-json-block');
  const btn = document.getElementById('btn-toggle-receipt');
  if (!pre || !btn) return;
  const isHidden = pre.classList.toggle('hidden');
  btn.textContent = isHidden ? 'Show receipt JSON' : 'Hide receipt JSON';
}
function downloadReceiptJson() {
  if (!lastReceipt) return;
  const blob = new Blob([JSON.stringify(lastReceipt, null, 2)], {type: 'application/json'});
  const url  = URL.createObjectURL(blob);
  const a    = document.createElement('a');
  a.href     = url;
  const hash = lastReceipt.receipt_hash ? lastReceipt.receipt_hash.slice(0, 12) : 'receipt';
  a.download = 'receipt_' + hash + '.json';
  a.click();
  URL.revokeObjectURL(url);
}
async function copyReceiptHash(btn, hash) {
  try {
    await navigator.clipboard.writeText(hash);
    btn.textContent = 'Copied';
    btn.style.color = '#3fb950';
  } catch(e) {
    btn.textContent = 'Copy failed';
    btn.style.color = '#f85149';
  }
  setTimeout(() => { btn.textContent = 'Copy'; btn.style.color = ''; }, 1500);
}
function show(id)      { document.getElementById(id).classList.remove('hidden'); }
function hide(id)      { document.getElementById(id).classList.add('hidden'); }
function setBtn(btn, label, disabled) { btn.textContent = label; btn.disabled = disabled; }

// Returns an operator-readable guidance hint for a routing error code.
function errorHint(code) {
  const c = String(code || '').toLowerCase();
  if (c.includes('normaliz') || c.includes('validat') || c.includes('parse'))
    return 'Check that all fields contain valid values. Example: restoration_type=crown, material=zirconia, jurisdiction=DE.';
  if (c.includes('no_eligible') || c.includes('routing') || c.includes('refused'))
    return 'No manufacturer matched the routing criteria. Try a different material, restoration type, or jurisdiction.';
  if (c.includes('registry') || c.includes('snapshot') || c.includes('fixture'))
    return 'The registry snapshot could not be loaded. Start the service from the repo root so examples/pilot/ is reachable.';
  return 'Clear the form and re-enter the values, or click \u2295 Load sample to use the canonical pilot input.';
}

// Ctrl+Enter / Cmd+Enter inside the normalized input section submits the form.
document.getElementById('norm-input-section').addEventListener('keydown', function(e) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
    const btn = document.getElementById('btn-route-norm');
    if (btn && !btn.disabled) { e.preventDefault(); routeNormalized(btn); }
  }
});

// Show a dispatch-section status message.
// kind: 'error' (red) | 'warn' (amber)
function showDispatchMsg(kind, text) {
  const el = document.getElementById('dispatch-error');
  el.className = (kind === 'warn' ? 'warn-note' : 'error-note');
  el.textContent = text;
  show('dispatch-error');
}
</script>
</body>
</html>"#;
