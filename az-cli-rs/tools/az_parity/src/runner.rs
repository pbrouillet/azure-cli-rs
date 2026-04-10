use crate::differ;
use crate::reporter::{TestResult, TestStatus};
use crate::suite::TestCase;
use std::time::{Duration, Instant};
use tokio::process::Command;

pub struct RunConfig {
    pub az_path: String,
    pub azrs_path: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone)]
pub struct CmdResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Run a single CLI invocation and capture output
pub async fn run_one(binary: &str, command: &str, timeout_secs: u64) -> Result<CmdResult, String> {
    let args: Vec<&str> = command.split_whitespace().collect();

    let start = Instant::now();
    let result = tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        Command::new(binary)
            .args(&args)
            .arg("--output").arg("json")
            .output(),
    ).await;

    match result {
        Ok(Ok(output)) => {
            let _ = start.elapsed();
            Ok(CmdResult {
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
        Ok(Err(e)) => Err(format!("failed to execute {}: {}", binary, e)),
        Err(_) => Err(format!("{} timed out after {}s", binary, timeout_secs)),
    }
}

/// Run a test: execute both az and azrs, compare outputs
pub async fn run_test(config: &RunConfig, test: &TestCase) -> TestResult {
    // Run az
    let az_start = Instant::now();
    let az_result = run_one(&config.az_path, &test.command, config.timeout_secs).await;
    let az_elapsed = az_start.elapsed();

    // Run azrs
    let azrs_start = Instant::now();
    let azrs_result = run_one(&config.azrs_path, &test.command, config.timeout_secs).await;
    let azrs_elapsed = azrs_start.elapsed();

    match (az_result, azrs_result) {
        (Ok(az), Ok(azrs)) => {
            compare_outputs(test, &az, Some(az_elapsed), &azrs, Some(azrs_elapsed))
        }
        (Err(e), _) => TestResult {
            name: test.name.clone(),
            command: test.command.clone(),
            status: TestStatus::Error(format!("az failed: {}", e)),
            az_time: None,
            azrs_time: None,
            diffs: vec![],
            az_stdout: None,
            azrs_stdout: None,
        },
        (_, Err(e)) => TestResult {
            name: test.name.clone(),
            command: test.command.clone(),
            status: TestStatus::Skip(format!("azrs failed: {}", e)),
            az_time: Some(az_elapsed),
            azrs_time: None,
            diffs: vec![],
            az_stdout: None,
            azrs_stdout: None,
        },
    }
}

/// Compare outputs from both CLIs
pub fn compare_outputs(
    test: &TestCase,
    az: &CmdResult,
    az_time: Option<Duration>,
    azrs: &CmdResult,
    azrs_time: Option<Duration>,
) -> TestResult {
    // If we expect an error, check both errored
    if test.expect_error {
        if az.exit_code != 0 && azrs.exit_code != 0 {
            return TestResult {
                name: test.name.clone(),
                command: test.command.clone(),
                status: TestStatus::Pass,
                az_time,
                azrs_time,
                diffs: vec![],
                az_stdout: None,
                azrs_stdout: None,
            };
        }
        if az.exit_code != 0 && azrs.exit_code == 0 {
            return TestResult {
                name: test.name.clone(),
                command: test.command.clone(),
                status: TestStatus::Diff,
                az_time,
                azrs_time,
                diffs: vec![differ::Difference {
                    path: "$".to_string(),
                    kind: differ::DiffKind::ExitCode {
                        az: az.exit_code,
                        azrs: azrs.exit_code,
                    },
                }],
                az_stdout: Some(az.stdout.clone()),
                azrs_stdout: Some(azrs.stdout.clone()),
            };
        }
    }

    // Check exit code mismatch
    if az.exit_code != azrs.exit_code {
        let mut diffs = vec![differ::Difference {
            path: "$".to_string(),
            kind: differ::DiffKind::ExitCode {
                az: az.exit_code,
                azrs: azrs.exit_code,
            },
        }];

        // If azrs errored but az didn't, include azrs stderr
        if azrs.exit_code != 0 && az.exit_code == 0 {
            diffs.push(differ::Difference {
                path: "stderr".to_string(),
                kind: differ::DiffKind::AzrsOnly(
                    serde_json::Value::String(azrs.stderr.trim().to_string()),
                ),
            });
        }

        return TestResult {
            name: test.name.clone(),
            command: test.command.clone(),
            status: TestStatus::Diff,
            az_time,
            azrs_time,
            diffs,
            az_stdout: Some(az.stdout.clone()),
            azrs_stdout: Some(azrs.stdout.clone()),
        };
    }

    // Both succeeded — parse JSON and compare
    let az_json = serde_json::from_str::<serde_json::Value>(az.stdout.trim());
    let azrs_json = serde_json::from_str::<serde_json::Value>(azrs.stdout.trim());

    match (az_json, azrs_json) {
        (Ok(az_val), Ok(azrs_val)) => {
            let diffs = differ::diff_json(&az_val, &azrs_val, "$", &test.ignore_fields);
            let status = if diffs.is_empty() {
                TestStatus::Pass
            } else {
                TestStatus::Diff
            };
            TestResult {
                name: test.name.clone(),
                command: test.command.clone(),
                status,
                az_time,
                azrs_time,
                diffs,
                az_stdout: Some(az.stdout.clone()),
                azrs_stdout: Some(azrs.stdout.clone()),
            }
        }
        (Err(_), Err(_)) => {
            // Neither produced valid JSON — compare raw strings
            if az.stdout.trim() == azrs.stdout.trim() {
                TestResult {
                    name: test.name.clone(),
                    command: test.command.clone(),
                    status: TestStatus::Pass,
                    az_time,
                    azrs_time,
                    diffs: vec![],
                    az_stdout: None,
                    azrs_stdout: None,
                }
            } else {
                TestResult {
                    name: test.name.clone(),
                    command: test.command.clone(),
                    status: TestStatus::Diff,
                    az_time,
                    azrs_time,
                    diffs: vec![differ::Difference {
                        path: "$".to_string(),
                        kind: differ::DiffKind::ValueMismatch {
                            az: serde_json::Value::String(az.stdout.trim().to_string()),
                            azrs: serde_json::Value::String(azrs.stdout.trim().to_string()),
                        },
                    }],
                    az_stdout: Some(az.stdout.clone()),
                    azrs_stdout: Some(azrs.stdout.clone()),
                }
            }
        }
        (Ok(_), Err(e)) => {
            TestResult {
                name: test.name.clone(),
                command: test.command.clone(),
                status: TestStatus::Diff,
                az_time,
                azrs_time,
                diffs: vec![differ::Difference {
                    path: "$".to_string(),
                    kind: differ::DiffKind::AzrsOnly(
                        serde_json::Value::String(format!("azrs output is not valid JSON: {}", e)),
                    ),
                }],
                az_stdout: Some(az.stdout.clone()),
                azrs_stdout: Some(azrs.stdout.clone()),
            }
        }
        (Err(e), Ok(_)) => {
            TestResult {
                name: test.name.clone(),
                command: test.command.clone(),
                status: TestStatus::Diff,
                az_time,
                azrs_time,
                diffs: vec![differ::Difference {
                    path: "$".to_string(),
                    kind: differ::DiffKind::AzOnly(
                        serde_json::Value::String(format!("az output is not valid JSON: {}", e)),
                    ),
                }],
                az_stdout: Some(az.stdout.clone()),
                azrs_stdout: Some(azrs.stdout.clone()),
            }
        }
    }
}
