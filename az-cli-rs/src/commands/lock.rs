/// Lock commands — `azrs lock create/delete/list/update` and
/// `azrs account lock create/delete/list/update`.
///
/// ARM API: Microsoft.Authorization/locks (api-version 2020-05-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2020-05-01";

/// `azrs lock list [--resource-group <rg>]`
pub async fn list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = match resource_group {
        Some(rg) => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Authorization/locks?api-version={API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Authorization/locks?api-version={API_VERSION}"
        ),
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs lock create --name <name> --lock-type <CanNotDelete|ReadOnly> [--resource-group <rg>] [--notes <notes>]`
pub async fn create(
    name: &str,
    lock_type: &str,
    resource_group: Option<&str>,
    notes: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = match resource_group {
        Some(rg) => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Authorization/locks/{name}?api-version={API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Authorization/locks/{name}?api-version={API_VERSION}"
        ),
    };

    let mut body = serde_json::json!({
        "properties": {
            "level": lock_type
        }
    });
    if let Some(n) = notes {
        body["properties"]["notes"] = serde_json::Value::String(n.to_string());
    }

    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs lock delete --name <name> [--resource-group <rg>]`
pub async fn delete(name: &str, resource_group: Option<&str>) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = match resource_group {
        Some(rg) => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Authorization/locks/{name}?api-version={API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Authorization/locks/{name}?api-version={API_VERSION}"
        ),
    };
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `azrs lock update --name <name> [--resource-group <rg>] [--lock-type <type>] [--notes <notes>]`
pub async fn update(
    name: &str,
    resource_group: Option<&str>,
    lock_type: Option<&str>,
    notes: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = match resource_group {
        Some(rg) => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Authorization/locks/{name}?api-version={API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Authorization/locks/{name}?api-version={API_VERSION}"
        ),
    };

    // GET current, merge, PUT
    let mut current = cmd.get(&path).await?;
    if let Some(lt) = lock_type {
        current["properties"]["level"] = serde_json::Value::String(lt.to_string());
    }
    if let Some(n) = notes {
        current["properties"]["notes"] = serde_json::Value::String(n.to_string());
    }

    let result = cmd.put(&path, &current).await?;
    cmd.save_cache()?;
    Ok(result)
}
