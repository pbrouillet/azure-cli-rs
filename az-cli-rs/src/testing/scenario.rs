/// ScenarioTest — high-level test harness for CLI scenario tests.
///
/// Provides a way to run CLI commands against recorded cassettes or live Azure,
/// with assertion helpers inspired by Python testsdk's ScenarioTest.
use super::cassette::cassette_path;
use super::checkers::{Checker, CmdResult};
use super::fixtures::TestEnv;
use super::processors::{self, GeneralNameReplacer, RecordingProcessor};
use super::recording_client::{RecordingHttpClient, TestMode};
use crate::commands::ArmCommand;
use crate::config::Config;
use std::sync::Arc;

/// Determine the test mode from environment.
pub fn test_mode() -> TestMode {
    if std::env::var("AZURE_TEST_RUN_LIVE").unwrap_or_default() == "1" {
        TestMode::Recording
    } else {
        TestMode::Playback
    }
}

/// High-level test harness — manages cassette, HTTP client, and assertion helpers.
pub struct ScenarioTest {
    pub mode: TestMode,
    pub test_env: TestEnv,
    pub name_replacer: GeneralNameReplacer,
    http: Arc<RecordingHttpClient>,
    moniker_counter: usize,
}

impl ScenarioTest {
    /// Create a new scenario test.
    ///
    /// - `module`: test module name (e.g., "group") — used for cassette path
    /// - `test_name`: test function name — used for cassette file name
    pub fn new(module: &str, test_name: &str) -> Self {
        let mode = test_mode();
        let name_replacer = GeneralNameReplacer::new();

        let mut recording_processors: Vec<Box<dyn RecordingProcessor>> =
            processors::default_recording_processors();
        recording_processors.push(Box::new(name_replacer.clone()));

        let path = cassette_path(module, test_name);
        let http = RecordingHttpClient::new(mode, path, recording_processors)
            .unwrap_or_else(|e| panic!("Failed to initialize test cassette: {e}"));

        let test_env = TestEnv::new();

        Self {
            mode,
            test_env,
            name_replacer,
            http: Arc::new(http),
            moniker_counter: 0,
        }
    }

    /// Get an ArmCommand wired to the recording/playback HTTP client.
    pub fn arm_command(&self) -> ArmCommand {
        ArmCommand::from_parts(
            self.test_env.cloud.clone(),
            self.test_env.profile.clone(),
            self.test_env.cache.clone(),
            Config::default(),
            Box::new(ArcHttpClient(self.http.clone())),
        )
    }

    /// Generate a resource name.
    /// - Recording mode: random name (registered with name replacer for sanitization)
    /// - Playback mode: deterministic moniker (e.g., "clitest.rg000001")
    pub fn create_random_name(&mut self, prefix: &str, len: usize) -> String {
        self.moniker_counter += 1;
        let moniker = format!("{}{:06}", prefix, self.moniker_counter);

        if self.mode == TestMode::Recording {
            // Generate actual random name
            let random_part: String = (0..(len.saturating_sub(prefix.len())))
                .map(|_| {
                    let idx = rand::random::<u8>() % 36;
                    if idx < 10 {
                        (b'0' + idx) as char
                    } else {
                        (b'a' + idx - 10) as char
                    }
                })
                .collect();
            let real_name = format!("{prefix}{random_part}");
            self.name_replacer
                .register_name_pair(&real_name, &moniker);
            real_name
        } else {
            moniker
        }
    }

    /// Finalize the test — save cassette if recording.
    pub fn finish(&self) {
        self.http
            .save_cassette()
            .unwrap_or_else(|e| eprintln!("Warning: failed to save cassette: {e}"));
    }

    // --- Checker builder methods (convenience wrappers) ---

    pub fn check(&self, query: &str, expected: serde_json::Value) -> Checker {
        super::checkers::check(query, expected)
    }

    pub fn exists(&self, query: &str) -> Checker {
        super::checkers::exists(query)
    }

    pub fn not_exists(&self, query: &str) -> Checker {
        super::checkers::not_exists(query)
    }

    pub fn greater_than(&self, query: &str, expected: f64) -> Checker {
        super::checkers::greater_than(query, expected)
    }

    pub fn check_pattern(&self, query: &str, pattern: &str) -> Checker {
        super::checkers::check_pattern(query, pattern)
    }

    pub fn is_empty(&self) -> Checker {
        super::checkers::is_empty()
    }
}

impl Drop for ScenarioTest {
    fn drop(&mut self) {
        // Auto-save cassette on drop if recording (best-effort)
        if self.mode == TestMode::Recording {
            let _ = self.http.save_cassette();
        }
    }
}

/// Wrapper to use Arc<RecordingHttpClient> as HttpClient.
struct ArcHttpClient(Arc<RecordingHttpClient>);

impl crate::http_client::HttpClient for ArcHttpClient {
    fn send(
        &self,
        request: crate::http_client::HttpRequest,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = crate::error::Result<crate::http_client::HttpResponse>>
                + Send
                + '_,
        >,
    > {
        self.0.send(request)
    }
}

/// Helper to create a CmdResult from a command's JSON output.
pub fn cmd_result_from_json(json: serde_json::Value) -> CmdResult {
    CmdResult {
        exit_code: 0,
        stdout: serde_json::to_string_pretty(&json).unwrap_or_default(),
        stderr: String::new(),
        json: Some(json),
    }
}

/// Helper to create a CmdResult from a command that returns no output.
pub fn cmd_result_none() -> CmdResult {
    CmdResult {
        exit_code: 0,
        stdout: String::new(),
        stderr: String::new(),
        json: None,
    }
}

/// Helper to create a CmdResult from an error.
pub fn cmd_result_error(err: &crate::error::AzrsError) -> CmdResult {
    CmdResult {
        exit_code: 1,
        stdout: String::new(),
        stderr: err.to_string(),
        json: None,
    }
}
