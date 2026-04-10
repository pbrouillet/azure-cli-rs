/// HTTP client abstraction — enables test recording/playback by decoupling
/// from reqwest::Client.
///
/// Production code uses `ReqwestClient`; tests inject a `RecordingHttpClient`
/// that records or plays back HTTP interactions from JSON cassettes.
use std::collections::HashMap;

/// An HTTP request in a transport-agnostic form.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// An HTTP response in a transport-agnostic form.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Read the body as a UTF-8 string.
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    /// Check if the status code is 2xx.
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Trait abstracting HTTP transport — implemented by `ReqwestClient` (production)
/// and `RecordingHttpClient` (tests).
pub trait HttpClient: Send + Sync {
    fn send(
        &self,
        request: HttpRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = crate::error::Result<HttpResponse>> + Send + '_>,
    >;
}

/// Production HTTP client backed by reqwest.
#[derive(Clone)]
pub struct ReqwestClient {
    inner: reqwest::Client,
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }
}

impl HttpClient for ReqwestClient {
    fn send(
        &self,
        request: HttpRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = crate::error::Result<HttpResponse>> + Send + '_>,
    > {
        Box::pin(async move {
            let method: reqwest::Method = request.method.parse().map_err(|_| {
                crate::error::AzrsError::General(format!(
                    "Invalid HTTP method: {}",
                    request.method
                ))
            })?;

            let mut builder = self.inner.request(method, &request.url);

            for (key, value) in &request.headers {
                builder = builder.header(key.as_str(), value.as_str());
            }

            if let Some(body) = request.body {
                builder = builder.body(body);
            }

            let resp = builder.send().await?;

            let status = resp.status().as_u16();
            let mut headers = HashMap::new();
            for (name, value) in resp.headers() {
                if let Ok(v) = value.to_str() {
                    headers.insert(name.as_str().to_string(), v.to_string());
                }
            }
            let body = resp.bytes().await?.to_vec();

            Ok(HttpResponse {
                status,
                headers,
                body,
            })
        })
    }
}
