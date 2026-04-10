use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct TestSuite {
    #[serde(default)]
    pub defaults: SuiteDefaults,
    #[serde(rename = "test", default)]
    tests_raw: Vec<TestCaseRaw>,
    // Populated after deserialization
    #[serde(skip)]
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Default, Deserialize)]
pub struct SuiteDefaults {
    #[serde(default)]
    pub subscription: Option<String>,
    #[serde(default)]
    pub ignore_fields: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TestCaseRaw {
    name: String,
    command: String,
    #[serde(default)]
    ignore_fields: Vec<String>,
    #[serde(default)]
    expect_error: bool,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub command: String,
    pub ignore_fields: Vec<String>,
    pub expect_error: bool,
}

pub fn load_suite(path: &Path) -> Result<TestSuite, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;

    let mut suite: TestSuite = toml::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {}", path.display(), e))?;

    // Merge defaults into each test case
    suite.tests = suite.tests_raw.iter().map(|raw| {
        let mut ignore = suite.defaults.ignore_fields.clone();
        ignore.extend(raw.ignore_fields.clone());

        let mut command = raw.command.clone();
        // Inject --subscription if set in defaults and not already present
        if let Some(ref sub) = suite.defaults.subscription {
            if !command.contains("--subscription") {
                command = format!("{} --subscription {}", command, sub);
            }
        }

        TestCase {
            name: raw.name.clone(),
            command,
            ignore_fields: ignore,
            expect_error: raw.expect_error,
        }
    }).collect();

    Ok(suite)
}
