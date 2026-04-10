/// Provider commands — `azrs provider list/show/register/unregister`.
///
/// ARM API: Microsoft.Resources (api-version 2024-03-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2024-03-01";

/// `azrs provider list`
pub async fn list(expand: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let mut path = format!("/subscriptions/{{subscriptionId}}/providers?api-version={API_VERSION}");
    if let Some(exp) = expand {
        path.push_str(&format!("&$expand={exp}"));
    }
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs provider show --namespace <namespace>`
pub async fn show(namespace: &str, expand: Option<&str>) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let mut path = format!("/subscriptions/{{subscriptionId}}/providers/{namespace}?api-version={API_VERSION}");
    if let Some(exp) = expand {
        path.push_str(&format!("&$expand={exp}"));
    }
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs provider register --namespace <namespace>`
pub async fn register(namespace: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/providers/{namespace}/register?api-version={API_VERSION}");
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Registered provider '{namespace}'.");
    Ok(result)
}

/// `azrs provider unregister --namespace <namespace>`
pub async fn unregister(namespace: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/providers/{namespace}/unregister?api-version={API_VERSION}");
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Unregistered provider '{namespace}'.");
    Ok(result)
}

/// `azrs provider operation list`
pub async fn operation_list(namespace: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = match namespace {
        Some(ns) => format!("/providers/{ns}/operations?api-version={API_VERSION}"),
        None => format!("/providers/Microsoft.Authorization/providerOperations?api-version=2022-04-01&$expand=resourceTypes"),
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs provider permission list`
pub async fn permission_list(namespace: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/providers/Microsoft.Authorization/providerOperations/{namespace}?api-version=2022-04-01&$expand=resourceTypes");
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}
