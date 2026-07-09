/// Azure Container Registry commands — `azrs acr create/list/show/delete/update/credential`.
///
/// ARM API: Microsoft.ContainerRegistry/registries (api-version 2023-07-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2023-07-01";

/// `azrs acr create -n <name> -g <rg> -l <location> --sku <Basic|Standard|Premium> [--admin-enabled]`
pub async fn create(
    name: &str,
    resource_group: &str,
    location: &str,
    sku: &str,
    admin_enabled: bool,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.ContainerRegistry/registries/{name}?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "location": location,
        "sku": { "name": sku },
        "properties": {
            "adminUserEnabled": admin_enabled,
        },
    });
    eprintln!("Creating container registry '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!();
    Ok(result)
}

/// `azrs acr list [-g <rg>]` — list in a resource group, or across the subscription.
pub async fn list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = if let Some(rg) = resource_group {
        format!("/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.ContainerRegistry/registries?api-version={API_VERSION}")
    } else {
        format!("/subscriptions/{{subscriptionId}}/providers/Microsoft.ContainerRegistry/registries?api-version={API_VERSION}")
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs acr show -n <name> -g <rg>`
pub async fn show(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.ContainerRegistry/registries/{name}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs acr delete -n <name> -g <rg>`
pub async fn delete(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.ContainerRegistry/registries/{name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    eprintln!("Container registry '{name}' deleted.");
    Ok(())
}

/// `azrs acr update -n <name> -g <rg> [--set key=value ...] [--tags key=value ...]`
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
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.ContainerRegistry/registries/{name}?api-version={API_VERSION}"
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

/// `azrs acr credential show -n <name> -g <rg>`
pub async fn credential_show(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.ContainerRegistry/registries/{name}/listCredentials?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs acr credential renew -n <name> -g <rg> --password-name <password|password2>`
pub async fn credential_renew(
    name: &str,
    resource_group: &str,
    password_name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.ContainerRegistry/registries/{name}/regenerateCredential?api-version={API_VERSION}"
    );
    let body = serde_json::json!({ "name": password_name });
    let result = cmd.post(&path, Some(&body)).await?;
    cmd.save_cache()?;
    Ok(result)
}

// TODO: Implement ACR data-plane subgroups such as repository after a data-plane client is available.

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
