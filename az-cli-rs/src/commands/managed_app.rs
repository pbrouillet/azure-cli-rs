/// Managed application commands — `azrs managedapp create/delete/list` and
/// `azrs managedapp definition create/update/delete/list`.
///
/// ARM API: Microsoft.Solutions/applications (api-version 2021-07-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2021-07-01";

/// `managedapp list [--resource-group <rg>]`
pub async fn list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = match resource_group {
        Some(rg) => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Solutions/applications?api-version={API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Solutions/applications?api-version={API_VERSION}"
        ),
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `managedapp show --resource-group <rg> --name <name>`
pub async fn show(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Solutions/applications/{name}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `managedapp create`
pub async fn create(
    resource_group: &str,
    name: &str,
    kind: &str,
    managed_rg_id: &str,
    location: &str,
    definition_id: Option<&str>,
    parameters: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Solutions/applications/{name}?api-version={API_VERSION}"
    );

    let mut props = serde_json::json!({
        "managedResourceGroupId": managed_rg_id
    });
    if let Some(def_id) = definition_id {
        props["applicationDefinitionId"] = serde_json::Value::String(def_id.to_string());
    }
    if let Some(params) = parameters {
        let parsed: serde_json::Value = serde_json::from_str(params)
            .map_err(|e| crate::error::AzrsError::General(format!("Invalid parameters JSON: {e}")))?;
        props["parameters"] = parsed;
    }

    let body = serde_json::json!({
        "kind": kind,
        "location": location,
        "properties": props
    });

    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `managedapp delete`
pub async fn delete(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Solutions/applications/{name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

// --- Managed Application Definitions ---

/// `managedapp definition list --resource-group <rg>`
pub async fn definition_list(resource_group: &str) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Solutions/applicationDefinitions?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `managedapp definition create`
pub async fn definition_create(
    resource_group: &str,
    name: &str,
    lock_level: &str,
    location: &str,
    display_name: Option<&str>,
    description: Option<&str>,
    package_file_uri: Option<&str>,
    authorizations: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Solutions/applicationDefinitions/{name}?api-version={API_VERSION}"
    );

    let mut props = serde_json::json!({
        "lockLevel": lock_level
    });
    if let Some(dn) = display_name {
        props["displayName"] = serde_json::Value::String(dn.to_string());
    }
    if let Some(desc) = description {
        props["description"] = serde_json::Value::String(desc.to_string());
    }
    if let Some(uri) = package_file_uri {
        props["packageFileUri"] = serde_json::Value::String(uri.to_string());
    }
    if let Some(auths) = authorizations {
        let parsed: serde_json::Value = serde_json::from_str(auths)
            .map_err(|e| crate::error::AzrsError::General(format!("Invalid authorizations JSON: {e}")))?;
        props["authorizations"] = parsed;
    }

    let body = serde_json::json!({
        "location": location,
        "properties": props
    });

    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `managedapp definition delete`
pub async fn definition_delete(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Solutions/applicationDefinitions/{name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `managedapp definition update`
pub async fn definition_update(
    resource_group: &str,
    name: &str,
    lock_level: Option<&str>,
    display_name: Option<&str>,
    description: Option<&str>,
    tags: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Solutions/applicationDefinitions/{name}?api-version={API_VERSION}"
    );

    let mut current = cmd.get(&path).await?;
    if let Some(ll) = lock_level {
        current["properties"]["lockLevel"] = serde_json::Value::String(ll.to_string());
    }
    if let Some(dn) = display_name {
        current["properties"]["displayName"] = serde_json::Value::String(dn.to_string());
    }
    if let Some(desc) = description {
        current["properties"]["description"] = serde_json::Value::String(desc.to_string());
    }
    if let Some(tag_list) = tags {
        current["tags"] = serde_json::to_value(crate::commands::group::parse_tags(tag_list))?;
    }

    let result = cmd.put(&path, &current).await?;
    cmd.save_cache()?;
    Ok(result)
}
