use engine_model::ExecutionEvent;
use gas_profiler::GasReport;
use debug_engine::analysis::SecurityFinding;
use test_runtime::TestResult;
use chrono::Local;

/// Generate a premium, glassmorphism-styled HTML report from the demo run results.
pub fn generate_html_report(
    results: &[TestResult],
    primary_trace: &[ExecutionEvent],
    gas_report: &GasReport,
    findings: &[SecurityFinding],
    migration_patterns: &[(String, String, String)],
) -> String {
    let test_rows = results
        .iter()
        .map(|r| {
            let status = if r.passed {
                r#"<span class="status-badge pass">PASS ✓</span>"#
            } else {
                r#"<span class="status-badge fail">FAIL ✗</span>"#
            };
            let _reason = r.failure_reason.as_deref().unwrap_or("—");
            let assertions_html: String = r.assertion_results.iter().map(|a| {
                let mark = if a.passed { "✓" } else { "✗" };
                let cls = if a.passed { "pass" } else { "fail" };
                format!(r#"<div class="assertion-item {cls}">{mark} {}</div>"#, a.assertion)
            }).collect();
            format!(
                r#"<tr class="glass-row">
                    <td><div class="test-name">{}</div><div class="test-desc">{}</div></td>
                    <td>{status}</td>
                    <td>{}</td>
                    <td>{assertions_html}</td>
                </tr>"#,
                r.name, r.description, r.trace.len()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut cumulative_gas: u64 = 0;
    let total_limit: u64 = 4_000_000; // Arbitrum block gas limit placeholder
    let trace_rows = primary_trace
        .iter()
        .map(|ev| {
            cumulative_gas += ev.gas_used;
            let remaining = total_limit.saturating_sub(cumulative_gas);
            let storage = if ev.storage_diff.is_empty() {
                "—".to_string()
            } else {
                ev.storage_diff
                    .iter()
                    .map(|s| {
                        format!(
                            r#"<div class="storage-change"><strong>{}</strong>: {} → {}</div>"#,
                            s.key,
                            s.old.as_deref().unwrap_or("∅"),
                            s.new.as_deref().unwrap_or("∅")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("")
            };
            let snippet = ev.source_line.as_deref().unwrap_or("—");
            format!(
                r#"<tr class="trace-row" onclick="inspectStep({}, '{}', {}, {}, {})">
                    <td>{}</td>
                    <td><code class="opcode-tag">{}</code></td>
                    <td>{}</td>
                    <td>{}</td>
                    <td><code class="rust-code">{}</code></td>
                    <td><div class="storage-cell">{}</div></td>
                </tr>"#,
                ev.step, snippet.replace("'", "\\'"), ev.stack.len(), ev.gas_used, remaining, ev.step, ev.opcode, ev.gas_used, ev.stack.len(), snippet, storage
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let hotspot_rows = gas_report
        .hotspots
        .iter()
        .map(|h| {
            format!(
                r#"<tr>
                    <td><code class="opcode-tag">{}</code></td>
                    <td>{}</td>
                    <td>{:.1}%</td>
                    <td width="200"><div class="progress-container"><div class="progress-bar" style="width:{}%"></div></div></td>
                </tr>"#,
                h.label, h.gas, h.percent, h.percent
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let suggestion_items = gas_report
        .suggestions
        .iter()
        .map(|s| format!(r#"<div class="suggestion-card">{}</div>"#, s))
        .collect::<Vec<_>>()
        .join("\n");

    let finding_cards = findings
        .iter()
        .map(|f| {
            let sev_class = match f.severity {
                debug_engine::analysis::Severity::Critical => "critical",
                debug_engine::analysis::Severity::Warning => "warning",
                debug_engine::analysis::Severity::Info => "info",
            };
            let step_info = f.step.map(|s| format!(r#"<div class="finding-step">Detected at step {}</div>"#, s)).unwrap_or_default();
            format!(
                r#"<div class="finding-card sev-{sev_class}">
                    <div class="finding-header">
                        <span class="sev-badge">{}</span>
                        <strong>{}</strong>
                    </div>
                    <div class="finding-body">{}</div>
                    {}
                </div>"#,
                f.severity, f.title, f.description, step_info
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let migration_rows = migration_patterns
        .iter()
        .map(|(sol, rust, desc)| {
            format!(
                r#"<tr class="glass-row">
                    <td><code>{sol}</code></td>
                    <td><code class="rust-code">{rust}</code></td>
                    <td>{desc}</td>
                </tr>"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Stylus Debug Suite Dashboard</title>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;600;700&family=JetBrains+Mono&display=swap" rel="stylesheet">
<style>
  :root {{
    --bg: #030712;
    --surface: rgba(17, 24, 39, 0.7);
    --border: rgba(255, 255, 255, 0.1);
    --accent: #3b82f6;
    --accent-glow: rgba(59, 130, 246, 0.3);
    --text: #f3f4f6;
    --text-muted: #9ca3af;
    --green: #10b981;
    --red: #ef4444;
    --yellow: #f59e0b;
    --glass-blur: blur(12px);
  }}
  * {{ margin:0; padding:0; box-sizing:border-box; }}
  body {{ 
    background: var(--bg);
    background-image: radial-gradient(circle at 50% -20%, #1e1b4b, transparent);
    color: var(--text);
    font-family: 'Inter', sans-serif;
    padding: 3rem;
    min-height: 100vh;
  }}
  .glass-card {{
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border);
    border-radius: 1.5rem;
    padding: 2rem;
    margin-bottom: 2.5rem;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  }}
  h1 {{ font-size: 2.5rem; font-weight: 700; letter-spacing: -0.025em; margin-bottom: 0.5rem; background: linear-gradient(to right, #fff, #94a3b8); -webkit-background-clip: text; -webkit-text-fill-color: transparent; }}
  h2 {{ font-size: 1.5rem; margin-bottom: 1.5rem; color: var(--accent); display: flex; align-items: center; gap: 0.75rem; }}
  h2::before {{ content: ''; display: inline-block; width: 4px; height: 1.5rem; background: var(--accent); border-radius: 2px; }}
  .header {{ margin-bottom: 4rem; position: relative; }}
  .badge {{ background: var(--accent-glow); color: var(--accent); padding: 0.5rem 1rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 700; border: 1px solid var(--accent); text-transform: uppercase; letter-spacing: 0.05em; }}
  
  table {{ width: 100%; border-collapse: separate; border-spacing: 0 0.5rem; margin: 1rem 0; }}
  th {{ text-align: left; padding: 1rem; color: var(--text-muted); font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; }}
  td {{ padding: 1.25rem 1rem; background: rgba(255,255,255,0.03); border-top: 1px solid var(--border); border-bottom: 1px solid var(--border); }}
  td:first-child {{ border-left: 1px solid var(--border); border-top-left-radius: 1rem; border-bottom-left-radius: 1rem; }}
  td:last-child {{ border-right: 1px solid var(--border); border-top-right-radius: 1rem; border-bottom-right-radius: 1rem; }}
  
  .status-badge {{ padding: 0.25rem 0.75rem; border-radius: 0.5rem; font-size: 0.75rem; font-weight: 600; display: inline-flex; align-items: center; gap: 0.25rem; }}
  .status-badge.pass {{ background: rgba(16, 185, 129, 0.1); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.2); }}
  .status-badge.fail {{ background: rgba(239, 68, 68, 0.1); color: var(--red); border: 1px solid rgba(239, 68, 68, 0.2); }}
  
  code {{ font-family: 'JetBrains Mono', monospace; font-size: 0.85rem; background: rgba(0,0,0,0.3); padding: 0.2rem 0.5rem; border-radius: 0.5rem; }}
  .opcode-tag {{ color: var(--green); border: 1px solid rgba(16, 185, 129, 0.2); }}
  .rust-code {{ color: var(--accent); }}
  
  .trace-row {{ cursor: pointer; transition: all 0.2s; }}
  .trace-row:hover {{ background: rgba(59, 130, 246, 0.1); transform: translateX(4px); }}
  
  .progress-container {{ background: rgba(255,255,255,0.05); height: 8px; border-radius: 4px; overflow: hidden; }}
  .progress-bar {{ background: var(--accent); height: 100%; border-radius: 4px; box-shadow: 0 0 10px var(--accent-glow); }}
  
  .finding-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1.5rem; }}
  .finding-card {{ padding: 1.5rem; border-radius: 1rem; border: 1px solid var(--border); position: relative; overflow: hidden; transition: transform 0.2s; }}
  .finding-card:hover {{ transform: translateY(-5px); }}
  .finding-card.sev-critical {{ border-color: var(--red); background: linear-gradient(135deg, rgba(239, 68, 68, 0.1), transparent); }}
  .finding-card.sev-warning {{ border-color: var(--yellow); background: linear-gradient(135deg, rgba(245, 158, 11, 0.1), transparent); }}
  .finding-card.sev-info {{ border-color: var(--accent); background: linear-gradient(135deg, rgba(59, 130, 246, 0.1), transparent); }}
  .sev-badge {{ font-size: 0.65rem; font-weight: 800; text-transform: uppercase; padding: 0.2rem 0.6rem; border-radius: 4px; margin-bottom: 0.75rem; display: inline-block; }}
  .sev-critical .sev-badge {{ background: var(--red); color: #fff; }}
  .sev-warning .sev-badge {{ background: var(--yellow); color: #000; }}
  .sev-info .sev-badge {{ background: var(--accent); color: #fff; }}
  
  .suggestion-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1rem; }}
  .suggestion-card {{ background: rgba(255,255,255,0.02); padding: 1.25rem; border-radius: 1rem; border: 1px solid var(--border); font-size: 0.9rem; position: relative; padding-left: 3rem; }}
  .suggestion-card::before {{ content: '💡'; position: absolute; left: 1rem; top: 1.25rem; font-size: 1.2rem; }}
  
  .storage-cell {{ font-size: 0.8rem; line-height: 1.4; }}
  .storage-change {{ margin-bottom: 0.5rem; }}
  .storage-change strong {{ color: var(--yellow); }}
  
  .inspector {{ position: fixed; right: 2rem; bottom: 2rem; width: 400px; max-height: 500px; background: rgba(17, 24, 39, 0.95); backdrop-filter: blur(20px); border: 1px solid var(--accent); border-radius: 1.5rem; padding: 1.5rem; box-shadow: 0 0 40px rgba(0,0,0,0.8); display: none; z-index: 100; }}
  .inspector h3 {{ margin-bottom: 1rem; color: var(--accent); font-size: 1.1rem; }}
  .inspector-list {{ list-style: none; font-size: 0.85rem; }}
  .inspector-list li {{ padding: 0.5rem 0; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; }}
  .inspector-list li span:last-child {{ color: var(--accent); font-family: 'JetBrains Mono', monospace; }}
  
  @keyframes fadeIn {{ from {{ opacity: 0; transform: translateY(20px); }} to {{ opacity: 1; transform: translateY(0); }} }}
  .glass-card {{ animation: fadeIn 0.6s ease-out forwards; }}
</style>
</head>
<body>

<div class="header">
  <span class="badge">Experimental POC</span>
  <h1>Stylus Debug Suite Dashboard</h1>
  <p style="color:var(--text-muted); font-size:1.1rem; max-width:600px;">
    Advanced developer toolkit for Arbitrum Stylus. Automated testing, gas-aware profiling, and security-first trace analysis.
  </p>
</div>

<section class="glass-card">
  <h2>Integration Test Scenarios</h2>
  <table>
    <thead>
      <tr><th>Capability / Scenario</th><th>Status</th><th>Events</th><th>Assertion Pipeline</th></tr>
    </thead>
    <tbody>
      {test_rows}
    </tbody>
  </table>
</section>

<div style="display: grid; grid-template-columns: 2fr 1fr; gap: 2.5rem;">
  <section class="glass-card">
    <h2>Execution Trace Explorer</h2>
    <p style="color:var(--text-muted); font-size:0.85rem; margin-bottom:1rem;">Click any step to inspect machine state.</p>
    <table>
      <thead>
        <tr><th>Step</th><th>Opcode</th><th>Gas</th><th>Stack</th><th>Source</th><th>State Mutations</th></tr>
      </thead>
      <tbody>
        {trace_rows}
      </tbody>
    </table>
  </section>

  <div>
    <section class="glass-card">
      <h2>Gas Hotspots</h2>
      <table>
        <thead>
          <tr><th>Opcode</th><th>Gas</th><th>%</th><th>Load</th></tr>
        </thead>
        <tbody>
          {hotspot_rows}
        </tbody>
      </table>
    </section>

    <section class="glass-card">
      <h2>Optimization Insights</h2>
      <div class="suggestion-grid">
        {suggestion_items}
      </div>
    </section>
  </div>
</div>

<section class="glass-card">
  <h2>Trace-Based Security Findings</h2>
  <div class="finding-grid">
    {finding_cards}
  </div>
</section>

<section class="glass-card">
  <h2>Solidity → Stylus Migration Assistant</h2>
  <table>
    <thead>
      <tr><th>Solidity Pattern</th><th>Stylus Rust Equivalent</th><th>Description</th></tr>
    </thead>
    <tbody>
      {migration_rows}
    </tbody>
  </table>
</section>

<div id="inspector" class="inspector">
  <div style="display:flex; justify-content:space-between; align-items:center;">
    <h3>Step Inspector</h3>
    <button onclick="document.getElementById('inspector').style.display='none'" style="background:none; border:none; color:var(--text-muted); cursor:pointer; font-size:1.5rem;">&times;</button>
  </div>
  <div id="inspector-content"></div>
</div>

<script>
  function inspectStep(step, snippet, stackDepth, gasUsed, gasRemaining) {{
    const inspector = document.getElementById('inspector');
    const content = document.getElementById('inspector-content');
    inspector.style.display = 'block';
    
    content.innerHTML = `
      <ul class="inspector-list">
        <li><span>Instruction ID</span> <span>#${{step}}</span></li>
        <li><span>Execution Frame</span> <span>0x42...f6</span></li>
        <li><span>Stack Depth</span> <span>${{stackDepth}}</span></li>
        <li><span>Gas Used (Step)</span> <span>${{gasUsed.toLocaleString()}}</span></li>
        <li><span>Gas Remaining</span> <span>${{gasRemaining.toLocaleString()}}</span></li>
      </ul>
      <div style="margin-top:1.5rem; padding:1rem; background:rgba(0,0,0,0.4); border-radius:0.5rem;">
        <div style="font-size:0.7rem; color:var(--accent); margin-bottom:0.5rem; text-transform:uppercase;">Source Snippet</div>
        <code style="display:block; color:var(--green);">${{snippet}}</code>
      </div>
    `;
  }}
</script>

<footer style="margin-top:2rem; font-size:0.8rem; color:var(--text-muted); display:flex; justify-content:space-between; align-items:center;">
  <div>Generated by Stylus Debug Suite · <strong>ILE Labs</strong></div>
  <div>{timestamp} · Arbitrum Stylus Alpha</div>
</footer>

</body>
</html>"##,
        timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    )
}
