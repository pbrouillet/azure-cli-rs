/// Webapp commands — `azrs webapp create/list/show/delete/stop/start/restart/...`.
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

/// `webapp list [--resource-group <rg>]`
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
    Ok(results)
}

/// `webapp show`
pub async fn show(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let result = cmd.get(&site_path(resource_group, name)).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp create`
pub async fn create(
    resource_group: &str,
    name: &str,
    plan: &str,
    runtime: Option<&str>,
    startup_file: Option<&str>,
    deployment_container_image: Option<&str>,
    tags: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    // Resolve plan — could be just a name or a full resource ID
    let plan_id = if plan.starts_with('/') {
        plan.to_string()
    } else {
        format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/serverfarms/{plan}"
        )
        // subscriptionId will be replaced by arm_url()
    };
    // We need the actual subscription ID for the plan_id
    let sub_id = cmd.subscription_id()?.to_string();
    let resolved_plan_id = plan_id.replace("{subscriptionId}", &sub_id);

    let mut site_config = serde_json::json!({});
    if let Some(rt) = runtime {
        // Map runtime string to site config properties
        let (linux_fx, _windows_fx) = parse_runtime(rt);
        if let Some(fx) = linux_fx {
            site_config["linuxFxVersion"] = serde_json::Value::String(fx);
        }
    }
    if let Some(sf) = startup_file {
        site_config["appCommandLine"] = serde_json::Value::String(sf.to_string());
    }

    let mut body = serde_json::json!({
        "location": "",
        "properties": {
            "serverFarmId": resolved_plan_id,
            "siteConfig": site_config
        }
    });

    if let Some(img) = deployment_container_image {
        body["properties"]["siteConfig"]["linuxFxVersion"] =
            serde_json::Value::String(format!("DOCKER|{img}"));
    }

    if let Some(tag_list) = tags {
        body["tags"] = serde_json::to_value(crate::commands::group::parse_tags(tag_list))?;
    }

    // Get plan location to set the site location
    let plan_path = format!("{resolved_plan_id}?api-version={API_VERSION}");
    let plan_info = cmd.get(&plan_path.replace(&format!("https://management.azure.com"), "")).await;
    if let Ok(plan_data) = &plan_info {
        if let Some(loc) = plan_data.get("location").and_then(|v| v.as_str()) {
            body["location"] = serde_json::Value::String(loc.to_string());
        }
    }
    // Fallback — if we couldn't get the plan, location must be provided
    if body["location"].as_str() == Some("") {
        // Try to get from the resource group
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

/// `webapp delete`
pub async fn delete(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    cmd.delete(&site_path(resource_group, name)).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `webapp stop`
pub async fn stop(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/stop?api-version={API_VERSION}"
    );
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Web app '{name}' stopped.");
    Ok(())
}

/// `webapp start`
pub async fn start(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/start?api-version={API_VERSION}"
    );
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Web app '{name}' started.");
    Ok(())
}

/// `webapp restart`
pub async fn restart(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/restart?api-version={API_VERSION}"
    );
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Web app '{name}' restarted.");
    Ok(())
}

/// `webapp update`
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

/// `webapp list-runtimes [--os <linux|windows>]`
pub async fn list_runtimes(os_type: Option<&str>) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/providers/Microsoft.Web/availableStacks?api-version={API_VERSION}&osTypeSelected={}",
        os_type.unwrap_or("Linux")
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp deploy --resource-group <rg> --name <name> --src-path <path> --type <zip|war|jar|...>`
pub async fn deploy(
    resource_group: &str,
    name: &str,
    src_path: &str,
    deploy_type: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    // Read the file
    let content = std::fs::read(src_path)
        .map_err(|e| AzrsError::General(format!("Cannot read deployment file: {e}")))?;

    let dtype = deploy_type.unwrap_or("zip");

    // Use the Kudu ZIP deploy endpoint via ARM
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/extensions/onedeploy?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "properties": {
            "packageUri": "",
            "type": dtype
        }
    });

    // For zip deploy, use the zipdeploy API directly
    if dtype == "zip" {
        let deploy_path = format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/extensions/zipdeploy?api-version={API_VERSION}"
        );

        // Use ArmCommand's request with a custom body
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
            eprintln!("Deployment initiated for '{name}'.");
            return Ok(serde_json::json!({"status": "success", "type": dtype}));
        } else {
            let body = resp.text().await.unwrap_or_default();
            return Err(AzrsError::General(format!("Deployment failed: HTTP {status}: {body}")));
        }
    }

    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

// --- Webapp Config ---

/// `webapp config appsettings list`
pub async fn config_appsettings_list(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings/list?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp config appsettings set`
pub async fn config_appsettings_set(
    resource_group: &str,
    name: &str,
    settings: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    // Get current settings first
    let list_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings/list?api-version={API_VERSION}"
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
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings?api-version={API_VERSION}"
    );
    let body = serde_json::json!({"properties": props});
    let result = cmd.put(&put_path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp config appsettings delete`
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

/// `webapp config connection-string list`
pub async fn config_connstr_list(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/connectionstrings/list?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp config connection-string set`
pub async fn config_connstr_set(
    resource_group: &str,
    name: &str,
    settings: &[String],
    conn_type: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let list_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/connectionstrings/list?api-version={API_VERSION}"
    );
    let current = cmd.post(&list_path, None).await?;
    let mut props = current.get("properties").cloned().unwrap_or(serde_json::json!({}));

    for setting in settings {
        if let Some((k, v)) = setting.split_once('=') {
            props[k] = serde_json::json!({"value": v, "type": conn_type});
        }
    }

    let put_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/connectionstrings?api-version={API_VERSION}"
    );
    let body = serde_json::json!({"properties": props});
    let result = cmd.put(&put_path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp config connection-string delete`
pub async fn config_connstr_delete(
    resource_group: &str,
    name: &str,
    keys: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let list_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/connectionstrings/list?api-version={API_VERSION}"
    );
    let current = cmd.post(&list_path, None).await?;
    let mut props: serde_json::Map<String, serde_json::Value> =
        serde_json::from_value(current.get("properties").cloned().unwrap_or(serde_json::json!({})))
            .unwrap_or_default();

    for key in keys {
        props.remove(key.as_str());
    }

    let put_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/connectionstrings?api-version={API_VERSION}"
    );
    let body = serde_json::json!({"properties": props});
    let result = cmd.put(&put_path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp config set`
pub async fn config_set(
    resource_group: &str,
    name: &str,
    set_values: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/web?api-version={API_VERSION}"
    );
    let mut current = cmd.get(&path).await?;

    for pair in set_values {
        if let Some((key, value)) = pair.split_once('=') {
            // Set in properties
            if let Some(props) = current.get_mut("properties") {
                let parsed = serde_json::from_str::<serde_json::Value>(value)
                    .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));
                props[key] = parsed;
            }
        }
    }

    let result = cmd.put(&path, &current).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp config hostname list`
pub async fn config_hostname_list(resource_group: &str, name: &str) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/hostNameBindings?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `webapp config hostname add`
pub async fn config_hostname_add(resource_group: &str, name: &str, hostname: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/hostNameBindings/{hostname}?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "properties": {
            "siteName": name
        }
    });
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp config hostname delete`
pub async fn config_hostname_delete(resource_group: &str, name: &str, hostname: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/hostNameBindings/{hostname}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

// --- Deployment Source ---

/// `webapp deployment source config-zip`
pub async fn deployment_source_config_zip(
    resource_group: &str,
    name: &str,
    src: &str,
) -> Result<serde_json::Value> {
    deploy(resource_group, name, src, Some("zip")).await
}

/// `webapp deployment list-publishing-profiles`
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

/// `webapp deployment source show`
pub async fn deployment_source_show(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/sourcecontrols/web?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp deployment source delete`
pub async fn deployment_source_delete(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/sourcecontrols/web?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `webapp deployment source sync`
pub async fn deployment_source_sync(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/sync?api-version={API_VERSION}"
    );
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Deployment source synced for '{name}'.");
    Ok(())
}

// --- Identity ---

/// `webapp identity assign`
pub async fn identity_assign(
    resource_group: &str,
    name: &str,
    identity_type: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = site_path(resource_group, name);
    let mut current = cmd.get(&path).await?;

    let id_type = identity_type.unwrap_or("SystemAssigned");
    current["identity"] = serde_json::json!({"type": id_type});

    let result = cmd.put(&path, &current).await?;
    cmd.save_cache()?;
    // Return just the identity section
    Ok(result.get("identity").cloned().unwrap_or(serde_json::Value::Null))
}

/// `webapp identity remove`
pub async fn identity_remove(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = site_path(resource_group, name);
    let mut current = cmd.get(&path).await?;

    current["identity"] = serde_json::json!({"type": "None"});

    let result = cmd.put(&path, &current).await?;
    cmd.save_cache()?;
    Ok(result.get("identity").cloned().unwrap_or(serde_json::Value::Null))
}

// --- Helpers ---

/// Parse a runtime string like "node:18-lts" or "PYTHON:3.11" into (linuxFxVersion, windowsFxVersion)
fn parse_runtime(runtime: &str) -> (Option<String>, Option<String>) {
    // Format: RUNTIME|VERSION or RUNTIME:VERSION
    let normalized = runtime.replace(':', "|").to_uppercase();
    (Some(normalized), None)
}

// ── Config SSL ─────────────────────────────────────────────────────

/// `webapp config ssl list`
pub async fn config_ssl_list(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let result = cmd.get(&site_path(resource_group, name)).await?;
    cmd.save_cache()?;
    Ok(result.get("properties")
        .and_then(|p| p.get("hostNameSslStates"))
        .cloned()
        .unwrap_or(serde_json::Value::Array(vec![])))
}

/// `webapp config ssl bind`
pub async fn config_ssl_bind(
    resource_group: &str,
    name: &str,
    ssl_type: &str,
    certificate_thumbprint: &str,
    hostname: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = site_path(resource_group, name);
    let mut site = cmd.get(&path).await?;

    if let Some(props) = site.get_mut("properties") {
        let states = props.get_mut("hostNameSslStates")
            .and_then(|s| s.as_array_mut());
        if let Some(arr) = states {
            let mut found = false;
            for state in arr.iter_mut() {
                if state.get("name").and_then(|n| n.as_str()) == Some(hostname) {
                    state["sslState"] = serde_json::Value::String(ssl_type.to_string());
                    state["thumbprint"] = serde_json::Value::String(certificate_thumbprint.to_string());
                    found = true;
                    break;
                }
            }
            if !found {
                arr.push(serde_json::json!({
                    "name": hostname,
                    "sslState": ssl_type,
                    "thumbprint": certificate_thumbprint
                }));
            }
        }
    }

    let result = cmd.put(&path, &site).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp config ssl unbind`
pub async fn config_ssl_unbind(
    resource_group: &str,
    name: &str,
    hostname: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = site_path(resource_group, name);
    let mut site = cmd.get(&path).await?;

    if let Some(props) = site.get_mut("properties") {
        let states = props.get_mut("hostNameSslStates")
            .and_then(|s| s.as_array_mut());
        if let Some(arr) = states {
            for state in arr.iter_mut() {
                if state.get("name").and_then(|n| n.as_str()) == Some(hostname) {
                    state["sslState"] = serde_json::Value::String("Disabled".to_string());
                    state["thumbprint"] = serde_json::Value::Null;
                    break;
                }
            }
        }
    }

    let result = cmd.put(&path, &site).await?;
    cmd.save_cache()?;
    Ok(result)
}

// ── Config Access Restriction ──────────────────────────────────────

/// `webapp config access-restriction add`
pub async fn config_access_restriction_add(
    resource_group: &str,
    name: &str,
    rule_name: &str,
    priority: u32,
    action: &str,
    ip_address: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/web?api-version={API_VERSION}"
    );
    let mut config = cmd.get(&path).await?;

    let new_rule = serde_json::json!({
        "name": rule_name,
        "priority": priority,
        "action": action,
        "ipAddress": ip_address.unwrap_or("0.0.0.0/0"),
    });

    if let Some(props) = config.get_mut("properties") {
        let restrictions = props.get_mut("ipSecurityRestrictions")
            .and_then(|r| r.as_array_mut());
        if let Some(arr) = restrictions {
            arr.push(new_rule);
        } else {
            props["ipSecurityRestrictions"] = serde_json::json!([new_rule]);
        }
    }

    let result = cmd.put(&path, &config).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp config access-restriction remove`
pub async fn config_access_restriction_remove(
    resource_group: &str,
    name: &str,
    rule_name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/web?api-version={API_VERSION}"
    );
    let mut config = cmd.get(&path).await?;

    if let Some(props) = config.get_mut("properties") {
        if let Some(restrictions) = props.get_mut("ipSecurityRestrictions").and_then(|r| r.as_array_mut()) {
            restrictions.retain(|r| r.get("name").and_then(|n| n.as_str()) != Some(rule_name));
        }
    }

    let result = cmd.put(&path, &config).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp config access-restriction set`
pub async fn config_access_restriction_set(
    resource_group: &str,
    name: &str,
    use_same_restrictions_for_scm_site: bool,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/web?api-version={API_VERSION}"
    );
    let mut config = cmd.get(&path).await?;

    if let Some(props) = config.get_mut("properties") {
        props["scmIpSecurityRestrictionsUseMain"] = serde_json::Value::Bool(use_same_restrictions_for_scm_site);
    }

    let result = cmd.put(&path, &config).await?;
    cmd.save_cache()?;
    Ok(result)
}

// ── Config Container ───────────────────────────────────────────────

/// `webapp config container set`
pub async fn config_container_set(
    resource_group: &str,
    name: &str,
    image: &str,
    registry_url: Option<&str>,
    registry_username: Option<&str>,
    registry_password: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let config_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/web?api-version={API_VERSION}"
    );
    let mut config = cmd.get(&config_path).await?;
    if let Some(props) = config.get_mut("properties") {
        props["linuxFxVersion"] = serde_json::Value::String(format!("DOCKER|{image}"));
    }
    cmd.put(&config_path, &config).await?;

    let list_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings/list?api-version={API_VERSION}"
    );
    let current = cmd.post(&list_path, None).await?;
    let mut props = current.get("properties").cloned().unwrap_or(serde_json::json!({}));

    props["DOCKER_CUSTOM_IMAGE_NAME"] = serde_json::Value::String(image.to_string());
    if let Some(url) = registry_url {
        props["DOCKER_REGISTRY_SERVER_URL"] = serde_json::Value::String(url.to_string());
    }
    if let Some(user) = registry_username {
        props["DOCKER_REGISTRY_SERVER_USERNAME"] = serde_json::Value::String(user.to_string());
    }
    if let Some(pass) = registry_password {
        props["DOCKER_REGISTRY_SERVER_PASSWORD"] = serde_json::Value::String(pass.to_string());
    }

    let put_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings?api-version={API_VERSION}"
    );
    let body = serde_json::json!({"properties": props});
    let result = cmd.put(&put_path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp config container delete`
pub async fn config_container_delete(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let config_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/web?api-version={API_VERSION}"
    );
    let mut config = cmd.get(&config_path).await?;
    if let Some(props) = config.get_mut("properties") {
        props["linuxFxVersion"] = serde_json::Value::String(String::new());
    }
    cmd.put(&config_path, &config).await?;

    let list_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings/list?api-version={API_VERSION}"
    );
    let current = cmd.post(&list_path, None).await?;
    let mut props: serde_json::Map<String, serde_json::Value> =
        serde_json::from_value(current.get("properties").cloned().unwrap_or(serde_json::json!({})))
            .unwrap_or_default();

    for key in &["DOCKER_CUSTOM_IMAGE_NAME", "DOCKER_REGISTRY_SERVER_URL", "DOCKER_REGISTRY_SERVER_USERNAME", "DOCKER_REGISTRY_SERVER_PASSWORD"] {
        props.remove(*key);
    }

    let put_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings?api-version={API_VERSION}"
    );
    let body = serde_json::json!({"properties": props});
    let result = cmd.put(&put_path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

// ── Config Backup ──────────────────────────────────────────────────

/// `webapp config backup list`
pub async fn config_backup_list(resource_group: &str, name: &str) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/backups?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `webapp config backup create`
pub async fn config_backup_create(
    resource_group: &str,
    name: &str,
    backup_name: &str,
    storage_account_url: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/backup?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "properties": {
            "backupName": backup_name,
            "storageAccountUrl": storage_account_url,
            "enabled": true
        }
    });
    let result = cmd.post(&path, Some(&body)).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp config backup delete`
pub async fn config_backup_delete(resource_group: &str, name: &str, backup_id: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/backups/{backup_id}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `webapp config backup restore`
pub async fn config_backup_restore(
    resource_group: &str,
    name: &str,
    backup_id: &str,
    storage_account_url: &str,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/backups/{backup_id}/restore?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "properties": {
            "storageAccountUrl": storage_account_url,
            "overwrite": true
        }
    });
    cmd.post_lro(&path, Some(&body)).await?;
    cmd.save_cache()?;
    Ok(())
}

// ── CORS ───────────────────────────────────────────────────────────

/// `webapp cors add`
pub async fn cors_add(
    resource_group: &str,
    name: &str,
    allowed_origins: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/web?api-version={API_VERSION}"
    );
    let mut config = cmd.get(&path).await?;

    if let Some(props) = config.get_mut("properties") {
        let mut existing: Vec<String> = props.get("cors")
            .and_then(|c| c.get("allowedOrigins"))
            .and_then(|a| serde_json::from_value(a.clone()).ok())
            .unwrap_or_default();

        for origin in allowed_origins {
            if !existing.contains(origin) {
                existing.push(origin.clone());
            }
        }

        props["cors"] = serde_json::json!({"allowedOrigins": existing});
    }

    let result = cmd.put(&path, &config).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp cors remove`
pub async fn cors_remove(
    resource_group: &str,
    name: &str,
    allowed_origins: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/web?api-version={API_VERSION}"
    );
    let mut config = cmd.get(&path).await?;

    if let Some(props) = config.get_mut("properties") {
        let mut existing: Vec<String> = props.get("cors")
            .and_then(|c| c.get("allowedOrigins"))
            .and_then(|a| serde_json::from_value(a.clone()).ok())
            .unwrap_or_default();

        existing.retain(|o| !allowed_origins.contains(o));
        props["cors"] = serde_json::json!({"allowedOrigins": existing});
    }

    let result = cmd.put(&path, &config).await?;
    cmd.save_cache()?;
    Ok(result)
}

// ── Deployment Slot ────────────────────────────────────────────────

/// `webapp deployment slot list`
pub async fn deployment_slot_list(resource_group: &str, name: &str) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/slots?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `webapp deployment slot create`
pub async fn deployment_slot_create(
    resource_group: &str,
    name: &str,
    slot: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let site = cmd.get(&site_path(resource_group, name)).await?;
    let location = site.get("location").and_then(|l| l.as_str()).unwrap_or("eastus");

    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/slots/{slot}?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "location": location,
        "properties": {}
    });
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp deployment slot delete`
pub async fn deployment_slot_delete(resource_group: &str, name: &str, slot: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/slots/{slot}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `webapp deployment slot swap`
pub async fn deployment_slot_swap(
    resource_group: &str,
    name: &str,
    slot: &str,
    target_slot: &str,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/slots/{slot}/slotsswap?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "targetSlot": target_slot
    });
    cmd.post_lro(&path, Some(&body)).await?;
    cmd.save_cache()?;
    eprintln!("Slot '{slot}' swapped with '{target_slot}'.");
    Ok(())
}

// ── Deployment GitHub Actions ──────────────────────────────────────

/// `webapp deployment github-actions add`
pub async fn deployment_github_actions_add(
    resource_group: &str,
    name: &str,
    repo: &str,
    branch: &str,
    _token: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/sourcecontrols/web?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "properties": {
            "repoUrl": format!("https://github.com/{repo}"),
            "branch": branch,
            "isManualIntegration": false,
            "isGitHubAction": true,
            "gitHubActionConfiguration": {
                "generateWorkflowFile": true
            }
        }
    });
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp deployment github-actions remove`
pub async fn deployment_github_actions_remove(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/sourcecontrols/web?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

// ── Deployment Container ───────────────────────────────────────────

/// `webapp deployment container config`
pub async fn deployment_container_config(
    resource_group: &str,
    name: &str,
    enable_cd: bool,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let list_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings/list?api-version={API_VERSION}"
    );
    let current = cmd.post(&list_path, None).await?;
    let mut props = current.get("properties").cloned().unwrap_or(serde_json::json!({}));

    props["DOCKER_ENABLE_CI"] = serde_json::Value::String(if enable_cd { "true" } else { "false" }.to_string());

    let put_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/appsettings?api-version={API_VERSION}"
    );
    let body = serde_json::json!({"properties": props});
    let result = cmd.put(&put_path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp deployment container show-cd-url`
pub async fn deployment_container_show_cd_url(
    resource_group: &str,
    name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/publishingcredentials/list?api-version={API_VERSION}"
    );
    let creds = cmd.post(&path, None).await?;

    let scm_uri = creds.get("properties")
        .and_then(|p| p.get("scmUri"))
        .and_then(|u| u.as_str())
        .unwrap_or("");

    let cd_url = if scm_uri.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(format!("{scm_uri}/api/registry/webhook"))
    };

    cmd.save_cache()?;
    Ok(serde_json::json!({"CI_CD_URL": cd_url}))
}

// ── Deployment User ────────────────────────────────────────────────

/// `webapp deployment user set`
pub async fn deployment_user_set(username: &str, password: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/providers/Microsoft.Web/publishingUsers/web?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "properties": {
            "publishingUserName": username,
            "publishingPassword": password
        }
    });
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

// ── VNet Integration ──────────────────────────────────────────────

/// `webapp vnet-integration list`
pub async fn vnet_integration_list(resource_group: &str, name: &str) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/virtualNetworkConnections?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `webapp vnet-integration add`
pub async fn vnet_integration_add(
    resource_group: &str,
    name: &str,
    vnet: &str,
    subnet: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let sub_id = cmd.subscription_id()?.to_string();
    let vnet_subnet_id = format!(
        "/subscriptions/{sub_id}/resourceGroups/{resource_group}/providers/Microsoft.Network/virtualNetworks/{vnet}/subnets/{subnet}"
    );
    let vnet_name = vnet.to_string();
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/virtualNetworkConnections/{vnet_name}?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "properties": {
            "vnetResourceId": vnet_subnet_id,
            "isSwift": true
        }
    });
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp vnet-integration remove`
pub async fn vnet_integration_remove(resource_group: &str, name: &str, vnet: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/virtualNetworkConnections/{vnet}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

// ── Log ───────────────────────────────────────────────────────────

/// `webapp log config` — read-modify-write the logging configuration
pub async fn log_config(
    resource_group: &str,
    name: &str,
    application_logging: Option<&str>,
    web_server_logging: Option<&str>,
    level: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/logs?api-version={API_VERSION}"
    );
    let mut config = cmd.get(&path).await?;

    if let Some(props) = config.get_mut("properties") {
        if let Some(app_log) = application_logging {
            props["applicationLogs"] = serde_json::json!({
                "fileSystem": { "level": level.unwrap_or("Information") }
            });
            if app_log == "azureblobstorage" {
                props["applicationLogs"]["azureBlobStorage"] = serde_json::json!({
                    "level": level.unwrap_or("Information"),
                    "retentionInDays": 30
                });
            }
        }
        if let Some(web_log) = web_server_logging {
            if web_log == "filesystem" {
                props["httpLogs"] = serde_json::json!({
                    "fileSystem": { "enabled": true, "retentionInMb": 35, "retentionInDays": 3 }
                });
            } else if web_log == "off" {
                props["httpLogs"] = serde_json::json!({
                    "fileSystem": { "enabled": false }
                });
            }
        }
    }

    let result = cmd.put(&path, &config).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp log download` — return log download info via the logstream URL
pub async fn log_download(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/publishingcredentials/list?api-version={API_VERSION}"
    );
    let creds = cmd.post(&path, None).await?;
    let scm_uri = creds.get("properties")
        .and_then(|p| p.get("scmUri"))
        .and_then(|u| u.as_str())
        .unwrap_or("");
    cmd.save_cache()?;
    Ok(serde_json::json!({
        "logStreamUrl": if scm_uri.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(format!("{scm_uri}/api/logstream")) }
    }))
}

/// `webapp log tail` — return the log streaming URL
pub async fn log_tail(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    log_download(resource_group, name).await
}

// ── Deleted ───────────────────────────────────────────────────────

/// `webapp deleted list`
pub async fn deleted_list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/providers/Microsoft.Web/deletedSites?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    if let Some(rg) = resource_group {
        let rg_lower = rg.to_lowercase();
        let filtered = results
            .into_iter()
            .filter(|site| {
                site.get("properties")
                    .and_then(|p| p.get("resourceGroup"))
                    .and_then(|r| r.as_str())
                    .map(|r| r.to_lowercase() == rg_lower)
                    .unwrap_or(false)
            })
            .collect();
        return Ok(filtered);
    }
    Ok(results)
}

/// `webapp deleted restore`
pub async fn deleted_restore(
    resource_group: &str,
    name: &str,
    deleted_id: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/restoreFromDeletedApp?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "deletedSiteId": deleted_id,
        "recoverConfiguration": false
    });
    let result = cmd.post(&path, Some(&body)).await?;
    cmd.save_cache()?;
    Ok(result)
}

// ── Webjob Continuous ─────────────────────────────────────────────

/// `webapp webjob continuous list`
pub async fn webjob_continuous_list(resource_group: &str, name: &str) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/continuouswebjobs?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `webapp webjob continuous start`
pub async fn webjob_continuous_start(resource_group: &str, name: &str, webjob_name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/continuouswebjobs/{webjob_name}/start?api-version={API_VERSION}"
    );
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Continuous webjob '{webjob_name}' started.");
    Ok(())
}

/// `webapp webjob continuous stop`
pub async fn webjob_continuous_stop(resource_group: &str, name: &str, webjob_name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/continuouswebjobs/{webjob_name}/stop?api-version={API_VERSION}"
    );
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Continuous webjob '{webjob_name}' stopped.");
    Ok(())
}

/// `webapp webjob continuous remove`
pub async fn webjob_continuous_remove(resource_group: &str, name: &str, webjob_name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/continuouswebjobs/{webjob_name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

// ── Webjob Triggered ──────────────────────────────────────────────

/// `webapp webjob triggered list`
pub async fn webjob_triggered_list(resource_group: &str, name: &str) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/triggeredwebjobs?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `webapp webjob triggered run`
pub async fn webjob_triggered_run(resource_group: &str, name: &str, webjob_name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/triggeredwebjobs/{webjob_name}/run?api-version={API_VERSION}"
    );
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Triggered webjob '{webjob_name}' started.");
    Ok(())
}

/// `webapp webjob triggered remove`
pub async fn webjob_triggered_remove(resource_group: &str, name: &str, webjob_name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/triggeredwebjobs/{webjob_name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `webapp webjob triggered log`
pub async fn webjob_triggered_log(resource_group: &str, name: &str, webjob_name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/triggeredwebjobs/{webjob_name}/history?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

// ── Traffic Routing ───────────────────────────────────────────────

/// `webapp traffic-routing set`
pub async fn traffic_routing_set(
    resource_group: &str,
    name: &str,
    distribution: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let config_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/web?api-version={API_VERSION}"
    );
    let mut config = cmd.get(&config_path).await?;

    let mut rules = Vec::new();
    for entry in distribution {
        if let Some((slot, pct)) = entry.split_once('=') {
            let percentage: f64 = pct.parse().unwrap_or(0.0);
            rules.push(serde_json::json!({
                "actionHostName": format!("{name}-{slot}.azurewebsites.net"),
                "reroutePercentage": percentage,
                "name": slot
            }));
        }
    }

    if let Some(props) = config.get_mut("properties") {
        props["experiments"] = serde_json::json!({
            "rampUpRules": rules
        });
    }

    let result = cmd.put(&config_path, &config).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `webapp traffic-routing clear`
pub async fn traffic_routing_clear(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let config_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}/config/web?api-version={API_VERSION}"
    );
    let mut config = cmd.get(&config_path).await?;

    if let Some(props) = config.get_mut("properties") {
        props["experiments"] = serde_json::json!({
            "rampUpRules": []
        });
    }

    let result = cmd.put(&config_path, &config).await?;
    cmd.save_cache()?;
    Ok(result)
}
