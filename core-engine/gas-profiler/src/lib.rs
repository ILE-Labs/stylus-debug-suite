use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use engine_model::ExecutionEvent;

/// Aggregate gas usage by logical hotspot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasReport {
    pub function: String,
    pub total_gas: u64,
    pub hotspots: Vec<GasHotspot>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasHotspot {
    pub label: String,
    pub gas: u64,
    pub percent: f32,
}

/// This profiles the execution trace by aggregating gas usage from the VM's fuel tracking.
/// It maps captured fuel consumption back to hotspots for optimization analysis.
pub fn profile(function: &str, trace: &[ExecutionEvent]) -> GasReport {
    let total_gas: u64 = trace.iter().map(|ev| ev.gas_used).sum();

    // Aggregate gas per opcode category.
    let mut by_opcode: HashMap<String, u64> = HashMap::new();
    for ev in trace {
        *by_opcode.entry(ev.opcode.clone()).or_default() += ev.gas_used;
    }

    // Sort hotspots by gas cost descending.
    let mut hotspots: Vec<GasHotspot> = by_opcode
        .into_iter()
        .map(|(label, gas)| {
            let percent = if total_gas > 0 {
                (gas as f64 / total_gas as f64 * 100.0) as f32
            } else {
                0.0
            };
            GasHotspot { label, gas, percent }
        })
        .collect();
    hotspots.sort_by(|a, b| b.gas.cmp(&a.gas));

    // Generate context-aware optimization suggestions based on what was detected.
    let mut suggestions = Vec::new();

    let sstore_count = trace.iter().filter(|e| e.opcode == "SSTORE").count();
    let sload_count = trace.iter().filter(|e| e.opcode == "SLOAD").count();
    let call_count = trace.iter().filter(|e| e.opcode == "CALL").count();
    let log_count = trace.iter().filter(|e| e.opcode == "LOG").count();

    if sstore_count > 1 {
        suggestions.push(format!(
            "Detected {sstore_count} SSTORE operations. Batch storage writes where possible \
             to reduce per-write overhead (each cold SSTORE costs ~20,000 gas)."
        ));
    }
    if sload_count > 0 {
        suggestions.push(format!(
            "Detected {sload_count} SLOAD operation(s). Cache storage reads in local variables \
             when the same slot is read more than once to avoid redundant cold reads (~2,100 gas each)."
        ));
    }
    if call_count > 2 {
        suggestions.push(format!(
            "Detected {call_count} external CALL operations. Consider consolidating external \
             calls to reduce base call overhead (~700 gas per call) and re-entrancy surface."
        ));
    }
    if log_count > 0 {
        suggestions.push(format!(
            "Detected {log_count} LOG event(s). While events are cheap relative to storage, \
             avoid emitting large event payloads in loops to keep gas predictable."
        ));
    }

    // Always include a general Stylus-specific tip.
    suggestions.push(
        "Stylus tip: prefer in-memory computation over repeated storage access. \
         Stylus WASM execution is ~10x cheaper than EVM per CPU cycle, but storage \
         costs remain L1-equivalent."
            .into(),
    );

    GasReport {
        function: function.into(),
        total_gas,
        hotspots,
        suggestions,
    }
}
