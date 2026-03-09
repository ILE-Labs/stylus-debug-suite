use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use debug_engine::vm::ScenarioParams;
use debug_engine::{DebugConfig, DebugSession, ExecutionEvent};

pub mod assertions;
use assertions::{check_assertions, Assertion, AssertionResult};

/// Top-level test configuration loaded from YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub network: String,
    pub fork: Option<String>,
    pub contracts: Vec<String>,
    pub scenarios: Vec<ScenarioConfig>,
}

/// A single scenario definition from the YAML config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioConfig {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub params: ScenarioParamsConfig,
    #[serde(default)]
    pub assertions: Vec<Assertion>,
}

/// Scenario parameters from YAML.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScenarioParamsConfig {
    pub deposit_amount: Option<u128>,
    pub withdraw_amount: Option<u128>,
}

impl From<&ScenarioParamsConfig> for ScenarioParams {
    fn from(cfg: &ScenarioParamsConfig) -> Self {
        ScenarioParams {
            deposit_amount: cfg.deposit_amount,
            withdraw_amount: cfg.withdraw_amount,
        }
    }
}

/// Result of running a test scenario through the VM + assertion engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub description: String,
    pub passed: bool,
    pub failure_reason: Option<String>,
    pub trace: Vec<ExecutionEvent>,
    pub final_storage: BTreeMap<String, String>,
    pub assertion_results: Vec<AssertionResult>,
}

/// Run a single scenario from its config, checking assertions against VM output.
pub async fn run_scenario(config: &TestConfig, scenario_cfg: &ScenarioConfig) -> anyhow::Result<TestResult> {
    let contract = config
        .contracts
        .first()
        .cloned()
        .unwrap_or_else(|| "examples/demo-contracts/vault.rs".into());

    let debug_config = DebugConfig {
        contract_path: contract,
        entrypoint: scenario_cfg.name.clone(),
        breakpoints: vec![],
    };

    let params: ScenarioParams = (&scenario_cfg.params).into();
    let mut session = DebugSession::new(debug_config);
    session.load_scenario_with_params(&params)?;
    let vm_result = session.run()?;

    // Run assertions against post-execution state.
    let assertion_results = check_assertions(
        &scenario_cfg.assertions,
        &vm_result.final_storage,
        vm_result.reverted,
    );

    let all_assertions_passed = assertion_results.iter().all(|a| a.passed);

    let failure_reason = if !all_assertions_passed {
        let failed: Vec<String> = assertion_results
            .iter()
            .filter(|a| !a.passed)
            .map(|a| format!("{}: expected {}, got {}", a.assertion, a.expected, a.actual))
            .collect();
        Some(failed.join("; "))
    } else if vm_result.reverted && scenario_cfg.assertions.is_empty() {
        vm_result.revert_reason.clone().map(|r| format!("transaction reverted: {r}"))
    } else {
        None
    };

    let passed = if scenario_cfg.assertions.is_empty() {
        !vm_result.reverted
    } else {
        all_assertions_passed
    };

    Ok(TestResult {
        name: scenario_cfg.name.clone(),
        description: scenario_cfg.description.clone(),
        passed,
        failure_reason,
        trace: vm_result.trace,
        final_storage: vm_result.final_storage,
        assertion_results,
    })
}
