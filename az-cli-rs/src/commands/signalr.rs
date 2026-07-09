/// Azure SignalR Service commands — `azrs signalr create/list/show/delete/update/key list/key renew`.
///
/// ARM API: Microsoft.SignalRService/signalR (api-version 2023-08-01-preview)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2023-08-01-preview";

/// `azrs signalr create -n <name> -g <rg> -l <location> --sku <Free_F1|Standard_S1|Premium_P1> [--unit-count <n>] [--service-mode <Default|Serverless|Classic>]`
pub async fn create(
    name: &str,
    resource_group: &str,
    location: &str,
    sku: &str,
    unit_count: i64,
    service_mode: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.SignalRService/signalR/{name}?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "location": location,
        "sku": { "name": sku, "capacity": unit_count },
        "properties": {
            "features": [
                { "flag": "ServiceMode", "value": service_mode }
            ]
        }
    });
    eprintln!("Creating SignalR service '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!();
    Ok(result)
}

/// `azrs signalr list [-g <rg>]` — list in a resource group, or across the subscription.
pub async fn list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = if let Some(rg) = resource_group {
        format!("/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.SignalRService/signalR?api-version={API_VERSION}")
    } else {
        format!("/subscriptions/{{subscriptionId}}/providers/Microsoft.SignalRService/signalR?api-version={API_VERSION}")
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs signalr show -n <name> -g <rg>`
pub async fn show(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.SignalRService/signalR/{name}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs signalr delete -n <name> -g <rg>`
pub async fn delete(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.SignalRService/signalR/{name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    eprintln!("SignalR service '{name}' deleted.");
    Ok(())
}

/// `azrs signalr update -n <name> -g <rg> [--set key=value ...] [--tags key=value ...]`
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
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.SignalRService/signalR/{name}?api-version={API_VERSION}"
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

/// `azrs signalr key list -n <name> -g <rg>`
pub async fn key_list(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.SignalRService/signalR/{name}/listKeys?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs signalr key renew -n <name> -g <rg> --key-type <Primary|Secondary>`
pub async fn key_renew(name: &str, resource_group: &str, key_type: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.SignalRService/signalR/{name}/regenerateKey?api-version={API_VERSION}"
    );
    let body = serde_json::json!({ "keyType": key_type });
    let result = cmd.post(&path, Some(&body)).await?;
    cmd.save_cache()?;
    Ok(result)
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

// TODO: Add SignalR subgroups (cors, network-rule, upstream, identity, custom-domain,
// custom-certificate, replica) when their az parity surface is implemented.
