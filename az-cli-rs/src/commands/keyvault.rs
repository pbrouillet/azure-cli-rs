/// Key Vault secret commands — data-plane operations.
///
/// Uses `https://<vault>.vault.azure.net/` with scope `https://vault.azure.net/.default`
/// (different from ARM management plane).
use crate::auth::TokenCache;
use crate::cloud::CloudConfig;
use crate::error::{AzrsError, Result};
use crate::http_client::{HttpClient, HttpRequest, HttpResponse, ReqwestClient};
use crate::profile::Profile;
use crate::commands::{DEBUG, subscription_override};
use std::collections::HashMap;
use std::sync::atomic::Ordering;

const API_VERSION: &str = "7.4";

/// Get an access token scoped to Key Vault data plane.
async fn vault_token(cloud: &CloudConfig) -> Result<String> {
    let profile = Profile::load()?;

    let sub = if let Some(over) = subscription_override() {
        profile.find_subscription(over)
            .ok_or_else(|| AzrsError::SubscriptionNotFound(over.to_string()))?
    } else {
        profile.active_subscription()
            .ok_or(AzrsError::NoActiveSubscription)?
    };

    let mut cache = TokenCache::load(cloud)?;
    let scopes = vec!["https://vault.azure.net/.default".to_string()];
    let token = cache
        .get_access_token(&sub.user.name, &sub.tenant_id, &scopes, cloud)
        .await?;
    cache.save()?;
    Ok(token.access_token)
}

fn vault_url(vault_name: &str, path: &str) -> String {
    format!("https://{vault_name}.vault.azure.net/{path}?api-version={API_VERSION}")
}

/// Helper to make an authenticated Key Vault request.
async fn vault_request(
    http: &dyn HttpClient,
    method: &str,
    url: &str,
    token: &str,
    body: Option<&serde_json::Value>,
) -> Result<HttpResponse> {
    let debug = DEBUG.load(Ordering::Relaxed);
    if debug {
        eprintln!("DEBUG: {method} {url}");
    }

    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), format!("Bearer {token}"));
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Accept".to_string(), "application/json".to_string());

    let body_bytes = body.map(|b| serde_json::to_vec(b).unwrap_or_default());

    let req = HttpRequest {
        method: method.to_string(),
        url: url.to_string(),
        headers,
        body: body_bytes,
    };
    let resp = http.send(req).await?;

    if debug {
        eprintln!("DEBUG: {method} {url} — {}", resp.status);
    }

    Ok(resp)
}

/// `azrs keyvault secret set --vault-name <v> -n <name> --value <val>`
pub async fn secret_set(vault_name: &str, name: &str, value: &str) -> Result<serde_json::Value> {
    let cloud = CloudConfig::default();
    let token = vault_token(&cloud).await?;
    let url = vault_url(vault_name, &format!("secrets/{name}"));
    let http = ReqwestClient::default();

    let resp = vault_request(&http, "PUT", &url, &token, Some(&serde_json::json!({ "value": value }))).await?;

    if !resp.is_success() {
        return Err(AzrsError::General(format!("Key Vault error ({}): {}", resp.status, resp.text())));
    }
    Ok(serde_json::from_str(&resp.text())?)
}

/// `azrs keyvault secret show --vault-name <v> -n <name>`
pub async fn secret_show(vault_name: &str, name: &str) -> Result<serde_json::Value> {
    let cloud = CloudConfig::default();
    let token = vault_token(&cloud).await?;
    let url = vault_url(vault_name, &format!("secrets/{name}"));
    let http = ReqwestClient::default();

    let resp = vault_request(&http, "GET", &url, &token, None).await?;

    if !resp.is_success() {
        return Err(AzrsError::General(format!("Key Vault error ({}): {}", resp.status, resp.text())));
    }
    Ok(serde_json::from_str(&resp.text())?)
}

/// `azrs keyvault secret list --vault-name <v>`
pub async fn secret_list(vault_name: &str) -> Result<Vec<serde_json::Value>> {
    let cloud = CloudConfig::default();
    let token = vault_token(&cloud).await?;
    let http = ReqwestClient::default();

    let mut results = Vec::new();
    let mut url = vault_url(vault_name, "secrets");

    loop {
        let resp = vault_request(&http, "GET", &url, &token, None).await?;

        if !resp.is_success() {
            return Err(AzrsError::General(format!("Key Vault error ({}): {}", resp.status, resp.text())));
        }

        let parsed: serde_json::Value = serde_json::from_str(&resp.text())?;
        if let Some(values) = parsed.get("value").and_then(|v| v.as_array()) {
            results.extend(values.iter().cloned());
        }

        match parsed.get("nextLink").and_then(|v| v.as_str()) {
            Some(next) if !next.is_empty() => url = next.to_string(),
            _ => break,
        }
    }

    Ok(results)
}

/// `azrs keyvault secret delete --vault-name <v> -n <name>`
pub async fn secret_delete(vault_name: &str, name: &str) -> Result<serde_json::Value> {
    let cloud = CloudConfig::default();
    let token = vault_token(&cloud).await?;
    let url = vault_url(vault_name, &format!("secrets/{name}"));
    let http = ReqwestClient::default();

    let resp = vault_request(&http, "DELETE", &url, &token, None).await?;

    if !resp.is_success() {
        return Err(AzrsError::General(format!("Key Vault error ({}): {}", resp.status, resp.text())));
    }
    let body_text = resp.text();
    if body_text.is_empty() {
        Ok(serde_json::Value::Null)
    } else {
        Ok(serde_json::from_str(&body_text)?)
    }
}
