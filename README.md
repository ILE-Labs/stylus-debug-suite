# Stylus Debug Suite

A local-first Stylus developer toolkit by **ILE Labs** for the Arbitrum ecosystem.

## Install

```bash
cargo install stylus-debug
```

Once installed, the `stylus-debug` CLI is available globally with three subcommands:

```bash
stylus-debug migrate   # Analyze Solidity contracts for Stylus migration
stylus-debug adapter   # Start the DAP-compatible debug adapter
stylus-debug demo      # Run the Stylus VM demo with report generation
```

## What This Toolkit Does

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

## Quick Start

### Install from crates.io

```bash
cargo install stylus-debug
```

### Or build from source

```bash
git clone https://github.com/ILE-Labs/stylus-debug-suite.git
cd stylus-debug-suite
cargo install --path stylus-debug
```

### Usage

```bash
# Run the full end-to-end demo
stylus-debug demo

# Export a self-contained HTML report
stylus-debug demo --export report.html

# Analyze your own Solidity contracts
stylus-debug migrate path/to/Contract.sol --verbose

# Start the DAP-compatible debug adapter (for IDE integration)
stylus-debug adapter
```

### Development (from source)

```bash
# Build the full workspace
cargo build

# Run tests
cargo test

# Run the demo directly from workspace
cargo run -p stylus-debug -- demo

# Analyze Solidity contracts
cargo run -p stylus-debug -- migrate path/to/Contract.sol --verbose
```

## Workspace Layout

| Crate | Purpose |
|---|---|
| `stylus-debug` | **CLI entrypoint** — unified binary with `migrate`, `adapter`, and `demo` subcommands. |
| `core-engine/debug-engine` | **Core VM**: stack-based simulator and security analyzer engine. |
| `core-engine/test-runtime` | **Assertion Engine**: YAML-driven testing with post-execution validation. |
| `core-engine/gas-profiler` | **Profiler**: Opcode aggregation and Stylus-specific efficiency tips. |
| `core-engine/model` | Shared data structures (events, storage diffs, config). |
| `migration-cli` | **Analyzer**: AST-Based transformation engine for Solidity → Stylus (Rust) guidance. |
| `debug-adapter` | **DAP server**: standalone Debug Adapter Protocol adapter. |
| `examples/demo-contracts` | Example Solidity/Rust contracts + structured test config. |
| `vscode-extension` | Demo VS Code extension shell for IDE integration. |

## What the Demo Shows

When you run `stylus-debug demo`, you will see:

1. **Integration tests** — VM-executed scenarios with structured pass/fail assertion results.
2. **Execution trace** — A live-computed trace showing opcodes, stack state, and storage diffs.
3. **Gas profiler** — Visual distribution of gas usage and tailored optimization suggestions.
4. **Security analysis** — Detections for reentrancy risk, unchecked calls, and CEI pattern violations.
5. **Storage snapshot** — Comparison of initial vs final values for every modified storage slot.
6. **Migration assistant** — Mapping of detected Solidity patterns to Stylus equivalents using real AST parsing.

## License

MIT
