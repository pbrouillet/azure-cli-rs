mod suite;
mod runner;
mod differ;
mod reporter;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "az-parity", about = "Compare az (Python) vs azrs (Rust) CLI behavior")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to az binary
    #[arg(long, global = true, default_value = "az")]
    az_path: String,

    /// Path to azrs binary
    #[arg(long, global = true, default_value = "azrs")]
    azrs_path: String,

    /// Command timeout in seconds
    #[arg(long, global = true, default_value = "30")]
    timeout: u64,

    /// Show full outputs for failed tests
    #[arg(long, global = true)]
    verbose: bool,

    /// Output format
    #[arg(long, global = true, default_value = "text")]
    output: OutputFormat,
}

#[derive(Clone, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a single command comparison
    Run {
        /// The az command to compare (without 'az' prefix), e.g. "group list"
        command: String,

        /// Fields to ignore in comparison
        #[arg(long, value_delimiter = ',')]
        ignore: Vec<String>,
    },

    /// Run a test suite from a TOML file
    Suite {
        /// Path to TOML test suite file
        path: PathBuf,

        /// Run only tests matching this name filter
        #[arg(long)]
        filter: Option<String>,
    },

    /// Record az outputs to fixtures for offline replay
    Record {
        /// Path to TOML test suite file
        path: PathBuf,

        /// Output directory for recordings
        #[arg(long, default_value = "tests/parity/recordings")]
        output_dir: PathBuf,
    },

    /// Replay: compare azrs live output against recorded az fixtures
    Replay {
        /// Path to TOML test suite file
        path: PathBuf,

        /// Directory containing recorded fixtures
        #[arg(long, default_value = "tests/parity/recordings")]
        recordings_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Run { ref command, ref ignore } => {
            cmd_run(&cli, command, ignore).await
        }
        Commands::Suite { ref path, ref filter } => {
            cmd_suite(&cli, path, filter.as_deref()).await
        }
        Commands::Record { ref path, ref output_dir } => {
            cmd_record(&cli, path, output_dir).await
        }
        Commands::Replay { ref path, ref recordings_dir } => {
            cmd_replay(&cli, path, recordings_dir).await
        }
    };

    std::process::exit(if result { 0 } else { 1 });
}

/// Run a single ad-hoc comparison
async fn cmd_run(cli: &Cli, command: &str, ignore_fields: &[String]) -> bool {
    let test = suite::TestCase {
        name: command.replace(' ', "-"),
        command: command.to_string(),
        ignore_fields: ignore_fields.to_vec(),
        expect_error: false,
    };

    let config = runner::RunConfig {
        az_path: cli.az_path.clone(),
        azrs_path: cli.azrs_path.clone(),
        timeout_secs: cli.timeout,
    };

    let outcome = runner::run_test(&config, &test).await;
    let results = vec![outcome];
    reporter::print_report(&results, cli.verbose, matches!(cli.output, OutputFormat::Json));
    results.iter().all(|r| matches!(r.status, reporter::TestStatus::Pass))
}

/// Run a full test suite
async fn cmd_suite(cli: &Cli, path: &std::path::Path, filter: Option<&str>) -> bool {
    let suite = match suite::load_suite(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error loading suite {}: {}", path.display(), e);
            return false;
        }
    };

    let tests: Vec<_> = if let Some(f) = filter {
        suite.tests.into_iter().filter(|t| t.name.contains(f)).collect()
    } else {
        suite.tests
    };

    if tests.is_empty() {
        eprintln!("No tests to run.");
        return true;
    }

    let config = runner::RunConfig {
        az_path: cli.az_path.clone(),
        azrs_path: cli.azrs_path.clone(),
        timeout_secs: cli.timeout,
    };

    let mut results = Vec::new();
    for test in &tests {
        let outcome = runner::run_test(&config, test).await;
        results.push(outcome);
    }

    reporter::print_report(&results, cli.verbose, matches!(cli.output, OutputFormat::Json));
    results.iter().all(|r| matches!(r.status, reporter::TestStatus::Pass))
}

/// Record az outputs to fixture files
async fn cmd_record(cli: &Cli, path: &std::path::Path, output_dir: &std::path::Path) -> bool {
    let suite = match suite::load_suite(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error loading suite {}: {}", path.display(), e);
            return false;
        }
    };

    std::fs::create_dir_all(output_dir).ok();

    let config = runner::RunConfig {
        az_path: cli.az_path.clone(),
        azrs_path: cli.azrs_path.clone(),
        timeout_secs: cli.timeout,
    };

    let mut ok = true;
    for test in &suite.tests {
        eprint!("Recording {}... ", test.name);
        match runner::run_one(&config.az_path, &test.command, config.timeout_secs).await {
            Ok(result) => {
                let recording = serde_json::json!({
                    "command": test.command,
                    "exit_code": result.exit_code,
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                });
                let file = output_dir.join(format!("{}.json", test.name));
                if let Err(e) = std::fs::write(&file, serde_json::to_string_pretty(&recording).unwrap()) {
                    eprintln!("FAIL (write: {})", e);
                    ok = false;
                } else {
                    eprintln!("OK");
                }
            }
            Err(e) => {
                eprintln!("FAIL ({})", e);
                ok = false;
            }
        }
    }
    ok
}

/// Replay: compare azrs live output against recorded az fixtures
async fn cmd_replay(cli: &Cli, path: &std::path::Path, recordings_dir: &std::path::Path) -> bool {
    let suite = match suite::load_suite(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error loading suite {}: {}", path.display(), e);
            return false;
        }
    };

    let config = runner::RunConfig {
        az_path: cli.az_path.clone(),
        azrs_path: cli.azrs_path.clone(),
        timeout_secs: cli.timeout,
    };

    let mut results = Vec::new();
    for test in &suite.tests {
        let recording_file = recordings_dir.join(format!("{}.json", test.name));
        let outcome = match std::fs::read_to_string(&recording_file) {
            Ok(content) => {
                let rec: serde_json::Value = match serde_json::from_str(&content) {
                    Ok(v) => v,
                    Err(e) => {
                        results.push(reporter::TestResult {
                            name: test.name.clone(),
                            command: test.command.clone(),
                            status: reporter::TestStatus::Error(format!("bad recording: {}", e)),
                            az_time: None,
                            azrs_time: None,
                            diffs: vec![],
                            az_stdout: None,
                            azrs_stdout: None,
                        });
                        continue;
                    }
                };

                let az_stdout = rec["stdout"].as_str().unwrap_or("").to_string();
                let az_exit = rec["exit_code"].as_i64().unwrap_or(-1) as i32;

                match runner::run_one(&config.azrs_path, &test.command, config.timeout_secs).await {
                    Ok(azrs_result) => {
                        runner::compare_outputs(
                            &test,
                            &runner::CmdResult { exit_code: az_exit, stdout: az_stdout.clone(), stderr: String::new() },
                            None,
                            &azrs_result,
                            None,
                        )
                    }
                    Err(e) => reporter::TestResult {
                        name: test.name.clone(),
                        command: test.command.clone(),
                        status: reporter::TestStatus::Error(format!("azrs failed: {}", e)),
                        az_time: None,
                        azrs_time: None,
                        diffs: vec![],
                        az_stdout: Some(az_stdout),
                        azrs_stdout: None,
                    },
                }
            }
            Err(_) => {
                reporter::TestResult {
                    name: test.name.clone(),
                    command: test.command.clone(),
                    status: reporter::TestStatus::Skip(format!("no recording at {}", recording_file.display())),
                    az_time: None,
                    azrs_time: None,
                    diffs: vec![],
                    az_stdout: None,
                    azrs_stdout: None,
                }
            }
        };
        results.push(outcome);
    }

    reporter::print_report(&results, cli.verbose, matches!(cli.output, OutputFormat::Json));
    results.iter().all(|r| matches!(r.status, reporter::TestStatus::Pass))
}
