use serde::{Deserialize, Serialize};

/// Core execution event emitted by the Stylus debug engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub step: u64,
    pub opcode: String,
    /// Gas consumed by this individual operation.
    pub gas_used: u64,
    pub stack: Vec<Value>,
    pub memory: MemorySnapshot,
    pub storage_diff: Vec<StorageChange>,
    pub source_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChange {
    pub key: String,
    pub old: Option<String>,
    pub new: Option<String>,
}

/// High‑level debug session configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub contract_path: String,
    pub entrypoint: String,
    /// Optional breakpoints expressed as "file:line" or symbolic form.
    pub breakpoints: Vec<String>,
}
