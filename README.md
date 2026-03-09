## Stylus Debug Suite — POC Demo

A local-first Stylus developer toolkit by **ILE Labs** for the Arbitrum ecosystem.

### What This Toolkit Does

| Capability | Description |
|---|---|
| **Integration Test Runner** | **VM-backed** execution of contract scenarios with YAML-defined assertions. |
| **Execution Trace** | Step-by-step opcode trace + memory snapshots, powered by a core simulation engine. |
| **Gas Profiler** | Dynamic hotspot analysis from real traces with 5 contextual optimization tips. |
| **Security Analysis** | Detects 5 patterns including reentrancy risk, unchecked calls, and gas-heavy loops. |
| **Storage Snapshot** | Detailed ledger of storage changes (before/after) with change markers. |
| **Migration Assistant** | **AST-based analysis** of Solidity files using `@solidity-parser/parser` for precise Stylus Rust equivalents. |
| **Interactive Dashboard** | **Glassmorphism-styled** HTML report with a clickable Trace Explorer and Source Inspector. |
| **DAP Protocol Hub** | **Engineering Ready**: Functional DAP server for direct IDE (VS Code) integration. |

### Quick Start

From **WSL** (or Linux) inside the repo:

```bash
# Run the full end-to-end demo
cargo run -p demo-runner --bin stylus-demo

# Export a self-contained HTML report
cargo run -p demo-runner --bin stylus-demo -- --export report.html
```

### Advanced Usage

```bash
# Run specific scenarios
cargo run -p demo-runner --bin stylus-demo -- --scenario overflow_withdraw

# Analyze your own Solidity contracts
cargo run -p migration-cli -- path/to/Contract.sol --verbose

# Run standalone integration tests with assertion output
cargo run -p test-runtime --bin stylus-test
```

### Workspace Layout

| Crate | Purpose |
|---|---|
| `demo-runner` | Main entrypoint — orchestrates testing, analysis, and HTML/JSON reporting. |
| `core-engine/debug-engine` | **Core VM**: stack-based simulator and security analyzer engine. |
| `core-engine/test-runtime` | **Assertion Engine**: YAML-driven testing with post-execution validation. |
| `core-engine/gas-profiler` | **Profiler**: Opcode aggregation and Stylus-specific efficiency tips. |
| `migration-cli` | **Analyzer**: AST-Based transformation engine for Solidity → Stylus (Rust) guidance. |
| `core-engine/model` | Shared data structures (events, storage diffs, config). |
| `examples/demo-contracts` | Example Solidity/Rust contracts + structured test config. |

### What the Demo Shows

When you run `cargo run -p demo-runner --bin stylus-demo`, you will see:

1. **Integration tests** — 3 VM-executed scenarios with structured pass/fail assertion results.
2. **Execution trace** — A live-computed trace showing opcodes, stack state, and storage diffs.
3. **Gas profiler** — Visual distribution of gas usage and 5 tailored optimization suggestions.
4. **Security analysis** — Detections for reentrancy risk, unchecked calls, and CEI pattern violations.
5. **Storage snapshot** — Comparison of initial vs final values for every modified storage slot.
6. **Migration assistant** — Detailed mapping of 11+ detected Solidity patterns to Stylus equivalents, using real Solidity AST parsing.

The full stack demonstrates the technical capability to build a robust, production-grade debugger.
