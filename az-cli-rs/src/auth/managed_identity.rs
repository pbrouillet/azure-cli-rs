/// Managed Identity authentication — acquires tokens from Azure IMDS or
/// App Service/Functions identity endpoints.
///
/// Supports system-assigned and user-assigned identities.
use crate::auth::oauth2::TokenResponse;
use crate::error::{AzrsError, Result};

/// IMDS endpoint for Azure VMs.
const IMDS_ENDPOINT: &str =
    "http://169.254.169.254/metadata/identity/oauth2/token";

/// Acquire a token using Azure Managed Identity.
///
/// Tries App Service identity endpoint first (IDENTITY_ENDPOINT + IDENTITY_HEADER),
/// then falls back to IMDS.
pub async fn login(
    resource: &str,
    client_id: Option<&str>,
    object_id: Option<&str>,
    resource_id: Option<&str>,
) -> Result<TokenResponse> {
    // App Service / Azure Functions identity endpoint
    if let (Ok(endpoint), Ok(header)) = (
        std::env::var("IDENTITY_ENDPOINT"),
        std::env::var("IDENTITY_HEADER"),
    ) {
        return login_app_service(&endpoint, &header, resource, client_id, object_id, resource_id).await;
    }

    // IMDS (Azure VM)
    login_imds(resource, client_id, object_id, resource_id).await
}

/// Acquire token from IMDS endpoint (Azure VMs).
async fn login_imds(
    resource: &str,
    client_id: Option<&str>,
    object_id: Option<&str>,
    resource_id: Option<&str>,
) -> Result<TokenResponse> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AzrsError::Auth(format!("Failed to create HTTP client: {e}")))?;

    let mut url = format!("{IMDS_ENDPOINT}?api-version=2018-02-01&resource={resource}");

    if let Some(cid) = client_id {
        url.push_str(&format!("&client_id={cid}"));
    }
    if let Some(oid) = object_id {
        url.push_str(&format!("&object_id={oid}"));
    }
    if let Some(rid) = resource_id {
        url.push_str(&format!("&mi_res_id={rid}"));
    }

    let resp = client
        .get(&url)
        .header("Metadata", "true")
        .send()
        .await
        .map_err(|e| {
            AzrsError::Auth(format!(
                "Failed to reach IMDS endpoint. Are you running on an Azure VM with managed identity enabled? Error: {e}"
            ))
        })?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AzrsError::Auth(format!(
            "Managed identity token acquisition failed (IMDS): {body}"
        )));
    }

    parse_msi_response(resp).await
}

/// Acquire token from App Service identity endpoint.
async fn login_app_service(
    endpoint: &str,
    identity_header: &str,
    resource: &str,
    client_id: Option<&str>,
    object_id: Option<&str>,
    resource_id: Option<&str>,
) -> Result<TokenResponse> {
    let client = reqwest::Client::new();

    let mut url = format!("{endpoint}?api-version=2019-08-01&resource={resource}");

    if let Some(cid) = client_id {
        url.push_str(&format!("&client_id={cid}"));
    }
    if let Some(oid) = object_id {
        url.push_str(&format!("&object_id={oid}"));
    }
    if let Some(rid) = resource_id {
        url.push_str(&format!("&mi_res_id={rid}"));
    }

    let resp = client
        .get(&url)
        .header("X-IDENTITY-HEADER", identity_header)
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AzrsError::Auth(format!(
            "Managed identity token acquisition failed (App Service): {body}"
        )));
    }

    parse_msi_response(resp).await
}

/// Parse MSI token response into our TokenResponse type.
async fn parse_msi_response(resp: reqwest::Response) -> Result<TokenResponse> {
    let raw: serde_json::Value = resp.json().await?;

    let access_token = raw
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AzrsError::Auth("Missing access_token in MSI response".into()))?
        .to_string();

    let expires_in = raw
        .get("expires_in")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<u64>().ok()).or_else(|| v.as_u64()))
        .unwrap_or(3600);

    let resource = raw
        .get("resource")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let token = TokenResponse {
        access_token,
        refresh_token: None,
        id_token: None,
        token_type: "Bearer".to_string(),
        expires_in,
        scope: resource,
        expires_on: chrono::Utc::now(),
        id_token_claims: None,
    };

    Ok(token.finalize())
}
