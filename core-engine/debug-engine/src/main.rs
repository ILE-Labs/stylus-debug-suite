use debug_engine::{DebugConfig, DebugSession};
use gas_profiler::profile;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Minimal CLI debug runner stub.
    //
    // For the demo, this accepts no arguments and runs a canned configuration
    // against a demo contract path and entrypoint, then prints a JSON trace.

    let config = DebugConfig {
        contract_path: "examples/demo-contracts/vault.rs".into(),
        entrypoint: "deposit_and_withdraw".into(),
        breakpoints: vec!["vault.rs:42".into()],
    };

    let mut session = DebugSession::new(config);
    let vm_result = session.run()?;

    // Emit raw execution trace.
    let json = serde_json::to_string_pretty(&vm_result.trace)?;
    println!("# Execution trace");
    println!("{json}");

    // Emit a simple gas profile derived from the same trace.
    let report = profile("deposit_and_withdraw", &vm_result.trace);
    let report_json = serde_json::to_string_pretty(&report)?;
    println!("\n# Gas report");
    println!("{report_json}");


    Ok(())
}

