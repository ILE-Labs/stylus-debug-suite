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

    // Call JS bridge to get AST. Use absolute path to node.exe as fallback if 'node' is not in path.
    let mut cmd = Command::new("node");
    let output = cmd.arg("migration-cli/src/parse_solidity.js")
        .arg(input_path)
        .output()
        .or_else(|_| {
            Command::new("/mnt/c/Program Files/nodejs/node.exe")
                .arg("migration-cli/src/parse_solidity.js")
                .arg(input_path)
                .output()
        });

    let output = match output {
        Ok(o) if o.status.success() => o.stdout,
        _ => {
            // Fallback to minimal detection if JS fails, but we want AST for real
            return results;
        }
    };

    let ast: Value = match serde_json::from_slice(&output) {
        Ok(v) => v,
        Err(_) => return results,
    };

    // Traverse AST nodes
    if let Some(children) = ast["children"].as_array() {
        for child in children {
            match child["type"].as_str() {
                Some("ContractDefinition") => {
                    // Check for Inheritance (Out of Scope for POC)
                    if let Some(base_contracts) = child["baseContracts"].as_array() {
                        if !base_contracts.is_empty() {
                            results.push(DetectedPattern {
                                pattern_id: "OUT_OF_SCOPE:Inheritance".into(),
                                solidity_construct: "is ContractName".into(),
                                description: "Inheritance is currently out of scope for the POC. \
                                              Stylus requires flat contract structures or manual composition.".into(),
                                stylus_equivalent: "// Flatten your contracts into a single struct".into(),
                                line_number: child["loc"]["start"]["line"].as_u64().map(|l| l as usize),
                                matched_text: "contract ... is ...".into(),
                            });
                        }
                    }

                    // Traverse sub-nodes
                    if let Some(sub_children) = child["subNodes"].as_array() {
                        for node in sub_children {
                            process_node(node, &mut results);
                        }
                    }
                }
                _ => {}
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

            let visibility = node["visibility"].as_str().unwrap_or("");
            if visibility == "external" || visibility == "public" {
                 results.push(DetectedPattern {
                    pattern_id: "external_function".into(),
                    solidity_construct: node["name"].as_str().unwrap_or("func").into(),
                    description: "External function. All #[public] methods in Stylus are ABI-exposed.".into(),
                    stylus_equivalent: "#[public]\nimpl Contract { ... }".into(),
                    line_number: line,
                    matched_text: format!("function {}() {}", node["name"].as_str().unwrap_or(""), visibility),
                });
            }

            if node["isPayable"].as_bool().unwrap_or(false) {
                results.push(DetectedPattern {
                    pattern_id: "payable_function".into(),
                    solidity_construct: node["name"].as_str().unwrap_or("func").into(),
                    description: "Payable function. Use #[payable] in Stylus.".into(),
                    stylus_equivalent: "#[payable]\nfn ...".into(),
                    line_number: line,
                    matched_text: "payable".into(),
                });
            }
        }
        "EventDefinition" => {
            results.push(DetectedPattern {
                pattern_id: "event_definition".into(),
                solidity_construct: node["name"].as_str().unwrap_or("event").into(),
                description: "Event definition. Use sol! macro in Stylus.".into(),
                stylus_equivalent: "sol! { event Name(...); }".into(),
                line_number: line,
                matched_text: format!("event {}(...)", node["name"].as_str().unwrap_or("")),
            });
        }
        "EmitStatement" => {
            results.push(DetectedPattern {
                pattern_id: "emit_event".into(),
                solidity_construct: "emit".into(),
                description: "Event emission. Use evm::log in Stylus.".into(),
                stylus_equivalent: "evm::log(EventName { ... });".into(),
                line_number: line,
                matched_text: "emit EventValue(...)".into(),
            });
        }
        _ => {}
    }

    // Recursively check for specific expressions like msg.sender or require
    check_expressions(node, results);
}

fn check_expressions(node: &Value, results: &mut Vec<DetectedPattern>) {
    // This is a simplified traversal for the POC
    let s = serde_json::to_string(node).unwrap_or_default();
    if s.contains("\"memberName\":\"sender\",\"expression\":{\"type\":\"Identifier\",\"name\":\"msg\"}") {
        results.push(DetectedPattern {
            pattern_id: "msg_sender".into(),
            solidity_construct: "msg.sender".into(),
            description: "Caller address. Use msg::sender() in Stylus.".into(),
            stylus_equivalent: "let caller: Address = msg::sender();".into(),
            line_number: None,
            matched_text: "msg.sender".into(),
        });
    }
    if s.contains("\"name\":\"require\"") && s.contains("\"type\":\"FunctionCall\"") {
        results.push(DetectedPattern {
            pattern_id: "require".into(),
            solidity_construct: "require()".into(),
            description: "Condition check. Use if/revert or Result match in Stylus.".into(),
            stylus_equivalent: "if !cond { return Err(...); }".into(),
            line_number: None,
            matched_text: "require(condition, ...)".into(),
        });
    }
}
