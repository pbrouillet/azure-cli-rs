/// Tag commands — `azrs tag list/create/delete/update/add-value/remove-value`.
///
/// ARM API: Microsoft.Resources/tags (api-version 2024-03-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2024-03-01";

/// `azrs tag list`
pub async fn list() -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/tagNames?api-version={API_VERSION}");
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs tag create --resource-id <id> --tags key=value ...` (at scope)
/// Or `azrs tag create --name <name>` (legacy tag name creation)
pub async fn create(
    resource_id: Option<&str>,
    tags: Option<&[String]>,
    name: Option<&str>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    if let Some(id) = resource_id {
        // Tags at scope — PUT
        let tag_map = crate::commands::group::parse_tags(tags.unwrap_or(&[]));
        let path = format!("{id}/providers/Microsoft.Resources/tags/default?api-version={API_VERSION}");
        let body = serde_json::json!({
            "properties": {
                "tags": tag_map
            }
        });
        let result = cmd.put(&path, &body).await?;
        cmd.save_cache()?;
        Ok(result)
    } else if let Some(tag_name) = name {
        // Legacy: create a tag name in the subscription
        let path = format!("/subscriptions/{{subscriptionId}}/tagNames/{tag_name}?api-version={API_VERSION}");
        let body = serde_json::json!({});
        let result = cmd.put(&path, &body).await?;
        cmd.save_cache()?;
        Ok(result)
    } else {
        Err(crate::error::AzrsError::General(
            "Either --resource-id or --name is required".into(),
        ))
    }
}

/// `azrs tag delete --resource-id <id>` or `azrs tag delete --name <name>`
pub async fn delete(resource_id: Option<&str>, name: Option<&str>) -> Result<()> {
    let mut cmd = ArmCommand::new()?;

    if let Some(id) = resource_id {
        let path = format!("{id}/providers/Microsoft.Resources/tags/default?api-version={API_VERSION}");
        cmd.delete(&path).await?;
    } else if let Some(tag_name) = name {
        let path = format!("/subscriptions/{{subscriptionId}}/tagNames/{tag_name}?api-version={API_VERSION}");
        cmd.delete(&path).await?;
    } else {
        return Err(crate::error::AzrsError::General(
            "Either --resource-id or --name is required".into(),
        ));
    }

    cmd.save_cache()?;
    Ok(())
}

/// `azrs tag update --resource-id <id> --operation <merge|replace|delete> --tags key=value ...`
pub async fn update(
    resource_id: &str,
    operation: &str,
    tags: &[String],
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let tag_map = crate::commands::group::parse_tags(tags);
    let path = format!("{resource_id}/providers/Microsoft.Resources/tags/default?api-version={API_VERSION}");

    let body = serde_json::json!({
        "operation": operation,
        "properties": {
            "tags": tag_map
        }
    });

    let resp = cmd.request("PATCH", &path, Some(&body)).await?;
    cmd.save_cache()?;

    if !resp.is_success() {
        return Err(crate::error::AzrsError::General(
            format!("HTTP {}: {}", resp.status, resp.text()),
        ));
    }
    Ok(serde_json::from_str(&resp.text())?)
}

/// `azrs tag add-value --name <tag-name> --value <value>`
pub async fn add_value(tag_name: &str, value: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/tagNames/{tag_name}/tagValues/{value}?api-version={API_VERSION}"
    );
    let body = serde_json::json!({});
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs tag remove-value --name <tag-name> --value <value>`
pub async fn remove_value(tag_name: &str, value: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/tagNames/{tag_name}/tagValues/{value}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}
