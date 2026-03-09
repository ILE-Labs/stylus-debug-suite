use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use debug_engine::DebugSession;
use engine_model::DebugConfig;

#[derive(Debug, Serialize, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: i64,
    pub column: i64,
}

#[allow(dead_code)]
pub struct PcToSourceMap {
    pub map: HashMap<u32, SourceLocation>,
}

#[allow(dead_code)]
impl PcToSourceMap {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }
    pub fn lookup(&self, pc: u32) -> Option<&SourceLocation> {
        self.map.get(&pc)
    }
}

#[allow(dead_code)]
pub fn parse_dwarf(wasm_path: &std::path::Path) -> Result<PcToSourceMap> {
    use object::{Object, ObjectSection};
    use std::fs;

    let bin_data = fs::read(wasm_path)?;
    let obj_file = object::File::parse(&*bin_data)?;
    
    let load_section = |id: gimli::SectionId| -> Result<gimli::EndianSlice<'_, gimli::RunTimeEndian>, gimli::Error> {
        let name = id.name();
        match obj_file.section_by_name(name) {
            Some(section) => Ok(gimli::EndianSlice::new(section.data().unwrap_or(&[]), gimli::RunTimeEndian::Little)),
            None => Ok(gimli::EndianSlice::new(&[], gimli::RunTimeEndian::Little)),
        }
    };

    let dwarf_sections = gimli::Dwarf::load(&load_section)?;
    let mut pc_to_source = PcToSourceMap::new();

    let mut iter = dwarf_sections.units();
    while let Some(header) = iter.next()? {
        let unit = dwarf_sections.unit(header)?;
        if let Some(program) = unit.line_program.clone() {
            let mut rows = program.rows();
            while let Some((header, row)) = rows.next_row()? {
                if row.end_sequence() { continue; }
                if let Some(file_entry) = row.file(header) {
                    let path_attr = file_entry.path_name();
                    let file_name = match path_attr {
                        gimli::AttributeValue::DebugStrRef(offset) => {
                            dwarf_sections.debug_str.get_str(offset).map(|s| s.to_string_lossy().into_owned()).unwrap_or("unknown".into())
                        }
                        gimli::AttributeValue::String(s) => s.to_string_lossy().into_owned(),
                        _ => "unknown".into(),
                    };
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

#[derive(Parser)]
pub struct AdapterArgs {
    /// DAP Port (for future use, currently stdio)
    #[arg(long, default_value = "0")]
    pub port: u16,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ProtocolRequest {
    seq: i64,
    #[serde(rename = "type")]
    type_field: String,
    command: String,
    #[serde(default)]
    arguments: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ProtocolResponse<T> {
    seq: i64,
    #[serde(rename = "type")]
    type_field: &'static str,
    request_seq: i64,
    success: bool,
    command: String,
    body: T,
}

pub async fn run(_args: AdapterArgs) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut active_session: Option<DebugSession> = None;
    let _pc_map = PcToSourceMap::new();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }

        let req: ProtocolRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(err) => {
                let resp: ProtocolResponse<String> = ProtocolResponse {
                    seq: 0,
                    type_field: "response",
                    request_seq: 0,
                    success: false,
                    command: "unknown".into(),
                    body: format!("invalid request: {err}"),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                continue;
            }
        };

        let command = req.command.clone();
        match command.as_str() {
            "initialize" => {
                let resp: ProtocolResponse<serde_json::Value> = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body: serde_json::json!({ "supportsConfigurationDoneRequest": true }),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
            "launch" => {
                let config = DebugConfig {
                    contract_path: "vault.rs".into(),
                    entrypoint: "deposit".into(),
                    breakpoints: vec![],
                };
                active_session = Some(DebugSession::new(config));
                let resp: ProtocolResponse<serde_json::Value> = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body: serde_json::json!({ "status": "launched" }),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
            "next" => {
                let mut success = false;
                if let Some(session) = &mut active_session {
                    success = session.step();
                }
                let resp: ProtocolResponse<serde_json::Value> = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body: serde_json::json!({ "stepped": success }),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
            "disconnect" => {
                let resp: ProtocolResponse<serde_json::Value> = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: true,
                    command,
                    body: serde_json::json!({ "status": "bye" }),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                break;
            }
            _ => {
                let resp: ProtocolResponse<serde_json::Value> = ProtocolResponse {
                    seq: req.seq,
                    type_field: "response",
                    request_seq: req.seq,
                    success: false,
                    command,
                    body: serde_json::json!({ "error": "unsupported" }),
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            }
        }
        stdout.flush()?;
    }

    Ok(())
}
