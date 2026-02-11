## Stylus Debug Suite (Demo Architecture)

This repository is a **demo architecture** for a local‑first Stylus developer toolkit.
It is organized as a modular Rust + TypeScript workspace, matching the grant demo narrative:

- **Run integration tests**
- **Step‑debug Stylus contracts**
- **Inspect storage/state**
- **Profile gas usage**
- **Prototype Solidity → Stylus migration**

### Workspace Layout

- `demo-runner` – **one-command grant demo** (`stylus-demo`) that orchestrates tests, trace, gas, storage, and migration hint
- `core-engine/debug-engine` – Rust core execution + tracing engine, CLI debug runner
- `core-engine/gas-profiler` – Rust gas profiling and trace analysis
- `core-engine/test-runtime` – Rust integration test runtime
- `core-engine/model` – shared data structures (`ExecutionEvent`, storage diffs, config)
- `debug-adapter` – Rust Debug Adapter Protocol (DAP‑style) server
- `vscode-extension` – VS Code extension (TypeScript) UI shell
- `migration-cli` – Rust CLI for Solidity → Stylus migration experiments
- `examples/demo-contracts` – Example Stylus contracts + demo scenario config

Each module is intended to be independently usable and locally runnable.

### Quick Grant Demo (high‑level)

From WSL inside the repo:

```bash
cargo run -p demo-runner --bin stylus-demo
```

This walks through, in one narrative:

1. Run an integration test scenario (`deposit_and_withdraw`) against the demo vault.
2. Show a human‑readable execution trace preview.
3. Generate a gas profiling report with hotspots and optimization suggestions.
4. Summarize storage changes as a storage snapshot (for a future VS Code pane).
5. Print a Solidity → Stylus migration hint for the paired `DemoVault` contract.



