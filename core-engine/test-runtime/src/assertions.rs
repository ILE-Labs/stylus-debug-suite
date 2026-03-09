use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Result of checking a single assertion against post-execution state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    pub assertion: String,
    pub passed: bool,
    pub expected: String,
    pub actual: String,
}

/// Assertion definition loaded from the YAML config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assertion {
    /// Type of assertion: "storage_eq", "reverted", "not_reverted"
    #[serde(rename = "type")]
    pub assert_type: String,
    /// For storage assertions: the slot name.
    pub slot: Option<String>,
    /// Expected value (hex string for storage, "true"/"false" for reverted).
    pub expected: String,
}

/// Check a list of assertions against the final VM state.
pub fn check_assertions(
    assertions: &[Assertion],
    final_storage: &BTreeMap<String, String>,
    reverted: bool,
) -> Vec<AssertionResult> {
    assertions
        .iter()
        .map(|a| check_one(a, final_storage, reverted))
        .collect()
}

fn check_one(
    assertion: &Assertion,
    final_storage: &BTreeMap<String, String>,
    reverted: bool,
) -> AssertionResult {
    match assertion.assert_type.as_str() {
        "storage_eq" => {
            let slot = assertion.slot.as_deref().unwrap_or("?");
            let actual = final_storage
                .get(slot)
                .cloned()
                .unwrap_or_else(|| "0x00".into());
            let expected = &assertion.expected;
            let passed = normalize_hex(&actual) == normalize_hex(expected);
            AssertionResult {
                assertion: format!("storage[{slot}] == {expected}"),
                passed,
                expected: expected.clone(),
                actual,
            }
        }
        "reverted" => {
            let expected_reverted = assertion.expected == "true";
            AssertionResult {
                assertion: format!("reverted == {}", assertion.expected),
                passed: reverted == expected_reverted,
                expected: assertion.expected.clone(),
                actual: reverted.to_string(),
            }
        }
        "not_reverted" => AssertionResult {
            assertion: "transaction should succeed".into(),
            passed: !reverted,
            expected: "false".into(),
            actual: reverted.to_string(),
        },
        other => AssertionResult {
            assertion: format!("unknown assertion type: {other}"),
            passed: false,
            expected: "?".into(),
            actual: "?".into(),
        },
    }
}

/// Normalize hex for comparison: lowercase, strip leading zeros after 0x.
fn normalize_hex(s: &str) -> String {
    let s = s.to_lowercase();
    if let Some(stripped) = s.strip_prefix("0x") {
        let trimmed = stripped.trim_start_matches('0');
        if trimmed.is_empty() {
            "0x0".to_string()
        } else {
            format!("0x{trimmed}")
        }
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_hex() {
        assert_eq!(normalize_hex("0x0032"), "0x32");
        assert_eq!(normalize_hex("0x00"), "0x0");
        assert_eq!(normalize_hex("0xFF"), "0xff");
    }

    #[test]
    fn test_storage_assertion_pass() {
        let mut storage = BTreeMap::new();
        storage.insert("balance".into(), "0x32".into());

        let assertions = vec![Assertion {
            assert_type: "storage_eq".into(),
            slot: Some("balance".into()),
            expected: "0x32".into(),
        }];

        let results = check_assertions(&assertions, &storage, false);
        assert!(results[0].passed);
    }

    #[test]
    fn test_reverted_assertion() {
        let assertions = vec![Assertion {
            assert_type: "reverted".into(),
            slot: None,
            expected: "true".into(),
        }];

        let results = check_assertions(&assertions, &BTreeMap::new(), true);
        assert!(results[0].passed);
    }
}
