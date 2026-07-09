/// Azure Maps account commands — `azrs maps account create/list/show/delete/update/keys`.
///
/// ARM API: Microsoft.Maps/accounts (api-version 2021-02-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2021-02-01";

fn account_path(name: &str, resource_group: &str) -> String {
    format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Maps/accounts/{name}?api-version={API_VERSION}"
    )
}

/// `azrs maps account create -n <name> -g <rg> --sku <S0|S1|G2> [-l <location>] [--kind <Gen1|Gen2>]`
pub async fn create(
    name: &str,
    resource_group: &str,
    sku: &str,
    kind: Option<&str>,
    location: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = account_path(name, resource_group);
    let mut body = serde_json::json!({
        "location": location,
        "sku": { "name": sku },
    });
    if let Some(kind) = kind {
        body["kind"] = serde_json::Value::String(kind.to_string());
    }
    eprintln!("Creating Azure Maps account '{name}'...");
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!();
    Ok(result)
}

/// `azrs maps account list [-g <rg>]` — list in a resource group, or across the subscription.
pub async fn list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = if let Some(rg) = resource_group {
        format!("/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Maps/accounts?api-version={API_VERSION}")
    } else {
        format!("/subscriptions/{{subscriptionId}}/providers/Microsoft.Maps/accounts?api-version={API_VERSION}")
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs maps account show -n <name> -g <rg>`
pub async fn show(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = account_path(name, resource_group);
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs maps account delete -n <name> -g <rg>`
pub async fn delete(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = account_path(name, resource_group);
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    eprintln!("Azure Maps account '{name}' deleted.");
    Ok(())
}

/// `azrs maps account update -n <name> -g <rg> [--set key=value ...] [--tags key=value ...]`
///
/// Applies a PATCH with tag updates and/or arbitrary `properties.*` values (dot-path `--set`).
pub async fn update(
    name: &str,
    resource_group: &str,
    tags: Option<&[String]>,
    set_props: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = account_path(name, resource_group);
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

/// `azrs maps account keys list -n <name> -g <rg>`
pub async fn keys_list(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Maps/accounts/{name}/listKeys?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs maps account keys regenerate -n <name> -g <rg> --key <primary|secondary>`
pub async fn keys_regenerate(
    name: &str,
    resource_group: &str,
    key_type: &str,
) -> Result<serde_json::Value> {
    let key_type = match key_type.to_ascii_lowercase().as_str() {
        "primary" => "primary",
        "secondary" => "secondary",
        _ => {
            return Err(crate::error::AzrsError::General(format!(
                "invalid --key '{key_type}': expected primary or secondary"
            )))
        }
    };
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Maps/accounts/{name}/regenerateKey?api-version={API_VERSION}"
    );
    let body = serde_json::json!({ "keyType": key_type });
    let result = cmd.post(&path, Some(&body)).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// TODO: Add Azure Maps creator/data-plane subgroups.

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
