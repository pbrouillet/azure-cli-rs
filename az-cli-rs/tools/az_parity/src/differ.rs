use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Difference {
    pub path: String,
    pub kind: DiffKind,
}

#[derive(Debug, Clone)]
pub enum DiffKind {
    /// Key exists in az output but not azrs
    AzOnly(Value),
    /// Key exists in azrs output but not az
    AzrsOnly(Value),
    /// Both have the key but values differ
    ValueMismatch { az: Value, azrs: Value },
    /// Type mismatch (e.g., string vs number)
    TypeMismatch { az: String, azrs: String },
    /// Array length mismatch
    ArrayLengthMismatch { az: usize, azrs: usize },
    /// Exit code mismatch
    ExitCode { az: i32, azrs: i32 },
}

/// Recursively diff two JSON values, returning a list of differences.
/// `path` is the JSONPath-style location (e.g., "$", "$.name", "$[0].id").
/// `ignore_fields` are field names to skip at any depth.
pub fn diff_json(az: &Value, azrs: &Value, path: &str, ignore_fields: &[String]) -> Vec<Difference> {
    let mut diffs = Vec::new();
    diff_recursive(az, azrs, path, ignore_fields, &mut diffs);
    diffs
}

fn diff_recursive(az: &Value, azrs: &Value, path: &str, ignore: &[String], diffs: &mut Vec<Difference>) {
    match (az, azrs) {
        (Value::Object(az_map), Value::Object(azrs_map)) => {
            // Keys only in az
            for key in az_map.keys() {
                if ignore.contains(key) {
                    continue;
                }
                let child_path = format!("{}.{}", path, key);
                match azrs_map.get(key) {
                    Some(azrs_val) => {
                        diff_recursive(&az_map[key], azrs_val, &child_path, ignore, diffs);
                    }
                    None => {
                        diffs.push(Difference {
                            path: child_path,
                            kind: DiffKind::AzOnly(az_map[key].clone()),
                        });
                    }
                }
            }
            // Keys only in azrs
            for key in azrs_map.keys() {
                if ignore.contains(key) {
                    continue;
                }
                if !az_map.contains_key(key) {
                    diffs.push(Difference {
                        path: format!("{}.{}", path, key),
                        kind: DiffKind::AzrsOnly(azrs_map[key].clone()),
                    });
                }
            }
        }
        (Value::Array(az_arr), Value::Array(azrs_arr)) => {
            if az_arr.len() != azrs_arr.len() {
                diffs.push(Difference {
                    path: path.to_string(),
                    kind: DiffKind::ArrayLengthMismatch {
                        az: az_arr.len(),
                        azrs: azrs_arr.len(),
                    },
                });
            }

            // Try to match array elements by "name" or "id" field
            let matched = try_match_by_key(az_arr, azrs_arr);

            match matched {
                Some(pairs) => {
                    for (az_item, azrs_item, match_key) in &pairs {
                        let child_path = format!("{}[{}={}]", path,
                            match_key.0, match_key.1);
                        diff_recursive(az_item, azrs_item, &child_path, ignore, diffs);
                    }
                    // Report unmatched az items
                    let matched_az: Vec<_> = pairs.iter().map(|(a, _, _)| *a as *const Value).collect();
                    for (i, item) in az_arr.iter().enumerate() {
                        if !matched_az.contains(&(item as *const Value)) {
                            diffs.push(Difference {
                                path: format!("{}[{}]", path, i),
                                kind: DiffKind::AzOnly(item.clone()),
                            });
                        }
                    }
                    let matched_azrs: Vec<_> = pairs.iter().map(|(_, a, _)| *a as *const Value).collect();
                    for (i, item) in azrs_arr.iter().enumerate() {
                        if !matched_azrs.contains(&(item as *const Value)) {
                            diffs.push(Difference {
                                path: format!("{}[{}]", path, i),
                                kind: DiffKind::AzrsOnly(item.clone()),
                            });
                        }
                    }
                }
                None => {
                    // Positional fallback
                    let len = az_arr.len().min(azrs_arr.len());
                    for i in 0..len {
                        let child_path = format!("{}[{}]", path, i);
                        diff_recursive(&az_arr[i], &azrs_arr[i], &child_path, ignore, diffs);
                    }
                }
            }
        }
        _ => {
            // Leaf comparison
            if type_name(az) != type_name(azrs) {
                diffs.push(Difference {
                    path: path.to_string(),
                    kind: DiffKind::TypeMismatch {
                        az: type_name(az).to_string(),
                        azrs: type_name(azrs).to_string(),
                    },
                });
            } else if az != azrs {
                diffs.push(Difference {
                    path: path.to_string(),
                    kind: DiffKind::ValueMismatch {
                        az: az.clone(),
                        azrs: azrs.clone(),
                    },
                });
            }
        }
    }
}

/// Try to match array elements by a common identifier field (name, id, etc.)
fn try_match_by_key<'a>(
    az_arr: &'a [Value],
    azrs_arr: &'a [Value],
) -> Option<Vec<(&'a Value, &'a Value, (String, String))>> {
    // Try matching by common key fields
    for key in &["name", "id", "displayName"] {
        // Check if all elements in both arrays have this key
        let az_has = az_arr.iter().all(|v| v.get(key).and_then(|v| v.as_str()).is_some());
        let azrs_has = azrs_arr.iter().all(|v| v.get(key).and_then(|v| v.as_str()).is_some());

        if az_has && azrs_has {
            let mut pairs = Vec::new();
            for az_item in az_arr {
                let az_key = az_item[key].as_str().unwrap();
                if let Some(azrs_item) = azrs_arr.iter().find(|v| v[key].as_str() == Some(az_key)) {
                    pairs.push((az_item, azrs_item, (key.to_string(), az_key.to_string())));
                }
            }
            if !pairs.is_empty() {
                return Some(pairs);
            }
        }
    }
    None
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn identical_objects_produce_no_diffs() {
        let v = json!({"name": "foo", "location": "eastus"});
        let diffs = diff_json(&v, &v, "$", &[]);
        assert!(diffs.is_empty());
    }

    #[test]
    fn missing_key_in_azrs() {
        let az = json!({"name": "foo", "location": "eastus", "extra": true});
        let azrs = json!({"name": "foo", "location": "eastus"});
        let diffs = diff_json(&az, &azrs, "$", &[]);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path, "$.extra");
        assert!(matches!(diffs[0].kind, DiffKind::AzOnly(_)));
    }

    #[test]
    fn extra_key_in_azrs() {
        let az = json!({"name": "foo"});
        let azrs = json!({"name": "foo", "bonus": 42});
        let diffs = diff_json(&az, &azrs, "$", &[]);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path, "$.bonus");
        assert!(matches!(diffs[0].kind, DiffKind::AzrsOnly(_)));
    }

    #[test]
    fn value_mismatch() {
        let az = json!({"name": "foo", "location": "eastus"});
        let azrs = json!({"name": "foo", "location": "westus"});
        let diffs = diff_json(&az, &azrs, "$", &[]);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path, "$.location");
        assert!(matches!(diffs[0].kind, DiffKind::ValueMismatch { .. }));
    }

    #[test]
    fn ignore_fields_respected() {
        let az = json!({"name": "foo", "etag": "abc123"});
        let azrs = json!({"name": "foo", "etag": "xyz789"});
        let diffs = diff_json(&az, &azrs, "$", &["etag".to_string()]);
        assert!(diffs.is_empty());
    }

    #[test]
    fn nested_diff() {
        let az = json!({"properties": {"status": "running"}});
        let azrs = json!({"properties": {"status": "stopped"}});
        let diffs = diff_json(&az, &azrs, "$", &[]);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path, "$.properties.status");
    }

    #[test]
    fn array_length_mismatch() {
        let az = json!([1, 2, 3]);
        let azrs = json!([1, 2]);
        let diffs = diff_json(&az, &azrs, "$", &[]);
        assert!(diffs.iter().any(|d| matches!(d.kind, DiffKind::ArrayLengthMismatch { .. })));
    }

    #[test]
    fn array_matching_by_name() {
        let az = json!([
            {"name": "a", "value": 1},
            {"name": "b", "value": 2}
        ]);
        let azrs = json!([
            {"name": "b", "value": 2},
            {"name": "a", "value": 1}
        ]);
        let diffs = diff_json(&az, &azrs, "$", &[]);
        // Same data, different order — should match by name and produce no diffs
        assert!(diffs.is_empty());
    }
}
