// Re‑export the shared model so downstream crates can depend on `debug-engine`
// without also having to depend on `engine-model` directly.
pub use engine_model::{DebugConfig, ExecutionEvent, MemorySnapshot, StorageChange, Value};

pub mod vm;
pub mod analysis;

use vm::{ScenarioCompiler, ScenarioParams, StylusVm, VmResult};

/// Represents a running debug session backed by the StylusVm.
pub struct DebugSession {
    pub config: DebugConfig,
    vm: StylusVm,
}

impl DebugSession {
    pub fn new(config: DebugConfig) -> Self {
        let vm = StylusVm::new();
        let mut session = Self { config, vm };
        let _ = session.load_scenario_with_params(&ScenarioParams::default());
        session
    }

    pub fn load_scenario_with_params(&mut self, params: &ScenarioParams) -> anyhow::Result<()> {
        let instructions = ScenarioCompiler::compile(&self.config.entrypoint, params)
            .map_err(|e| anyhow::anyhow!(e))?;
        self.vm.load_instructions(instructions);
        Ok(())
    }

    /// Load a WASM contract into the session.
    pub fn load_contract(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.vm.load_contract(bytes)
    }

    /// Advance the VM by one instruction.
    pub fn step(&mut self) -> bool {
        self.vm.step()
    }

    /// Run the contract scenario through the VM and return the execution trace.
    pub fn run(&mut self) -> anyhow::Result<VmResult> {
        while self.step() {}
        Ok(VmResult {
            trace: self.vm.trace().to_vec(),
            final_storage: self.vm.storage().clone(),
            reverted: self.vm.is_reverted(),
            revert_reason: None, // Simplified for now
        })
    }

    pub fn vm(&self) -> &StylusVm {
        &self.vm
    }
}
