mod report;

use std::collections::BTreeMap;
use std::fs;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use engine_model::{ExecutionEvent, StorageChange};
use debug_engine::analysis::{analyze_trace, SecurityFinding, Severity};
use gas_profiler::{profile, GasReport};
use test_runtime::{run_scenario, TestConfig, TestResult};

/// Stylus Debug Suite — one-command POC demo runner.
///
/// Orchestrates all five capabilities: integration tests, execution trace,
/// gas profiler, storage snapshot, security analysis, and migration hints.
#[derive(Parser)]
#[command(name = "stylus-demo", version, about)]
struct Cli {
    /// Test config YAML path.
    #[arg(long, default_value = "examples/demo-contracts/demo-test.yml")]
    config: String,

    /// Output format: text or json.
    #[arg(long, default_value = "text")]
    format: String,

    /// Run only this scenario (by name). If omitted, runs all.
    #[arg(long)]
    scenario: Option<String>,

    /// Export a self-contained HTML report to this path.
    #[arg(long)]
    export: Option<String>,

    /// Show verbose output including full JSON traces.
    #[arg(long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let yaml = fs::read_to_string(&cli.config)?;
    let config: TestConfig = serde_yaml::from_str(&yaml)?;

    // Filter scenarios if --scenario is specified.
    let scenarios: Vec<_> = if let Some(ref name) = cli.scenario {
        config
            .scenarios
            .iter()
            .filter(|s| s.name == *name)
            .collect()
    } else {
        config.scenarios.iter().collect()
    };

    if scenarios.is_empty() {
        anyhow::bail!("no matching scenarios found");
    }

    if cli.format == "text" {
        print_banner();
    }

    // ── Simulation Phase (with progress bar) ─────────────────────────────
    let pb = if cli.format == "text" {
        let pb = ProgressBar::new(scenarios.len() as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .expect("valid template")
            .progress_chars("#>-"));
        Some(pb)
    } else {
        None
    };

    let mut results: Vec<TestResult> = Vec::new();
    for scenario_cfg in &scenarios {
        if let Some(ref p) = pb {
            p.set_message(format!("Simulating {}...", scenario_cfg.name));
        }
        // Small delay to show progress bar
        tokio::time::sleep(Duration::from_millis(300)).await;
        
        let result = run_scenario(&config, scenario_cfg).await?;
        results.push(result);
        
        if let Some(ref p) = pb {
            p.inc(1);
        }
    }

    if let Some(p) = pb {
        p.finish_with_message("Simulation complete!");
    }

    let primary = results
        .iter()
        .find(|r| r.passed)
        .unwrap_or(&results[0]);

    // ── Analysis Phase ───────────────────────────────────────────────────
    if cli.format == "text" {
        let apb = ProgressBar::new_spinner();
        apb.set_message("Performing deep security analysis...");
        apb.enable_steady_tick(Duration::from_millis(100));
        tokio::time::sleep(Duration::from_millis(800)).await;
        apb.finish_and_clear();
    }

    let gas_report = profile(&primary.name, &primary.trace);
    let findings = analyze_trace(&primary.trace);

    // Migration patterns (summary form for demo output).
    let migration_patterns = vec![
        ("uint256 public balance".into(), "sol_storage! { balance: StorageU256 }".into(), "State variable → StorageU256".into()),
        ("msg.sender".into(), "msg::sender()".into(), "Caller address → SDK function".into()),
        ("msg.value".into(), "msg::value()".into(), "Attached value → SDK function".into()),
        ("require(cond, reason)".into(), "if !cond { return Err(...) }".into(), "Require → Rust Result".into()),
        ("event Deposited(...)".into(), "sol! { event Deposited(...); }".into(), "Events → sol! macro".into()),
        ("external payable".into(), "#[payable] fn deposit()".into(), "Payable → #[payable] attribute".into()),
        ("address.call{value}(\"\")".into(), "call::Call::new().value(amt)".into(), "Low-level call → Call builder".into()),
    ];

    match cli.format.as_str() {
        "json" => print_json_output(&results, &gas_report, &findings)?,
        _ => print_text_output(&results, &config, primary, &gas_report, &findings, &migration_patterns, cli.verbose),
    }

    // ── HTML export ──────────────────────────────────────────────────────
    if let Some(ref path) = cli.export {
        let html = report::generate_html_report(
            &results,
            &primary.trace,
            &gas_report,
            &findings,
            &migration_patterns,
        );
        fs::write(path, &html)?;
        println!("\n  📄 {} saved to: {}", "HTML report".bold().cyan(), path.bold().white());
    }

    Ok(())
}

// ─── JSON output ─────────────────────────────────────────────────────────────

fn print_json_output(
    results: &[TestResult],
    gas_report: &GasReport,
    findings: &[SecurityFinding],
) -> Result<()> {
    #[derive(serde::Serialize)]
    struct FullReport<'a> {
        tests: &'a [TestResult],
        gas: &'a GasReport,
        security: &'a [SecurityFinding],
    }

    let report = FullReport {
        tests: results,
        gas: gas_report,
        security: findings,
    };
    let json = serde_json::to_string_pretty(&report)?;
    println!("{json}");
    Ok(())
}

// ─── Text output ─────────────────────────────────────────────────────────────

fn print_text_output(
    results: &[TestResult],
    config: &TestConfig,
    primary: &TestResult,
    gas_report: &GasReport,
    findings: &[SecurityFinding],
    migration_patterns: &[(String, String, String)],
    verbose: bool,
) {
    print_banner();

    // 1) Integration Tests
    section("1 ▸ Integration Test Runner");
    println!("  Running {} scenario(s) against DemoVault contract...\n", results.len());

    for result in results {
        print_test_result(result, config);
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();
    println!("  ─────────────────────────────────────────────────");
    println!("  Results: {} passed, {} failed, {} total", passed, failed, results.len());

    // 2) Execution Trace
    section("2 ▸ Execution Trace");
    print_trace(&primary.trace);

    if verbose {
        println!("\n  Full JSON trace:");
        if let Ok(json) = serde_json::to_string_pretty(&primary.trace) {
            for line in json.lines() {
                println!("    {line}");
            }
        }
    } else {
        println!("\n  (use --verbose for full JSON trace)");
    }

    // Show failure trace if any
    for fail_result in results.iter().filter(|r| !r.passed) {
        println!();
        println!("  ┌ Failure Trace ({})", fail_result.name);
        println!("  │");
        for ev in &fail_result.trace {
            let marker = if ev.opcode == "REVERT" { "✗" } else { "│" };
            println!(
                "  {}  step {:>3} │ {:8} │ gas: {:>6}",
                marker, ev.step, ev.opcode, ev.gas_used
            );
        }
        if let Some(ref reason) = fail_result.failure_reason {
            println!("  ✗  Reason: {reason}");
        }
        println!("  └────────────────────────────────────────────");
    }

    // 3) Gas Profile
    section("3 ▸ Gas Profiler Report");
    print_gas_report(gas_report);

    // 4) Security Analysis
    section("4 ▸ Security Analysis");
    print_security_findings(findings);

    // 5) Storage Snapshot
    section("5 ▸ Storage Snapshot");
    print_storage_snapshot(&primary.trace, &primary.final_storage);

    // 6) Migration Hints
    section("6 ▸ Solidity → Stylus Migration Hints");
    print_migration_hints(migration_patterns);

    // Done
    section("✓ Demo Complete");
    println!("  The Stylus Debug Suite demonstrated:");
    println!("    1. Integration test runner    — VM-executed scenarios with assertions");
    println!("    2. Execution trace            — step-by-step opcode trace + JSON");
    println!("    3. Gas profiler               — hotspot analysis + optimization tips");
    println!("    4. Security analysis           — reentrancy, unchecked calls, gas patterns");
    println!("    5. Storage snapshot            — before/after for every slot write");
    println!("    6. Migration hints             — Solidity patterns → Stylus Rust");
    println!();
    println!("  Use --format json for machine-readable output.");
    println!("  Use --export report.html for a self-contained HTML report.");
}

fn print_banner() {
    println!();
    println!("{}", "  ╔══════════════════════════════════════════════════╗".cyan());
    println!("  ║        {}        ║", "Stylus Debug Suite — POC Demo Run".bold().white());
    println!("{}", "  ╠══════════════════════════════════════════════════╣".cyan());
    println!("  ║  {}   ║", "ILE Labs · Arbitrum Stylus Developer Toolkit".italic().bright_black());
    println!("{}", "  ╚══════════════════════════════════════════════════╝".cyan());
}

fn section(title: &str) {
    println!();
    println!("{}", format!("  ╭──────────────────────────────────────────────────╮").blue());
    println!("  {} {:48} {}", "│".blue(), title.bold().yellow(), "│".blue());
    println!("{}", format!("  ╰──────────────────────────────────────────────────╯").blue());
    println!();
}

fn print_test_result(result: &TestResult, cfg: &TestConfig) {
    let status = if result.passed { 
        "PASS ✓".bold().green() 
    } else { 
        "FAIL ✗".bold().red() 
    };
    println!("  {} Scenario: {}", "┌".bright_black(), result.name.bold().white());
    if !result.description.is_empty() {
        println!("  {} Info    : {}", "│".bright_black(), result.description.italic().bright_black());
    }
    println!("  {} Status  : {}", "│".bright_black(), status);
    println!(
        "  {} Network : {}{}",
        "│".bright_black(),
        cfg.network.cyan(),
        cfg.fork
            .as_ref()
            .map(|f| format!(" (fork: {})", f.bright_black()))
            .unwrap_or_default()
    );

    // Print assertion results
    for ar in &result.assertion_results {
        let mark = if ar.passed { "✓".green() } else { "✗".red() };
        let msg = format!("{} (expected: {}, actual: {})", ar.assertion, ar.expected.white(), ar.actual.white());
        println!("  {}   {} {}", "│".bright_black(), mark, msg.bright_black());
    }

    if let Some(ref reason) = result.failure_reason {
        println!("  {} {} : {}", "│".bright_black(), "Reason".bold().red(), reason.red());
    }
    println!("  {} Trace   : {} execution events recorded", "│".bright_black(), result.trace.len().to_string().cyan());
    println!("  {}─────────────────────────────────────────────────", "└".bright_black());
    println!();
}

fn print_trace(trace: &[ExecutionEvent]) {
    println!(
        "  {:>5} │ {:8} │ {:>8} │ {:>5} │ {:25} │ storage",
        "step".bold(), "opcode".bold(), "gas".bold(), "stack".bold(), "source snippet".bold()
    );
    println!("{}", "  ──────┼──────────┼──────────┼───────┼───────────────────────────┼────────────────────".bright_black());
    for ev in trace {
        let storage_info = if ev.storage_diff.is_empty() {
            "—".bright_black().to_string()
        } else {
            ev.storage_diff
                .iter()
                .map(|s| {
                    format!(
                        "{}: {} → {}",
                        s.key.yellow(),
                        s.old.as_deref().unwrap_or("∅").bright_black(),
                        s.new.as_deref().unwrap_or("∅").cyan()
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        let snippet = ev.source_line.as_deref().unwrap_or("—");
        println!(
            "  {:>5} │ {:8} │ {:>8} │ {:>5} │ {:25} │ {}",
            ev.step.to_string().bright_black(),
            ev.opcode.green(),
            ev.gas_used.to_string().white(),
            ev.stack.len().to_string().cyan(),
            snippet.italic().bright_black(),
            storage_info
        );
    }
}

fn print_gas_report(report: &GasReport) {
    println!("  {}  : {}", "Scenario".bold(), report.function.cyan());
    println!("  {} : {} ({})", "Total gas".bold(), report.total_gas.to_string().white(), "VM-computed".italic().bright_black());
    println!();
    println!("  {}", "Hotspots:".bold().yellow());
    for h in &report.hotspots {
        let bar_len = (h.percent / 5.0).round() as usize;
        let bar: String = "█".repeat(bar_len);
        println!(
            "    {:<14} {:>6} gas  {:>5.1}%  {}",
            h.label.green(), h.gas.to_string().white(), h.percent.to_string().cyan(), bar.blue()
        );
    }
    println!();
    println!("  {}", "Optimization suggestions:".bold().yellow());
    for (i, s) in report.suggestions.iter().enumerate() {
        println!("    {}. {}", (i + 1).to_string().bold().white(), s.bright_white());
    }
}

fn print_security_findings(findings: &[SecurityFinding]) {
    if findings.is_empty() {
        println!("  {}", "No security findings detected.".green());
        return;
    }

    println!("  Found {} finding(s):\n", findings.len().to_string().bold().red());
    for (i, f) in findings.iter().enumerate() {
        let (icon, color) = match f.severity {
            Severity::Critical => ("🔴", "red"),
            Severity::Warning => ("🟡", "yellow"),
            Severity::Info => ("🔵", "blue"),
        };
        
        let sev_str = format!("[{:8}]", f.severity.to_string()).color(color).bold();
        println!("  {} {}. {} {}", icon, (i + 1).to_string().bold(), sev_str, f.title.bold());
        println!("     {}", f.description.bright_black());
        if let Some(step) = f.step {
            println!("     (at step {})", step.to_string().cyan());
        }
        println!();
    }
}

fn print_storage_snapshot(trace: &[ExecutionEvent], final_storage: &BTreeMap<String, String>) {
    let mut slots: BTreeMap<String, Vec<&StorageChange>> = BTreeMap::new();
    for ev in trace {
        for change in &ev.storage_diff {
            slots.entry(change.key.clone()).or_default().push(change);
        }
    }

    if slots.is_empty() && final_storage.is_empty() {
        println!("  (no storage writes observed)");
        return;
    }

    println!(
        "  {:16} │ {:>12} │ {:>12} │ writes",
        "slot", "initial", "final"
    );
    println!("  ─────────────────┼──────────────┼──────────────┼───────");
    for (key, changes) in &slots {
        let first_old = changes
            .first()
            .and_then(|c| c.old.as_deref())
            .unwrap_or("∅");
        let last_new = changes
            .last()
            .and_then(|c| c.new.as_deref())
            .unwrap_or("∅");
        let changed = first_old != last_new;
        let marker = if changed { "◆" } else { "○" };
        println!(
            "  {marker} {:<14} │ {:>12} │ {:>12} │ {}",
            key,
            first_old,
            last_new,
            changes.len()
        );
    }
    println!();
    println!("  Legend: ◆ = changed, ○ = unchanged (read-only)");

    println!();
    println!("  Final VM storage state:");
    for (k, v) in final_storage {
        println!("    {k} = {v}");
    }
}

fn print_migration_hints(patterns: &[(String, String, String)]) {
    println!("  Source: examples/demo-contracts/Demo.sol (DemoVault)\n");
    println!(
        "  {:>2}  {:30}  {:38}  {}",
        "#", "Solidity Pattern", "Stylus Rust Equivalent", "Description"
    );
    println!("  ──  ──────────────────────────  ──────────────────────────────────────  ────────────────────────────────────────────");
    for (i, (sol, rust, desc)) in patterns.iter().enumerate() {
        println!("  {:>2}. {:30}  {:38}  {}", i + 1, sol, rust, desc);
    }
    println!();
    println!("  Run `cargo run -p migration-cli -- --verbose` for detailed analysis.");
}
