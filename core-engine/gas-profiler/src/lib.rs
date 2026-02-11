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
    pub percent: f32,
}

/// Very rough, placeholder gas profiler that derives a synthetic gas report
/// from an execution trace. The real implementation would plug into Stylus
/// opcode pricing.
pub fn profile(function: &str, trace: &[ExecutionEvent]) -> GasReport {
    let total_gas = trace.len() as u64 * 100;

    let hotspots = vec![
        GasHotspot {
            label: "storage_write".into(),
            percent: 35.0,
        },
        GasHotspot {
            label: "loop_iteration".into(),
            percent: 22.0,
        },
    ];

    let suggestions = vec![
        "Cache storage reads outside of tight loops".into(),
        "Batch storage writes where possible".into(),
    ];

    GasReport {
        function: function.into(),
        total_gas,
        hotspots,
        suggestions,
    }
}


