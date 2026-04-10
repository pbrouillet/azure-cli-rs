/// Recording processors — sanitize sensitive data in cassette recordings.
///
/// Inspired by Python testsdk's `recording_processors.py`.
/// Processors transform requests and responses before they are written to cassettes,
/// ensuring no secrets (subscription IDs, tokens, emails) leak into recordings.
use super::cassette::{RecordedRequest, RecordedResponse};
use std::sync::{Arc, Mutex};

/// Trait for processors that transform recorded HTTP interactions.
pub trait RecordingProcessor: Send + Sync {
    /// Transform a request before recording. Return None to omit it from the cassette.
    fn process_request(&self, request: &mut RecordedRequest);
    /// Transform a response before recording.
    fn process_response(&self, response: &mut RecordedResponse);
}

/// Replace subscription IDs in URLs and bodies with a mock value.
pub struct SubscriptionIdProcessor {
    pub mock_id: String,
}

impl SubscriptionIdProcessor {
    pub fn new() -> Self {
        Self {
            mock_id: "00000000-0000-0000-0000-000000000000".to_string(),
        }
    }
}

impl RecordingProcessor for SubscriptionIdProcessor {
    fn process_request(&self, request: &mut RecordedRequest) {
        // Replace UUID-shaped subscription IDs in URL
        let re = regex::Regex::new(
            r"(?i)/subscriptions/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})"
        ).unwrap();
        request.url = re
            .replace_all(&request.url, format!("/subscriptions/{}", self.mock_id))
            .to_string();

        if let Some(ref mut body) = request.body {
            *body = re
                .replace_all(body, format!("/subscriptions/{}", self.mock_id))
                .to_string();
        }
    }

    fn process_response(&self, response: &mut RecordedResponse) {
        let re = regex::Regex::new(
            r"(?i)([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})"
        ).unwrap();
        // Only replace subscription-like IDs in subscription contexts
        let sub_re = regex::Regex::new(
            r"(?i)/subscriptions/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})"
        ).unwrap();
        response.body = sub_re
            .replace_all(&response.body, format!("/subscriptions/{}", self.mock_id))
            .to_string();
        // Also replace in "subscriptionId" JSON fields
        let field_re = regex::Regex::new(
            r#"(?i)"subscriptionId"\s*:\s*"([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})""#
        ).unwrap();
        response.body = field_re
            .replace_all(&response.body, format!(r#""subscriptionId":"{}""#, self.mock_id))
            .to_string();
        // Replace in "/subscriptions/<id>" patterns in id fields
        let _ = re; // used indirectly through sub_re patterns
    }
}

/// Strip or replace Authorization headers.
pub struct AuthHeaderProcessor;

impl RecordingProcessor for AuthHeaderProcessor {
    fn process_request(&self, request: &mut RecordedRequest) {
        // Remove authorization header from recordings
        request.headers.remove("authorization");
        request.headers.remove("Authorization");
    }
    fn process_response(&self, _response: &mut RecordedResponse) {}
}

/// Strip x-ms-client-request-id headers (non-deterministic UUIDs).
pub struct RequestIdProcessor;

impl RecordingProcessor for RequestIdProcessor {
    fn process_request(&self, request: &mut RecordedRequest) {
        request.headers.remove("x-ms-client-request-id");
    }
    fn process_response(&self, response: &mut RecordedResponse) {
        response.headers.remove("x-ms-request-id");
        response.headers.remove("x-ms-correlation-request-id");
    }
}

/// Registry-based name replacement — maps random names to deterministic monikers.
/// Used by preparers to ensure playback works with fixed names.
#[derive(Clone)]
pub struct GeneralNameReplacer {
    pairs: Arc<Mutex<Vec<(String, String)>>>,
}

impl GeneralNameReplacer {
    pub fn new() -> Self {
        Self {
            pairs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a real→mock name pair.
    pub fn register_name_pair(&self, real_name: &str, mock_name: &str) {
        let mut pairs = self.pairs.lock().unwrap();
        pairs.push((real_name.to_string(), mock_name.to_string()));
    }

    fn replace_in_string(&self, input: &str) -> String {
        let pairs = self.pairs.lock().unwrap();
        let mut result = input.to_string();
        for (real, mock) in pairs.iter() {
            result = result.replace(real, mock);
        }
        result
    }
}

impl RecordingProcessor for GeneralNameReplacer {
    fn process_request(&self, request: &mut RecordedRequest) {
        request.url = self.replace_in_string(&request.url);
        if let Some(ref mut body) = request.body {
            *body = self.replace_in_string(body);
        }
    }

    fn process_response(&self, response: &mut RecordedResponse) {
        response.body = self.replace_in_string(&response.body);
    }
}

/// Replace access tokens in response bodies.
pub struct AccessTokenReplacer;

impl RecordingProcessor for AccessTokenReplacer {
    fn process_request(&self, _request: &mut RecordedRequest) {}
    fn process_response(&self, response: &mut RecordedResponse) {
        let re = regex::Regex::new(r#""access_token"\s*:\s*"[^"]+""#).unwrap();
        response.body = re
            .replace_all(&response.body, r#""access_token":"***""#)
            .to_string();
    }
}

/// Build the default processor chain for recording.
pub fn default_recording_processors() -> Vec<Box<dyn RecordingProcessor>> {
    vec![
        Box::new(SubscriptionIdProcessor::new()),
        Box::new(AuthHeaderProcessor),
        Box::new(RequestIdProcessor),
        Box::new(AccessTokenReplacer),
    ]
}
