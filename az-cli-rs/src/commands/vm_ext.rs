/// Hand-written VM commands — `vm create`, `vm update`, `vm resize`, `vm open-port`,
/// `vm list-ip-addresses`, `vm get-instance-view`, `vm auto-shutdown`, `vm install-patches`,
/// `vm disk attach/detach`, `vm identity assign/remove`, `vm user update/delete/reset-ssh`,
/// `vm nic add/remove/set/list`, `vm image list/list-offers/list-publishers/list-skus/accept-terms`,
/// `vm encryption enable/disable`.
///
/// ARM API: Microsoft.Compute (api-version 2024-07-01)
///
/// NOTE: CLI wiring is pending — these functions are not yet dispatched from cli.rs/main.rs.
/// They will be integrated when the architecture supports merging generated + hand-written
/// commands under the same `vm` prefix.
use super::ArmCommand;
use crate::commands::group::parse_tags;
use crate::error::{AzrsError, Result};

const API_VERSION: &str = "2024-07-01";
const NETWORK_API_VERSION: &str = "2024-07-01";
const DEVTESTLAB_API_VERSION: &str = "2018-09-15";
const MARKETPLACE_API_VERSION: &str = "2021-01-01";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn vm_path(resource_group: &str, name: &str) -> String {
    format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}\
         /providers/Microsoft.Compute/virtualMachines/{name}?api-version={API_VERSION}"
    )
}

/// Parse an image URN `publisher:offer:sku:version` into an imageReference object.
fn parse_image_urn(urn: &str) -> Result<serde_json::Value> {
    let parts: Vec<&str> = urn.split(':').collect();
    if parts.len() != 4 {
        return Err(AzrsError::General(format!(
            "Invalid image URN '{urn}'. Expected format: publisher:offer:sku:version"
        )));
    }
    Ok(serde_json::json!({
        "publisher": parts[0],
        "offer": parts[1],
        "sku": parts[2],
        "version": parts[3],
    }))
}

/// Extract the resource group from an ARM resource ID.
fn resource_group_from_id(id: &str) -> Option<String> {
    let lower = id.to_lowercase();
    let idx = lower.find("/resourcegroups/")?;
    let after = &id[idx + "/resourcegroups/".len()..];
    Some(after.split('/').next()?.to_string())
}

// ---------------------------------------------------------------------------
// vm create
// ---------------------------------------------------------------------------

/// `azrs vm create -g <rg> -n <name> --image <urn> -l <location> [options]`
///
/// Simplified VM creation: builds the VM JSON body and PUTs with LRO.
/// Creates a default NIC (named `{name}VMNic`) in a default VNet/Subnet if
/// no `--nics` argument is supplied.
#[allow(clippy::too_many_arguments)]
pub async fn create(
    resource_group: &str,
    name: &str,
    image: &str,
    location: &str,
    size: Option<&str>,
    admin_username: Option<&str>,
    admin_password: Option<&str>,
    ssh_key_values: Option<&[String]>,
    generate_ssh_keys: bool,
    os_type: Option<&str>,
    tags: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let vm_size = size.unwrap_or("Standard_DS1_v2");
    let admin_user = admin_username.unwrap_or("azureuser");
    let image_ref = parse_image_urn(image)?;

    // Determine OS type from image or explicit flag
    let is_linux = match os_type {
        Some(t) => t.eq_ignore_ascii_case("linux"),
        None => !image.to_lowercase().contains("windows"),
    };

    // --- Create default networking resources ---
    let nic_name = format!("{name}VMNic");
    let vnet_name = format!("{name}VNET");
    let subnet_name = format!("{name}Subnet");
    let ip_name = format!("{name}PublicIP");

    // 1. Public IP
    let pip_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}\
         /providers/Microsoft.Network/publicIPAddresses/{ip_name}?api-version={NETWORK_API_VERSION}"
    );
    let pip_body = serde_json::json!({
        "location": location,
        "properties": { "publicIPAllocationMethod": "Dynamic" }
    });
    eprintln!("Creating public IP address '{ip_name}'...");
    cmd.put_lro(&pip_path, &pip_body).await?;

    // 2. VNet + Subnet
    let vnet_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}\
         /providers/Microsoft.Network/virtualNetworks/{vnet_name}?api-version={NETWORK_API_VERSION}"
    );
    let vnet_body = serde_json::json!({
        "location": location,
        "properties": {
            "addressSpace": { "addressPrefixes": ["10.0.0.0/16"] },
            "subnets": [{
                "name": subnet_name,
                "properties": { "addressPrefix": "10.0.0.0/24" }
            }]
        }
    });
    eprintln!("Creating virtual network '{vnet_name}'...");
    cmd.put_lro(&vnet_path, &vnet_body).await?;

    let sub_id = cmd.subscription_id()?.to_string();
    let subnet_id = format!(
        "/subscriptions/{sub_id}/resourceGroups/{resource_group}\
         /providers/Microsoft.Network/virtualNetworks/{vnet_name}/subnets/{subnet_name}"
    );
    let pip_id = format!(
        "/subscriptions/{sub_id}/resourceGroups/{resource_group}\
         /providers/Microsoft.Network/publicIPAddresses/{ip_name}"
    );

    // 3. NIC
    let nic_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}\
         /providers/Microsoft.Network/networkInterfaces/{nic_name}?api-version={NETWORK_API_VERSION}"
    );
    let nic_body = serde_json::json!({
        "location": location,
        "properties": {
            "ipConfigurations": [{
                "name": "ipconfig1",
                "properties": {
                    "subnet": { "id": subnet_id },
                    "publicIPAddress": { "id": pip_id },
                    "privateIPAllocationMethod": "Dynamic"
                }
            }]
        }
    });
    eprintln!("Creating network interface '{nic_name}'...");
    cmd.put_lro(&nic_path, &nic_body).await?;

    let nic_id = format!(
        "/subscriptions/{sub_id}/resourceGroups/{resource_group}\
         /providers/Microsoft.Network/networkInterfaces/{nic_name}"
    );

    // --- Build VM body ---
    let os_disk = serde_json::json!({
        "createOption": "FromImage",
        "managedDisk": { "storageAccountType": "Premium_LRS" }
    });

    let mut os_profile = serde_json::json!({
        "computerName": name,
        "adminUsername": admin_user,
    });

    if let Some(password) = admin_password {
        os_profile["adminPassword"] = serde_json::Value::String(password.to_string());
    }

    // SSH keys
    if is_linux {
        let mut ssh_keys_json = Vec::new();
        if let Some(keys) = ssh_key_values {
            for key in keys {
                ssh_keys_json.push(serde_json::json!({
                    "path": format!("/home/{admin_user}/.ssh/authorized_keys"),
                    "keyData": key,
                }));
            }
        }
        if generate_ssh_keys && ssh_keys_json.is_empty() {
            // Read default SSH public key
            if let Some(home) = dirs::home_dir() {
                let pub_key_path = home.join(".ssh").join("id_rsa.pub");
                if pub_key_path.exists() {
                    let key_data = std::fs::read_to_string(&pub_key_path)?;
                    ssh_keys_json.push(serde_json::json!({
                        "path": format!("/home/{admin_user}/.ssh/authorized_keys"),
                        "keyData": key_data.trim(),
                    }));
                }
            }
        }
        if !ssh_keys_json.is_empty() {
            os_profile["linuxConfiguration"] = serde_json::json!({
                "disablePasswordAuthentication": admin_password.is_none(),
                "ssh": { "publicKeys": ssh_keys_json },
            });
        }
    }

    let mut vm_body = serde_json::json!({
        "location": location,
        "properties": {
            "hardwareProfile": { "vmSize": vm_size },
            "storageProfile": {
                "imageReference": image_ref,
                "osDisk": os_disk,
            },
            "osProfile": os_profile,
            "networkProfile": {
                "networkInterfaces": [{
                    "id": nic_id,
                    "properties": { "primary": true }
                }]
            }
        }
    });

    if let Some(tag_list) = tags {
        vm_body["tags"] = serde_json::to_value(parse_tags(tag_list))?;
    }

    let path = vm_path(resource_group, name);
    eprintln!("Creating VM '{name}'...");
    let result = cmd.put_lro(&path, &vm_body).await?;
    cmd.save_cache()?;
    eprintln!("\nVM '{name}' created.");
    Ok(result)
}

// ---------------------------------------------------------------------------
// vm update
// ---------------------------------------------------------------------------

/// `azrs vm update -g <rg> -n <name> --set key=value ...`
///
/// GET the VM, merge the `--set` key=value pairs into the JSON, PUT back.
pub async fn update(
    resource_group: &str,
    name: &str,
    set_values: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = vm_path(resource_group, name);

    let mut vm = cmd.get(&path).await?;

    for kv in set_values {
        let (key, value) = kv.split_once('=').ok_or_else(|| {
            AzrsError::General(format!("Invalid --set format '{kv}'. Expected key=value"))
        })?;
        set_json_path(&mut vm, key, value);
    }

    eprintln!("Updating VM '{name}'...");
    let result = cmd.put_lro(&path, &vm).await?;
    cmd.save_cache()?;
    eprintln!("\nVM '{name}' updated.");
    Ok(result)
}

/// Set a dotted path in a JSON value (e.g. "properties.hardwareProfile.vmSize" = "Standard_D2s_v3").
fn set_json_path(root: &mut serde_json::Value, path: &str, value: &str) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = root;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Try to parse as JSON, fall back to string
            let parsed = serde_json::from_str(value)
                .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));
            current[part] = parsed;
        } else {
            if !current[part].is_object() {
                current[part] = serde_json::json!({});
            }
            current = &mut current[part];
        }
    }
}

// ---------------------------------------------------------------------------
// vm get-instance-view
// ---------------------------------------------------------------------------

/// `azrs vm get-instance-view -g <rg> -n <name>`
pub async fn get_instance_view(
    resource_group: &str,
    name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}\
         /providers/Microsoft.Compute/virtualMachines/{name}\
         ?api-version={API_VERSION}&$expand=instanceView"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

// ---------------------------------------------------------------------------
// vm resize
// ---------------------------------------------------------------------------

/// `azrs vm resize -g <rg> -n <name> --size <size>`
pub async fn resize(
    resource_group: &str,
    name: &str,
    size: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = vm_path(resource_group, name);

    let mut vm = cmd.get(&path).await?;
    vm["properties"]["hardwareProfile"]["vmSize"] = serde_json::Value::String(size.to_string());

    eprintln!("Resizing VM '{name}' to {size}...");
    let result = cmd.put_lro(&path, &vm).await?;
    cmd.save_cache()?;
    eprintln!("\nVM '{name}' resized.");
    Ok(result)
}

// ---------------------------------------------------------------------------
// vm open-port
// ---------------------------------------------------------------------------

/// `azrs vm open-port -g <rg> -n <name> --port <port> [--priority <priority>]`
///
/// Creates or updates an NSG rule on the VM's first NIC's NSG to allow inbound traffic.
pub async fn open_port(
    resource_group: &str,
    name: &str,
    port: &str,
    priority: Option<u32>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = vm_path(resource_group, name);
    let vm = cmd.get(&path).await?;

    // Extract first NIC ID
    let nic_id = vm
        .pointer("/properties/networkProfile/networkInterfaces/0/id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AzrsError::General("VM has no network interfaces".into()))?;

    // GET the NIC
    let nic_rg = resource_group_from_id(nic_id)
        .unwrap_or_else(|| resource_group.to_string());
    let nic_name = nic_id.rsplit('/').next().unwrap_or("unknown");
    let nic_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{nic_rg}\
         /providers/Microsoft.Network/networkInterfaces/{nic_name}\
         ?api-version={NETWORK_API_VERSION}"
    );
    let nic = cmd.get(&nic_path).await?;

    // Check if NIC has an NSG
    let nsg_id = nic
        .pointer("/properties/networkSecurityGroup/id")
        .and_then(|v| v.as_str());

    let nsg_name;
    let nsg_rg;
    if let Some(id) = nsg_id {
        nsg_name = id.rsplit('/').next().unwrap_or("unknown").to_string();
        nsg_rg = resource_group_from_id(id).unwrap_or_else(|| resource_group.to_string());
    } else {
        // Create a new NSG and attach to NIC
        nsg_name = format!("{name}NSG");
        nsg_rg = resource_group.to_string();
        let location = vm
            .get("location")
            .and_then(|v| v.as_str())
            .unwrap_or("eastus");
        let nsg_create_path = format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{nsg_rg}\
             /providers/Microsoft.Network/networkSecurityGroups/{nsg_name}\
             ?api-version={NETWORK_API_VERSION}"
        );
        let nsg_body = serde_json::json!({ "location": location });
        eprintln!("Creating NSG '{nsg_name}'...");
        cmd.put_lro(&nsg_create_path, &nsg_body).await?;

        // Attach NSG to NIC
        let sub_id = cmd.subscription_id()?.to_string();
        let new_nsg_id = format!(
            "/subscriptions/{sub_id}/resourceGroups/{nsg_rg}\
             /providers/Microsoft.Network/networkSecurityGroups/{nsg_name}"
        );
        let mut nic_update = nic.clone();
        nic_update["properties"]["networkSecurityGroup"] =
            serde_json::json!({ "id": new_nsg_id });
        eprintln!("Attaching NSG to NIC...");
        cmd.put_lro(&nic_path, &nic_update).await?;
    }

    // Create security rule
    let rule_name = format!("open-port-{port}");
    let rule_priority = priority.unwrap_or(900);
    let rule_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{nsg_rg}\
         /providers/Microsoft.Network/networkSecurityGroups/{nsg_name}\
         /securityRules/{rule_name}?api-version={NETWORK_API_VERSION}"
    );
    let rule_body = serde_json::json!({
        "properties": {
            "protocol": "*",
            "sourceAddressPrefix": "*",
            "destinationAddressPrefix": "*",
            "sourcePortRange": "*",
            "destinationPortRange": port,
            "access": "Allow",
            "priority": rule_priority,
            "direction": "Inbound"
        }
    });
    eprintln!("Creating security rule '{rule_name}'...");
    let result = cmd.put_lro(&rule_path, &rule_body).await?;
    cmd.save_cache()?;
    eprintln!("\nPort {port} opened on VM '{name}'.");
    Ok(result)
}

// ---------------------------------------------------------------------------
// vm auto-shutdown
// ---------------------------------------------------------------------------

/// `azrs vm auto-shutdown -g <rg> -n <name> --time <HH:MM> [--timezone <tz>]`
///
/// Configures DevTestLab auto-shutdown schedule.
/// Use `--off` to disable.
pub async fn auto_shutdown(
    resource_group: &str,
    name: &str,
    time: Option<&str>,
    timezone: Option<&str>,
    off: bool,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let sub_id = cmd.subscription_id()?.to_string();
    let vm_id = format!(
        "/subscriptions/{sub_id}/resourceGroups/{resource_group}\
         /providers/Microsoft.Compute/virtualMachines/{name}"
    );
    let schedule_name = format!("shutdown-computevm-{name}");
    let schedule_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}\
         /providers/Microsoft.DevTestLab/schedules/{schedule_name}\
         ?api-version={DEVTESTLAB_API_VERSION}"
    );

    if off {
        let body = serde_json::json!({
            "properties": {
                "status": "Disabled",
                "taskType": "ComputeVmShutdownTask",
                "targetResourceId": vm_id,
            }
        });
        let result = cmd.put(&schedule_path, &body).await?;
        cmd.save_cache()?;
        eprintln!("Auto-shutdown disabled for VM '{name}'.");
        return Ok(result);
    }

    let shutdown_time = time.ok_or_else(|| {
        AzrsError::General("--time is required (format HH:MM, e.g. 19:00)".into())
    })?;
    // Normalize "HH:MM" → "HHMM"
    let daily_recurrence = shutdown_time.replace(':', "");
    let tz = timezone.unwrap_or("UTC");

    // GET VM to find its location
    let vm_path_str = vm_path(resource_group, name);
    let vm = cmd.get(&vm_path_str).await?;
    let location = vm
        .get("location")
        .and_then(|v| v.as_str())
        .unwrap_or("eastus");

    let body = serde_json::json!({
        "location": location,
        "properties": {
            "status": "Enabled",
            "taskType": "ComputeVmShutdownTask",
            "dailyRecurrence": { "time": daily_recurrence },
            "timeZoneId": tz,
            "targetResourceId": vm_id,
        }
    });

    eprintln!("Configuring auto-shutdown for VM '{name}' at {shutdown_time} {tz}...");
    let result = cmd.put(&schedule_path, &body).await?;
    cmd.save_cache()?;
    eprintln!("Auto-shutdown configured.");
    Ok(result)
}

// ---------------------------------------------------------------------------
// vm install-patches
// ---------------------------------------------------------------------------

/// `azrs vm install-patches -g <rg> -n <name> --maximum-duration <dur> --reboot-setting <setting>`
pub async fn install_patches(
    resource_group: &str,
    name: &str,
    maximum_duration: &str,
    reboot_setting: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}\
         /providers/Microsoft.Compute/virtualMachines/{name}\
         /installPatches?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "maximumDuration": maximum_duration,
        "rebootSetting": reboot_setting,
    });
    eprintln!("Installing patches on VM '{name}'...");
    // installPatches is a POST LRO that returns a result body; use request + poll manually
    let resp = cmd.request("POST", &path, Some(&body)).await?;
    if !resp.is_success() && resp.status != 202 {
        return Err(AzrsError::General(format!(
            "HTTP {}: {}",
            resp.status,
            resp.text()
        )));
    }
    // For 202, wait via post_lro pattern then GET the result
    if resp.status == 202 {
        cmd.post_lro(&path, Some(&body)).await?;
    }
    cmd.save_cache()?;
    eprintln!("\nPatch installation completed on VM '{name}'.");
    // Return the VM's instance view to show patch status
    get_instance_view(resource_group, name).await
}

// ---------------------------------------------------------------------------
// vm list-ip-addresses
// ---------------------------------------------------------------------------

/// `azrs vm list-ip-addresses -g <rg> -n <name>`
///
/// Returns public and private IP addresses for the VM.
pub async fn list_ip_addresses(
    resource_group: &str,
    name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = vm_path(resource_group, name);
    let vm = cmd.get(&path).await?;

    let nics = vm
        .pointer("/properties/networkProfile/networkInterfaces")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut ip_entries = Vec::new();

    for nic_ref in &nics {
        let nic_id = match nic_ref.get("id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => continue,
        };
        let nic_rg =
            resource_group_from_id(nic_id).unwrap_or_else(|| resource_group.to_string());
        let nic_name = nic_id.rsplit('/').next().unwrap_or("unknown");
        let nic_path = format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{nic_rg}\
             /providers/Microsoft.Network/networkInterfaces/{nic_name}\
             ?api-version={NETWORK_API_VERSION}"
        );
        let nic = cmd.get(&nic_path).await?;

        let ip_configs = nic
            .pointer("/properties/ipConfigurations")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        for ip_config in &ip_configs {
            let private_ip = ip_config
                .pointer("/properties/privateIPAddress")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let mut public_ip = String::new();
            if let Some(pip_id) = ip_config
                .pointer("/properties/publicIPAddress/id")
                .and_then(|v| v.as_str())
            {
                let pip_rg = resource_group_from_id(pip_id)
                    .unwrap_or_else(|| resource_group.to_string());
                let pip_name = pip_id.rsplit('/').next().unwrap_or("unknown");
                let pip_path = format!(
                    "/subscriptions/{{subscriptionId}}/resourceGroups/{pip_rg}\
                     /providers/Microsoft.Network/publicIPAddresses/{pip_name}\
                     ?api-version={NETWORK_API_VERSION}"
                );
                if let Ok(pip) = cmd.get(&pip_path).await {
                    public_ip = pip
                        .pointer("/properties/ipAddress")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                }
            }

            ip_entries.push(serde_json::json!({
                "virtualMachine": { "name": name, "resourceGroup": resource_group },
                "networkInterfaceName": nic_name,
                "privateIpAddress": private_ip,
                "publicIpAddress": public_ip,
            }));
        }
    }

    cmd.save_cache()?;
    Ok(serde_json::Value::Array(ip_entries))
}

// ---------------------------------------------------------------------------
// vm disk attach / detach
// ---------------------------------------------------------------------------

/// `azrs vm disk attach -g <rg> --vm-name <vm> --name <disk> [--lun <lun>] [--size-gb <size>]`
///
/// Attaches a managed data disk to the VM.
pub async fn disk_attach(
    resource_group: &str,
    vm_name: &str,
    disk_name: &str,
    lun: Option<i64>,
    size_gb: Option<i64>,
    new: bool,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = vm_path(resource_group, vm_name);
    let mut vm = cmd.get(&path).await?;

    let data_disks = vm
        .pointer_mut("/properties/storageProfile/dataDisks")
        .and_then(|v| v.as_array_mut());

    let existing_luns: Vec<i64> = if let Some(disks) = &data_disks {
        disks
            .iter()
            .filter_map(|d| d.get("lun").and_then(|l| l.as_i64()))
            .collect()
    } else {
        vec![]
    };

    let actual_lun = lun.unwrap_or_else(|| {
        // Auto-assign next available LUN
        let mut l = 0i64;
        while existing_luns.contains(&l) {
            l += 1;
        }
        l
    });

    let new_disk = if new {
        // Create a new empty managed disk inline
        serde_json::json!({
            "lun": actual_lun,
            "name": disk_name,
            "createOption": "Empty",
            "diskSizeGB": size_gb.unwrap_or(32),
            "managedDisk": { "storageAccountType": "Premium_LRS" }
        })
    } else {
        // Attach existing managed disk by name
        let sub_id = cmd.subscription_id()?.to_string();
        let disk_id = format!(
            "/subscriptions/{sub_id}/resourceGroups/{resource_group}\
             /providers/Microsoft.Compute/disks/{disk_name}"
        );
        serde_json::json!({
            "lun": actual_lun,
            "name": disk_name,
            "createOption": "Attach",
            "managedDisk": { "id": disk_id }
        })
    };

    // Ensure dataDisks array exists and append
    if !vm["properties"]["storageProfile"]["dataDisks"].is_array() {
        vm["properties"]["storageProfile"]["dataDisks"] = serde_json::json!([]);
    }
    vm["properties"]["storageProfile"]["dataDisks"]
        .as_array_mut()
        .unwrap()
        .push(new_disk);

    eprintln!("Attaching disk '{disk_name}' to VM '{vm_name}' at LUN {actual_lun}...");
    let result = cmd.put_lro(&path, &vm).await?;
    cmd.save_cache()?;
    eprintln!("\nDisk attached.");
    Ok(result)
}

/// `azrs vm disk detach -g <rg> --vm-name <vm> --name <disk>`
///
/// Detaches a data disk from the VM.
pub async fn disk_detach(
    resource_group: &str,
    vm_name: &str,
    disk_name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = vm_path(resource_group, vm_name);
    let mut vm = cmd.get(&path).await?;

    let data_disks = vm
        .pointer_mut("/properties/storageProfile/dataDisks")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| AzrsError::General("VM has no data disks".into()))?;

    let orig_len = data_disks.len();
    data_disks.retain(|d| {
        d.get("name").and_then(|n| n.as_str()) != Some(disk_name)
    });

    if data_disks.len() == orig_len {
        return Err(AzrsError::General(format!(
            "Disk '{disk_name}' not found on VM '{vm_name}'"
        )));
    }

    eprintln!("Detaching disk '{disk_name}' from VM '{vm_name}'...");
    let result = cmd.put_lro(&path, &vm).await?;
    cmd.save_cache()?;
    eprintln!("\nDisk detached.");
    Ok(result)
}

// ---------------------------------------------------------------------------
// vm identity assign / remove
// ---------------------------------------------------------------------------

/// `azrs vm identity assign -g <rg> -n <name> [--scope <scope>] [--identities <id>...]`
///
/// Assigns system or user-assigned managed identity to the VM.
pub async fn identity_assign(
    resource_group: &str,
    name: &str,
    identities: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = vm_path(resource_group, name);
    let mut vm = cmd.get(&path).await?;

    if let Some(user_ids) = identities {
        if user_ids.is_empty() || (user_ids.len() == 1 && user_ids[0] == "[system]") {
            // SystemAssigned
            vm["identity"] = serde_json::json!({ "type": "SystemAssigned" });
        } else {
            // UserAssigned (possibly also SystemAssigned)
            let current_type = vm
                .pointer("/identity/type")
                .and_then(|v| v.as_str())
                .unwrap_or("None");

            let new_type = if current_type.contains("SystemAssigned") {
                "SystemAssigned, UserAssigned"
            } else {
                "UserAssigned"
            };

            let mut user_assigned = vm
                .pointer("/identity/userAssignedIdentities")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));

            for id in user_ids {
                user_assigned[id] = serde_json::json!({});
            }

            vm["identity"] = serde_json::json!({
                "type": new_type,
                "userAssignedIdentities": user_assigned,
            });
        }
    } else {
        // Default: SystemAssigned
        vm["identity"] = serde_json::json!({ "type": "SystemAssigned" });
    }

    eprintln!("Assigning identity to VM '{name}'...");
    let result = cmd.put_lro(&path, &vm).await?;
    cmd.save_cache()?;
    eprintln!("\nIdentity assigned.");
    Ok(result)
}

/// `azrs vm identity remove -g <rg> -n <name> [--identities <id>...]`
///
/// Removes managed identity from the VM.
pub async fn identity_remove(
    resource_group: &str,
    name: &str,
    identities: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = vm_path(resource_group, name);
    let mut vm = cmd.get(&path).await?;

    if let Some(user_ids) = identities {
        if user_ids.len() == 1 && user_ids[0] == "[system]" {
            // Remove SystemAssigned only
            let current_type = vm
                .pointer("/identity/type")
                .and_then(|v| v.as_str())
                .unwrap_or("None");
            if current_type.contains("UserAssigned") {
                vm["identity"]["type"] =
                    serde_json::Value::String("UserAssigned".to_string());
            } else {
                vm["identity"] = serde_json::json!({ "type": "None" });
            }
        } else {
            // Remove specific user-assigned identities
            if let Some(ua) = vm
                .pointer_mut("/identity/userAssignedIdentities")
                .and_then(|v| v.as_object_mut())
            {
                for id in user_ids {
                    ua.remove(id);
                }
                if ua.is_empty() {
                    let current_type = vm
                        .pointer("/identity/type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("None");
                    if current_type.contains("SystemAssigned") {
                        vm["identity"]["type"] =
                            serde_json::Value::String("SystemAssigned".to_string());
                    } else {
                        vm["identity"] = serde_json::json!({ "type": "None" });
                    }
                }
            }
        }
    } else {
        // Remove all identities
        vm["identity"] = serde_json::json!({ "type": "None" });
    }

    eprintln!("Removing identity from VM '{name}'...");
    let result = cmd.put_lro(&path, &vm).await?;
    cmd.save_cache()?;
    eprintln!("\nIdentity removed.");
    Ok(result)
}

// ---------------------------------------------------------------------------
// vm user update / delete / reset-ssh
// ---------------------------------------------------------------------------

/// `azrs vm user update -g <rg> -n <name> -u <username> [-p <password>]`
///
/// Updates a user account on a Linux VM via VMAccessForLinux extension.
pub async fn user_update(
    resource_group: &str,
    vm_name: &str,
    username: &str,
    password: Option<&str>,
    ssh_key_value: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let mut protected_settings = serde_json::json!({ "username": username });
    if let Some(pw) = password {
        protected_settings["password"] = serde_json::Value::String(pw.to_string());
    }
    if let Some(key) = ssh_key_value {
        protected_settings["ssh_key"] = serde_json::Value::String(key.to_string());
    }

    let ext_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}\
         /providers/Microsoft.Compute/virtualMachines/{vm_name}\
         /extensions/VMAccessForLinux?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "location": "",
        "properties": {
            "publisher": "Microsoft.OSTCExtensions",
            "type": "VMAccessForLinux",
            "typeHandlerVersion": "1.5",
            "autoUpgradeMinorVersion": true,
            "protectedSettings": protected_settings,
        }
    });

    // Need VM location
    let vm_path_str = vm_path(resource_group, vm_name);
    let vm = cmd.get(&vm_path_str).await?;
    let location = vm
        .get("location")
        .and_then(|v| v.as_str())
        .unwrap_or("eastus");

    let mut body = body;
    body["location"] = serde_json::Value::String(location.to_string());

    eprintln!("Updating user '{username}' on VM '{vm_name}'...");
    let result = cmd.put_lro(&ext_path, &body).await?;
    cmd.save_cache()?;
    eprintln!("\nUser updated.");
    Ok(result)
}

/// `azrs vm user delete -g <rg> -n <name> -u <username>`
pub async fn user_delete(
    resource_group: &str,
    vm_name: &str,
    username: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let vm_path_str = vm_path(resource_group, vm_name);
    let vm = cmd.get(&vm_path_str).await?;
    let location = vm
        .get("location")
        .and_then(|v| v.as_str())
        .unwrap_or("eastus");

    let ext_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}\
         /providers/Microsoft.Compute/virtualMachines/{vm_name}\
         /extensions/VMAccessForLinux?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "location": location,
        "properties": {
            "publisher": "Microsoft.OSTCExtensions",
            "type": "VMAccessForLinux",
            "typeHandlerVersion": "1.5",
            "autoUpgradeMinorVersion": true,
            "protectedSettings": {
                "remove_user": username,
            },
        }
    });

    eprintln!("Deleting user '{username}' from VM '{vm_name}'...");
    let result = cmd.put_lro(&ext_path, &body).await?;
    cmd.save_cache()?;
    eprintln!("\nUser deleted.");
    Ok(result)
}

/// `azrs vm user reset-ssh -g <rg> -n <name>`
///
/// Resets SSH configuration on the VM.
pub async fn user_reset_ssh(
    resource_group: &str,
    vm_name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let vm_path_str = vm_path(resource_group, vm_name);
    let vm = cmd.get(&vm_path_str).await?;
    let location = vm
        .get("location")
        .and_then(|v| v.as_str())
        .unwrap_or("eastus");

    let ext_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}\
         /providers/Microsoft.Compute/virtualMachines/{vm_name}\
         /extensions/VMAccessForLinux?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "location": location,
        "properties": {
            "publisher": "Microsoft.OSTCExtensions",
            "type": "VMAccessForLinux",
            "typeHandlerVersion": "1.5",
            "autoUpgradeMinorVersion": true,
            "protectedSettings": {
                "reset_ssh": true,
            },
        }
    });

    eprintln!("Resetting SSH configuration on VM '{vm_name}'...");
    let result = cmd.put_lro(&ext_path, &body).await?;
    cmd.save_cache()?;
    eprintln!("\nSSH configuration reset.");
    Ok(result)
}

// ---------------------------------------------------------------------------
// vm nic add / remove / set / list
// ---------------------------------------------------------------------------

/// `azrs vm nic add -g <rg> --vm-name <vm> --nics <nic_id>...`
pub async fn nic_add(
    resource_group: &str,
    vm_name: &str,
    nic_ids: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = vm_path(resource_group, vm_name);
    let mut vm = cmd.get(&path).await?;

    let nics = vm
        .pointer_mut("/properties/networkProfile/networkInterfaces")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| AzrsError::General("VM has no network profile".into()))?;

    for nic_id in nic_ids {
        nics.push(serde_json::json!({
            "id": nic_id,
            "properties": { "primary": false }
        }));
    }

    eprintln!("Adding NIC(s) to VM '{vm_name}'...");
    let result = cmd.put_lro(&path, &vm).await?;
    cmd.save_cache()?;
    eprintln!("\nNIC(s) added.");
    Ok(result)
}

/// `azrs vm nic remove -g <rg> --vm-name <vm> --nics <nic_id>...`
pub async fn nic_remove(
    resource_group: &str,
    vm_name: &str,
    nic_ids: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = vm_path(resource_group, vm_name);
    let mut vm = cmd.get(&path).await?;

    let nics = vm
        .pointer_mut("/properties/networkProfile/networkInterfaces")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| AzrsError::General("VM has no network profile".into()))?;

    let orig_len = nics.len();
    let remove_set: std::collections::HashSet<&str> =
        nic_ids.iter().map(|s| s.as_str()).collect();
    nics.retain(|n| {
        n.get("id")
            .and_then(|v| v.as_str())
            .map(|id| !remove_set.contains(id))
            .unwrap_or(true)
    });

    if nics.len() == orig_len {
        return Err(AzrsError::General(
            "None of the specified NICs were found on the VM".into(),
        ));
    }

    eprintln!("Removing NIC(s) from VM '{vm_name}'...");
    let result = cmd.put_lro(&path, &vm).await?;
    cmd.save_cache()?;
    eprintln!("\nNIC(s) removed.");
    Ok(result)
}

/// `azrs vm nic set -g <rg> --vm-name <vm> --nics <nic_id>... [--primary-nic <nic_id>]`
///
/// Replaces all NICs on the VM.
pub async fn nic_set(
    resource_group: &str,
    vm_name: &str,
    nic_ids: &[String],
    primary_nic: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = vm_path(resource_group, vm_name);
    let mut vm = cmd.get(&path).await?;

    let new_nics: Vec<serde_json::Value> = nic_ids
        .iter()
        .map(|id| {
            let is_primary = primary_nic.map(|p| p == id).unwrap_or(false)
                || (primary_nic.is_none() && id == &nic_ids[0]);
            serde_json::json!({
                "id": id,
                "properties": { "primary": is_primary }
            })
        })
        .collect();

    vm["properties"]["networkProfile"]["networkInterfaces"] =
        serde_json::Value::Array(new_nics);

    eprintln!("Setting NIC(s) on VM '{vm_name}'...");
    let result = cmd.put_lro(&path, &vm).await?;
    cmd.save_cache()?;
    eprintln!("\nNIC(s) set.");
    Ok(result)
}

/// `azrs vm nic list -g <rg> --vm-name <vm>`
pub async fn nic_list(
    resource_group: &str,
    vm_name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = vm_path(resource_group, vm_name);
    let vm = cmd.get(&path).await?;
    cmd.save_cache()?;

    let nics = vm
        .pointer("/properties/networkProfile/networkInterfaces")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    Ok(nics)
}

// ---------------------------------------------------------------------------
// vm image list / list-offers / list-publishers / list-skus / accept-terms
// ---------------------------------------------------------------------------

/// `azrs vm image list -l <location> -p <publisher> -f <offer> -s <sku>`
///
/// Lists VM image versions.
pub async fn image_list(
    location: &str,
    publisher: &str,
    offer: &str,
    sku: &str,
) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/providers/Microsoft.Compute\
         /locations/{location}/publishers/{publisher}\
         /artifacttypes/vmimage/offers/{offer}/skus/{sku}/versions\
         ?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs vm image list-offers -l <location> -p <publisher>`
pub async fn image_list_offers(
    location: &str,
    publisher: &str,
) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/providers/Microsoft.Compute\
         /locations/{location}/publishers/{publisher}\
         /artifacttypes/vmimage/offers?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs vm image list-publishers -l <location>`
pub async fn image_list_publishers(location: &str) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/providers/Microsoft.Compute\
         /locations/{location}/publishers?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs vm image list-skus -l <location> -p <publisher> -f <offer>`
pub async fn image_list_skus(
    location: &str,
    publisher: &str,
    offer: &str,
) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/providers/Microsoft.Compute\
         /locations/{location}/publishers/{publisher}\
         /artifacttypes/vmimage/offers/{offer}/skus?api-version={API_VERSION}"
    );
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs vm image accept-terms -p <publisher> -f <offer> --plan <plan>`
///
/// Accepts Marketplace terms for a VM image.
pub async fn image_accept_terms(
    publisher: &str,
    offer: &str,
    plan: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    // GET current agreement
    let path = format!(
        "/subscriptions/{{subscriptionId}}/providers/Microsoft.MarketplaceOrdering\
         /offerTypes/virtualmachine/publishers/{publisher}\
         /offers/{offer}/plans/{plan}/agreements/current\
         ?api-version={MARKETPLACE_API_VERSION}"
    );
    let mut agreement = cmd.get(&path).await?;

    // Set accepted = true and PUT back
    agreement["properties"]["accepted"] = serde_json::Value::Bool(true);
    let result = cmd.put(&path, &agreement).await?;
    cmd.save_cache()?;
    eprintln!("Terms accepted for {publisher}:{offer}:{plan}.");
    Ok(result)
}

// ---------------------------------------------------------------------------
// vm encryption enable / disable
// ---------------------------------------------------------------------------

/// `azrs vm encryption enable -g <rg> -n <name> --disk-encryption-keyvault <vault_url>
///   [--volume-type <OS|Data|All>]`
///
/// Enables Azure Disk Encryption via the AzureDiskEncryption extension.
pub async fn encryption_enable(
    resource_group: &str,
    vm_name: &str,
    disk_encryption_keyvault: &str,
    volume_type: Option<&str>,
    key_encryption_key: Option<&str>,
    key_encryption_algorithm: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    // Get VM location
    let vm_path_str = vm_path(resource_group, vm_name);
    let vm = cmd.get(&vm_path_str).await?;
    let location = vm
        .get("location")
        .and_then(|v| v.as_str())
        .unwrap_or("eastus");

    let vol_type = volume_type.unwrap_or("All");

    let mut settings = serde_json::json!({
        "KeyVaultURL": disk_encryption_keyvault,
        "VolumeType": vol_type,
        "EncryptionOperation": "EnableEncryption",
    });

    // Extract keyvault resource ID from the URL if possible
    // The user may pass the keyvault URL or resource ID
    if disk_encryption_keyvault.starts_with('/') {
        settings["KeyVaultResourceId"] =
            serde_json::Value::String(disk_encryption_keyvault.to_string());
    }

    if let Some(kek) = key_encryption_key {
        settings["KeyEncryptionKeyURL"] = serde_json::Value::String(kek.to_string());
    }
    if let Some(algo) = key_encryption_algorithm {
        settings["KeyEncryptionAlgorithm"] = serde_json::Value::String(algo.to_string());
    }

    let ext_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}\
         /providers/Microsoft.Compute/virtualMachines/{vm_name}\
         /extensions/AzureDiskEncryption?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "location": location,
        "properties": {
            "publisher": "Microsoft.Azure.Security",
            "type": "AzureDiskEncryption",
            "typeHandlerVersion": "2.2",
            "autoUpgradeMinorVersion": true,
            "settings": settings,
        }
    });

    eprintln!("Enabling disk encryption on VM '{vm_name}'...");
    let result = cmd.put_lro(&ext_path, &body).await?;
    cmd.save_cache()?;
    eprintln!("\nDisk encryption enabled.");
    Ok(result)
}

/// `azrs vm encryption disable -g <rg> -n <name> [--volume-type <OS|Data|All>]`
pub async fn encryption_disable(
    resource_group: &str,
    vm_name: &str,
    volume_type: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    let vm_path_str = vm_path(resource_group, vm_name);
    let vm = cmd.get(&vm_path_str).await?;
    let location = vm
        .get("location")
        .and_then(|v| v.as_str())
        .unwrap_or("eastus");

    let vol_type = volume_type.unwrap_or("All");

    let ext_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}\
         /providers/Microsoft.Compute/virtualMachines/{vm_name}\
         /extensions/AzureDiskEncryption?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "location": location,
        "properties": {
            "publisher": "Microsoft.Azure.Security",
            "type": "AzureDiskEncryption",
            "typeHandlerVersion": "2.2",
            "autoUpgradeMinorVersion": true,
            "settings": {
                "VolumeType": vol_type,
                "EncryptionOperation": "DisableEncryption",
            },
        }
    });

    eprintln!("Disabling disk encryption on VM '{vm_name}'...");
    let result = cmd.put_lro(&ext_path, &body).await?;
    cmd.save_cache()?;
    eprintln!("\nDisk encryption disabled.");
    Ok(result)
}
