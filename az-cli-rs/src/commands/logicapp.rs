/// Logic App (Standard) commands — `azrs logicapp create/list/show/delete/...`.
///
/// ARM API: Microsoft.Web/sites (api-version 2024-11-01)
/// Logic Apps Standard are Web/sites with kind="functionapp,workflowapp".
use super::ArmCommand;
use crate::error::{AzrsError, Result};

const API_VERSION: &str = "2024-11-01";

fn site_path(resource_group: &str, name: &str) -> String {
    format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}?api-version={API_VERSION}"
    )
}

/// `logicapp list [--resource-group <rg>]`
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
    let filtered = results
        .into_iter()
        .filter(|site| {
            site.get("kind")
                .and_then(|k| k.as_str())
                .map(|k| k.to_lowercase().contains("workflowapp"))
                .unwrap_or(false)
        })
        .collect();
    Ok(filtered)
}

/// `logicapp show`
pub async fn show(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let result = cmd.get(&site_path(resource_group, name)).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `logicapp create`
pub async fn create(
    resource_group: &str,
    name: &str,
    plan: &str,
    location: &str,
    storage_account: Option<&str>,
    tags: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let sub_id = cmd.subscription_id()?.to_string();

    let plan_id = if plan.starts_with('/') {
        plan.to_string()
    } else {
        format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/serverfarms/{plan}"
        )
    };
    let resolved_plan_id = plan_id.replace("{subscriptionId}", &sub_id);

    let mut body = serde_json::json!({
        "kind": "functionapp,workflowapp",
        "location": location,
        "properties": {
            "serverFarmId": resolved_plan_id,
            "siteConfig": {
                "appSettings": [
                    {"name": "FUNCTIONS_EXTENSION_VERSION", "value": "~4"},
                    {"name": "FUNCTIONS_WORKER_RUNTIME", "value": "node"},
                    {"name": "WEBSITE_NODE_DEFAULT_VERSION", "value": "~18"},
                    {"name": "AzureWebJobsFeatureFlags", "value": "EnableMultiLanguageWorker"}
                ]
            }
        }
    });

    if let Some(sa) = storage_account {
        if let Some(settings) = body["properties"]["siteConfig"]["appSettings"].as_array_mut() {
            settings.push(serde_json::json!({"name": "AzureWebJobsStorage", "value": format!("DefaultEndpointsProtocol=https;AccountName={sa}")}));
        }
    }

    if let Some(tag_list) = tags {
        body["tags"] = serde_json::to_value(crate::commands::group::parse_tags(tag_list))?;
    }

    let result = cmd.put_lro(&site_path(resource_group, name), &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `logicapp delete`
pub async fn delete(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    cmd.delete(&site_path(resource_group, name)).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `logicapp stop`
pub async fn stop(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/stop?api-version={API_VERSION}"
    );
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Logic app '{name}' stopped.");
    Ok(())
}

/// `logicapp start`
pub async fn start(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/start?api-version={API_VERSION}"
    );
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Logic app '{name}' started.");
    Ok(())
}

/// `logicapp restart`
pub async fn restart(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/restart?api-version={API_VERSION}"
    );
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Logic app '{name}' restarted.");
    Ok(())
}

/// `logicapp update`
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

// ── Config ────────────────────────────────────────────────────────

/// `logicapp config appsettings list`
pub async fn config_appsettings_list(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings/list?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `logicapp config appsettings set`
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

/// `logicapp config appsettings delete`
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

// ── Deployment ────────────────────────────────────────────────────

/// `logicapp deployment source config-zip`
pub async fn deployment_source_config_zip(
    resource_group: &str,
    name: &str,
    src: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let content = std::fs::read(src)
        .map_err(|e| AzrsError::General(format!("Cannot read deployment file: {e}")))?;

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
        eprintln!("Deployment initiated for logic app '{name}'.");
        Ok(serde_json::json!({"status": "success", "type": "zip"}))
    } else {
        let body = resp.text().await.unwrap_or_default();
        Err(AzrsError::General(format!("Deployment failed: HTTP {status}: {body}")))
    }
}
