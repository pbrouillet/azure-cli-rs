use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2024-07-01";

pub async fn vnet_create(name: &str, resource_group: &str, location: &str, address_prefixes: &[String]) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Network/virtualNetworks/{name}?api-version={API_VERSION}");
    let body = serde_json::json!({ "location": location, "properties": { "addressSpace": { "addressPrefixes": address_prefixes } } });
    eprintln!("Creating virtual network '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!();
    Ok(result)
}

pub async fn vnet_list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = if let Some(rg) = resource_group {
        format!("/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Network/virtualNetworks?api-version={API_VERSION}")
    } else {
        format!("/subscriptions/{{subscriptionId}}/providers/Microsoft.Network/virtualNetworks?api-version={API_VERSION}")
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

pub async fn vnet_show(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Network/virtualNetworks/{name}?api-version={API_VERSION}");
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

pub async fn vnet_delete(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Network/virtualNetworks/{name}?api-version={API_VERSION}");
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    eprintln!("Virtual network '{name}' deleted.");
    Ok(())
}

pub async fn nsg_create(name: &str, resource_group: &str, location: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Network/networkSecurityGroups/{name}?api-version={API_VERSION}");
    let body = serde_json::json!({ "location": location });
    eprintln!("Creating network security group '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!();
    Ok(result)
}

pub async fn nsg_list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = if let Some(rg) = resource_group {
        format!("/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Network/networkSecurityGroups?api-version={API_VERSION}")
    } else {
        format!("/subscriptions/{{subscriptionId}}/providers/Microsoft.Network/networkSecurityGroups?api-version={API_VERSION}")
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

pub async fn nsg_show(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Network/networkSecurityGroups/{name}?api-version={API_VERSION}");
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}
