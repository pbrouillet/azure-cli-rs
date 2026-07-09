/// Azure App Configuration store commands — `azrs appconfig create/list/show/delete/update/credential list`.
///
/// ARM API: Microsoft.AppConfiguration/configurationStores (api-version 2025-08-01-preview)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2025-08-01-preview";

/// `azrs appconfig create -n <name> -g <rg> -l <location> --sku <Free|Developer|Standard|Premium> [--enable-purge-protection]`
pub async fn create(
    name: &str,
    resource_group: &str,
    location: &str,
    sku: &str,
    enable_purge_protection: bool,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.AppConfiguration/configurationStores/{name}?api-version={API_VERSION}"
    );
    let mut body = serde_json::json!({
        "location": location,
        "sku": { "name": sku },
    });
    if enable_purge_protection {
        body["properties"] = serde_json::json!({ "enablePurgeProtection": true });
    }
    eprintln!("Creating App Configuration store '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!();
    Ok(result)
}

/// `azrs appconfig list [-g <rg>]` — list in a resource group, or across the subscription.
pub async fn list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = if let Some(rg) = resource_group {
        format!("/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.AppConfiguration/configurationStores?api-version={API_VERSION}")
    } else {
        format!("/subscriptions/{{subscriptionId}}/providers/Microsoft.AppConfiguration/configurationStores?api-version={API_VERSION}")
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs appconfig show -n <name> -g <rg>`
pub async fn show(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.AppConfiguration/configurationStores/{name}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs appconfig delete -n <name> -g <rg>`
pub async fn delete(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.AppConfiguration/configurationStores/{name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    eprintln!("App Configuration store '{name}' deleted.");
    Ok(())
}

/// `azrs appconfig credential list -n <name> -g <rg>`
pub async fn credential_list(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.AppConfiguration/configurationStores/{name}/listKeys?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs appconfig update -n <name> -g <rg> [--tags key=value ...] [--set key=value ...]`
///
/// Applies a PATCH with tag updates and/or arbitrary `properties.*` values (dot-path `--set`).
pub async fn update(
    name: &str,
    resource_group: &str,
    tags: Option<&[String]>,
    set_props: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.AppConfiguration/configurationStores/{name}?api-version={API_VERSION}"
    );
    let mut body = serde_json::json!({});
    if let Some(tag_list) = tags {
        body["tags"] = serde_json::to_value(super::group::parse_tags(tag_list))?;
    }
    if let Some(sets) = set_props {
        let mut properties = serde_json::Map::new();
        for s in sets {
            if let Some((k, v)) = s.split_once('=') {
                properties.insert(k.to_string(), parse_scalar(v));
            }
        }
        if !properties.is_empty() {
            body["properties"] = serde_json::Value::Object(properties);
        }
    }
    let resp = cmd.request("PATCH", &path, Some(&body)).await?;
    cmd.save_cache()?;
    if !resp.is_success() {
        return Err(crate::error::AzrsError::General(format!(
            "HTTP {}: {}",
            resp.status,
            resp.text()
        )));
    }
    Ok(serde_json::from_str(&resp.text())?)
}

/// Best-effort scalar coercion for `--set key=value` (bool, integer, else string).
fn parse_scalar(v: &str) -> serde_json::Value {
    if let Ok(b) = v.parse::<bool>() {
        return serde_json::Value::Bool(b);
    }
    if let Ok(i) = v.parse::<i64>() {
        return serde_json::Value::Number(i.into());
    }
    serde_json::Value::String(v.to_string())
}

// TODO: Add App Configuration key-value data-plane commands.
// TODO: Add replica management commands.
// TODO: Add key-value revision commands.
