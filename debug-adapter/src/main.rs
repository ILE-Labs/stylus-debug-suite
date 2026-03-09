use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use anyhow::Result;
use debug_engine::{DebugConfig, DebugSession, ExecutionEvent, StorageChange};
use gas_profiler::profile;
use serde::{Deserialize, Serialize};

/// DAP‑shaped protocol message for requests.
#[derive(Debug, Deserialize)]
struct ProtocolRequest {
    seq: i64,
    #[serde(rename = "type")]
    type_field: String, // expect "request"
    command: String,
    #[serde(default)]
    arguments: serde_json::Value,
}

/// DAP‑shaped protocol response envelope.
#[derive(Debug, Serialize)]
struct ProtocolResponse<T> {
    seq: i64,
    #[serde(rename = "type")]
    type_field: &'static str, // "response"
    request_seq: i64,
    success: bool,
    command: String,
    body: T,
}

#[derive(Debug, Serialize, Clone)]
struct SourceLocation {
    file: String,
    line: i64,
    column: i64,
}

struct PcToSourceMap {
    map: HashMap<u32, SourceLocation>,
}

impl PcToSourceMap {
    fn new() -> Self {
        Self { map: HashMap::new() }
    }

    fn lookup(&self, pc: u32) -> Option<&SourceLocation> {
        self.map.get(&pc)
    }
}

fn parse_dwarf(wasm_path: &str) -> Result<PcToSourceMap> {
    use object::{Object, ObjectSection};
    use std::fs;

    let bin_data = fs::read(wasm_path)?;
    let obj_file = object::File::parse(&*bin_data)?;
    
    let load_section = |id: gimli::SectionId| -> Result<DefaultCow, gimli::Error> {
        let name = id.name();
        match obj_file.section_by_name(name) {
            Some(section) => Ok(section.cow_data().into()),
            None => Ok(DefaultCow::new()),
        }
    };

    let drown_sections = gimli::Dwarf::load(&load_section)?;
    let mut pc_to_source = PcToSourceMap::new();

    let mut iter = drown_sections.units();
    while let Some(header) = iter.next()? {
        let unit = drown_sections.unit(header)?;
        if let Some(program) = unit.line_program.clone() {
            let mut rows = program.rows();
            while let Some((header, row)) = rows.next_row()? {
                if row.end_sequence() {
                    continue;
                }
                if let Some(file_entry) = row.file(header) {
                    let file_name = drown_sections.attr_string(&unit, file_entry.path_name())?
                        .to_string_lossy()
                        .into_owned();
                    pc_to_source.map.insert(row.address() as u32, SourceLocation {
                        file: file_name,
                        line: row.line().map(|l| l.get() as i64).unwrap_or(0),
                        column: match row.column() {
                            gimli::ColumnType::LeftEdge => 0,
                            gimli::ColumnType::Column(c) => c.get() as i64,
                        },
                    });
                }
            }
        }
    }

    Ok(pc_to_source)
}

type DefaultCow = std::borrow::Cow<'static, [u8]>;

#[tokio::main]
async fn main() -> Result<()> {
    // For the demo, we implement a JSON‑over‑stdio loop that is shaped like
    // DAP messages (`seq`, `type`, `command`, `arguments`) but omits HTTP
    // headers and full DAP surface area.
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut active_session: Option<DebugSession> = None;
    let mut last_trace: Option<Vec<ExecutionEvent>> = None;
    let mut pc_map: PcToSourceMap = PcToSourceMap::new();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let req: ProtocolRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(err) => {
                let resp = ProtocolResponse {
                    seq: 0,
                    type_field: "response",
                    request_seq: 0,
                    success: false,
                    command: "unknown".into(),
                    body: format!("invalid request: {err}"),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                stdout.flush()?;
                continue;
            }
        };

        let command = req.command.clone();

        match command.as_str() {
            "initialize" => {
                let body = serde_json::json!({
                    "supportsConfigurationDoneRequest": true,
                    "supportsEvaluateForHovers": false,
                    "supportsStepBack": false,
                    "supportsRestartRequest": true,
                });
                let resp = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body,
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
            "launch" => {
                let contract_path = req.arguments.get("contractPath").and_then(|v| v.as_str()).unwrap_or("examples/demo-contracts/vault.rs").to_string();
                let wasm_path = req.arguments.get("wasmPath").and_then(|v| v.as_str()).unwrap_or("examples/demo-contracts/vault.wasm").to_string();
                
                // Load real DWARF mapping if WASM exists
                if let Ok(map) = parse_dwarf(&wasm_path) {
                    pc_map = map;
                }

                let config = DebugConfig {
                    contract_path: contract_path.clone(),
                    entrypoint: "deposit_and_withdraw".into(),
                    breakpoints: vec![],
                };
                active_session = Some(DebugSession::new(config));

                let resp = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body: serde_json::json!({ "status": "launched", "contract": contract_path, "dwarf": !pc_map.map.is_empty() }),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
            "setBreakpoints" => {
                let resp = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body: serde_json::json!({ "breakpoints": [] }),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
            "next" => {
                let mut success = false;
                if let Some(session) = &mut active_session {
                    success = session.step();
                }
                let resp = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body: serde_json::json!({ "stepped": success }),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
            "stackTrace" => {
                let mut frames = Vec::new();
                if let Some(session) = &active_session {
                    let vm = session.vm();
                    let pc = vm.current_ptr() as u32; // Currently ptr is scenario index, will be real PC soon
                    
                    let loc = pc_map.lookup(pc);
                    let line = loc.map(|l| l.line).unwrap_or(10 + pc as i64); // Fallback to heuristic
                    let file = loc.map(|l| l.file.as_str()).unwrap_or("vault.rs");

                    frames.push(serde_json::json!({
                        "id": 1,
                        "name": "contract_execution",
                        "source": { "name": file, "path": format!("examples/demo-contracts/{file}") },
                        "line": line,
                        "column": loc.map(|l| l.column).unwrap_or(0)
                    }));
                } else {
                    let resp = ProtocolResponse {
                        seq: req.seq,
                        type_field: "response",
                        request_seq: req.seq,
                        success: false,
                        command: "stackTrace".into(),
                        body: "Debug session not initialized".into(),
                    };
                    writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                    continue;
                }
                let resp = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body: serde_json::json!({ "stackFrames": frames, "totalFrames": frames.len() }),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
            "scopes" => {
                let body = serde_json::json!({
                    "scopes": [
                        { "name": "Stack", "variablesReference": 1001, "expensive": false },
                        { "name": "Storage", "variablesReference": 1002, "expensive": true }
                    ]
                });
                let resp = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body,
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
            "variables" => {
                let ref_id = req.arguments.get("variablesReference").and_then(|v| v.as_i64()).unwrap_or(0);
                let mut vars = Vec::new();
                if let Some(session) = &active_session {
                    let vm = session.vm();
                    match ref_id {
                        1001 => { // Stack variables
                            for (i, val) in vm.stack().iter().enumerate() {
                                vars.push(serde_json::json!({
                                    "name": format!("stack[{i}]"),
                                    "value": val,
                                    "type": "U256"
                                }));
                            }
                        }
                        1002 => { // Storage variables
                            for (key, val) in vm.storage() {
                                vars.push(serde_json::json!({
                                    "name": key,
                                    "value": val,
                                    "type": "StorageSlot"
                                }));
                            }
                        }
                        _ => {}
                    }
                } else {
                    let resp = ProtocolResponse {
                        seq: req.seq,
                        type_field: "response",
                        request_seq: req.seq,
                        success: false,
                        command: "variables".into(),
                        body: "Debug session not initialized".into(),
                    };
                    writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                    continue;
                }
                let resp = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body: serde_json::json!({ "variables": vars }),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
            "stylusStorage" => {
                let mut storage = Vec::new();
                if let Some(session) = &active_session {
                    for (key, val) in session.vm().storage() {
                        storage.push(serde_json::json!({ "key": key, "value": val }));
                    }
                }
                let resp = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body: serde_json::json!({ "storage": storage }),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
            "stylusGas" => {
                let mut gas_info = serde_json::json!({ "consumed": 0, "remaining": 0 });
                if let Some(session) = &active_session {
                    let trace = session.vm().trace();
                    let consumed: u64 = trace.iter().map(|e| e.gas_used).sum();
                    gas_info = serde_json::json!({
                        "consumed": consumed,
                        "remaining": 4_000_000 - consumed // Default Arbitrum block gas limit placeholder
                    });
                }
                let resp = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body: gas_info,
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
            "disconnect" => {
                let body = serde_json::json!({ "status": "bye" });
                let resp = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body,
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                break;
            }
            _ => {
                let resp = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: false,
                    command,
                    body: serde_json::json!({ "error": "unknown command" }),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
        }

        stdout.flush()?;
    }

    Ok(())
}

fn build_storage_view(trace: Option<&Vec<ExecutionEvent>>) -> StorageView {
    let mut map: HashMap<String, Option<String>> = HashMap::new();

    if let Some(events) = trace {
        for event in events {
            for StorageChange { key, old: _, new } in &event.storage_diff {
                map.insert(key.clone(), new.clone());
            }
        }
    }

    let slots = map
        .into_iter()
        .map(|(key, value)| StorageSlotView { key, value })
        .collect();

    StorageView { slots }
}



