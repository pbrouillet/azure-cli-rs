/// Function App commands — `azrs functionapp create/list/show/delete/stop/start/restart/...`.
///
/// ARM API: Microsoft.Web/sites (api-version 2024-11-01)
use super::ArmCommand;
use crate::error::{AzrsError, Result};

const API_VERSION: &str = "2024-11-01";

fn site_path(resource_group: &str, name: &str) -> String {
    format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}?api-version={API_VERSION}"
    )
}

/// `functionapp list [--resource-group <rg>]`
/// Returns only sites whose `kind` contains "functionapp".
pub async fn list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = match resource_group {
        Some(rg) => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Web/sites?api-version={API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Web/sites?api-version={API_VERSION}"
        ),
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    // Filter to function apps only
    let filtered = results
        .into_iter()
        .filter(|site| {
            site.get("kind")
                .and_then(|k| k.as_str())
                .map(|k| k.to_lowercase().contains("functionapp"))
                .unwrap_or(false)
        })
        .collect();
    Ok(filtered)
}

/// `functionapp show`
pub async fn show(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let result = cmd.get(&site_path(resource_group, name)).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp create`
pub async fn create(
    resource_group: &str,
    name: &str,
    plan: Option<&str>,
    consumption: bool,
    runtime: Option<&str>,
    os_type: Option<&str>,
    storage_account: Option<&str>,
    location: Option<&str>,
    tags: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let sub_id = cmd.subscription_id()?.to_string();

    let mut site_config = serde_json::json!({});
    let os = os_type.unwrap_or("Linux");

    if let Some(rt) = runtime {
        let normalized = rt.replace(':', "|").to_uppercase();
        if os.eq_ignore_ascii_case("linux") {
            site_config["linuxFxVersion"] = serde_json::Value::String(normalized);
        }
    }

    let mut properties = serde_json::json!({
        "siteConfig": site_config
    });

    // Resolve plan or use consumption
    if let Some(p) = plan {
        let plan_id = if p.starts_with('/') {
            p.to_string()
        } else {
            format!(
                "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/serverfarms/{p}"
            )
        };
        let resolved_plan_id = plan_id.replace("{subscriptionId}", &sub_id);
        properties["serverFarmId"] = serde_json::Value::String(resolved_plan_id);
    }

    if consumption {
        // For consumption plan, set reserved=true for Linux
        if os.eq_ignore_ascii_case("linux") {
            properties["reserved"] = serde_json::Value::Bool(true);
        }
    }

    let mut body = serde_json::json!({
        "kind": "functionapp",
        "location": "",
        "properties": properties
    });

    if os.eq_ignore_ascii_case("linux") {
        body["kind"] = serde_json::Value::String("functionapp,linux".to_string());
    }

    // Set storage account connection string in app settings if provided
    if let Some(sa) = storage_account {
        body["properties"]["siteConfig"]["appSettings"] = serde_json::json!([
            {"name": "AzureWebJobsStorage", "value": sa},
            {"name": "FUNCTIONS_EXTENSION_VERSION", "value": "~4"},
            {"name": "FUNCTIONS_WORKER_RUNTIME", "value": runtime.unwrap_or("node")}
        ]);
    }

    if let Some(tag_list) = tags {
        body["tags"] = serde_json::to_value(crate::commands::group::parse_tags(tag_list))?;
    }

    // Resolve location
    if let Some(loc) = location {
        body["location"] = serde_json::Value::String(loc.to_string());
    } else if let Some(p) = plan {
        // Try to get location from plan
        let plan_id = if p.starts_with('/') {
            p.to_string()
        } else {
            format!(
                "/subscriptions/{sub_id}/resourceGroups/{resource_group}/providers/Microsoft.Web/serverfarms/{p}"
            )
        };
        let plan_path = format!("{plan_id}?api-version={API_VERSION}");
        if let Ok(plan_data) = cmd.get(&plan_path).await {
            if let Some(loc) = plan_data.get("location").and_then(|v| v.as_str()) {
                body["location"] = serde_json::Value::String(loc.to_string());
            }
        }
    }

    // Fallback — get location from resource group
    if body["location"].as_str() == Some("") {
        let rg_path = format!("/subscriptions/{{subscriptionId}}/resourcegroups/{resource_group}?api-version=2024-03-01");
        if let Ok(rg_data) = cmd.get(&rg_path).await {
            if let Some(loc) = rg_data.get("location").and_then(|v| v.as_str()) {
                body["location"] = serde_json::Value::String(loc.to_string());
            }
        }
    }

    let result = cmd.put_lro(&site_path(resource_group, name), &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp delete`
pub async fn delete(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    cmd.delete(&site_path(resource_group, name)).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `functionapp stop`
pub async fn stop(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/stop?api-version={API_VERSION}"
    );
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Function app '{name}' stopped.");
    Ok(())
}

/// `functionapp start`
pub async fn start(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/start?api-version={API_VERSION}"
    );
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Function app '{name}' started.");
    Ok(())
}

/// `functionapp restart`
pub async fn restart(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/restart?api-version={API_VERSION}"
    );
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Function app '{name}' restarted.");
    Ok(())
}

/// `functionapp update`
pub async fn update(
    resource_group: &str,
    name: &str,
    set_values: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = site_path(resource_group, name);
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

/// `functionapp list-runtimes`
/// Returns known Azure Functions runtimes as a static JSON list.
pub async fn list_runtimes() -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "linux": [
            {"runtime": "dotnet", "versions": ["6.0", "7.0", "8.0"]},
            {"runtime": "dotnet-isolated", "versions": ["6.0", "7.0", "8.0"]},
            {"runtime": "node", "versions": ["16", "18", "20"]},
            {"runtime": "python", "versions": ["3.9", "3.10", "3.11"]},
            {"runtime": "java", "versions": ["8", "11", "17", "21"]},
            {"runtime": "powershell", "versions": ["7.2", "7.4"]},
            {"runtime": "custom", "versions": []}
        ],
        "windows": [
            {"runtime": "dotnet", "versions": ["6.0", "8.0"]},
            {"runtime": "dotnet-isolated", "versions": ["6.0", "7.0", "8.0"]},
            {"runtime": "node", "versions": ["16", "18", "20"]},
            {"runtime": "java", "versions": ["8", "11", "17", "21"]},
            {"runtime": "powershell", "versions": ["7.2", "7.4"]},
            {"runtime": "custom", "versions": []}
        ]
    }))
}

/// `functionapp deploy`
pub async fn deploy(
    resource_group: &str,
    name: &str,
    src_path: &str,
    deploy_type: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let content = std::fs::read(src_path)
        .map_err(|e| AzrsError::General(format!("Cannot read deployment file: {e}")))?;

    let dtype = deploy_type.unwrap_or("zip");

    if dtype == "zip" {
        let deploy_path = format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/extensions/zipdeploy?api-version={API_VERSION}"
        );

        let url = cmd.arm_url(&deploy_path)?;
        let token = {
            let username = cmd.username()?.to_string();
            let tenant = cmd.tenant_id()?.to_string();
            let scopes = vec![cmd.cloud.default_scope()];
            cmd.cache.get_access_token(&username, &tenant, &scopes, &cmd.cloud).await?
        };

        let client = reqwest::Client::new();
        let resp = client
            .put(&url)
            .header("Authorization", format!("Bearer {}", token.access_token))
            .header("Content-Type", "application/octet-stream")
            .body(content)
            .send()
            .await
            .map_err(|e| AzrsError::General(format!("Deployment request failed: {e}")))?;

        cmd.save_cache()?;
        let status = resp.status().as_u16();
        if status == 200 || status == 202 {
            eprintln!("Deployment initiated for function app '{name}'.");
            return Ok(serde_json::json!({"status": "success", "type": dtype}));
        } else {
            let body = resp.text().await.unwrap_or_default();
            return Err(AzrsError::General(format!("Deployment failed: HTTP {status}: {body}")));
        }
    }

    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/extensions/onedeploy?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "properties": {
            "packageUri": "",
            "type": dtype
        }
    });
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

// --- Config ---

/// `functionapp config appsettings list`
pub async fn config_appsettings_list(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings/list?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp config appsettings set`
pub async fn config_appsettings_set(
    resource_group: &str,
    name: &str,
    settings: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let list_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings/list?api-version={API_VERSION}"
    );
    let current = cmd.post(&list_path, None).await?;
    let mut props = current.get("properties").cloned().unwrap_or(serde_json::json!({}));

    for setting in settings {
        if let Some((k, v)) = setting.split_once('=') {
            props[k] = serde_json::Value::String(v.to_string());
        }
    }

    let put_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings?api-version={API_VERSION}"
    );
    let body = serde_json::json!({"properties": props});
    let result = cmd.put(&put_path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp config appsettings delete`
pub async fn config_appsettings_delete(
    resource_group: &str,
    name: &str,
    keys: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let list_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings/list?api-version={API_VERSION}"
    );
    let current = cmd.post(&list_path, None).await?;
    let mut props: serde_json::Map<String, serde_json::Value> =
        serde_json::from_value(current.get("properties").cloned().unwrap_or(serde_json::json!({})))
            .unwrap_or_default();

    for key in keys {
        props.remove(key.as_str());
    }

    let put_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings?api-version={API_VERSION}"
    );
    let body = serde_json::json!({"properties": props});
    let result = cmd.put(&put_path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

// --- Keys ---

/// `functionapp keys list`
pub async fn keys_list(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/host/default/listkeys?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp keys set`
pub async fn keys_set(
    resource_group: &str,
    name: &str,
    key_name: &str,
    key_value: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/host/default/functionkeys/{key_name}?api-version={API_VERSION}"
    );
    let body = if let Some(v) = key_value {
        serde_json::json!({"properties": {"name": key_name, "value": v}})
    } else {
        serde_json::json!({"properties": {"name": key_name}})
    };
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp keys delete`
pub async fn keys_delete(
    resource_group: &str,
    name: &str,
    key_name: &str,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/host/default/functionkeys/{key_name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

// --- Function ---

/// `functionapp function list`
pub async fn function_list(resource_group: &str, name: &str) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/functions?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `functionapp function show`
pub async fn function_show(
    resource_group: &str,
    name: &str,
    function_name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/functions/{function_name}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp function delete`
pub async fn function_delete(
    resource_group: &str,
    name: &str,
    function_name: &str,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/functions/{function_name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `functionapp function keys list`
pub async fn function_keys_list(
    resource_group: &str,
    name: &str,
    function_name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/functions/{function_name}/listkeys?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp function keys set`
pub async fn function_keys_set(
    resource_group: &str,
    name: &str,
    function_name: &str,
    key_name: &str,
    key_value: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/functions/{function_name}/keys/{key_name}?api-version={API_VERSION}"
    );
    let body = if let Some(v) = key_value {
        serde_json::json!({"properties": {"name": key_name, "value": v}})
    } else {
        serde_json::json!({"properties": {"name": key_name}})
    };
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp function keys delete`
pub async fn function_keys_delete(
    resource_group: &str,
    name: &str,
    function_name: &str,
    key_name: &str,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/functions/{function_name}/keys/{key_name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

// --- Deployment ---

/// `functionapp deployment list-publishing-profiles`
pub async fn deployment_list_publishing_profiles(
    resource_group: &str,
    name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/publishxml?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp deployment source config-zip`
pub async fn deployment_source_config_zip(
    resource_group: &str,
    name: &str,
    src: &str,
) -> Result<serde_json::Value> {
    deploy(resource_group, name, src, Some("zip")).await
}
