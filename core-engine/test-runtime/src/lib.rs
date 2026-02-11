use serde::{Deserialize, Serialize};

use debug_engine::{DebugConfig, DebugSession, ExecutionEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub network: String,
    pub fork: Option<String>,
    pub contracts: Vec<String>,
    pub scenarios: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub trace: Vec<ExecutionEvent>,
}

/// Run a single, config‑driven scenario.
pub async fn run_scenario(config: &TestConfig, scenario: &str) -> anyhow::Result<TestResult> {
    let contract = config
        .contracts
        .get(0)
        .cloned()
        .unwrap_or_else(|| "examples/demo-contracts/vault.rs".into());

    let debug_config = DebugConfig {
        contract_path: contract,
        entrypoint: scenario.to_string(),
        breakpoints: vec![],
    };

    let session = DebugSession::new(debug_config);
    let trace = session.run()?;

    Ok(TestResult {
        name: scenario.to_string(),
        passed: true,
        trace,
    })
}


