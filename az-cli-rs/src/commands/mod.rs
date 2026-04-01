/// ARM command framework — shared infrastructure for typed service commands.
///
/// Provides authenticated HTTP methods (get/put/delete/list) against ARM,
/// with subscription context, pagination, and error handling.
pub mod appservice;
pub mod deployment;
pub mod feature;
pub mod functionapp;
pub mod functionapp_ext;
pub mod group;
pub mod keyvault;
pub mod lock;
pub mod logicapp;
pub mod managed_app;
pub mod management_group;
pub mod network;
pub mod provider;
pub mod resource;
pub mod stack;
pub mod staticwebapp;
pub mod storage;
pub mod tag;
pub mod template_specs;
pub mod vm;
pub mod vm_ext;
pub mod vmss_ext;
pub mod webapp;

use crate::auth::TokenCache;
use crate::cloud::CloudConfig;
use crate::config::Config;
use crate::error::{AzrsError, Result};
use crate::http_client::{HttpClient, HttpRequest, HttpResponse, ReqwestClient};
use crate::profile::Profile;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

/// Global debug flag — when set, ARM requests print method+URL+status to stderr.
pub static DEBUG: AtomicBool = AtomicBool::new(false);

/// Global subscription override (set from --subscription CLI arg).
static SUBSCRIPTION_OVERRIDE: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Set the global debug flag.
pub fn set_debug(enabled: bool) {
    DEBUG.store(enabled, Ordering::Relaxed);
}

/// Set the global subscription override.
pub fn set_subscription_override(sub: Option<String>) {
    if let Some(s) = sub {
        let _ = SUBSCRIPTION_OVERRIDE.set(s);
    }
}

fn subscription_override() -> Option<&'static str> {
    SUBSCRIPTION_OVERRIDE.get().map(|s| s.as_str())
}

/// Shared context for ARM commands — handles auth, subscription, and HTTP.
pub struct ArmCommand {
    pub cloud: CloudConfig,
    pub profile: Profile,
    pub cache: TokenCache,
    #[allow(dead_code)]
    pub config: Config,
    http: Box<dyn HttpClient>,
}

impl ArmCommand {
    /// Create a new ArmCommand, loading profile, token cache, and config.
    pub fn new() -> Result<Self> {
        Self::new_with_http(Box::new(ReqwestClient::default()))
    }

    /// Create a new ArmCommand with a custom HTTP client (for testing).
    pub fn new_with_http(http: Box<dyn HttpClient>) -> Result<Self> {
        let cloud = CloudConfig::default();
        let profile = Profile::load()?;
        let cache = TokenCache::load(&cloud)?;
        let config = Config::load();
        Ok(Self {
            cloud,
            profile,
            cache,
            config,
            http,
        })
    }

    /// Create an ArmCommand from pre-built components (for testing).
    pub fn from_parts(
        cloud: CloudConfig,
        profile: Profile,
        cache: TokenCache,
        config: Config,
        http: Box<dyn HttpClient>,
    ) -> Self {
        Self {
            cloud,
            profile,
            cache,
            config,
            http,
        }
    }

    /// Get active subscription ID — respects --subscription override.
    pub fn subscription_id(&self) -> Result<&str> {
        if let Some(over) = subscription_override() {
            // Find by ID or name
            return self.profile
                .find_subscription(over)
                .map(|s| s.id.as_str())
                .ok_or_else(|| AzrsError::SubscriptionNotFound(over.to_string()));
        }
        self.profile
            .active_subscription()
            .map(|s| s.id.as_str())
            .ok_or(AzrsError::NoActiveSubscription)
    }

    /// Get active subscription's tenant ID.
    pub fn tenant_id(&self) -> Result<&str> {
        if let Some(over) = subscription_override() {
            return self.profile
                .find_subscription(over)
                .map(|s| s.tenant_id.as_str())
                .ok_or_else(|| AzrsError::SubscriptionNotFound(over.to_string()));
        }
        self.profile
            .active_subscription()
            .map(|s| s.tenant_id.as_str())
            .ok_or(AzrsError::NoActiveSubscription)
    }

    /// Get active subscription's username.
    pub fn username(&self) -> Result<&str> {
        if let Some(over) = subscription_override() {
            return self.profile
                .find_subscription(over)
                .map(|s| s.user.name.as_str())
                .ok_or_else(|| AzrsError::SubscriptionNotFound(over.to_string()));
        }
        self.profile
            .active_subscription()
            .map(|s| s.user.name.as_str())
            .ok_or(AzrsError::NoActiveSubscription)
    }

    /// Resolve a resource group — uses explicit value, or falls back to config default.
    #[allow(dead_code)]
    pub fn resolve_group<'a>(&'a self, explicit: Option<&'a str>) -> Result<&'a str> {
        explicit
            .or_else(|| self.config.default_group())
            .ok_or_else(|| AzrsError::General(
                "No resource group specified. Use -g/--resource-group or set a default with: azrs config set defaults.group=<name>".into()
            ))
    }

    /// Resolve a location — uses explicit value, or falls back to config default.
    #[allow(dead_code)]
    pub fn resolve_location<'a>(&'a self, explicit: Option<&'a str>) -> Result<&'a str> {
        explicit
            .or_else(|| self.config.default_location())
            .ok_or_else(|| AzrsError::General(
                "No location specified. Use -l/--location or set a default with: azrs config set defaults.location=<name>".into()
            ))
    }

    /// Build a full ARM URL from a relative path.
    /// Replaces {subscriptionId} with the active subscription.
    pub fn arm_url(&self, path: &str) -> Result<String> {
        let base = self.cloud.resource_manager.trim_end_matches('/');
        let sub_id = self.subscription_id()?;
        let url = format!("{base}{path}").replace("{subscriptionId}", sub_id);
        Ok(url)
    }

    /// Get a Bearer token for ARM.
    async fn bearer_token(&mut self) -> Result<String> {
        let username = self.username()?.to_string();
        let tenant = self.tenant_id()?.to_string();
        let scopes = vec![self.cloud.default_scope()];
        let token = self
            .cache
            .get_access_token(&username, &tenant, &scopes, &self.cloud)
            .await?;
        Ok(token.access_token)
    }

    /// Make an authenticated ARM request and return the response.
    pub async fn request(
        &mut self,
        method: &str,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<HttpResponse> {
        let url = self.arm_url(path)?;
        let token = self.bearer_token().await?;
        let debug = DEBUG.load(Ordering::Relaxed);

        if debug {
            eprintln!("DEBUG: {} {}", method, url);
        }

        let start = std::time::Instant::now();

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {token}"));
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Accept".to_string(), "application/json".to_string());
        headers.insert(
            "x-ms-client-request-id".to_string(),
            uuid::Uuid::new_v4().to_string(),
        );

        let body_bytes = body.map(|b| serde_json::to_vec(b).unwrap_or_default());

        let req = HttpRequest {
            method: method.to_string(),
            url: url.clone(),
            headers,
            body: body_bytes,
        };

        let resp = self.http.send(req).await?;

        if debug {
            let elapsed = start.elapsed();
            eprintln!("DEBUG: {} {} — {} ({:.0?})", method, url, resp.status, elapsed);
        }

        Ok(resp)
    }

    /// GET a resource and return the JSON body.
    pub async fn get(&mut self, path: &str) -> Result<serde_json::Value> {
        let resp = self.request("GET", path, None).await?;

        if !resp.is_success() {
            return Err(parse_arm_error(resp.status, &resp.text()));
        }

        Ok(serde_json::from_str(&resp.text())?)
    }

    /// PUT a resource and return the JSON response.
    pub async fn put(
        &mut self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let resp = self.request("PUT", path, Some(body)).await?;

        if !resp.is_success() {
            return Err(parse_arm_error(resp.status, &resp.text()));
        }

        let body_text = resp.text();
        if body_text.is_empty() {
            Ok(serde_json::Value::Null)
        } else {
            Ok(serde_json::from_str(&body_text)?)
        }
    }

    /// DELETE a resource.
    pub async fn delete(&mut self, path: &str) -> Result<()> {
        let resp = self.request("DELETE", path, None).await?;

        if !resp.is_success() && resp.status != 204 {
            return Err(parse_arm_error(resp.status, &resp.text()));
        }

        Ok(())
    }

    /// PUT a resource with LRO polling (for creates/updates that return 202).
    /// Polls until the operation reaches a terminal state, then returns the final resource.
    pub async fn put_lro(
        &mut self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.put_lro_opt(path, body, false).await
    }

    /// PUT with LRO, with optional --no-wait.
    pub async fn put_lro_opt(
        &mut self,
        path: &str,
        body: &serde_json::Value,
        no_wait: bool,
    ) -> Result<serde_json::Value> {
        let resp = self.request("PUT", path, Some(body)).await?;
        let status = resp.status;

        match status {
            200 | 201 => {
                // Synchronous success — but may still have an async operation header
                let async_url = extract_poll_url(&resp);
                let result: serde_json::Value = serde_json::from_str(&resp.text())?;

                if let Some(poll_url) = async_url {
                    // Poll even on 200/201 if Azure-AsyncOperation is present
                    let token = self.bearer_token().await?;
                    self.poll_until_done(&poll_url, &token).await?;
                }
                Ok(result)
            }
            202 => {
                // Accepted — must poll (unless --no-wait)
                let poll_url = extract_poll_url(&resp);

                if no_wait {
                    let result = serde_json::json!({
                        "status": "InProgress",
                        "operationUrl": poll_url,
                    });
                    return Ok(result);
                }

                let poll_url = poll_url.ok_or_else(|| {
                    AzrsError::General(
                        "LRO returned 202 but no polling URL in response headers".into(),
                    )
                })?;
                let location = resp
                    .headers
                    .get("location")
                    .or_else(|| resp.headers.get("Location"))
                    .cloned();

                let token = self.bearer_token().await?;
                self.poll_until_done(&poll_url, &token).await?;

                // After polling completes, GET the resource to return the final state
                if let Some(loc) = location {
                    let mut headers = HashMap::new();
                    headers.insert("Authorization".to_string(), format!("Bearer {token}"));
                    headers.insert("Accept".to_string(), "application/json".to_string());
                    let req = HttpRequest {
                        method: "GET".to_string(),
                        url: loc,
                        headers,
                        body: None,
                    };
                    let final_resp = self.http.send(req).await?;
                    let body_text = final_resp.text();
                    if body_text.is_empty() {
                        // Fallback: GET the original resource
                        return self.get(path).await;
                    }
                    Ok(serde_json::from_str(&body_text)?)
                } else {
                    // Fallback: GET the original resource
                    self.get(path).await
                }
            }
            _ => {
                Err(parse_arm_error(status, &resp.text()))
            }
        }
    }

    /// POST an action with LRO polling (e.g. vm start, vm deallocate).
    /// These typically return 202 Accepted with no body, just poll headers.
    pub async fn post_lro(
        &mut self,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<()> {
        self.post_lro_opt(path, body, false).await
    }

    /// POST action with LRO, with optional --no-wait.
    pub async fn post_lro_opt(
        &mut self,
        path: &str,
        body: Option<&serde_json::Value>,
        no_wait: bool,
    ) -> Result<()> {
        let resp = self.request("POST", path, body).await?;
        let status = resp.status;

        match status {
            200 | 204 => Ok(()),
            202 => {
                if no_wait {
                    eprintln!("Operation accepted (--no-wait).");
                    if let Some(url) = extract_poll_url(&resp) {
                        eprintln!("Poll URL: {url}");
                    }
                    return Ok(());
                }
                let poll_url = extract_poll_url(&resp).ok_or_else(|| {
                    AzrsError::General(
                        "LRO returned 202 but no polling URL in response headers".into(),
                    )
                })?;
                let token = self.bearer_token().await?;
                self.poll_until_done(&poll_url, &token).await
            }
            _ => {
                Err(parse_arm_error(status, &resp.text()))
            }
        }
    }

    /// POST a sync action that returns a JSON body (e.g. listKeys).
    pub async fn post(
        &mut self,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let resp = self.request("POST", path, body).await?;

        if !resp.is_success() {
            return Err(parse_arm_error(resp.status, &resp.text()));
        }

        let body_text = resp.text();
        if body_text.is_empty() {
            Ok(serde_json::Value::Null)
        } else {
            Ok(serde_json::from_str(&body_text)?)
        }
    }

    /// Poll an LRO URL until terminal state.
    async fn poll_until_done(&self, poll_url: &str, token: &str) -> Result<()> {
        let mut interval = std::time::Duration::from_secs(5);

        loop {
            tokio::time::sleep(interval).await;

            let mut headers = HashMap::new();
            headers.insert("Authorization".to_string(), format!("Bearer {token}"));
            headers.insert("Accept".to_string(), "application/json".to_string());
            let req = HttpRequest {
                method: "GET".to_string(),
                url: poll_url.to_string(),
                headers,
                body: None,
            };
            let resp = self.http.send(req).await?;

            let retry_after = resp
                .headers
                .get("retry-after")
                .or_else(|| resp.headers.get("Retry-After"))
                .and_then(|v| v.parse::<u64>().ok());

            let status_code = resp.status;

            // 200/201 with a body containing "status"
            if status_code == 200 || status_code == 201 {
                let body = resp.text();
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(op_status) = parsed.get("status").and_then(|v| v.as_str()) {
                        match op_status {
                            "Succeeded" => return Ok(()),
                            "Failed" => {
                                let msg = parsed
                                    .get("error")
                                    .and_then(|e| e.get("message"))
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("LRO operation failed");
                                return Err(AzrsError::General(msg.to_string()));
                            }
                            "Canceled" | "Cancelled" => {
                                return Err(AzrsError::General(
                                    "Operation was canceled".to_string(),
                                ));
                            }
                            _ => {
                                // Still in progress
                                eprint!(".");
                            }
                        }
                    } else {
                        // No "status" field — treat as complete
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            } else if status_code == 202 {
                // Still processing
                eprint!(".");
            } else if status_code == 204 {
                return Ok(());
            } else {
                return Err(parse_arm_error(status_code, &resp.text()));
            }

            if let Some(secs) = retry_after {
                interval = std::time::Duration::from_secs(secs);
            }
        }
    }

    /// LIST resources with automatic nextLink pagination.
    /// Returns the combined `value` arrays from all pages.
    pub async fn list(&mut self, path: &str) -> Result<Vec<serde_json::Value>> {
        let mut results = Vec::new();
        let mut url = self.arm_url(path)?;
        let token = self.bearer_token().await?;

        loop {
            let mut headers = HashMap::new();
            headers.insert("Authorization".to_string(), format!("Bearer {token}"));
            headers.insert("Accept".to_string(), "application/json".to_string());
            headers.insert(
                "x-ms-client-request-id".to_string(),
                uuid::Uuid::new_v4().to_string(),
            );
            let req = HttpRequest {
                method: "GET".to_string(),
                url: url.clone(),
                headers,
                body: None,
            };
            let resp = self.http.send(req).await?;

            if !resp.is_success() {
                return Err(parse_arm_error(resp.status, &resp.text()));
            }

            let parsed: serde_json::Value = serde_json::from_str(&resp.text())?;

            if let Some(values) = parsed.get("value").and_then(|v| v.as_array()) {
                results.extend(values.iter().cloned());
            } else {
                // Single object response (not a list)
                results.push(parsed);
                break;
            }

            // Follow nextLink
            match parsed.get("nextLink").and_then(|v| v.as_str()) {
                Some(next) if !next.is_empty() => url = next.to_string(),
                _ => break,
            }
        }

        Ok(results)
    }

    /// Check if a resource exists (HEAD or GET returning 204/200 vs 404).
    pub async fn exists(&mut self, path: &str) -> Result<bool> {
        let resp = self.request("HEAD", path, None).await?;
        match resp.status {
            200 | 204 => Ok(true),
            404 => Ok(false),
            status => {
                Err(parse_arm_error(status, &resp.text()))
            }
        }
    }

    /// Save cache after operations.
    pub fn save_cache(&self) -> Result<()> {
        self.cache.save()
    }
}

/// Parse an ARM error response into an AzrsError.
fn parse_arm_error(status: u16, body: &str) -> AzrsError {
    // Try to extract error.message from ARM error envelope
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(error) = parsed.get("error") {
            let code = error
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let message = error
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return AzrsError::General(format!("({code}) {message}"));
        }
    }
    AzrsError::General(format!("HTTP {status}: {body}"))
}

/// Extract the LRO polling URL from response headers.
/// Prefers `Azure-AsyncOperation` over `Location`.
fn extract_poll_url(resp: &HttpResponse) -> Option<String> {
    resp.headers
        .get("azure-asyncoperation")
        .or_else(|| resp.headers.get("Azure-AsyncOperation"))
        .or_else(|| resp.headers.get("location"))
        .or_else(|| resp.headers.get("Location"))
        .cloned()
}
