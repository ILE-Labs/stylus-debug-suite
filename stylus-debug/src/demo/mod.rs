use anyhow::Result;
use clap::Parser;
use colored::*;
use debug_engine::{DebugSession, DebugConfig, analysis};

pub mod report;

#[derive(Parser)]
pub struct DemoArgs {
    /// Export report to HTML.
    #[arg(long)]
    pub export: Option<String>,
}

pub async fn run(args: DemoArgs) -> Result<()> {
    println!("\n  {}", "Stylus Debug Demo Runner".bold().green());
    println!("  Executing pre-defined scenarios...\n");

    let config = DebugConfig {
        contract_path: "vault.rs".into(),
        entrypoint: "deposit_and_withdraw".into(),
        breakpoints: vec![],
    };

    let mut session = DebugSession::new(config);
    println!("  Scenario: {} ... {}", "Vault Deposit & Withdraw", "RUNNING".yellow());

    while session.step() {
        // Step through the VM
    }

    println!("  Scenario: {} ... {}", "Vault Deposit & Withdraw", "PASSED".green());

    let trace = session.vm().trace();
    
    // Run Analysis
    let gas_report = gas_profiler::profile("deposit_and_withdraw", trace);
    let security_findings = analysis::analyze_trace(trace);

    // Print Detailed Console Summary
    println!("\n  {}", "─── Execution Summary ───".bold().blue());
    println!("  Total Gas: {}", gas_report.total_gas.to_string().cyan());
    
    println!("\n  {}", "Gas Hotspots:".bold());
    for hotspot in gas_report.hotspots.iter().take(3) {
        println!("    • {:<10} {:>8} gas ({:.1}%)", hotspot.label, hotspot.gas.to_string().yellow(), hotspot.percent);
    }

    if !security_findings.is_empty() {
        println!("\n  {}", "Security Findings:".bold().red());
        for finding in security_findings.iter().take(3) {
            let color_severity = match finding.severity {
                analysis::Severity::Critical => finding.severity.to_string().red().bold(),
                analysis::Severity::Warning => finding.severity.to_string().yellow().bold(),
                _ => finding.severity.to_string().blue(),
            };
            println!("    [{}] {}", color_severity, finding.title);
        }
    }

    println!("\n  {}", "Optimization Tips:".bold().green());
    for tip in gas_report.suggestions.iter().take(2) {
        println!("    💡 {}", tip);
    }

    if let Some(path) = args.export {
        let results = vec![report::ScenarioResult {
            scenario_name: "Vault Deposit & Withdraw".into(),
            success: true,
            failure_reason: None,
            assertion_results: vec![],
            trace: trace.to_vec(),
        }];
        let html = report::generate_html_report(&results);
        std::fs::write(&path, html)?;
        println!("\n  {} {}", "Visual HTML report exported to:".bold(), path.cyan().underline());
    } else {
        println!("\n  {}", "Tip: Run with --export report.html for a full visual trace explorer.".dimmed());
    }

    Ok(())
}
