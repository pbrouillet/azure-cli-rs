/// Azure RBAC role commands — `azrs role assignment/definition` operations.
///
/// ARM API: Microsoft.Authorization (api-version 2022-04-01)
use super::ArmCommand;
use crate::error::{AzrsError, Result};

const API_VERSION: &str = "2022-04-01";

/// Resolve the ARM scope for role operations.
///
/// - If `scope` is given, use it directly (replacing `{subscriptionId}` if present).
/// - If only `resource_group` is given, build `/subscriptions/<sub>/resourceGroups/<rg>`.
/// - Otherwise, default to `/subscriptions/<sub>`.
fn resolve_scope(
    cmd: &ArmCommand,
    scope: Option<&str>,
    resource_group: Option<&str>,
) -> Result<String> {
    let sub_id = cmd.subscription_id()?;
    if let Some(s) = scope {
        Ok(s.replace("{subscriptionId}", sub_id))
    } else if let Some(rg) = resource_group {
        Ok(format!("/subscriptions/{sub_id}/resourceGroups/{rg}"))
    } else {
        Ok(format!("/subscriptions/{sub_id}"))
    }
}

/// Resolve a human-readable role name (e.g. "Contributor") to its full ARM role-definition ID.
async fn resolve_role_definition_id(
    cmd: &mut ArmCommand,
    scope: &str,
    role_name: &str,
) -> Result<String> {
    let path = format!(
        "{scope}/providers/Microsoft.Authorization/roleDefinitions?api-version={API_VERSION}&$filter=roleName%20eq%20'{role_name}'"
    );
    let resp = cmd.request("GET", &path, None).await?;
    if !resp.is_success() {
        return Err(AzrsError::General(format!(
            "Failed to resolve role '{}': HTTP {}",
            role_name, resp.status
        )));
    }
    let body: serde_json::Value = serde_json::from_str(&resp.text())?;
    let id = body
        .pointer("/value/0/id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            AzrsError::General(format!("Role '{}' not found in scope '{}'", role_name, scope))
        })?;
    Ok(id.to_string())
}

// ---------------------------------------------------------------------------
// Role assignments
// ---------------------------------------------------------------------------

/// `azrs role assignment list`
pub async fn assignment_list(
    scope: Option<&str>,
    resource_group: Option<&str>,
    assignee: Option<&str>,
    role: Option<&str>,
    include_inherited: bool,
    all: bool,
) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let resolved = resolve_scope(&cmd, scope, resource_group)?;

    let mut path = format!(
        "{resolved}/providers/Microsoft.Authorization/roleAssignments?api-version={API_VERSION}"
    );

    // Build $filter
    if let Some(principal) = assignee {
        path.push_str(&format!("&$filter=principalId%20eq%20'{principal}'"));
    } else if !include_inherited && !all {
        path.push_str("&$filter=atScope()");
    }

    let mut results = cmd.list(&path).await?;

    // Post-filter by role if requested
    if let Some(role_name) = role {
        let role_def_id = resolve_role_definition_id(&mut cmd, &resolved, role_name).await?;
        results.retain(|item| {
            item.pointer("/properties/roleDefinitionId")
                .and_then(|v| v.as_str())
                .map(|id| id.ends_with(&role_def_id))
                .unwrap_or(false)
        });
    }

    cmd.save_cache()?;
    Ok(results)
}

/// `azrs role assignment create`
pub async fn assignment_create(
    scope: &str,
    role: &str,
    assignee_object_id: &str,
    assignee_principal_type: Option<&str>,
    name: Option<&str>,
    description: Option<&str>,
    condition: Option<&str>,
    condition_version: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let resolved = resolve_scope(&cmd, Some(scope), None)?;

    let role_def_id = resolve_role_definition_id(&mut cmd, &resolved, role).await?;

    let assignment_name = match name {
        Some(n) => n.to_string(),
        None => uuid::Uuid::new_v4().to_string(),
    };

    let mut props = serde_json::json!({
        "roleDefinitionId": role_def_id,
        "principalId": assignee_object_id,
    });
    if let Some(pt) = assignee_principal_type {
        props["principalType"] = serde_json::Value::String(pt.to_string());
    }
    if let Some(d) = description {
        props["description"] = serde_json::Value::String(d.to_string());
    }
    if let Some(c) = condition {
        props["condition"] = serde_json::Value::String(c.to_string());
        // Default condition version to 2.0 if condition is provided but version is not
        if condition_version.is_none() {
            props["conditionVersion"] = serde_json::Value::String("2.0".to_string());
        }
    }
    if let Some(cv) = condition_version {
        props["conditionVersion"] = serde_json::Value::String(cv.to_string());
    }

    let body = serde_json::json!({ "properties": props });

    let path = format!(
        "{resolved}/providers/Microsoft.Authorization/roleAssignments/{assignment_name}?api-version={API_VERSION}"
    );
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs role assignment delete`
pub async fn assignment_delete(
    scope: Option<&str>,
    resource_group: Option<&str>,
    assignee: Option<&str>,
    role: Option<&str>,
    ids: Option<&[String]>,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;

    if let Some(id_list) = ids {
        for id in id_list {
            let path = format!("{id}?api-version={API_VERSION}");
            cmd.delete(&path).await?;
            eprintln!("Deleted role assignment '{id}'.");
        }
    } else {
        let resolved = resolve_scope(&cmd, scope, resource_group)?;
        let mut list_path = format!(
            "{resolved}/providers/Microsoft.Authorization/roleAssignments?api-version={API_VERSION}"
        );
        if let Some(principal) = assignee {
            list_path.push_str(&format!("&$filter=principalId%20eq%20'{principal}'"));
        }

        let mut results = cmd.list(&list_path).await?;

        if let Some(role_name) = role {
            let role_def_id =
                resolve_role_definition_id(&mut cmd, &resolved, role_name).await?;
            results.retain(|item| {
                item.pointer("/properties/roleDefinitionId")
                    .and_then(|v| v.as_str())
                    .map(|id| id.ends_with(&role_def_id))
                    .unwrap_or(false)
            });
        }

        if results.is_empty() {
            eprintln!("No matching role assignments found.");
        }
        for item in &results {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                let path = format!("{id}?api-version={API_VERSION}");
                cmd.delete(&path).await?;
                eprintln!("Deleted role assignment '{id}'.");
            }
        }
    }

    cmd.save_cache()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Role definitions
// ---------------------------------------------------------------------------

/// `azrs role definition list`
pub async fn definition_list(
    scope: Option<&str>,
    resource_group: Option<&str>,
    name: Option<&str>,
    custom_role_only: bool,
) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let resolved = resolve_scope(&cmd, scope, resource_group)?;

    let mut path = format!(
        "{resolved}/providers/Microsoft.Authorization/roleDefinitions?api-version={API_VERSION}"
    );
    if let Some(n) = name {
        path.push_str(&format!("&$filter=roleName%20eq%20'{n}'"));
    }

    let mut results = cmd.list(&path).await?;

    if custom_role_only {
        results.retain(|item| {
            item.pointer("/properties/type")
                .and_then(|v| v.as_str())
                .map(|t| t == "CustomRole")
                .unwrap_or(false)
        });
    }

    cmd.save_cache()?;
    Ok(results)
}

/// `azrs role definition create --role-definition <json-or-file>`
///
/// If `role_definition` starts with `@`, reads the file at the given path.
pub async fn definition_create(role_definition: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let json_str = if let Some(path) = role_definition.strip_prefix('@') {
        std::fs::read_to_string(path)?
    } else {
        role_definition.to_string()
    };
    let mut def: serde_json::Value = serde_json::from_str(&json_str)?;

    // Normalize: if the JSON has top-level fields rather than a `properties` wrapper,
    // wrap them for the ARM PUT body.
    if def.get("properties").is_none() {
        def = serde_json::json!({ "properties": def });
    }

    let scope = def
        .pointer("/properties/assignableScopes/0")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            AzrsError::General(
                "Role definition must include at least one assignableScopes entry".into(),
            )
        })?
        .to_string();

    let role_id = uuid::Uuid::new_v4().to_string();
    let path = format!(
        "{scope}/providers/Microsoft.Authorization/roleDefinitions/{role_id}?api-version={API_VERSION}"
    );

    let result = cmd.put(&path, &def).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs role definition update --role-definition <json-or-file>`
pub async fn definition_update(role_definition: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let json_str = if let Some(path) = role_definition.strip_prefix('@') {
        std::fs::read_to_string(path)?
    } else {
        role_definition.to_string()
    };
    let mut def: serde_json::Value = serde_json::from_str(&json_str)?;

    if def.get("properties").is_none() {
        def = serde_json::json!({ "properties": def });
    }

    // Try to use the existing `id` field for the PUT path.
    // If missing, resolve by roleName within assignableScopes.
    let put_path = if let Some(id) = def.get("id").and_then(|v| v.as_str()) {
        format!("{id}?api-version={API_VERSION}")
    } else {
        let scope = def
            .pointer("/properties/assignableScopes/0")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AzrsError::General(
                    "Role definition must include 'id' or 'assignableScopes'".into(),
                )
            })?
            .to_string();

        let role_name = def
            .pointer("/properties/roleName")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AzrsError::General(
                    "Role definition must include 'id' or 'properties.roleName'".into(),
                )
            })?;

        let resolved_id =
            resolve_role_definition_id(&mut cmd, &scope, role_name).await?;
        format!("{resolved_id}?api-version={API_VERSION}")
    };

    let result = cmd.put(&put_path, &def).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs role definition delete --name <name>`
pub async fn definition_delete(
    scope: Option<&str>,
    resource_group: Option<&str>,
    name: &str,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let resolved = resolve_scope(&cmd, scope, resource_group)?;

    let list_path = format!(
        "{resolved}/providers/Microsoft.Authorization/roleDefinitions?api-version={API_VERSION}&$filter=roleName%20eq%20'{name}'"
    );
    let results = cmd.list(&list_path).await?;

    if results.is_empty() {
        return Err(AzrsError::General(format!(
            "Role definition '{}' not found in scope '{}'",
            name, resolved
        )));
    }
    for item in &results {
        if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
            let path = format!("{id}?api-version={API_VERSION}");
            cmd.delete(&path).await?;
            eprintln!("Deleted role definition '{name}'.");
        }
    }

    cmd.save_cache()?;
    Ok(())
}
