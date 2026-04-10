/// Assertion checkers — JMESPath-based output validation.
///
/// Inspired by Python testsdk's `checkers.py`.
/// Each checker is a function that validates a command result, returning
/// Ok(()) on success or Err(message) on failure.

/// Result of executing a CLI command in tests.
#[derive(Debug)]
pub struct CmdResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub json: Option<serde_json::Value>,
}

impl CmdResult {
    /// Run all checkers against this result, panicking on the first failure.
    pub fn assert_with_checks(&self, checks: &[Checker]) {
        for check in checks {
            if let Err(msg) = (check)(self) {
                panic!("Check failed: {msg}");
            }
        }
    }

    /// Get the JSON output, panicking if it's absent.
    pub fn get_json(&self) -> &serde_json::Value {
        self.json
            .as_ref()
            .expect("Expected JSON output but got none")
    }
}

/// A checker function that validates a CmdResult.
pub type Checker = Box<dyn Fn(&CmdResult) -> Result<(), String>>;

/// Assert that a JMESPath query on the JSON output equals the expected value.
pub fn check(query: &str, expected: serde_json::Value) -> Checker {
    let query = query.to_string();
    Box::new(move |result: &CmdResult| {
        let json = result
            .json
            .as_ref()
            .ok_or("No JSON output to check")?;
        let expr = jmespath::compile(&query)
            .map_err(|e| format!("Invalid JMESPath '{query}': {e}"))?;
        let data = json_to_jmespath(json)?;
        let found = expr.search(&data).map_err(|e| format!("JMESPath search failed: {e}"))?;

        let found_json = jmespath_to_json(&found);
        if found_json != expected {
            return Err(format!(
                "JMESPath '{query}': expected {expected}, got {found_json}"
            ));
        }
        Ok(())
    })
}

/// Assert that a JMESPath query returns a truthy (non-null, non-empty) value.
pub fn exists(query: &str) -> Checker {
    let query = query.to_string();
    Box::new(move |result: &CmdResult| {
        let json = result.json.as_ref().ok_or("No JSON output to check")?;
        let expr = jmespath::compile(&query)
            .map_err(|e| format!("Invalid JMESPath '{query}': {e}"))?;
        let data = json_to_jmespath(json)?;
        let found = expr.search(&data).map_err(|e| format!("JMESPath search failed: {e}"))?;

        if found.is_null() {
            return Err(format!("JMESPath '{query}': expected to exist, got null"));
        }
        Ok(())
    })
}

/// Assert that a JMESPath query returns null/empty.
pub fn not_exists(query: &str) -> Checker {
    let query = query.to_string();
    Box::new(move |result: &CmdResult| {
        let json = result.json.as_ref().ok_or("No JSON output to check")?;
        let expr = jmespath::compile(&query)
            .map_err(|e| format!("Invalid JMESPath '{query}': {e}"))?;
        let data = json_to_jmespath(json)?;
        let found = expr.search(&data).map_err(|e| format!("JMESPath search failed: {e}"))?;

        if !found.is_null() {
            let found_json = jmespath_to_json(&found);
            return Err(format!(
                "JMESPath '{query}': expected not to exist, got {found_json}"
            ));
        }
        Ok(())
    })
}

/// Assert that a JMESPath query returns a value greater than expected.
pub fn greater_than(query: &str, expected: f64) -> Checker {
    let query = query.to_string();
    Box::new(move |result: &CmdResult| {
        let json = result.json.as_ref().ok_or("No JSON output to check")?;
        let expr = jmespath::compile(&query)
            .map_err(|e| format!("Invalid JMESPath '{query}': {e}"))?;
        let data = json_to_jmespath(json)?;
        let found = expr.search(&data).map_err(|e| format!("JMESPath search failed: {e}"))?;

        let found_json = jmespath_to_json(&found);
        let val = found_json
            .as_f64()
            .ok_or_else(|| format!("JMESPath '{query}': expected number, got {found_json}"))?;
        if val <= expected {
            return Err(format!(
                "JMESPath '{query}': expected > {expected}, got {val}"
            ));
        }
        Ok(())
    })
}

/// Assert that a JMESPath query result matches a regex pattern.
pub fn check_pattern(query: &str, pattern: &str) -> Checker {
    let query = query.to_string();
    let pattern = pattern.to_string();
    Box::new(move |result: &CmdResult| {
        let json = result.json.as_ref().ok_or("No JSON output to check")?;
        let expr = jmespath::compile(&query)
            .map_err(|e| format!("Invalid JMESPath '{query}': {e}"))?;
        let data = json_to_jmespath(json)?;
        let found = expr.search(&data).map_err(|e| format!("JMESPath search failed: {e}"))?;

        let found_json = jmespath_to_json(&found);
        let val = found_json
            .as_str()
            .ok_or_else(|| format!("JMESPath '{query}': expected string, got {found_json}"))?;

        let re = regex::Regex::new(&pattern)
            .map_err(|e| format!("Invalid regex '{pattern}': {e}"))?;
        if !re.is_match(val) {
            return Err(format!(
                "JMESPath '{query}': '{val}' does not match pattern '{pattern}'"
            ));
        }
        Ok(())
    })
}

/// Assert the JSON output is empty (null, [], {}, or "false").
pub fn is_empty() -> Checker {
    Box::new(move |result: &CmdResult| match &result.json {
        None => Ok(()),
        Some(serde_json::Value::Null) => Ok(()),
        Some(serde_json::Value::Array(arr)) if arr.is_empty() => Ok(()),
        Some(serde_json::Value::Object(obj)) if obj.is_empty() => Ok(()),
        Some(serde_json::Value::Bool(false)) => Ok(()),
        Some(other) => Err(format!("Expected empty output, got {other}")),
    })
}

/// Assert the stdout contains a substring.
pub fn string_contains(substring: &str) -> Checker {
    let substring = substring.to_string();
    Box::new(move |result: &CmdResult| {
        if result.stdout.contains(&substring) {
            Ok(())
        } else {
            Err(format!(
                "Expected stdout to contain '{substring}', got '{}'",
                result.stdout
            ))
        }
    })
}

/// Assert the command succeeded (exit code 0).
pub fn success() -> Checker {
    Box::new(move |result: &CmdResult| {
        if result.exit_code == 0 {
            Ok(())
        } else {
            Err(format!(
                "Expected exit code 0, got {}. stderr: {}",
                result.exit_code, result.stderr
            ))
        }
    })
}

/// Convert a jmespath::Variable to a serde_json::Value.
fn jmespath_to_json(var: &jmespath::Variable) -> serde_json::Value {
    // jmespath::Variable implements Display as JSON
    let s = var.to_string();
    serde_json::from_str(&s).unwrap_or(serde_json::Value::String(s))
}

/// Convert a serde_json::Value to a jmespath::Variable.
fn json_to_jmespath(value: &serde_json::Value) -> Result<jmespath::Variable, String> {
    let json_str = serde_json::to_string(value).map_err(|e| format!("JSON serialization failed: {e}"))?;
    jmespath::Variable::from_json(&json_str).map_err(|e| format!("JMESPath conversion failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(json: serde_json::Value) -> CmdResult {
        CmdResult {
            exit_code: 0,
            stdout: serde_json::to_string_pretty(&json).unwrap(),
            stderr: String::new(),
            json: Some(json),
        }
    }

    #[test]
    fn test_check_simple() {
        let result = make_result(serde_json::json!({"name": "test-rg", "location": "eastus"}));
        let checker = check("name", serde_json::json!("test-rg"));
        assert!(checker(&result).is_ok());
    }

    #[test]
    fn test_check_fails() {
        let result = make_result(serde_json::json!({"name": "test-rg"}));
        let checker = check("name", serde_json::json!("other"));
        assert!(checker(&result).is_err());
    }

    #[test]
    fn test_exists_ok() {
        let result = make_result(serde_json::json!({"id": "/sub/123"}));
        let checker = exists("id");
        assert!(checker(&result).is_ok());
    }

    #[test]
    fn test_not_exists_ok() {
        let result = make_result(serde_json::json!({"name": "test"}));
        let checker = not_exists("missing_field");
        assert!(checker(&result).is_ok());
    }

    #[test]
    fn test_is_empty_array() {
        let result = make_result(serde_json::json!([]));
        let checker = is_empty();
        assert!(checker(&result).is_ok());
    }

    #[test]
    fn test_greater_than() {
        let result = make_result(serde_json::json!({"count": 5}));
        let checker = greater_than("count", 3.0);
        assert!(checker(&result).is_ok());
    }

    #[test]
    fn test_check_pattern() {
        let result = make_result(serde_json::json!({"id": "/subscriptions/abc-123/resourceGroups/my-rg"}));
        let checker = check_pattern("id", r"/subscriptions/.+/resourceGroups/.+");
        assert!(checker(&result).is_ok());
    }
}
