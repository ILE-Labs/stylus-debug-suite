use std::fs;

use test_runtime::{run_scenario, TestConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let yaml = fs::read_to_string("examples/demo-contracts/demo-test.yml")?;
    let config: TestConfig = serde_yaml::from_str(&yaml)?;

    println!("╔══════════════════════════════════════════════════╗");
    println!("║           Stylus Test Runner                     ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();

    let mut passed = 0usize;
    let mut failed = 0usize;

    for scenario_cfg in &config.scenarios {
        let result = run_scenario(&config, scenario_cfg).await?;
        let status = if result.passed { "PASS ✓" } else { "FAIL ✗" };
        println!("  ┌ [{status}] {}", result.name);
        println!("  │ {}", result.description);

        // Print assertion results
        for ar in &result.assertion_results {
            let mark = if ar.passed { "✓" } else { "✗" };
            println!("  │   {mark} {} (expected: {}, actual: {})", ar.assertion, ar.expected, ar.actual);
        }

        if let Some(ref reason) = result.failure_reason {
            println!("  │ Reason: {reason}");
        }
        println!("  └─────────────────────────────────────────────────");
        println!();

        if result.passed {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    println!("  {passed} passed, {failed} failed, {} total", passed + failed);

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
