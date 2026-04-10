/// Generic REST command — mirrors `az rest` / `send_raw_request` from Python CLI.
///
/// Handles URL normalization, auto-auth, {subscriptionId} replacement,
/// header/body parsing, and response output.
use crate::auth::TokenCache;
use crate::cloud::CloudConfig;
use crate::error::{AzrsError, Result};
use crate::http_client::{HttpClient, HttpRequest, ReqwestClient};
use crate::profile::Profile;
use std::collections::HashMap;

/// Execute a raw REST request with automatic authentication.
pub async fn send_raw_request(
    method: &str,
    url: &str,
    headers: Option<&[String]>,
    uri_parameters: Option<&[String]>,
    body: Option<&str>,
    skip_authorization_header: bool,
    resource_override: Option<&str>,
    output_file: Option<&str>,
) -> Result<()> {
    let cloud = CloudConfig::default();
    let profile = Profile::load()?;

    let active_sub = profile.active_subscription();
    let subscription_id = active_sub.map(|s| s.id.as_str()).unwrap_or("");

    // --- URL normalization ---
    let mut final_url = if !url.contains("://") {
        // Treat as ARM resource ID — prepend ARM endpoint
        let base = cloud.resource_manager.trim_end_matches('/');
        format!("{base}{url}")
    } else {
        url.to_string()
    };

    // Replace {subscriptionId} placeholder
    if final_url.contains("{subscriptionId}") {
        if subscription_id.is_empty() {
            return Err(AzrsError::General(
                "URL contains {subscriptionId} but no active subscription is set.".into(),
            ));
        }
        final_url = final_url.replace("{subscriptionId}", subscription_id);
    }

    // --- Build request ---
    let http: Box<dyn HttpClient> = Box::new(ReqwestClient::default());

    let mut custom_headers: HashMap<String, String> = HashMap::new();

    // Add User-Agent
    custom_headers.insert(
        "user-agent".to_string(),
        format!("azrs/{}", env!("CARGO_PKG_VERSION")),
    );

    // Add x-ms-client-request-id
    custom_headers.insert(
        "x-ms-client-request-id".to_string(),
        uuid::Uuid::new_v4().to_string(),
    );

    if let Some(hdrs) = headers {
        for hdr in hdrs {
            // Try JSON first
            if hdr.starts_with('{') {
                if let Ok(map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(hdr) {
                    for (k, v) in map {
                        let val = match v {
                            serde_json::Value::String(s) => s,
                            other => other.to_string(),
                        };
                        custom_headers.insert(k.to_lowercase(), val);
                    }
                    continue;
                }
            }
            // KEY=VALUE format
            if let Some((k, v)) = hdr.split_once('=') {
                custom_headers.insert(k.to_lowercase().to_string(), v.to_string());
            }
        }
    }

    // --- Body ---
    let body_content = body.map(|b| load_body(b));
    if let Some(ref body_str) = body_content {
        // If body is valid JSON, set Content-Type if not already set
        if serde_json::from_str::<serde_json::Value>(body_str).is_ok()
            && !custom_headers.contains_key("content-type")
        {
            custom_headers.insert("content-type".to_string(), "application/json".to_string());
        }
    }

    // --- Authentication ---
    if !skip_authorization_header
        && !custom_headers.contains_key("authorization")
        && final_url.to_lowercase().starts_with("https://")
    {
        let token_resource = if let Some(res) = resource_override {
            Some(res.to_string())
        } else {
            detect_resource_from_url(&final_url, &cloud)
        };

        if let Some(resource) = token_resource {
            let scopes = vec![crate::cloud::resource_to_scope(&resource)];

            // Determine tenant and username from active subscription
            if let Some(sub) = active_sub {
                let mut cache = TokenCache::load(&cloud)?;
                match cache
                    .get_access_token(&sub.user.name, &sub.tenant_id, &scopes, &cloud)
                    .await
                {
                    Ok(token) => {
                        custom_headers.insert(
                            "authorization".to_string(),
                            format!("Bearer {}", token.access_token),
                        );
                        cache.save()?;
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            } else {
                return Err(AzrsError::NoActiveSubscription);
            }
        }
    }

    // --- Query parameters ---
    if let Some(params) = uri_parameters {
        let parsed = parse_key_value_pairs(params)?;
        let query_string: String = parsed
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        if final_url.contains('?') {
            final_url = format!("{final_url}&{query_string}");
        } else {
            final_url = format!("{final_url}?{query_string}");
        }
    }

    let req = HttpRequest {
        method: method.to_uppercase(),
        url: final_url,
        headers: custom_headers,
        body: body_content.map(|s| s.into_bytes()),
    };

    // --- Send request ---
    let response = http.send(req).await?;
    let status = response.status;

    // --- Handle response ---
    if let Some(out_path) = output_file {
        std::fs::write(out_path, &response.body)?;
        eprintln!("Response saved to {out_path} ({} bytes)", response.body.len());
    } else {
        let body_text = response.text();
        let is_success = (200..300).contains(&status);

        if !is_success {
            eprintln!("ERROR: HTTP {status}");
        }

        if body_text.is_empty() {
            if is_success {
                // No content — nothing to print
            }
        } else {
            // Try to pretty-print as JSON
            match serde_json::from_str::<serde_json::Value>(&body_text) {
                Ok(json) => {
                    println!("{}", serde_json::to_string_pretty(&json)?);
                }
                Err(_) => {
                    println!("{body_text}");
                }
            }
        }

        if !is_success {
            return Err(AzrsError::General(format!("Request failed with status {status}")));
        }
    }

    Ok(())
}

/// Detect the token resource from the URL by matching against cloud endpoints.
/// Matches Python `send_raw_request` logic from util.py:1041-1078.
fn detect_resource_from_url(url: &str, cloud: &CloudConfig) -> Option<String> {
    let lower = url.to_lowercase();
    let arm_base = cloud.resource_manager.trim_end_matches('/').to_lowercase();

    // ARM URLs → use active_directory_resource_id
    if lower.starts_with(&arm_base) {
        return Some(cloud.active_directory_resource_id.clone());
    }

    // Check other well-known endpoints
    let known_endpoints = [
        ("https://graph.microsoft.com", "https://graph.microsoft.com/"),
        ("https://graph.windows.net", "https://graph.windows.net/"),
    ];

    for (prefix, resource) in &known_endpoints {
        if lower.starts_with(*prefix) {
            return Some(resource.to_string());
        }
    }

    // Fallback: if it looks like an Azure URL, use ARM resource
    if lower.contains(".azure.com")
        || lower.contains(".azure.net")
        || lower.contains(".windows.net")
    {
        return Some(cloud.active_directory_resource_id.clone());
    }

    None
}

/// Load body content, supporting @file syntax.
fn load_body(body: &str) -> String {
    if let Some(path) = body.strip_prefix('@') {
        match std::fs::read_to_string(path) {
            Ok(content) => content.trim_end_matches('\n').to_string(),
            Err(e) => {
                eprintln!("Warning: Could not read file '{path}': {e}");
                body.to_string()
            }
        }
    } else {
        body.to_string()
    }
}

fn parse_key_value_pairs(items: &[String]) -> Result<Vec<(String, String)>> {
    let mut result = Vec::new();
    for item in items {
        // Try JSON first
        if item.starts_with('{') {
            if let Ok(map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(item) {
                for (k, v) in map {
                    let val = match v {
                        serde_json::Value::String(s) => s,
                        other => other.to_string(),
                    };
                    result.push((k, val));
                }
                continue;
            }
        }
        // KEY=VALUE format
        if let Some((k, v)) = item.split_once('=') {
            result.push((k.to_string(), v.to_string()));
        } else {
            return Err(AzrsError::General(format!(
                "Invalid parameter format '{item}'. Expected KEY=VALUE."
            )));
        }
    }
    Ok(result)
}
