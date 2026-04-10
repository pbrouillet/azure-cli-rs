/// Static Web App commands — `azrs staticwebapp create/list/show/delete/update/...`.
///
/// ARM API: Microsoft.Web/staticSites (api-version 2024-11-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2024-11-01";

fn static_site_path(resource_group: &str, name: &str) -> String {
    format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/staticSites/{name}?api-version={API_VERSION}"
    )
}

/// `staticwebapp list [--resource-group <rg>]`
pub async fn list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = match resource_group {
        Some(rg) => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Web/staticSites?api-version={API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Web/staticSites?api-version={API_VERSION}"
        ),
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `staticwebapp show`
pub async fn show(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let result = cmd.get(&static_site_path(resource_group, name)).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `staticwebapp create`
pub async fn create(
    resource_group: &str,
    name: &str,
    location: &str,
    source: Option<&str>,
    branch: Option<&str>,
    token: Option<&str>,
    sku: Option<&str>,
    tags: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let mut properties = serde_json::json!({});
    if let Some(s) = source {
        properties["repositoryUrl"] = serde_json::Value::String(s.to_string());
    }
    if let Some(b) = branch {
        properties["branch"] = serde_json::Value::String(b.to_string());
    }
    if let Some(t) = token {
        properties["repositoryToken"] = serde_json::Value::String(t.to_string());
    }

    let mut body = serde_json::json!({
        "location": location,
        "properties": properties,
        "sku": {
            "name": sku.unwrap_or("Free"),
            "tier": sku.unwrap_or("Free")
        }
    });

    if let Some(tag_list) = tags {
        body["tags"] = serde_json::to_value(crate::commands::group::parse_tags(tag_list))?;
    }

    let result = cmd.put(&static_site_path(resource_group, name), &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `staticwebapp delete`
pub async fn delete(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    cmd.delete(&static_site_path(resource_group, name)).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `staticwebapp update`
pub async fn update(
    resource_group: &str,
    name: &str,
    set_values: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = static_site_path(resource_group, name);
    let mut current = cmd.get(&path).await?;

    if let Some(pairs) = set_values {
        for pair in pairs {
            if let Some((key, value)) = pair.split_once('=') {
                crate::commands::resource::set_json_path_pub(&mut current, key, value);
            }
        }
    }

    let result = cmd.put(&path, &current).await?;
    cmd.save_cache()?;
    Ok(result)
}

// --- Appsettings ---

/// `staticwebapp appsettings list`
pub async fn appsettings_list(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/staticSites/{name}/listFunctionAppSettings?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `staticwebapp appsettings set`
pub async fn appsettings_set(
    resource_group: &str,
    name: &str,
    settings: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    // Get current settings
    let list_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/staticSites/{name}/listFunctionAppSettings?api-version={API_VERSION}"
    );
    let current = cmd.post(&list_path, None).await?;
    let mut props = current.get("properties").cloned().unwrap_or(serde_json::json!({}));

    // Merge new settings
    for setting in settings {
        if let Some((k, v)) = setting.split_once('=') {
            props[k] = serde_json::Value::String(v.to_string());
        }
    }

    let put_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/staticSites/{name}/config/functionappsettings?api-version={API_VERSION}"
    );
    let body = serde_json::json!({"properties": props});
    let result = cmd.put(&put_path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `staticwebapp appsettings delete`
pub async fn appsettings_delete(
    resource_group: &str,
    name: &str,
    keys: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let list_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/staticSites/{name}/listFunctionAppSettings?api-version={API_VERSION}"
    );
    let current = cmd.post(&list_path, None).await?;
    let mut props: serde_json::Map<String, serde_json::Value> =
        serde_json::from_value(current.get("properties").cloned().unwrap_or(serde_json::json!({})))
            .unwrap_or_default();

    for key in keys {
        props.remove(key.as_str());
    }

    let put_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/staticSites/{name}/config/functionappsettings?api-version={API_VERSION}"
    );
    let body = serde_json::json!({"properties": props});
    let result = cmd.put(&put_path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

// --- Hostname ---

/// `staticwebapp hostname list`
pub async fn hostname_list(resource_group: &str, name: &str) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/staticSites/{name}/customDomains?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `staticwebapp hostname set`
pub async fn hostname_set(
    resource_group: &str,
    name: &str,
    hostname: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/staticSites/{name}/customDomains/{hostname}?api-version={API_VERSION}"
    );
    let body = serde_json::json!({});
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `staticwebapp hostname delete`
pub async fn hostname_delete(resource_group: &str, name: &str, hostname: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/staticSites/{name}/customDomains/{hostname}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

// --- Environment ---

/// `staticwebapp environment list`
pub async fn environment_list(resource_group: &str, name: &str) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/staticSites/{name}/builds?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `staticwebapp environment show`
pub async fn environment_show(
    resource_group: &str,
    name: &str,
    environment_name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/staticSites/{name}/builds/{environment_name}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `staticwebapp environment delete`
pub async fn environment_delete(
    resource_group: &str,
    name: &str,
    environment_name: &str,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/staticSites/{name}/builds/{environment_name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}
