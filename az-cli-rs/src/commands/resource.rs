/// Generic resource commands — `azrs resource create/list/show/delete/update/tag/invoke-action/wait`.
///
/// These operate on any ARM resource using its full resource ID or the combination of
/// --resource-group, --namespace, --resource-type, --name, and optional --parent.
///
/// ARM API: api-version is user-specified (varies per resource type).
use super::ArmCommand;
use crate::error::{AzrsError, Result};

/// Build an ARM resource path from individual components.
///
/// Supports two modes:
/// 1. Full resource ID (starts with `/`) — used as-is
/// 2. Individual parts: resource-group + namespace + resource-type + name + optional parent
fn build_resource_path(
    resource_id: Option<&str>,
    resource_group: Option<&str>,
    namespace: Option<&str>,
    resource_type: Option<&str>,
    name: Option<&str>,
    parent: Option<&str>,
    api_version: &str,
) -> Result<String> {
    if let Some(id) = resource_id {
        // Full resource ID — append api-version
        let sep = if id.contains('?') { '&' } else { '?' };
        return Ok(format!("{id}{sep}api-version={api_version}"));
    }

    let rg = resource_group.ok_or_else(|| {
        AzrsError::General("--resource-group is required when --ids is not specified".into())
    })?;
    let ns = namespace.ok_or_else(|| {
        AzrsError::General("--namespace is required when --ids is not specified".into())
    })?;
    let rt = resource_type.ok_or_else(|| {
        AzrsError::General("--resource-type is required when --ids is not specified".into())
    })?;
    let n = name.ok_or_else(|| {
        AzrsError::General("--name is required when --ids is not specified".into())
    })?;

    let parent_segment = match parent {
        Some(p) if !p.is_empty() => format!("/{p}"),
        _ => String::new(),
    };

    Ok(format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/{ns}{parent_segment}/{rt}/{n}?api-version={api_version}"
    ))
}

/// `azrs resource list [--resource-group] [--resource-type] [--tag] [--name]`
pub async fn list(
    resource_group: Option<&str>,
    resource_type: Option<&str>,
    tag: Option<&str>,
    name: Option<&str>,
) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;

    let mut filters = Vec::new();
    if let Some(rt) = resource_type {
        filters.push(format!("resourceType eq '{rt}'"));
    }
    if let Some(n) = name {
        filters.push(format!("name eq '{n}'"));
    }
    if let Some(t) = tag {
        if let Some((k, v)) = t.split_once('=') {
            filters.push(format!("tagName eq '{k}' and tagValue eq '{v}'"));
        } else {
            filters.push(format!("tagName eq '{t}'"));
        }
    }

    let filter_str = if filters.is_empty() {
        String::new()
    } else {
        format!("&$filter={}", filters.join(" and "))
    };

    let path = match resource_group {
        Some(rg) => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/resources?api-version=2024-03-01{filter_str}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/resources?api-version=2024-03-01{filter_str}"
        ),
    };

    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs resource show`
pub async fn show(
    resource_id: Option<&str>,
    resource_group: Option<&str>,
    namespace: Option<&str>,
    resource_type: Option<&str>,
    name: Option<&str>,
    parent: Option<&str>,
    api_version: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = build_resource_path(resource_id, resource_group, namespace, resource_type, name, parent, api_version)?;
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs resource delete`
pub async fn delete(
    resource_id: Option<&str>,
    resource_group: Option<&str>,
    namespace: Option<&str>,
    resource_type: Option<&str>,
    name: Option<&str>,
    parent: Option<&str>,
    api_version: &str,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = build_resource_path(resource_id, resource_group, namespace, resource_type, name, parent, api_version)?;
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `azrs resource create`
pub async fn create(
    resource_id: Option<&str>,
    resource_group: Option<&str>,
    namespace: Option<&str>,
    resource_type: Option<&str>,
    name: Option<&str>,
    parent: Option<&str>,
    api_version: &str,
    properties: &str,
    location: Option<&str>,
    tags: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = build_resource_path(resource_id, resource_group, namespace, resource_type, name, parent, api_version)?;

    let mut body: serde_json::Value = serde_json::from_str(properties)
        .map_err(|e| AzrsError::General(format!("Invalid JSON properties: {e}")))?;

    if let Some(loc) = location {
        body["location"] = serde_json::Value::String(loc.to_string());
    }
    if let Some(tag_list) = tags {
        body["tags"] = serde_json::to_value(crate::commands::group::parse_tags(tag_list))?;
    }

    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs resource update` — GET + merge + PUT
pub async fn update(
    resource_id: Option<&str>,
    resource_group: Option<&str>,
    namespace: Option<&str>,
    resource_type: Option<&str>,
    name: Option<&str>,
    parent: Option<&str>,
    api_version: &str,
    set_values: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = build_resource_path(resource_id, resource_group, namespace, resource_type, name, parent, api_version)?;

    // GET current state
    let mut current = cmd.get(&path).await?;

    // Apply --set key=value updates
    if let Some(pairs) = set_values {
        for pair in pairs {
            if let Some((key, value)) = pair.split_once('=') {
                set_json_path(&mut current, key, value);
            }
        }
    }

    let result = cmd.put_lro(&path, &current).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs resource tag`
pub async fn tag(
    resource_id: Option<&str>,
    resource_group: Option<&str>,
    namespace: Option<&str>,
    resource_type: Option<&str>,
    name: Option<&str>,
    parent: Option<&str>,
    api_version: &str,
    tags: &[String],
    is_incremental: bool,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = build_resource_path(resource_id, resource_group, namespace, resource_type, name, parent, api_version)?;

    let new_tags = crate::commands::group::parse_tags(tags);

    if is_incremental {
        // Merge with existing tags
        let mut current = cmd.get(&path).await?;
        let existing = current.get("tags").cloned().unwrap_or(serde_json::json!({}));
        let mut merged: serde_json::Map<String, serde_json::Value> = serde_json::from_value(existing).unwrap_or_default();
        for (k, v) in &new_tags {
            merged.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        current["tags"] = serde_json::Value::Object(merged);
        let result = cmd.put(&path, &current).await?;
        cmd.save_cache()?;
        Ok(result)
    } else {
        // Replace all tags
        let mut current = cmd.get(&path).await?;
        current["tags"] = serde_json::to_value(&new_tags)?;
        let result = cmd.put(&path, &current).await?;
        cmd.save_cache()?;
        Ok(result)
    }
}

/// `azrs resource invoke-action`
pub async fn invoke_action(
    resource_id: Option<&str>,
    resource_group: Option<&str>,
    namespace: Option<&str>,
    resource_type: Option<&str>,
    name: Option<&str>,
    parent: Option<&str>,
    api_version: &str,
    action: &str,
    request_body: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    // Build path to the resource then append the action
    let base = build_resource_path(resource_id, resource_group, namespace, resource_type, name, parent, api_version)?;
    // Insert the action before the query string
    let path = if let Some(idx) = base.find('?') {
        format!("{}/{action}{}", &base[..idx], &base[idx..])
    } else {
        format!("{base}/{action}")
    };

    let body = match request_body {
        Some(b) => {
            let v: serde_json::Value = serde_json::from_str(b)
                .map_err(|e| AzrsError::General(format!("Invalid JSON body: {e}")))?;
            Some(v)
        }
        None => None,
    };

    let result = cmd.post(&path, body.as_ref()).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs resource wait`
pub async fn wait(
    resource_id: Option<&str>,
    resource_group: Option<&str>,
    namespace: Option<&str>,
    resource_type: Option<&str>,
    name: Option<&str>,
    parent: Option<&str>,
    api_version: &str,
    created: bool,
    updated: bool,
    deleted: bool,
    exists_flag: bool,
    custom: Option<&str>,
    interval: u64,
    timeout: u64,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = build_resource_path(resource_id, resource_group, namespace, resource_type, name, parent, api_version)?;
    let start = std::time::Instant::now();

    loop {
        if start.elapsed().as_secs() >= timeout {
            return Err(AzrsError::General("Timed out waiting for resource".into()));
        }

        let resp = cmd.request("GET", &path, None).await?;
        let status = resp.status;

        if deleted {
            if status == 404 {
                cmd.save_cache()?;
                return Ok(());
            }
        } else if exists_flag {
            if status == 200 {
                cmd.save_cache()?;
                return Ok(());
            }
        } else if created || updated {
            if status == 200 {
                let body: serde_json::Value = serde_json::from_str(&resp.text())?;
                let prov_state = body
                    .pointer("/properties/provisioningState")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if prov_state == "Succeeded" {
                    cmd.save_cache()?;
                    return Ok(());
                }
            }
        } else if let Some(jmespath) = custom {
            if status == 200 {
                let body: serde_json::Value = serde_json::from_str(&resp.text())?;
                let result = crate::output::jmespath_eval(&body, jmespath);
                if result_is_truthy(&result) {
                    cmd.save_cache()?;
                    return Ok(());
                }
            }
        }

        eprint!(".");
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
    }
}

/// Set a value at a dotted JSON path (e.g., "properties.sku.name" = "Standard").
fn set_json_path(value: &mut serde_json::Value, path: &str, val: &str) {
    set_json_path_pub(value, path, val);
}

/// Public version of set_json_path for use by other modules.
pub fn set_json_path_pub(value: &mut serde_json::Value, path: &str, val: &str) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            let parsed = serde_json::from_str::<serde_json::Value>(val)
                .unwrap_or_else(|_| serde_json::Value::String(val.to_string()));
            current[*part] = parsed;
        } else {
            if !current[*part].is_object() {
                current[*part] = serde_json::json!({});
            }
            current = &mut current[*part];
        }
    }
}

fn result_is_truthy(val: &serde_json::Value) -> bool {
    match val {
        serde_json::Value::Null => false,
        serde_json::Value::Bool(b) => *b,
        serde_json::Value::String(s) => !s.is_empty(),
        serde_json::Value::Number(_) => true,
        serde_json::Value::Array(a) => !a.is_empty(),
        serde_json::Value::Object(o) => !o.is_empty(),
    }
}

// --- Resource Link ---

const LINK_API_VERSION: &str = "2016-09-01";

/// `azrs resource link list [--scope <scope>]`
pub async fn link_list(scope: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = match scope {
        Some(s) => format!("{s}/providers/Microsoft.Resources/links?api-version={LINK_API_VERSION}"),
        None => format!("/subscriptions/{{subscriptionId}}/providers/Microsoft.Resources/links?api-version={LINK_API_VERSION}"),
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs resource link show --link-id <id>`
pub async fn link_show(link_id: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{link_id}?api-version={LINK_API_VERSION}");
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs resource link create --link-id <id> --target-id <target> [--notes <notes>]`
pub async fn link_create(link_id: &str, target_id: &str, notes: Option<&str>) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{link_id}?api-version={LINK_API_VERSION}");
    let mut body = serde_json::json!({
        "properties": {
            "targetId": target_id
        }
    });
    if let Some(n) = notes {
        body["properties"]["notes"] = serde_json::Value::String(n.to_string());
    }
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs resource link delete --link-id <id>`
pub async fn link_delete(link_id: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{link_id}?api-version={LINK_API_VERSION}");
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `azrs resource link update --link-id <id> [--target-id <target>] [--notes <notes>]`
pub async fn link_update(link_id: &str, target_id: Option<&str>, notes: Option<&str>) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{link_id}?api-version={LINK_API_VERSION}");
    let mut current = cmd.get(&path).await?;
    if let Some(t) = target_id {
        current["properties"]["targetId"] = serde_json::Value::String(t.to_string());
    }
    if let Some(n) = notes {
        current["properties"]["notes"] = serde_json::Value::String(n.to_string());
    }
    let result = cmd.put(&path, &current).await?;
    cmd.save_cache()?;
    Ok(result)
}
