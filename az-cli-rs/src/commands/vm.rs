use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2024-07-01";

pub async fn list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = if let Some(rg) = resource_group {
        format!("/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Compute/virtualMachines?api-version={API_VERSION}")
    } else {
        format!("/subscriptions/{{subscriptionId}}/providers/Microsoft.Compute/virtualMachines?api-version={API_VERSION}")
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

pub async fn show(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Compute/virtualMachines/{name}?api-version={API_VERSION}&$expand=instanceView");
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

pub async fn start(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Compute/virtualMachines/{name}/start?api-version={API_VERSION}");
    eprintln!("Starting VM '{name}'...");
    cmd.post_lro(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("\nVM '{name}' started.");
    Ok(())
}

pub async fn stop(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Compute/virtualMachines/{name}/powerOff?api-version={API_VERSION}");
    eprintln!("Stopping VM '{name}'...");
    cmd.post_lro(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("\nVM '{name}' stopped.");
    Ok(())
}

pub async fn restart(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Compute/virtualMachines/{name}/restart?api-version={API_VERSION}");
    eprintln!("Restarting VM '{name}'...");
    cmd.post_lro(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("\nVM '{name}' restarted.");
    Ok(())
}

pub async fn deallocate(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Compute/virtualMachines/{name}/deallocate?api-version={API_VERSION}");
    eprintln!("Deallocating VM '{name}'...");
    cmd.post_lro(&path, None).await?;
    cmd.save_cache()?;
    eprintln!("\nVM '{name}' deallocated.");
    Ok(())
}
