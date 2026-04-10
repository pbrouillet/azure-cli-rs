/// Management group commands — `azrs account management-group`.
///
/// ARM API: Microsoft.Management/managementGroups (api-version 2021-04-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2021-04-01";

/// `account management-group list`
pub async fn list() -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/providers/Microsoft.Management/managementGroups?api-version={API_VERSION}");
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `account management-group show --name <name>`
pub async fn show(name: &str, expand: Option<&str>, recurse: bool) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let mut path = format!("/providers/Microsoft.Management/managementGroups/{name}?api-version={API_VERSION}");
    if let Some(exp) = expand {
        path.push_str(&format!("&$expand={exp}"));
    }
    if recurse {
        path.push_str("&$recurse=true");
    }
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `account management-group create --name <name>`
pub async fn create(name: &str, display_name: Option<&str>, parent: Option<&str>) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/providers/Microsoft.Management/managementGroups/{name}?api-version={API_VERSION}");

    let mut body = serde_json::json!({});
    if let Some(dn) = display_name {
        body["properties"] = serde_json::json!({"displayName": dn});
    }
    if let Some(p) = parent {
        body["properties"]["details"] = serde_json::json!({
            "parent": {"id": format!("/providers/Microsoft.Management/managementGroups/{p}")}
        });
    }

    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `account management-group delete --name <name>`
pub async fn delete(name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/providers/Microsoft.Management/managementGroups/{name}?api-version={API_VERSION}");
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `account management-group check-name-availability --name <name>`
pub async fn check_name_availability(name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/providers/Microsoft.Management/checkNameAvailability?api-version={API_VERSION}");
    let body = serde_json::json!({
        "name": name,
        "type": "Microsoft.Management/managementGroups"
    });
    let result = cmd.post(&path, Some(&body)).await?;
    cmd.save_cache()?;
    Ok(result)
}

// --- Management Group Subscriptions ---

/// `account management-group subscription add`
pub async fn subscription_add(group_name: &str, subscription_id: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/providers/Microsoft.Management/managementGroups/{group_name}/subscriptions/{subscription_id}?api-version={API_VERSION}"
    );
    let body = serde_json::json!({});
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `account management-group subscription remove`
pub async fn subscription_remove(group_name: &str, subscription_id: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/providers/Microsoft.Management/managementGroups/{group_name}/subscriptions/{subscription_id}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `account management-group subscription show-sub-under-mg`
pub async fn subscription_show(group_name: &str, subscription_id: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/providers/Microsoft.Management/managementGroups/{group_name}/subscriptions/{subscription_id}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

// --- Entities ---

/// `account management-group entities list`
pub async fn entities_list() -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/providers/Microsoft.Management/getEntities?api-version={API_VERSION}");
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    // The result contains a "value" array
    if let Some(arr) = result.get("value").and_then(|v| v.as_array()) {
        Ok(arr.clone())
    } else {
        Ok(vec![result])
    }
}

// --- Hierarchy Settings ---

const HS_API_VERSION: &str = "2021-04-01";

/// `account management-group hierarchy-settings list`
pub async fn hierarchy_settings_list(group_name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/providers/Microsoft.Management/managementGroups/{group_name}/settings?api-version={HS_API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `account management-group hierarchy-settings create`
pub async fn hierarchy_settings_create(
    group_name: &str,
    require_authorization: Option<bool>,
    default_mg: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/providers/Microsoft.Management/managementGroups/{group_name}/settings/default?api-version={HS_API_VERSION}"
    );
    let mut props = serde_json::json!({});
    if let Some(ra) = require_authorization {
        props["requireAuthorizationForGroupCreation"] = serde_json::Value::Bool(ra);
    }
    if let Some(mg) = default_mg {
        props["defaultManagementGroup"] = serde_json::Value::String(
            format!("/providers/Microsoft.Management/managementGroups/{mg}")
        );
    }
    let body = serde_json::json!({"properties": props});
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `account management-group hierarchy-settings delete`
pub async fn hierarchy_settings_delete(group_name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/providers/Microsoft.Management/managementGroups/{group_name}/settings/default?api-version={HS_API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

// --- Tenant Backfill ---

/// `account management-group tenant-backfill get`
pub async fn tenant_backfill_get() -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/providers/Microsoft.Management/tenantBackfillStatus?api-version={API_VERSION}");
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `account management-group tenant-backfill start`
pub async fn tenant_backfill_start() -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/providers/Microsoft.Management/startTenantBackfill?api-version={API_VERSION}");
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}
