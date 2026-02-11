// Re‑export the shared model so downstream crates can depend on `debug-engine`
// without also having to depend on `engine-model` directly.
pub use engine_model::{DebugConfig, ExecutionEvent, MemorySnapshot, StorageChange, Value};

/// Represents a running debug session.
pub struct DebugSession {
    pub config: DebugConfig,
}

impl DebugSession {
    /// Create a new debug session from a configuration.
    pub fn new(config: DebugConfig) -> Self {
        Self { config }
    }

    /// Run the contract and yield a synthetic execution trace.
    ///
    /// For the demo, this is stubbed out to emit a handful of fake events;
    /// the API is shaped to later wrap real Stylus execution.
    pub fn run(&self) -> anyhow::Result<Vec<ExecutionEvent>> {
        let mut events = Vec::new();

        events.push(ExecutionEvent {
            step: 0,
            opcode: "PUSH".into(),
            stack: vec![Value { hex: "0x01".into() }],
            memory: MemorySnapshot { bytes: vec![] },
            storage_diff: vec![],
        });

        events.push(ExecutionEvent {
            step: 1,
            opcode: "SSTORE".into(),
            stack: vec![Value { hex: "0x01".into() }, Value { hex: "0x10".into() }],
            memory: MemorySnapshot { bytes: vec![] },
            storage_diff: vec![StorageChange {
                key: "slot_0".into(),
                old: None,
                new: Some("0x10".into()),
            }],
        });

        Ok(events)
    }
}


