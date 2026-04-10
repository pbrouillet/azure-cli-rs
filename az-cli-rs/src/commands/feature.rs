/// Feature commands — `azrs feature list/show/register/unregister` and
/// `azrs feature registration list/show/create/delete`.
///
/// ARM API: Microsoft.Features (api-version 2021-07-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2021-07-01";

/// `azrs feature list [--namespace <namespace>]`
pub async fn list(namespace: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = match namespace {
        Some(ns) => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Features/providers/{ns}/features?api-version={API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Features/features?api-version={API_VERSION}"
        ),
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs feature show --namespace <namespace> --name <name>`
pub async fn show(namespace: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/providers/Microsoft.Features/providers/{namespace}/features/{name}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs feature register --namespace <namespace> --name <name>`
pub async fn register(namespace: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/providers/Microsoft.Features/providers/{namespace}/features/{name}/register?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Registered feature '{namespace}/{name}'.");
    Ok(result)
}

/// `azrs feature unregister --namespace <namespace> --name <name>`
pub async fn unregister(namespace: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/providers/Microsoft.Features/providers/{namespace}/features/{name}/unregister?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Unregistered feature '{namespace}/{name}'.");
    Ok(result)
}

// --- Feature Registration ---

const REG_API_VERSION: &str = "2021-07-01";

/// `azrs feature registration list [--namespace <namespace>]`
pub async fn registration_list(namespace: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = match namespace {
        Some(ns) => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Features/featureProviders/{ns}/subscriptionFeatureRegistrations?api-version={REG_API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Features/subscriptionFeatureRegistrations?api-version={REG_API_VERSION}"
        ),
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs feature registration show --namespace <namespace> --name <name>`
pub async fn registration_show(namespace: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/providers/Microsoft.Features/featureProviders/{namespace}/subscriptionFeatureRegistrations/{name}?api-version={REG_API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs feature registration create --namespace <namespace> --name <name>`
pub async fn registration_create(namespace: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/providers/Microsoft.Features/featureProviders/{namespace}/subscriptionFeatureRegistrations/{name}?api-version={REG_API_VERSION}"
    );
    let body = serde_json::json!({});
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs feature registration delete --namespace <namespace> --name <name>`
pub async fn registration_delete(namespace: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/providers/Microsoft.Features/featureProviders/{namespace}/subscriptionFeatureRegistrations/{name}?api-version={REG_API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}
