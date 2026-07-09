/// Azure Event Grid commands — `azrs eventgrid topic/domain create/list/show/delete/update/key`.
///
/// ARM API: Microsoft.EventGrid (api-version 2022-06-15)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2022-06-15";

/// `azrs eventgrid topic create -n <name> -g <rg> -l <location> [--input-schema <schema>] [--public-network-access <enabled|disabled>]`
pub async fn topic_create(
    name: &str,
    resource_group: &str,
    location: &str,
    input_schema: Option<&str>,
    public_network_access: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.EventGrid/topics/{name}?api-version={API_VERSION}"
    );
    let properties = create_properties(input_schema, public_network_access)?;
    let body = serde_json::json!({ "location": location, "properties": properties });
    eprintln!("Creating Event Grid topic '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!();
    Ok(result)
}

/// `azrs eventgrid topic list [-g <rg>]` — list in a resource group, or across the subscription.
pub async fn topic_list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = if let Some(rg) = resource_group {
        format!("/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.EventGrid/topics?api-version={API_VERSION}")
    } else {
        format!("/subscriptions/{{subscriptionId}}/providers/Microsoft.EventGrid/topics?api-version={API_VERSION}")
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs eventgrid topic show -n <name> -g <rg>`
pub async fn topic_show(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.EventGrid/topics/{name}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs eventgrid topic delete -n <name> -g <rg>`
pub async fn topic_delete(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.EventGrid/topics/{name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    eprintln!("Event Grid topic '{name}' deleted.");
    Ok(())
}

/// `azrs eventgrid topic update -n <name> -g <rg> [--set key=value ...] [--tags key=value ...]`
///
/// Applies a PATCH with tag updates and/or arbitrary `properties.*` values (dot-path `--set`).
pub async fn topic_update(
    name: &str,
    resource_group: &str,
    tags: Option<&[String]>,
    set_props: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.EventGrid/topics/{name}?api-version={API_VERSION}"
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

/// `azrs eventgrid topic key list -n <name> -g <rg>`
pub async fn topic_key_list(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.EventGrid/topics/{name}/listKeys?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs eventgrid topic key regenerate -n <name> -g <rg> --key-name <key1|key2>`
pub async fn topic_key_regenerate(
    name: &str,
    resource_group: &str,
    key_name: &str,
) -> Result<serde_json::Value> {
    validate_key_name(key_name)?;
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.EventGrid/topics/{name}/regenerateKey?api-version={API_VERSION}"
    );
    let body = serde_json::json!({ "keyName": key_name });
    let result = cmd.post(&path, Some(&body)).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs eventgrid domain create -n <name> -g <rg> -l <location> [--input-schema <schema>] [--public-network-access <enabled|disabled>]`
pub async fn domain_create(
    name: &str,
    resource_group: &str,
    location: &str,
    input_schema: Option<&str>,
    public_network_access: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.EventGrid/domains/{name}?api-version={API_VERSION}"
    );
    let properties = create_properties(input_schema, public_network_access)?;
    let body = serde_json::json!({ "location": location, "properties": properties });
    eprintln!("Creating Event Grid domain '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!();
    Ok(result)
}

/// `azrs eventgrid domain list [-g <rg>]` — list in a resource group, or across the subscription.
pub async fn domain_list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = if let Some(rg) = resource_group {
        format!("/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.EventGrid/domains?api-version={API_VERSION}")
    } else {
        format!("/subscriptions/{{subscriptionId}}/providers/Microsoft.EventGrid/domains?api-version={API_VERSION}")
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs eventgrid domain show -n <name> -g <rg>`
pub async fn domain_show(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.EventGrid/domains/{name}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs eventgrid domain delete -n <name> -g <rg>`
pub async fn domain_delete(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.EventGrid/domains/{name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    eprintln!("Event Grid domain '{name}' deleted.");
    Ok(())
}

// TODO: Add Event Grid event-subscription, system-topic, and partner subgroups.

fn create_properties(
    input_schema: Option<&str>,
    public_network_access: Option<&str>,
) -> Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    if let Some(schema) = input_schema {
        properties.insert(
            "inputSchema".to_string(),
            serde_json::Value::String(normalize_input_schema(schema)?),
        );
    }
    if let Some(access) = public_network_access {
        properties.insert(
            "publicNetworkAccess".to_string(),
            serde_json::Value::String(normalize_public_network_access(access)?),
        );
    }
    Ok(serde_json::Value::Object(properties))
}

fn normalize_input_schema(input_schema: &str) -> Result<String> {
    match input_schema.to_ascii_lowercase().as_str() {
        "eventgridschema" => Ok("EventGridSchema".to_string()),
        "customeventschema" => Ok("CustomEventSchema".to_string()),
        "cloudeventschemav1_0" => Ok("CloudEventSchemaV1_0".to_string()),
        _ => Err(crate::error::AzrsError::General(format!(
            "invalid --input-schema '{input_schema}': expected eventgridschema, customeventschema, or cloudeventschemav1_0"
        ))),
    }
}

fn normalize_public_network_access(public_network_access: &str) -> Result<String> {
    match public_network_access.to_ascii_lowercase().as_str() {
        "enabled" => Ok("Enabled".to_string()),
        "disabled" => Ok("Disabled".to_string()),
        _ => Err(crate::error::AzrsError::General(format!(
            "invalid --public-network-access '{public_network_access}': expected enabled or disabled"
        ))),
    }
}

fn validate_key_name(key_name: &str) -> Result<()> {
    match key_name {
        "key1" | "key2" => Ok(()),
        _ => Err(crate::error::AzrsError::General(format!(
            "invalid --key-name '{key_name}': expected key1 or key2"
        ))),
    }
}

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
