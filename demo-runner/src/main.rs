use anyhow::Result;
use engine_model::ExecutionEvent;
use gas_profiler::profile;
use test_runtime::{run_scenario, TestConfig};

#[tokio::main]
async fn main() -> Result<()> {
    print_banner();

    // 1) Run an integration-style scenario against the demo vault.
    section("1) Integration Test");
    let cfg = TestConfig {
        network: "local".into(),
        fork: Some("arbitrum-mainnet".into()),
        contracts: vec!["examples/demo-contracts/vault.rs".into()],
        scenarios: vec!["deposit_and_withdraw".into()],
    };
    let result = run_scenario(&cfg, "deposit_and_withdraw").await?;
    println!("Scenario: {}", result.name);
    println!("Status  : {}", if result.passed { "PASSED" } else { "FAILED" });
    println!("Network : {}{}", cfg.network, cfg.fork.as_ref().map(|f| format!(" (fork: {f})")).unwrap_or_default());

    // 2) Show a compact execution trace preview.
    section("2) Step Trace Preview");
    print_trace_preview(&result.trace);

    // 3) Run gas profiling on the same trace.
    section("3) Gas Profile");
    let report = profile("deposit_and_withdraw", &result.trace);
    println!("Function   : {}", report.function);
    println!("Total gas* : {} (synthetic)", report.total_gas);
    println!("Hotspots   :");
    for h in &report.hotspots {
        println!("  - {:<18} {:>4.1}%", h.label, h.percent);
    }
    println!("Suggestions:");
    for s in &report.suggestions {
        println!("  - {}", s);
    }

    // 4) Show a simple storage snapshot based on the trace.
    section("4) Storage Snapshot");
    let storage = build_storage_snapshot(&result.trace);
    if storage.is_empty() {
        println!("(no storage writes observed)");
    } else {
        for (key, value) in storage {
            println!("  - {} => {}", key, value.unwrap_or_else(|| "null".into()));
        }
    }

    // 5) Show a migration hint from Solidity → Stylus.
    section("5) Migration Hint (Solidity -> Stylus)");
    print_migration_hint();

    section("Done");
    println!("Narrative: local node -> integration tests -> step trace -> gas insights -> migration guidance.");

    Ok(())
}

fn print_banner() {
    println!("+------------------------------------------------+");
    println!("|      Stylus Debug Suite - End-to-End Demo      |");
    println!("+------------------------------------------------+");
}

fn section(title: &str) {
    println!();
    println!("+------------------------------------------------+");
    println!("| {:<46} |", title);
    println!("+------------------------------------------------+");
}

fn print_trace_preview(trace: &[ExecutionEvent]) {
    let preview_len = trace.len().min(5);
    for ev in &trace[..preview_len] {
        println!(
            "  step {:>3} | {:<8} | stack depth: {} | storage writes: {}",
            ev.step,
            ev.opcode,
            ev.stack.len(),
            ev.storage_diff.len()
        );
    }
    if trace.len() > preview_len {
        println!("  ... {} more steps omitted", trace.len() - preview_len);
    }
}

fn build_storage_snapshot(trace: &[ExecutionEvent]) -> Vec<(String, Option<String>)> {
    use std::collections::BTreeMap;

    let mut map = BTreeMap::<String, Option<String>>::new();
    for ev in trace {
        for change in &ev.storage_diff {
            map.insert(change.key.clone(), change.new.clone());
        }
    }

    map.into_iter().collect()
}

fn print_migration_hint() {
    println!("Source Solidity (examples/demo-contracts/Demo.sol):");
    println!("  - contract DemoVault {{ uint256 public balance; ... }}");
    println!("Mapped Stylus Rust skeleton might look like:");
    println!("  struct DemoVault {{");
    println!("      balance: u128,");
    println!("  }}");
    println!("  impl DemoVault {{");
    println!("      pub fn deposit(&mut self, amount: u128) {{");
    println!("          self.balance += amount;");
    println!("      }}");
    println!("      pub fn withdraw(&mut self, amount: u128) {{");
    println!("          self.balance -= amount;");
    println!("      }}");
    println!("  }}");
}


