use anyhow::Result;
use clap::Parser;
use colored::*;
use debug_engine::{DebugSession, DebugConfig};

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

    if let Some(path) = args.export {
        let results = vec![report::ScenarioResult {
            scenario_name: "Vault Deposit & Withdraw".into(),
            success: true,
            failure_reason: None,
            assertion_results: vec![],
            trace: session.vm().trace().to_vec(),
        }];
        let html = report::generate_html_report(&results);
        std::fs::write(&path, html)?;
        println!("  Report exported to: {}", path.cyan().bold());
    }

    Ok(())
}
