/// Deployment stack commands — `azrs stack group/sub/mg list/delete/export`.
///
/// ARM API: Microsoft.Resources/deploymentStacks (api-version 2024-03-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2024-03-01";

/// Stack scope
pub enum StackScope<'a> {
    ResourceGroup(&'a str),
    Subscription,
    ManagementGroup(&'a str),
}

impl<'a> StackScope<'a> {
    fn base_path(&self) -> String {
        match self {
            StackScope::ResourceGroup(rg) => format!(
                "/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Resources/deploymentStacks"
            ),
            StackScope::Subscription => {
                "/subscriptions/{subscriptionId}/providers/Microsoft.Resources/deploymentStacks".to_string()
            }
            StackScope::ManagementGroup(mg) => format!(
                "/providers/Microsoft.Management/managementGroups/{mg}/providers/Microsoft.Resources/deploymentStacks"
            ),
        }
    }
}

/// `stack list`
pub async fn list(scope: StackScope<'_>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}?api-version={API_VERSION}", scope.base_path());
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `stack show`
pub async fn show(scope: StackScope<'_>, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}?api-version={API_VERSION}", scope.base_path(), name);
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `stack delete`
pub async fn delete(scope: StackScope<'_>, name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}?api-version={API_VERSION}", scope.base_path(), name);
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `stack export`
pub async fn export(scope: StackScope<'_>, name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("{}/{}/exportTemplate?api-version={API_VERSION}", scope.base_path(), name);
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}
