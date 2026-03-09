# Stylus Debug Suite

A comprehensive toolkit for Arbitrum Stylus (Rust-based smart contracts) development, debugging, and migration.

Built by [ILE Labs](https://ilelabs.com).

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [CLI Reference](#cli-reference)
- [Library Usage](#library-usage)
- [Feature Details](#feature-details)
- [Architecture](#architecture)
- [License](#license)

## Overview

The Stylus Debug Suite provides essential tools for the next generation of smart contract development on Arbitrum Stylus:

- **High-Fidelity VM Debugging**: Real `wasmtime` execution with DWARF source mapping.
- **Smart Migration**: AST-based Solidity-to-Rust conversion guidance.
- **Security First**: Automated trace analysis for reentrancy, CEI violations, and gas-heavy patterns.
- **Actionable Profiling**: Deep gas hotspot analysis with contextual optimization tips.

## Installation

### Globally via crates.io

```bash
cargo install stylus-debug
```

### From source

```bash
git clone https://github.com/ILE-Labs/stylus-debug-suite.git
cd stylus-debug-suite
cargo install --path stylus-debug
```

## Quick Start

Run the end-to-end demo to see the suite in action:

```bash
stylus-debug demo
```

Export a visual report:

```bash
stylus-debug demo --export report.html
```

## CLI Reference

The unified `stylus-debug` binary provides the following commands:

### `stylus-debug migrate [INPUT]`

Analyzes Solidity source files and provides high-fidelity Rust/Stylus equivalents.

- **Arguments**:
  - `INPUT`: Path to the `.sol` file (optional, defaults to `examples/demo-contracts/Demo.sol`).
- **Options**:
  - `--format <format>`: Output format (`text` or `json`).
  - `--verbose`: Shows matched source lines and line numbers.

### `stylus-debug adapter`

Starts a Debug Adapter Protocol (DAP) server on `stdio`.

- **Options**:
  - `--port <port>`: Port for the DAP server (currently `stdio` by default).

### `stylus-debug demo`

Runs a pre-defined debug scenario through the Stylus VM.

- **Options**:
  - `--export <path>`: Path to export the HTML report (e.g., `report.html`).

## Library Usage

You can also use the suite's core logic as a library in your own Rust projects. Add the following to your `Cargo.toml`:

```toml
[dependencies]
stylus-debug = "0.1.0"
```

### Example: Running a Debug Session

```rust
use stylus_debug::{DebugSession, DebugConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = DebugConfig {
        contract_path: "path/to/contract.rs".into(),
        entrypoint: "my_function".into(),
        breakpoints: vec!["contract.rs:42".into()],
    };

    let mut session = DebugSession::new(config);
    
    // Step-by-step execution
    while session.step() {
        println!("PC: {}", session.vm().current_ptr());
    }

    Ok(())
}
```

## Feature Details

### 🛡️ Security Analysis
Automatically detects 5 core vulnerability patterns in execution traces:
- **Reentrancy**: External `CALL` after `SSTORE` without a guard.
- **Unchecked Calls**: `CALL` operations not followed by status checks.
- **CEI Violations**: Non-standard Checks-Effects-Interactions flows.
- **Gas Inefficiencies**: Redundant `SSTORE` and large loops.

### ⛽ Gas Profiling
Provides a breakdown of gas consumption by opcode and contract function, highlighting expensive hotspots and offering Stylus-specific optimizations (e.g., caching storage in memory).

### 🔍 DWARF Debugging
Leverages `gimli` to parse DWARF debug information from WASM binaries, enabling accurate source-to-PC mapping and a real-world debugging experience.

## Architecture

The suite is modularized into several crates:

- `stylus-debug`: Unified CLI entrypoint.
- `debug-engine`: Core VM execution engine and analyzer.
- `engine-model`: Shared data structures.
- `gas-profiler`: Gas analysis and optimization.

## License

This project is licensed under the [MIT License](LICENSE).
