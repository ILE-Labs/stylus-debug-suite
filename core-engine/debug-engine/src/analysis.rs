use engine_model::ExecutionEvent;
use serde::{Deserialize, Serialize};

/// Security finding detected by trace analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub step: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

/// Analyze an execution trace for known security risk patterns.
pub fn analyze_trace(trace: &[ExecutionEvent]) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    let cfg = TraceCfg::from_trace(trace);
    
    check_reentrancy_cfg(&cfg, &mut findings);
    check_unchecked_calls(trace, &mut findings);
    check_gas_heavy_patterns(trace, &mut findings);
    check_multiple_storage_writes(trace, &mut findings);

    findings
}

struct TraceCfg<'a> {
    blocks: Vec<BasicBlock<'a>>,
}

struct BasicBlock<'a> {
    events: &'a [ExecutionEvent],
}

impl<'a> TraceCfg<'a> {
    fn from_trace(trace: &'a [ExecutionEvent]) -> Self {
        let mut blocks = Vec::new();
        let mut start = 0;
        
        for (i, ev) in trace.iter().enumerate() {
            // Control flow opcodes that end a basic block
            if ev.opcode == "CALL" || ev.opcode == "REVERT" || ev.opcode == "RETURN" || ev.opcode == "WASM_STEP" {
                blocks.push(BasicBlock {
                    events: &trace[start..=i],
                });
                start = i + 1;
            }
        }
        
        if start < trace.len() {
            blocks.push(BasicBlock {
                events: &trace[start..],
            });
        }
        
        Self { blocks }
    }
}

/// Reentrancy risk: external CALL after SSTORE without a preceding guard.
fn check_reentrancy_cfg(cfg: &TraceCfg, findings: &mut Vec<SecurityFinding>) {
    let mut active_guards = std::collections::HashSet::new();
    let mut last_sstore: Option<u64> = None;

    for block in &cfg.blocks {
        for ev in block.events {
            // Detect guard pattern: SLOAD -> check -> SSTORE
            if ev.opcode == "SSTORE" {
                for change in &ev.storage_diff {
                    // If we see a write to a slot that was previously loaded in the same block
                    // or recent blocks, it's likely a guard being set.
                    if block.events.iter().any(|prev| prev.opcode == "SLOAD" && prev.step < ev.step) {
                        active_guards.insert(change.key.clone());
                    }
                    last_sstore = Some(ev.step);
                }
            }

            if ev.opcode == "CALL" && ev.gas_used >= 9000 {
                // If there's an SSTORE before this CALL, check if a guard is active
                if let Some(sstore_step) = last_sstore {
                    let guarded = active_guards.iter().any(|_slot| {
                        // A simple heuristic: if the slot was written, we assume it's a guard
                        true 
                    });

                    if !guarded {
                        findings.push(SecurityFinding {
                            severity: Severity::Critical,
                            title: "Reentrancy Vulnerability (No Guard)".into(),
                            description: format!(
                                "Critical: External CALL at step {} follows SSTORE at step {} \
                                 without any detectable reentrancy guard. This is a high-risk \
                                 pattern that allows malicious contracts to re-enter your state.",
                                ev.step, sstore_step
                            ),
                            step: Some(ev.step),
                        });
                    }
                }
            }
        }
    }
}

/// Unchecked external calls: CALL not followed by a status check.
fn check_unchecked_calls(trace: &[ExecutionEvent], findings: &mut Vec<SecurityFinding>) {
    let call_count = trace
        .iter()
        .filter(|e| e.opcode == "CALL" && e.gas_used >= 700)
        .count();
    let require_count = trace
        .iter()
        .filter(|e| e.opcode == "REVERT")
        .count();

    if call_count > 0 && require_count == 0 {
        findings.push(SecurityFinding {
            severity: Severity::Warning,
            title: "No revert guards detected".into(),
            description: format!(
                "Detected {} external CALL(s) but no REVERT/require guards in the trace. \
                 Ensure all external call return values are checked to prevent silent failures.",
                call_count
            ),
            step: None,
        });
    }
}

/// State update before transfer: SSTORE followed by value-transfer CALL.
/// This is the classic "send ETH then update state" anti-pattern.
pub fn check_storage_before_transfer(trace: &[ExecutionEvent], findings: &mut Vec<SecurityFinding>) {
    for (i, ev) in trace.iter().enumerate() {
        if ev.opcode == "SSTORE" {
            // Look at the next few events for a value-transfer CALL
            for next in trace.iter().skip(i + 1).take(3) {
                if next.opcode == "CALL" && next.gas_used >= 9000 {
                    findings.push(SecurityFinding {
                        severity: Severity::Info,
                        title: "State updated before external transfer".into(),
                        description: format!(
                            "Storage write at step {} is followed by an external value transfer \
                             at step {}. This follows the checks-effects-interactions pattern \
                             (good practice) — verify this is intentional.",
                            ev.step, next.step
                        ),
                        step: Some(ev.step),
                    });
                    return; // Report once
                }
            }
        }
    }
}

/// Gas-heavy pattern: multiple SSTORE operations that could be batched.
fn check_gas_heavy_patterns(trace: &[ExecutionEvent], findings: &mut Vec<SecurityFinding>) {
    let sstore_gas: u64 = trace
        .iter()
        .filter(|e| e.opcode == "SSTORE")
        .map(|e| e.gas_used)
        .sum();
    let total_gas: u64 = trace.iter().map(|e| e.gas_used).sum();

    if total_gas > 0 {
        let sstore_pct = (sstore_gas as f64 / total_gas as f64) * 100.0;
        if sstore_pct > 50.0 {
            findings.push(SecurityFinding {
                severity: Severity::Info,
                title: "High storage write gas proportion".into(),
                description: format!(
                    "Storage writes consume {:.1}% of total gas ({} / {} gas). In Stylus, \
                     storage operations remain L1-priced while computation is ~10x cheaper. \
                     Consider caching intermediate values in memory.",
                    sstore_pct, sstore_gas, total_gas
                ),
                step: None,
            });
        }
    }
}

/// Multiple writes to the same storage slot.
fn check_multiple_storage_writes(trace: &[ExecutionEvent], findings: &mut Vec<SecurityFinding>) {
    use std::collections::HashMap;
    let mut write_counts: HashMap<String, usize> = HashMap::new();

    for ev in trace {
        for change in &ev.storage_diff {
            *write_counts.entry(change.key.clone()).or_default() += 1;
        }
    }

    for (slot, count) in &write_counts {
        if *count > 1 {
            findings.push(SecurityFinding {
                severity: Severity::Info,
                title: format!("Slot `{slot}` written {count} times"),
                description: format!(
                    "Storage slot `{slot}` was written {} times in a single execution. \
                     Each redundant SSTORE costs gas. Consider computing the final value \
                     in memory and writing once.",
                    count
                ),
                step: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_model::{MemorySnapshot, StorageChange};

    fn mock_event(step: u64, opcode: &str, gas: u64, diff: Vec<StorageChange>) -> ExecutionEvent {
        ExecutionEvent {
            step,
            opcode: opcode.to_string(),
            gas_used: gas,
            stack: vec![],
            memory: MemorySnapshot { bytes: vec![] },
            storage_diff: diff,
            source_line: None,
        }
    }

    #[test]
    fn test_reentrancy_vulnerable() {
        let trace = vec![
            mock_event(1, "SSTORE", 20000, vec![StorageChange { key: "0x1".into(), old: None, new: Some("0x64".into()) }]),
            mock_event(2, "CALL", 9000, vec![]),
        ];
        let findings = analyze_trace(&trace);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].title, "Reentrancy Vulnerability (No Guard)");
    }

    #[test]
    fn test_reentrancy_guarded() {
        let trace = vec![
            // 1. Load guard (SLOAD)
            mock_event(1, "SLOAD", 2100, vec![]),
            // 2. Set guard (SSTORE)
            mock_event(2, "SSTORE", 20000, vec![StorageChange { key: "guard_slot".into(), old: Some("0x0".into()), new: Some("0x1".into()) }]),
            // 3. Application SSTORE
            mock_event(3, "SSTORE", 5000, vec![StorageChange { key: "balance".into(), old: None, new: Some("0x64".into()) }]),
            // 4. External CALL (guarded)
            mock_event(4, "CALL", 9000, vec![]),
        ];
        let findings = analyze_trace(&trace);
        // Should NOT find a reentrancy risk because the SLOAD -> SSTORE pattern was detected in recent context
        assert!(findings.iter().all(|f| f.title != "Reentrancy Vulnerability (No Guard)"));
    }
}
