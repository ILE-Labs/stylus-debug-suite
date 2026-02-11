## Stylus Debug Suite – Demo Architecture

This document maps the **grant demo narrative** onto the actual modules in this repository.

### Layers

- **VS Code Extension (`vscode-extension`)**
  - TypeScript UI shell.
  - Exposes command `Stylus: Start Debug Session`.
  - Spawns the Rust debug adapter (`stylus-dap`) as a child process.

- **Local Debug Engine / Orchestrator**
  - `core-engine/debug-engine`
    - Library: execution tracing model (`ExecutionEvent`, storage diffs, memory snapshots).
    - Binary `stylus-debug`: CLI debug runner that prints a JSON execution trace and a gas report.
  - `debug-adapter`
    - Binary `stylus-dap`: JSON‑over‑stdio, DAP‑style server that wraps `debug-engine`.

- **Stylus Node Adapter (Execution Layer)**
  - Not fully implemented in the demo; the `DebugSession::run` API is shaped so it can
    later wrap real Stylus execution and node interaction.

- **Supporting Modules**
  - `core-engine/test-runtime`
    - Library + `stylus-test` binary for integration‑style scenario execution.
  - `core-engine/gas-profiler`
    - Consumes execution traces and emits a high‑level gas usage report.
  - `migration-cli`
    - `stylus-migrate` binary; placeholder pipeline for Solidity → Stylus Rust skeletons.

### Demo Scenario Walkthrough

1. **Deploy Stylus contract locally (conceptual)**
   - The contract stub lives in `examples/demo-contracts/vault.rs`.
   - In a full implementation, this would be compiled and deployed to a local Stylus dev node.

2. **Run integration test**
   - Config file: `examples/demo-contracts/demo-test.yml`.
   - Command: `cargo run -p test-runtime --bin stylus-test`.
   - The test runtime loads the YAML config, runs the `deposit_and_withdraw` scenario,
     and prints a JSON `TestResult` (including the execution trace).

3. **Pause at breakpoint / step‑debug contracts**
   - The `debug-engine` exposes a `DebugSession` API and emits per‑step `ExecutionEvent`s.
   - The `debug-adapter` crate provides a JSON‑RPC‑like, DAP‑style server that VS Code
     (or tools) can speak to over stdio.
   - The VS Code extension demonstrates how an editor command can spin up the adapter
     and send `initialize`, `launch`, and `step` requests.

4. **Inspect storage / state**
   - `ExecutionEvent` carries `storage_diff` and `memory` snapshots, which can be surfaced
     by the debug adapter and rendered in the VS Code UI.
   - The demo emits this information as structured JSON for inspection.

5. **View gas report**
   - The `gas-profiler` module consumes a trace and emits a `GasReport` with hotspots
     and optimization suggestions.
   - The `stylus-debug` CLI wires this in and prints both the raw trace and the gas report.

6. **Generate migration suggestion**
   - The `migration-cli` tool (`stylus-migrate`) takes a Solidity file path and prints
     a pseudo‑Rust Stylus skeleton to illustrate the intended migration flow.

Each module is **independently runnable**, and together they form a coherent, local‑first
Stylus debugging and tooling story suitable for a grant demo.


