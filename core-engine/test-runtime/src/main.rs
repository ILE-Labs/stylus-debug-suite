use std::fs;

use test_runtime::{run_scenario, TestConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Minimal integration test runner stub.
    //
    // For the demo it loads a single YAML config and runs the first scenario.

    let yaml = fs::read_to_string("examples/demo-contracts/demo-test.yml")?;
    let config: TestConfig = serde_yaml::from_str(&yaml)?;

    let scenario = config
        .scenarios
        .get(0)
        .cloned()
        .unwrap_or_else(|| "deposit_and_withdraw".into());

    let result = run_scenario(&config, &scenario).await?;
    let json = serde_json::to_string_pretty(&result)?;
    println!("{json}");

    Ok(())
}


