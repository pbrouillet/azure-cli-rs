/// Deployment commands — ARM template deployments at multiple scopes.
///
/// Supports deployment at resource group, subscription, management group, and tenant scopes.
/// ARM API: Microsoft.Resources/deployments (api-version 2024-03-01)
use super::ArmCommand;
use crate::error::{AzrsError, Result};

const API_VERSION: &str = "2024-03-01";

/// Scope for deployment operations.
pub enum Scope<'a> {
    ResourceGroup(&'a str),
    Subscription,
    ManagementGroup(&'a str),
    Tenant,
}

impl<'a> Scope<'a> {
    fn base_path(&self) -> String {
        match self {
            Scope::ResourceGroup(rg) => {
                format!("/subscriptions/{{subscriptionId}}/resourcegroups/{rg}/providers/Microsoft.Resources/deployments")
            }
            Scope::Subscription => {
                "/subscriptions/{subscriptionId}/providers/Microsoft.Resources/deployments".to_string()
            }
            Scope::ManagementGroup(mg) => {
                format!("/providers/Microsoft.Management/managementGroups/{mg}/providers/Microsoft.Resources/deployments")
            }
            Scope::Tenant => {
                "/providers/Microsoft.Resources/deployments".to_string()
            }
        }
    }
}

/// `deployment list`
pub async fn list(scope: Scope<'_>, filter: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let mut path = format!("{}?api-version={API_VERSION}", scope.base_path());
    if let Some(f) = filter {
        path.push_str(&format!("&$filter={f}"));
    }
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `deployment show`
pub async fn show(scope: Scope<'_>, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}?api-version={API_VERSION}", scope.base_path(), name);
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `deployment delete`
pub async fn delete(scope: Scope<'_>, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}?api-version={API_VERSION}", scope.base_path(), name);
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `deployment cancel`
pub async fn cancel(scope: Scope<'_>, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}/cancel?api-version={API_VERSION}", scope.base_path(), name);
    cmd.post(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("Deployment '{name}' canceled.");
    Ok(())
}

/// `deployment export`
pub async fn export(scope: Scope<'_>, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}/exportTemplate?api-version={API_VERSION}", scope.base_path(), name);
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `deployment validate`
pub async fn validate(
    scope: Scope<'_>,
    name: &str,
    template_file: Option<&str>,
    template_uri: Option<&str>,
    parameters: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}/validate?api-version={API_VERSION}", scope.base_path(), name);

    let body = build_deployment_body(template_file, template_uri, parameters, None)?;
    let result = cmd.post(&path, Some(&body)).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `deployment create`
pub async fn create(
    scope: Scope<'_>,
    name: &str,
    template_file: Option<&str>,
    template_uri: Option<&str>,
    parameters: Option<&str>,
    mode: Option<&str>,
    no_wait: bool,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}?api-version={API_VERSION}", scope.base_path(), name);

    let body = build_deployment_body(template_file, template_uri, parameters, mode)?;

    let result = if no_wait {
        cmd.put(&path, &body).await?
    } else {
        cmd.put_lro(&path, &body).await?
    };
    cmd.save_cache()?;
    Ok(result)
}

/// `deployment what-if`
pub async fn what_if(
    scope: Scope<'_>,
    name: &str,
    template_file: Option<&str>,
    template_uri: Option<&str>,
    parameters: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}/whatIf?api-version={API_VERSION}", scope.base_path(), name);

    let body = build_deployment_body(template_file, template_uri, parameters, None)?;
    // what-if is a POST with LRO
    let resp = cmd.request("POST", &path, Some(&body)).await?;

    if !resp.is_success() && resp.status != 202 {
        return Err(AzrsError::General(format!("HTTP {}: {}", resp.status, resp.text())));
    }

    // For 202, try to parse the polling result
    if resp.status == 202 {
        // Return the initial response — full LRO polling for what-if would need
        // the poll_until_done mechanism, which we'd need to expose differently
        let body_text = resp.text();
        if body_text.is_empty() {
            return Ok(serde_json::json!({"status": "Accepted"}));
        }
        return Ok(serde_json::from_str(&body_text)?);
    }

    Ok(serde_json::from_str(&resp.text())?)
}

/// `deployment operation list`
pub async fn operation_list(scope: Scope<'_>, name: &str) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}/operations?api-version={API_VERSION}", scope.base_path(), name);
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// Build the deployment request body from template and parameters.
fn build_deployment_body(
    template_file: Option<&str>,
    template_uri: Option<&str>,
    parameters: Option<&str>,
    mode: Option<&str>,
) -> Result<serde_json::Value> {
    let mut properties = serde_json::json!({});

    // Template: file or URI
    if let Some(file) = template_file {
        let content = if file.starts_with('@') {
            std::fs::read_to_string(&file[1..])
                .map_err(|e| AzrsError::General(format!("Cannot read template file: {e}")))?
        } else {
            std::fs::read_to_string(file)
                .map_err(|e| AzrsError::General(format!("Cannot read template file: {e}")))?
        };
        let template: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| AzrsError::General(format!("Invalid template JSON: {e}")))?;
        properties["template"] = template;
    } else if let Some(uri) = template_uri {
        properties["templateLink"] = serde_json::json!({"uri": uri});
    } else {
        return Err(AzrsError::General(
            "Either --template-file or --template-uri is required".into(),
        ));
    }

    // Parameters
    if let Some(params) = parameters {
        let params_value = if params.starts_with('@') {
            let content = std::fs::read_to_string(&params[1..])
                .map_err(|e| AzrsError::General(format!("Cannot read parameters file: {e}")))?;
            let parsed: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| AzrsError::General(format!("Invalid parameters JSON: {e}")))?;
            // Handle both { "parameters": { ... } } and bare { "key": { "value": ... } } formats
            if let Some(inner) = parsed.get("parameters") {
                inner.clone()
            } else {
                parsed
            }
        } else {
            serde_json::from_str(params)
                .map_err(|e| AzrsError::General(format!("Invalid parameters JSON: {e}")))?
        };
        properties["parameters"] = params_value;
    }

    // Mode (Incremental or Complete, defaults to Incremental)
    properties["mode"] = serde_json::Value::String(
        mode.unwrap_or("Incremental").to_string()
    );

    Ok(serde_json::json!({"properties": properties}))
}

// --- Deployment Scripts ---

const DS_API_VERSION: &str = "2023-08-01";

/// `deployment-scripts list [--resource-group <rg>]`
pub async fn scripts_list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = match resource_group {
        Some(rg) => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Resources/deploymentScripts?api-version={DS_API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Resources/deploymentScripts?api-version={DS_API_VERSION}"
        ),
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `deployment-scripts show-log --resource-group <rg> --name <name>`
pub async fn scripts_show_log(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Resources/deploymentScripts/{name}/logs?api-version={DS_API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `deployment-scripts delete --resource-group <rg> --name <name>`
pub async fn scripts_delete(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Resources/deploymentScripts/{name}?api-version={DS_API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}
