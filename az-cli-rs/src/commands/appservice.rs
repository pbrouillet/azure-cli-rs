/// Appservice commands — plan, ASE, domain, plan identity.
///
/// ARM API: Microsoft.Web/serverfarms, hostingEnvironments (api-version 2024-11-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2024-11-01";

/// `appservice list-locations`
pub async fn list_locations() -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/providers/Microsoft.Web/geoRegions?api-version={API_VERSION}");
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `appservice plan list [--resource-group <rg>]`
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

/// `appservice plan show`
pub async fn plan_show(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/serverfarms/{name}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `appservice plan create`
pub async fn plan_create(
    resource_group: &str,
    name: &str,
    location: &str,
    sku: Option<&str>,
    is_linux: bool,
    tags: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/serverfarms/{name}?api-version={API_VERSION}"
    );

    let sku_name = sku.unwrap_or("B1");
    let mut body = serde_json::json!({
        "location": location,
        "sku": {
            "name": sku_name
        },
        "properties": {
            "reserved": is_linux
        }
    });

    if let Some(tag_list) = tags {
        body["tags"] = serde_json::to_value(crate::commands::group::parse_tags(tag_list))?;
    }

    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `appservice plan delete`
pub async fn plan_delete(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/serverfarms/{name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `appservice plan update`
pub async fn plan_update(
    resource_group: &str,
    name: &str,
    sku: Option<&str>,
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

    let result = cmd.put(&path, &current).await?;
    cmd.save_cache()?;
    Ok(result)
}

// ── ASE (App Service Environment) ──────────────────────────────────

fn ase_base(resource_group: &str) -> String {
    format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/hostingEnvironments"
    )
}

/// `appservice ase list [--resource-group <rg>]`
pub async fn ase_list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = match resource_group {
        Some(rg) => format!("{}?api-version={API_VERSION}", ase_base(rg)),
        None => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Web/hostingEnvironments?api-version={API_VERSION}"
        ),
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `appservice ase show`
pub async fn ase_show(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}?api-version={API_VERSION}", ase_base(resource_group), name);
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `appservice ase create`
pub async fn ase_create(
    resource_group: &str,
    name: &str,
    location: &str,
    vnet_name: &str,
    subnet: &str,
    kind: Option<&str>,
    tags: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let sub_id = cmd.subscription_id()?.to_string();
    let path = format!("{}/{}?api-version={API_VERSION}", ase_base(resource_group), name);

    let subnet_id = if subnet.starts_with('/') {
        subnet.to_string()
    } else {
        format!(
            "/subscriptions/{sub_id}/resourceGroups/{resource_group}/providers/Microsoft.Network/virtualNetworks/{vnet_name}/subnets/{subnet}"
        )
    };

    let ase_kind = kind.unwrap_or("ASEv3");
    let mut body = serde_json::json!({
        "kind": ase_kind,
        "location": location,
        "properties": {
            "virtualNetwork": {
                "id": subnet_id
            }
        }
    });

    if let Some(tag_list) = tags {
        body["tags"] = serde_json::to_value(crate::commands::group::parse_tags(tag_list))?;
    }

    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `appservice ase delete`
pub async fn ase_delete(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}?api-version={API_VERSION}", ase_base(resource_group), name);
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `appservice ase update` — GET + merge set values + PUT
pub async fn ase_update(
    resource_group: &str,
    name: &str,
    set_values: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}?api-version={API_VERSION}", ase_base(resource_group), name);
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

/// `appservice ase list-addresses`
pub async fn ase_list_addresses(resource_group: &str, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/{}/capacities?api-version={API_VERSION}",
        ase_base(resource_group),
        name
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `appservice ase list-plans`
pub async fn ase_list_plans(resource_group: &str, name: &str) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/{}/serverfarms?api-version={API_VERSION}",
        ase_base(resource_group),
        name
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `appservice ase upgrade` — POST LRO
pub async fn ase_upgrade(resource_group: &str, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/{}/upgrade?api-version={API_VERSION}",
        ase_base(resource_group),
        name
    );
    cmd.post_lro(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("ASE '{name}' upgrade initiated.");
    Ok(())
}

// ── Domain ─────────────────────────────────────────────────────────

const DOMAIN_API_VERSION: &str = "2024-11-01";

/// `appservice domain create`
pub async fn domain_create(
    resource_group: &str,
    hostname: &str,
    contact_info_json: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.DomainRegistration/domains/{hostname}?api-version={DOMAIN_API_VERSION}"
    );

    let contact_info: serde_json::Value = serde_json::from_str(contact_info_json)
        .map_err(|e| crate::error::AzrsError::General(format!("Invalid contact info JSON: {e}")))?;

    let body = serde_json::json!({
        "location": "global",
        "properties": {
            "contactAdmin": contact_info,
            "contactBilling": contact_info,
            "contactRegistrant": contact_info,
            "contactTech": contact_info,
            "autoRenew": true
        }
    });

    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `appservice domain show-terms`
pub async fn domain_show_terms() -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/providers/Microsoft.DomainRegistration/topLevelDomainAgreements?api-version={DOMAIN_API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

// ── Plan Identity ──────────────────────────────────────────────────

/// `appservice plan identity assign`
pub async fn plan_identity_assign(
    resource_group: &str,
    name: &str,
    identity_type: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/serverfarms/{name}?api-version={API_VERSION}"
    );
    let mut current = cmd.get(&path).await?;

    let id_type = identity_type.unwrap_or("SystemAssigned");
    current["identity"] = serde_json::json!({
        "type": id_type
    });

    let result = cmd.put(&path, &current).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `appservice plan identity remove`
pub async fn plan_identity_remove(
    resource_group: &str,
    name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Web/serverfarms/{name}?api-version={API_VERSION}"
    );
    let mut current = cmd.get(&path).await?;

    current["identity"] = serde_json::json!({
        "type": "None"
    });

    let result = cmd.put(&path, &current).await?;
    cmd.save_cache()?;
    Ok(result)
}
