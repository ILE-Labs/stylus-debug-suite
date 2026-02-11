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

/// High‑level storage viewer payload intended for a VS Code "Storage" pane.
#[derive(Debug, Serialize)]
struct StorageView {
    slots: Vec<StorageSlotView>,
}

#[derive(Debug, Serialize)]
struct StorageSlotView {
    key: String,
    value: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // For the demo, we implement a JSON‑over‑stdio loop that is shaped like
    // DAP messages (`seq`, `type`, `command`, `arguments`) but omits HTTP
    // headers and full DAP surface area.
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut active_session: Option<DebugSession> = None;
    let mut last_trace: Option<Vec<ExecutionEvent>> = None;

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
                    "adapterId": "stylus-debug",
                    "capabilities": {
                        "supportsConfigurationDoneRequest": true,
                        "supportsEvaluateForHovers": false
                    }
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
                let contract_path = req
                    .arguments
                    .get("contractPath")
                    .and_then(|v| v.as_str())
                    .unwrap_or("examples/demo-contracts/vault.rs")
                    .to_string();
                let entrypoint = req
                    .arguments
                    .get("entrypoint")
                    .and_then(|v| v.as_str())
                    .unwrap_or("deposit_and_withdraw")
                    .to_string();

                let config = DebugConfig {
                    contract_path,
                    entrypoint,
                    breakpoints: vec![],
                };
                active_session = Some(DebugSession::new(config));

                let body = serde_json::json!({
                    "status": "launch acknowledged"
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
            // `next` is analogous to DAP's "next" step request; here it just
            // runs the full debug session and returns the trace.
            "next" => {
                if let Some(session) = &active_session {
                    let trace = session.run()?;
                    last_trace = Some(trace.clone());
                    let body = serde_json::json!({ "events": trace });
                    let resp = ProtocolResponse {
                        seq: req.seq,
                        type_field: "response",
                        request_seq: req.seq,
                        success: true,
                        command,
                        body,
                    };
                    writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                } else {
                    let resp = ProtocolResponse {
                        seq: req.seq,
                        type_field: "response",
                        request_seq: req.seq,
                        success: false,
                        command,
                        body: serde_json::json!({ "error": "no active session" }),
                    };
                    writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                }
            }
            // Custom command for a storage viewer pane in VS Code.
            "stylusStorage" => {
                let storage_view = build_storage_view(last_trace.as_ref());
                let resp = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body: storage_view,
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
            // Custom command for a gas profiler pane in VS Code.
            "stylusGas" => {
                if let Some(trace) = last_trace.as_ref() {
                    let report = profile("entrypoint", trace);
                    let resp = ProtocolResponse {
                        seq: req.seq,
                        type_field: "response",
                        request_seq: req.seq,
                        success: true,
                        command,
                        body: report,
                    };
                    writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                } else {
                    let resp = ProtocolResponse {
                        seq: req.seq,
                        type_field: "response",
                        request_seq: req.seq,
                        success: false,
                        command,
                        body: serde_json::json!({ "error": "no trace available; run next first" }),
                    };
                    writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                }
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



