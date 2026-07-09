/// Azure Cache for Redis commands — `azrs redis create/list/show/delete/list-keys/update/regenerate-keys`.
///
/// ARM API: Microsoft.Cache/redis (api-version 2024-11-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2024-11-01";

/// Split an `az`-style `--vm-size` (e.g. `c0`, `p1`) into the ARM sku `family` and `capacity`.
fn parse_vm_size(vm_size: &str) -> Result<(String, i64)> {
    let vm_size = vm_size.trim();
    let mut chars = vm_size.chars();
    let family_char = chars
        .next()
        .ok_or_else(|| crate::error::AzrsError::General("empty --vm-size".into()))?;
    let family = family_char.to_ascii_uppercase().to_string();
    if family != "C" && family != "P" {
        return Err(crate::error::AzrsError::General(format!(
            "invalid --vm-size '{vm_size}': must start with C (Basic/Standard) or P (Premium)"
        )));
    }
    let capacity: i64 = chars.as_str().parse().map_err(|_| {
        crate::error::AzrsError::General(format!(
            "invalid --vm-size '{vm_size}': expected a family letter followed by a capacity number, e.g. c0 or p1"
        ))
    })?;
    Ok((family, capacity))
}

/// `azrs redis create -n <name> -g <rg> -l <location> --sku <Basic|Standard|Premium> --vm-size <c0..p5> [--enable-non-ssl-port] [--redis-version <ver>]`
#[allow(clippy::too_many_arguments)]
pub async fn create(
    name: &str,
    resource_group: &str,
    location: &str,
    sku: &str,
    vm_size: &str,
    enable_non_ssl_port: bool,
    redis_version: Option<&str>,
) -> Result<serde_json::Value> {
    let (family, capacity) = parse_vm_size(vm_size)?;
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Cache/redis/{name}?api-version={API_VERSION}"
    );
    let mut properties = serde_json::json!({
        "sku": { "name": sku, "family": family, "capacity": capacity },
        "enableNonSslPort": enable_non_ssl_port,
    });
    if let Some(ver) = redis_version {
        properties["redisVersion"] = serde_json::Value::String(ver.to_string());
    }
    let body = serde_json::json!({ "location": location, "properties": properties });
    eprintln!("Creating Redis cache '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!();
    Ok(result)
}

/// `azrs redis list [-g <rg>]` — list in a resource group, or across the subscription.
pub async fn list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = if let Some(rg) = resource_group {
        format!("/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Cache/redis?api-version={API_VERSION}")
    } else {
        format!("/subscriptions/{{subscriptionId}}/providers/Microsoft.Cache/redis?api-version={API_VERSION}")
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs redis show -n <name> -g <rg>`
pub async fn show(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Cache/redis/{name}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs redis delete -n <name> -g <rg>`
pub async fn delete(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Cache/redis/{name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    eprintln!("Redis cache '{name}' deleted.");
    Ok(())
}

/// `azrs redis list-keys -n <name> -g <rg>`
pub async fn list_keys(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Cache/redis/{name}/listKeys?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs redis regenerate-keys -n <name> -g <rg> --key-type <Primary|Secondary>`
pub async fn regenerate_keys(name: &str, resource_group: &str, key_type: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Cache/redis/{name}/regenerateKey?api-version={API_VERSION}"
    );
    let body = serde_json::json!({ "keyType": key_type });
    let result = cmd.post(&path, Some(&body)).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs redis update -n <name> -g <rg> [--set key=value ...] [--tags key=value ...]`
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
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Cache/redis/{name}?api-version={API_VERSION}"
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
