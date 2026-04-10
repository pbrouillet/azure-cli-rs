/// Test support infrastructure — recording/playback, processors, checkers, fixtures.
///
/// Inspired by the Python Azure CLI test SDK (`azure-cli-testsdk`).
/// Tests run in two modes:
///   - **Playback** (default): reads HTTP interactions from JSON cassettes
///   - **Recording** (`AZURE_TEST_RUN_LIVE=1`): sends real HTTP requests, records to cassettes
pub mod cassette;
pub mod checkers;
pub mod fixtures;
pub mod preparers;
pub mod processors;
pub mod recording_client;
pub mod scenario;

#[cfg(test)]
mod test_group;
