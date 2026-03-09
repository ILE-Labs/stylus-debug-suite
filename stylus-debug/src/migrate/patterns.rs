use std::process::Command;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A migration pattern detected in Solidity source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub pattern_id: String,
    pub solidity_construct: String,
    pub description: String,
    pub stylus_equivalent: String,
    pub line_number: Option<usize>,
    pub matched_text: String,
}

/// Scan Solidity source code via AST and return detected migration patterns.
pub fn detect_patterns(input_path: &std::path::Path) -> Vec<DetectedPattern> {
    let mut results = Vec::new();

    // Call JS bridge to get AST.
    let mut cmd = Command::new("node");
    let output = cmd.arg("migration-cli/src/parse_solidity.js")
        .arg(input_path)
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o.stdout,
        _ => return results,
    };

    let ast: Value = serde_json::from_slice(&output).unwrap_or(Value::Null);
    if ast.is_null() { return results; }

    // Traverse AST nodes
    if let Some(children) = ast["children"].as_array() {
        for child in children {
            if child["type"] == "ContractDefinition" {
                if let Some(base_contracts) = child["baseContracts"].as_array() {
                    if !base_contracts.is_empty() {
                        results.push(DetectedPattern {
                            pattern_id: "OUT_OF_SCOPE:Inheritance".into(),
                            solidity_construct: "is ContractName".into(),
                            description: "Inheritance is currently out of scope. Stylus requires flat contract structures.".into(),
                            stylus_equivalent: "// Flatten your contracts into a single struct".into(),
                            line_number: child["loc"]["start"]["line"].as_u64().map(|l| l as usize),
                            matched_text: "contract ... is ...".into(),
                        });
                    }
                }

                if let Some(sub_children) = child["subNodes"].as_array() {
                    for node in sub_children {
                        process_node(node, &mut results);
                    }
                }
            }
        }
    }

    results
}

fn process_node(node: &Value, results: &mut Vec<DetectedPattern>) {
    let node_type = node["type"].as_str().unwrap_or("");
    let line = node["loc"]["start"]["line"].as_u64().map(|l| l as usize);

    match node_type {
        "StateVariableDeclaration" => {
            if let Some(variables) = node["variables"].as_array() {
                for var in variables {
                    let type_name = var["typeName"]["type"].as_str().unwrap_or("");
                    if type_name == "Mapping" {
                        results.push(DetectedPattern {
                            pattern_id: "mapping".into(),
                            solidity_construct: var["name"].as_str().unwrap_or("var").into(),
                            description: "Mapping type. Use StorageMap in Stylus.".into(),
                            stylus_equivalent: "sol_storage! { mapping: StorageMap<Key, Value> }".into(),
                            line_number: line,
                            matched_text: "mapping(...)".into(),
                        });
                    } else {
                        results.push(DetectedPattern {
                            pattern_id: "state_variable".into(),
                            solidity_construct: var["name"].as_str().unwrap_or("var").into(),
                            description: "State variable. Use sol_storage! in Stylus.".into(),
                            stylus_equivalent: "sol_storage! { pub <name>: Storage<Type> }".into(),
                            line_number: line,
                            matched_text: format!("{} {}", var["typeName"]["name"].as_str().unwrap_or(""), var["name"].as_str().unwrap_or("")),
                        });
                    }
                }
            }
        }
        "FunctionDefinition" => {
            if let Some(modifiers) = node["modifiers"].as_array() {
                if !modifiers.is_empty() {
                    results.push(DetectedPattern {
                        pattern_id: "OUT_OF_SCOPE:Modifiers".into(),
                        solidity_construct: "custom_modifier".into(),
                        description: "Custom modifiers are out of scope. In Stylus, inline the logic.".into(),
                        stylus_equivalent: "// Inline logic".into(),
                        line_number: line,
                        matched_text: "modifier_name".into(),
                    });
                }
            }
            // Additional function logic porting ...
        }
        _ => {}
    }
}
