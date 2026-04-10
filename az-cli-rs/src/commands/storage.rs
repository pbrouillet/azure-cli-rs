use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2023-05-01";

pub async fn create(name: &str, resource_group: &str, location: &str, sku: &str, kind: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Storage/storageAccounts/{name}?api-version={API_VERSION}");
    let body = serde_json::json!({
        "sku": { "name": sku },
        "kind": kind,
        "location": location,
        "properties": { "encryption": { "services": { "blob": { "enabled": true } }, "keySource": "Microsoft.Storage" } }
    });
    eprintln!("Creating storage account '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!();
    Ok(result)
}

pub async fn list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = if let Some(rg) = resource_group {
        format!("/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Storage/storageAccounts?api-version={API_VERSION}")
    } else {
        format!("/subscriptions/{{subscriptionId}}/providers/Microsoft.Storage/storageAccounts?api-version={API_VERSION}")
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

pub async fn show(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Storage/storageAccounts/{name}?api-version={API_VERSION}");
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

pub async fn delete(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Storage/storageAccounts/{name}?api-version={API_VERSION}");
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    eprintln!("Storage account '{name}' deleted.");
    Ok(())
}

pub async fn keys_list(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Storage/storageAccounts/{name}/listKeys?api-version={API_VERSION}");
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}
