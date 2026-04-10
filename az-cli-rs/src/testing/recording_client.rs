/// RecordingHttpClient — implements HttpClient with recording/playback support.
///
/// In **recording** mode: delegates to a real HTTP client, captures request/response pairs,
/// applies processor pipeline, and saves to a cassette file.
///
/// In **playback** mode: matches incoming requests against cassette entries (by method +
/// URL path + sorted query params) and returns the recorded response.
use super::cassette::{Cassette, CassetteEntry, RecordedRequest, RecordedResponse};
use super::processors::RecordingProcessor;
use crate::http_client::{HttpClient, HttpRequest, HttpResponse, ReqwestClient};
use std::path::PathBuf;
use std::sync::Mutex;

/// Test mode — recording (live) or playback (from cassette).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestMode {
    Recording,
    Playback,
}

/// HTTP client that records or plays back interactions.
pub struct RecordingHttpClient {
    mode: TestMode,
    cassette_path: PathBuf,
    /// Playback: pre-loaded entries; Recording: empty, filled during test.
    cassette: Mutex<Cassette>,
    /// Next playback index (sequential matching).
    playback_index: Mutex<usize>,
    /// Processor pipeline applied during recording.
    processors: Vec<Box<dyn RecordingProcessor>>,
    /// Backing real client (recording mode only).
    real_client: Option<ReqwestClient>,
}

impl RecordingHttpClient {
    /// Create a new recording client in the specified mode.
    pub fn new(
        mode: TestMode,
        cassette_path: PathBuf,
        processors: Vec<Box<dyn RecordingProcessor>>,
    ) -> std::io::Result<Self> {
        let cassette = match mode {
            TestMode::Playback => Cassette::load(&cassette_path)?,
            TestMode::Recording => Cassette::new(),
        };
        let real_client = match mode {
            TestMode::Recording => Some(ReqwestClient::default()),
            TestMode::Playback => None,
        };
        Ok(Self {
            mode,
            cassette_path,
            cassette: Mutex::new(cassette),
            playback_index: Mutex::new(0),
            processors,
            real_client,
        })
    }

    /// Save the recorded cassette to disk (call after test completes successfully).
    pub fn save_cassette(&self) -> std::io::Result<()> {
        if self.mode == TestMode::Recording {
            let cassette = self.cassette.lock().unwrap();
            cassette.save(&self.cassette_path)?;
        }
        Ok(())
    }

    /// Find the best-matching playback entry for a request.
    fn find_playback_match(&self, request: &HttpRequest) -> Option<RecordedResponse> {
        let cassette = self.cassette.lock().unwrap();
        let mut index = self.playback_index.lock().unwrap();

        // Try sequential match first (most common case)
        if *index < cassette.entries.len() {
            let entry = &cassette.entries[*index];
            if matches_request(&entry.request, request) {
                let response = entry.response.clone();
                *index += 1;
                return Some(response);
            }
        }

        // Fallback: scan remaining entries for best match
        for i in *index..cassette.entries.len() {
            let entry = &cassette.entries[i];
            if matches_request(&entry.request, request) {
                let response = entry.response.clone();
                *index = i + 1;
                return Some(response);
            }
        }

        None
    }
}

/// Check if a recorded request matches an incoming request.
/// Compares method, URL path, and sorted query parameters.
fn matches_request(recorded: &RecordedRequest, incoming: &HttpRequest) -> bool {
    if recorded.method.to_uppercase() != incoming.method.to_uppercase() {
        return false;
    }

    let (recorded_path, recorded_params) = split_url(&recorded.url);
    let (incoming_path, incoming_params) = split_url(&incoming.url);

    if recorded_path != incoming_path {
        return false;
    }

    // Compare sorted query params (ignoring x-ms-client-request-id and api-version ordering)
    let mut rec_params = parse_query_params(&recorded_params);
    let mut inc_params = parse_query_params(&incoming_params);

    // Remove non-deterministic params
    rec_params.retain(|(k, _)| k != "x-ms-client-request-id");
    inc_params.retain(|(k, _)| k != "x-ms-client-request-id");

    rec_params.sort();
    inc_params.sort();

    rec_params == inc_params
}

/// Split a URL into path and query string.
fn split_url(url: &str) -> (String, String) {
    if let Some(idx) = url.find('?') {
        (url[..idx].to_string(), url[idx + 1..].to_string())
    } else {
        (url.to_string(), String::new())
    }
}

/// Parse query string into sorted key=value pairs.
fn parse_query_params(query: &str) -> Vec<(String, String)> {
    if query.is_empty() {
        return Vec::new();
    }
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.to_lowercase();
            let value = parts.next().unwrap_or("").to_string();
            Some((key, value))
        })
        .collect()
}

impl HttpClient for RecordingHttpClient {
    fn send(
        &self,
        request: HttpRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = crate::error::Result<HttpResponse>> + Send + '_>,
    > {
        Box::pin(async move {
            match self.mode {
                TestMode::Recording => {
                    // Send real request
                    let real = self.real_client.as_ref().unwrap();
                    let response = real.send(request.clone()).await?;

                    // Build recorded entry
                    let mut recorded_req = RecordedRequest {
                        method: request.method.clone(),
                        url: request.url.clone(),
                        headers: request.headers.clone(),
                        body: request
                            .body
                            .as_ref()
                            .map(|b| String::from_utf8_lossy(b).to_string()),
                    };
                    let mut recorded_resp = RecordedResponse {
                        status: response.status,
                        headers: response.headers.clone(),
                        body: response.text(),
                    };

                    // Apply processors
                    for proc in &self.processors {
                        proc.process_request(&mut recorded_req);
                        proc.process_response(&mut recorded_resp);
                    }

                    // Store in cassette
                    self.cassette.lock().unwrap().push(CassetteEntry {
                        request: recorded_req,
                        response: recorded_resp,
                    });

                    Ok(response)
                }
                TestMode::Playback => {
                    // Find matching recorded response
                    let recorded = self.find_playback_match(&request).ok_or_else(|| {
                        crate::error::AzrsError::General(format!(
                            "No cassette match for {} {}",
                            request.method, request.url
                        ))
                    })?;

                    Ok(HttpResponse {
                        status: recorded.status,
                        headers: recorded.headers,
                        body: recorded.body.into_bytes(),
                    })
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_request_basic() {
        let recorded = RecordedRequest {
            method: "GET".to_string(),
            url: "https://management.azure.com/subscriptions/00000000-0000-0000-0000-000000000000/resourcegroups?api-version=2024-03-01".to_string(),
            headers: Default::default(),
            body: None,
        };
        let incoming = HttpRequest {
            method: "GET".to_string(),
            url: "https://management.azure.com/subscriptions/00000000-0000-0000-0000-000000000000/resourcegroups?api-version=2024-03-01".to_string(),
            headers: Default::default(),
            body: None,
        };
        assert!(matches_request(&recorded, &incoming));
    }

    #[test]
    fn test_matches_request_ignores_request_id() {
        let recorded = RecordedRequest {
            method: "GET".to_string(),
            url: "https://example.com/path?api-version=1".to_string(),
            headers: Default::default(),
            body: None,
        };
        let incoming = HttpRequest {
            method: "GET".to_string(),
            url: "https://example.com/path?api-version=1&x-ms-client-request-id=abc".to_string(),
            headers: Default::default(),
            body: None,
        };
        assert!(matches_request(&recorded, &incoming));
    }

    #[test]
    fn test_matches_request_different_method() {
        let recorded = RecordedRequest {
            method: "GET".to_string(),
            url: "https://example.com/path".to_string(),
            headers: Default::default(),
            body: None,
        };
        let incoming = HttpRequest {
            method: "PUT".to_string(),
            url: "https://example.com/path".to_string(),
            headers: Default::default(),
            body: None,
        };
        assert!(!matches_request(&recorded, &incoming));
    }
}
