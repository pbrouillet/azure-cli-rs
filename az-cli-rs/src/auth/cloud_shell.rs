/// Cloud Shell authentication — acquires tokens from the Cloud Shell MSI endpoint.
///
/// Cloud Shell sets `MSI_ENDPOINT` and `MSI_SECRET` environment variables.
/// Token acquisition is a simple POST to the endpoint with the resource.
use crate::auth::oauth2::TokenResponse;
use crate::error::{AzrsError, Result};

/// Check if running inside Azure Cloud Shell.
pub fn is_cloud_shell() -> bool {
    std::env::var("MSI_ENDPOINT").is_ok()
}

/// Acquire a token from the Cloud Shell MSI endpoint.
///
/// POST $MSI_ENDPOINT
///   resource=<resource>
/// Headers:
///   Metadata: true
///   Content-Type: application/x-www-form-urlencoded
pub async fn login(resource: &str) -> Result<TokenResponse> {
    let endpoint = std::env::var("MSI_ENDPOINT").map_err(|_| {
        AzrsError::Auth("Not running in Cloud Shell (MSI_ENDPOINT not set)".into())
    })?;

    let client = reqwest::Client::new();
    let resp = client
        .post(&endpoint)
        .header("Metadata", "true")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[("resource", resource)])
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AzrsError::Auth(format!(
            "Cloud Shell token acquisition failed: {body}"
        )));
    }

    // Cloud Shell returns a slightly different format — map it to TokenResponse
    let raw: serde_json::Value = resp.json().await?;

    let access_token = raw
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AzrsError::Auth("Missing access_token in Cloud Shell response".into()))?
        .to_string();

    let expires_in = raw
        .get("expires_in")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<u64>().ok()).or_else(|| v.as_u64()))
        .unwrap_or(3600);

    let token = TokenResponse {
        access_token,
        refresh_token: None,
        id_token: None,
        token_type: "Bearer".to_string(),
        expires_in,
        scope: Some(resource.to_string()),
        expires_on: chrono::Utc::now(),
        id_token_claims: None,
    };

    Ok(token.finalize())
}
