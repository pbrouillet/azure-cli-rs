/// Extended Function App commands — plan, deployment slot, vnet-integration, scale config.
///
/// ARM API: Microsoft.Web (api-version 2024-11-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2024-11-01";

// ── Functionapp Plan ───────────────────────────────────────────────

/// `functionapp plan list [--resource-group <rg>]`
pub async fn plan_list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = match resource_group {
        Some(rg) => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Web/serverfarms?api-version={API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Web/serverfarms?api-version={API_VERSION}"
        ),
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `functionapp plan show`
pub async fn plan_show(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/serverfarms/{name}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp plan create`
pub async fn plan_create(
    resource_group: &str,
    name: &str,
    location: &str,
    sku: Option<&str>,
    is_linux: bool,
    max_burst: Option<i64>,
    tags: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/serverfarms/{name}?api-version={API_VERSION}"
    );

    let sku_name = sku.unwrap_or("Y1");
    let mut body = serde_json::json!({
        "location": location,
        "sku": {
            "name": sku_name
        },
        "properties": {
            "reserved": is_linux
        }
    });

    if let Some(burst) = max_burst {
        body["properties"]["maximumElasticWorkerCount"] =
            serde_json::Value::Number(serde_json::Number::from(burst));
    }

    if let Some(tag_list) = tags {
        body["tags"] = serde_json::to_value(crate::commands::group::parse_tags(tag_list))?;
    }

    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp plan delete`
pub async fn plan_delete(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/serverfarms/{name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `functionapp plan update`
pub async fn plan_update(
    resource_group: &str,
    name: &str,
    sku: Option<&str>,
    max_burst: Option<i64>,
    number_of_workers: Option<i64>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/serverfarms/{name}?api-version={API_VERSION}"
    );

    let mut current = cmd.get(&path).await?;
    if let Some(s) = sku {
        current["sku"]["name"] = serde_json::Value::String(s.to_string());
    }
    if let Some(n) = number_of_workers {
        current["sku"]["capacity"] = serde_json::Value::Number(serde_json::Number::from(n));
    }
    if let Some(burst) = max_burst {
        current["properties"]["maximumElasticWorkerCount"] =
            serde_json::Value::Number(serde_json::Number::from(burst));
    }

    let result = cmd.put(&path, &current).await?;
    cmd.save_cache()?;
    Ok(result)
}

// ── Deployment Slot ────────────────────────────────────────────────

fn site_path(resource_group: &str, name: &str) -> String {
    format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/sites/{name}"
    )
}

/// `functionapp deployment slot list`
pub async fn deployment_slot_list(
    resource_group: &str,
    name: &str,
) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/slots?api-version={API_VERSION}", site_path(resource_group, name));
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `functionapp deployment slot create`
pub async fn deployment_slot_create(
    resource_group: &str,
    name: &str,
    slot: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/slots/{}?api-version={API_VERSION}",
        site_path(resource_group, name),
        slot
    );

    // GET the parent app to copy location
    let parent_path = format!("{}?api-version={API_VERSION}", site_path(resource_group, name));
    let parent = cmd.get(&parent_path).await?;
    let location = parent
        .get("location")
        .and_then(|l| l.as_str())
        .unwrap_or("eastus");

    let body = serde_json::json!({
        "location": location,
        "properties": {}
    });

    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp deployment slot delete`
pub async fn deployment_slot_delete(
    resource_group: &str,
    name: &str,
    slot: &str,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/slots/{}?api-version={API_VERSION}",
        site_path(resource_group, name),
        slot
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `functionapp deployment slot swap`
pub async fn deployment_slot_swap(
    resource_group: &str,
    name: &str,
    slot: &str,
    target_slot: Option<&str>,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let target = target_slot.unwrap_or("production");
    let path = format!(
        "{}/slots/{}/slotsswap?api-version={API_VERSION}",
        site_path(resource_group, name),
        slot
    );
    let body = serde_json::json!({
        "targetSlot": target
    });
    cmd.post_lro(&path, Some(&body)).await?;
    cmd.save_cache()?;
    eprintln!("Swapped slot '{slot}' with '{target}'.");
    Ok(())
}

// ── VNet Integration ───────────────────────────────────────────────

/// `functionapp vnet-integration list`
pub async fn vnet_integration_list(
    resource_group: &str,
    name: &str,
) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/virtualNetworkConnections?api-version={API_VERSION}",
        site_path(resource_group, name)
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `functionapp vnet-integration add`
pub async fn vnet_integration_add(
    resource_group: &str,
    name: &str,
    vnet: &str,
    subnet: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let sub_id = cmd.subscription_id()?.to_string();

    let subnet_id = if subnet.starts_with('/') {
        subnet.to_string()
    } else {
        format!(
            "/subscriptions/{sub_id}/resourceGroups/{resource_group}/providers/Microsoft.Network/virtualNetworks/{vnet}/subnets/{subnet}"
        )
    };

    let path = format!(
        "{}/networkConfig/virtualNetwork?api-version={API_VERSION}",
        site_path(resource_group, name)
    );
    let body = serde_json::json!({
        "properties": {
            "subnetResourceId": subnet_id
        }
    });

    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `functionapp vnet-integration remove`
pub async fn vnet_integration_remove(
    resource_group: &str,
    name: &str,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/networkConfig/virtualNetwork?api-version={API_VERSION}",
        site_path(resource_group, name)
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

// ── Scale Config ───────────────────────────────────────────────────

/// `functionapp scale config set`
pub async fn scale_config_set(
    resource_group: &str,
    name: &str,
    max_burst: Option<i64>,
    trigger_type: Option<&str>,
    trigger_value: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    // GET the current site config
    let site = format!("{}?api-version={API_VERSION}", site_path(resource_group, name));
    let mut current = cmd.get(&site).await?;

    if let Some(burst) = max_burst {
        current["properties"]["siteConfig"]["functionAppScaleLimit"] =
            serde_json::Value::Number(serde_json::Number::from(burst));
    }

    if let (Some(t_type), Some(t_val)) = (trigger_type, trigger_value) {
        current["properties"]["siteConfig"]["minimumElasticInstanceCount"] =
            serde_json::from_str::<serde_json::Value>(t_val)
                .unwrap_or_else(|_| serde_json::Value::String(t_val.to_string()));
        // Store trigger type as a tag for reference
        if current.get("tags").is_none() {
            current["tags"] = serde_json::json!({});
        }
        current["tags"]["scaleConfig.triggerType"] =
            serde_json::Value::String(t_type.to_string());
    }

    let result = cmd.put(&site, &current).await?;
    cmd.save_cache()?;
    Ok(result)
}
