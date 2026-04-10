/// Hand-written VMSS commands — `azrs vmss create/update/scale/deallocate/restart/stop/reimage/...`
///
/// ARM API: Microsoft.Compute/virtualMachineScaleSets (api-version 2024-07-01)
///
/// These complement the generated AAZ commands (delete, list, start, etc.) with
/// operations that need custom body construction or multi-step logic.
use super::ArmCommand;
use crate::commands::group::parse_tags;
use crate::error::Result;
use serde_json::{json, Value};

const API_VERSION: &str = "2024-07-01";

fn base_path(resource_group: &str, name: &str) -> String {
    format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}\
         /providers/Microsoft.Compute/virtualMachineScaleSets/{name}"
    )
}

/// `vmss create` — create a VMSS with a simplified parameter set.
///
/// Builds the ARM body with sku, upgradePolicy, and virtualMachineProfile
/// (storageProfile, osProfile, networkProfile).
#[allow(clippy::too_many_arguments)]
pub async fn create(
    resource_group: &str,
    name: &str,
    image: &str,
    location: &str,
    instance_count: Option<i64>,
    vm_sku: Option<&str>,
    admin_username: Option<&str>,
    admin_password: Option<&str>,
    upgrade_policy_mode: Option<&str>,
    tags: Option<&[String]>,
) -> Result<Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}?api-version={API_VERSION}", base_path(resource_group, name));

    let capacity = instance_count.unwrap_or(2);
    let sku_name = vm_sku.unwrap_or("Standard_DS1_v2");
    let admin = admin_username.unwrap_or("azureuser");
    let upgrade_mode = upgrade_policy_mode.unwrap_or("Manual");

    let mut os_profile = json!({
        "computerNamePrefix": &name[..name.len().min(9)],
        "adminUsername": admin,
    });
    if let Some(pw) = admin_password {
        os_profile["adminPassword"] = Value::String(pw.to_string());
    }

    let mut body = json!({
        "location": location,
        "sku": {
            "name": sku_name,
            "tier": "Standard",
            "capacity": capacity,
        },
        "properties": {
            "upgradePolicy": {
                "mode": upgrade_mode,
            },
            "virtualMachineProfile": {
                "storageProfile": {
                    "imageReference": parse_image_reference(image),
                    "osDisk": {
                        "createOption": "FromImage",
                        "caching": "ReadWrite",
                        "managedDisk": {
                            "storageAccountType": "Premium_LRS"
                        }
                    }
                },
                "osProfile": os_profile,
                "networkProfile": {
                    "networkInterfaceConfigurations": [{
                        "name": format!("{name}Nic"),
                        "properties": {
                            "primary": true,
                            "ipConfigurations": [{
                                "name": format!("{name}IpConfig"),
                                "properties": {
                                    "subnet": {
                                        "id": ""
                                    }
                                }
                            }]
                        }
                    }]
                }
            }
        }
    });

    if let Some(tag_list) = tags {
        body["tags"] = serde_json::to_value(parse_tags(tag_list))?;
    }

    eprintln!("Creating VMSS '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!("\nVMSS '{name}' created.");
    Ok(result)
}

/// `vmss update` — GET + merge `--set` property paths + PUT.
pub async fn update(
    resource_group: &str,
    name: &str,
    set_values: Option<&[String]>,
    tags: Option<&[String]>,
) -> Result<Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}?api-version={API_VERSION}", base_path(resource_group, name));

    let mut body = cmd.get(&path).await?;

    if let Some(sets) = set_values {
        for kv in sets {
            if let Some((key, val)) = kv.split_once('=') {
                set_nested_value(&mut body, key, val);
            }
        }
    }
    if let Some(tag_list) = tags {
        body["tags"] = serde_json::to_value(parse_tags(tag_list))?;
    }

    eprintln!("Updating VMSS '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!("\nVMSS '{name}' updated.");
    Ok(result)
}

/// `vmss scale` — change instance count by updating sku.capacity.
pub async fn scale(
    resource_group: &str,
    name: &str,
    new_capacity: i64,
) -> Result<Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}?api-version={API_VERSION}", base_path(resource_group, name));

    let mut body = cmd.get(&path).await?;
    body["sku"]["capacity"] = json!(new_capacity);

    eprintln!("Scaling VMSS '{name}' to {new_capacity} instances...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!("\nVMSS '{name}' scaled to {new_capacity} instances.");
    Ok(result)
}

/// `vmss deallocate` — POST .../deallocate with LRO.
pub async fn deallocate(
    resource_group: &str,
    name: &str,
    instance_ids: Option<&[String]>,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/deallocate?api-version={API_VERSION}",
        base_path(resource_group, name)
    );

    let body = instance_ids.map(|ids| json!({ "instanceIds": ids }));
    eprintln!("Deallocating VMSS '{name}'...");
    cmd.post_lro(&path, body.as_ref()).await?;
    cmd.save_cache()?;
    eprintln!("\nVMSS '{name}' deallocated.");
    Ok(())
}

/// `vmss restart` — POST .../restart with LRO.
pub async fn restart(
    resource_group: &str,
    name: &str,
    instance_ids: Option<&[String]>,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/restart?api-version={API_VERSION}",
        base_path(resource_group, name)
    );

    let body = instance_ids.map(|ids| json!({ "instanceIds": ids }));
    eprintln!("Restarting VMSS '{name}'...");
    cmd.post_lro(&path, body.as_ref()).await?;
    cmd.save_cache()?;
    eprintln!("\nVMSS '{name}' restarted.");
    Ok(())
}

/// `vmss stop` — POST .../powerOff with LRO.
pub async fn stop(
    resource_group: &str,
    name: &str,
    instance_ids: Option<&[String]>,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/powerOff?api-version={API_VERSION}",
        base_path(resource_group, name)
    );

    let body = instance_ids.map(|ids| json!({ "instanceIds": ids }));
    eprintln!("Stopping VMSS '{name}'...");
    cmd.post_lro(&path, body.as_ref()).await?;
    cmd.save_cache()?;
    eprintln!("\nVMSS '{name}' stopped.");
    Ok(())
}

/// `vmss reimage` — POST .../reimage with LRO.
pub async fn reimage(
    resource_group: &str,
    name: &str,
    instance_ids: Option<&[String]>,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/reimage?api-version={API_VERSION}",
        base_path(resource_group, name)
    );

    let body = instance_ids.map(|ids| json!({ "instanceIds": ids }));
    eprintln!("Reimaging VMSS '{name}'...");
    cmd.post_lro(&path, body.as_ref()).await?;
    cmd.save_cache()?;
    eprintln!("\nVMSS '{name}' reimaged.");
    Ok(())
}

/// `vmss get-instance-view` — GET with `$expand=instanceView`.
pub async fn get_instance_view(
    resource_group: &str,
    name: &str,
) -> Result<Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}?api-version={API_VERSION}&$expand=instanceView",
        base_path(resource_group, name)
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `vmss update-instances` — POST .../manualupgrade with instance IDs.
pub async fn update_instances(
    resource_group: &str,
    name: &str,
    instance_ids: &[String],
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/manualupgrade?api-version={API_VERSION}",
        base_path(resource_group, name)
    );

    let body = json!({ "instanceIds": instance_ids });
    eprintln!("Updating instances for VMSS '{name}'...");
    cmd.post_lro(&path, Some(&body)).await?;
    cmd.save_cache()?;
    eprintln!("\nVMSS '{name}' instances updated.");
    Ok(())
}

/// `vmss list-instance-connection-info` — list instance NICs to extract IPs/ports.
pub async fn list_instance_connection_info(
    resource_group: &str,
    name: &str,
) -> Result<Value> {
    let mut cmd = ArmCommand::new()?;

    // List VMSS VM instances
    let instances_path = format!(
        "{}/virtualMachines?api-version={API_VERSION}",
        base_path(resource_group, name)
    );
    let instances = cmd.list(&instances_path).await?;

    // List public IPs for the VMSS
    let pips_path = format!(
        "{}/publicipaddresses?api-version={API_VERSION}",
        base_path(resource_group, name)
    );
    let pips = cmd.list(&pips_path).await?;

    // Build a map: instance ID → connection info
    let mut info = Vec::new();
    for instance in &instances {
        let instance_id = instance.get("instanceId").and_then(|v| v.as_str()).unwrap_or("");
        let vm_name = instance.get("name").and_then(|v| v.as_str()).unwrap_or("");

        // Find a public IP for this instance
        let ip = pips.iter().find_map(|pip| {
            let pip_id = pip.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if pip_id.contains(&format!("/{instance_id}/")) || pip_id.contains(&format!("/{vm_name}/")) {
                pip.pointer("/properties/ipAddress").and_then(|v| v.as_str())
            } else {
                None
            }
        });

        info.push(json!({
            "instanceId": instance_id,
            "name": vm_name,
            "connectionInfo": ip.map(|addr| format!("{addr}:22")).unwrap_or_default(),
        }));
    }

    cmd.save_cache()?;
    Ok(json!(info))
}

/// `vmss list-instance-public-ips` — GET public IPs for all instances.
pub async fn list_instance_public_ips(
    resource_group: &str,
    name: &str,
) -> Result<Vec<Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/publicipaddresses?api-version={API_VERSION}",
        base_path(resource_group, name)
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `vmss identity assign` — add managed identity to VMSS.
pub async fn identity_assign(
    resource_group: &str,
    name: &str,
    system_assigned: bool,
    user_assigned: Option<&[String]>,
) -> Result<Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}?api-version={API_VERSION}", base_path(resource_group, name));

    let mut body = cmd.get(&path).await?;

    let existing_type = body
        .pointer("/identity/type")
        .and_then(|v| v.as_str())
        .unwrap_or("None")
        .to_string();

    let has_system = existing_type.contains("SystemAssigned");
    let has_user = existing_type.contains("UserAssigned");

    let want_system = system_assigned || has_system;
    let want_user = user_assigned.is_some() || has_user;

    let identity_type = match (want_system, want_user) {
        (true, true) => "SystemAssigned, UserAssigned",
        (true, false) => "SystemAssigned",
        (false, true) => "UserAssigned",
        (false, false) => "None",
    };

    if body.get("identity").is_none() {
        body["identity"] = json!({});
    }
    body["identity"]["type"] = json!(identity_type);

    if let Some(ids) = user_assigned {
        let ua = body["identity"]
            .as_object_mut()
            .unwrap()
            .entry("userAssignedIdentities")
            .or_insert_with(|| json!({}));
        for id in ids {
            ua[id] = json!({});
        }
    }

    eprintln!("Assigning identity to VMSS '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!("\nIdentity assigned to VMSS '{name}'.");
    Ok(result)
}

/// `vmss identity remove` — remove managed identity from VMSS.
pub async fn identity_remove(
    resource_group: &str,
    name: &str,
    system_assigned: bool,
    user_assigned: Option<&[String]>,
) -> Result<Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}?api-version={API_VERSION}", base_path(resource_group, name));

    let mut body = cmd.get(&path).await?;

    let existing_type = body
        .pointer("/identity/type")
        .and_then(|v| v.as_str())
        .unwrap_or("None")
        .to_string();

    let mut has_system = existing_type.contains("SystemAssigned");
    let mut has_user = existing_type.contains("UserAssigned");

    if system_assigned {
        has_system = false;
    }

    if let Some(ids) = user_assigned {
        if let Some(ua) = body.pointer_mut("/identity/userAssignedIdentities") {
            if let Some(obj) = ua.as_object_mut() {
                for id in ids {
                    obj.remove(id);
                }
                if obj.is_empty() {
                    has_user = false;
                }
            }
        }
    }

    let identity_type = match (has_system, has_user) {
        (true, true) => "SystemAssigned, UserAssigned",
        (true, false) => "SystemAssigned",
        (false, true) => "UserAssigned",
        (false, false) => "None",
    };
    body["identity"]["type"] = json!(identity_type);

    if !has_user {
        if let Some(obj) = body.pointer_mut("/identity") {
            if let Some(map) = obj.as_object_mut() {
                map.remove("userAssignedIdentities");
            }
        }
    }

    eprintln!("Removing identity from VMSS '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!("\nIdentity removed from VMSS '{name}'.");
    Ok(result)
}

/// `vmss set-orchestration-service-state` — POST .../setOrchestrationServiceState.
pub async fn set_orchestration_service_state(
    resource_group: &str,
    name: &str,
    service_name: &str,
    action: &str,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "{}/setOrchestrationServiceState?api-version={API_VERSION}",
        base_path(resource_group, name)
    );

    let body = json!({
        "serviceName": service_name,
        "action": action,
    });

    eprintln!("Setting orchestration service state for VMSS '{name}'...");
    cmd.post_lro(&path, Some(&body)).await?;
    cmd.save_cache()?;
    eprintln!("\nOrchestration service state updated for VMSS '{name}'.");
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Parse an image reference string into an ARM imageReference object.
///
/// Accepts either an URN (`publisher:offer:sku:version`) or a resource ID
/// (starting with `/`).
fn parse_image_reference(image: &str) -> Value {
    if image.starts_with('/') {
        // Image resource ID
        json!({ "id": image })
    } else {
        let parts: Vec<&str> = image.splitn(4, ':').collect();
        match parts.len() {
            4 => json!({
                "publisher": parts[0],
                "offer": parts[1],
                "sku": parts[2],
                "version": parts[3],
            }),
            _ => json!({
                "publisher": "Canonical",
                "offer": "0001-com-ubuntu-server-jammy",
                "sku": "22_04-lts-gen2",
                "version": "latest",
            }),
        }
    }
}

/// Set a nested value in a JSON object using dot-separated path.
///
/// E.g. `set_nested_value(&mut v, "sku.capacity", "5")` sets `v["sku"]["capacity"] = 5`.
fn set_nested_value(root: &mut Value, path: &str, val: &str) {
    let keys: Vec<&str> = path.split('.').collect();
    let mut current = root;

    for key in &keys[..keys.len() - 1] {
        if current.get(*key).is_none() {
            current[*key] = json!({});
        }
        current = current.get_mut(*key).unwrap();
    }

    let last = keys[keys.len() - 1];

    // Try to parse as number or bool, else keep as string
    let parsed: Value = if let Ok(n) = val.parse::<i64>() {
        json!(n)
    } else if let Ok(f) = val.parse::<f64>() {
        json!(f)
    } else if val == "true" {
        json!(true)
    } else if val == "false" {
        json!(false)
    } else if val == "null" {
        Value::Null
    } else {
        json!(val)
    };

    current[last] = parsed;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image_reference_urn() {
        let img = parse_image_reference("Canonical:0001-com-ubuntu-server-jammy:22_04-lts-gen2:latest");
        assert_eq!(img["publisher"], "Canonical");
        assert_eq!(img["offer"], "0001-com-ubuntu-server-jammy");
        assert_eq!(img["sku"], "22_04-lts-gen2");
        assert_eq!(img["version"], "latest");
    }

    #[test]
    fn test_parse_image_reference_id() {
        let img = parse_image_reference("/subscriptions/sub/resourceGroups/rg/providers/Microsoft.Compute/images/myImage");
        assert_eq!(img["id"], "/subscriptions/sub/resourceGroups/rg/providers/Microsoft.Compute/images/myImage");
    }

    #[test]
    fn test_parse_image_reference_fallback() {
        let img = parse_image_reference("ubuntu");
        assert_eq!(img["publisher"], "Canonical");
    }

    #[test]
    fn test_set_nested_value() {
        let mut v = json!({"sku": {"name": "Standard_DS1_v2"}});
        set_nested_value(&mut v, "sku.capacity", "5");
        assert_eq!(v["sku"]["capacity"], 5);
    }

    #[test]
    fn test_set_nested_value_deep() {
        let mut v = json!({});
        set_nested_value(&mut v, "a.b.c", "hello");
        assert_eq!(v["a"]["b"]["c"], "hello");
    }

    #[test]
    fn test_set_nested_value_bool() {
        let mut v = json!({});
        set_nested_value(&mut v, "enabled", "true");
        assert_eq!(v["enabled"], true);
    }
}
