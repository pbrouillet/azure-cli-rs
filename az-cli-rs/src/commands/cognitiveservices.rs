/// Azure Cognitive Services account commands — `azrs cognitiveservices account create/list/show/delete/update/keys`.
///
/// ARM API: Microsoft.CognitiveServices/accounts (api-version 2023-05-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2023-05-01";

/// `azrs cognitiveservices account create -n <name> -g <rg> -l <location> --kind <kind> --sku <sku> [--yes]`
pub async fn account_create(
    name: &str,
    resource_group: &str,
    location: &str,
    kind: &str,
    sku: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.CognitiveServices/accounts/{name}?api-version={API_VERSION}"
    );
    let body = serde_json::json!({
        "location": location,
        "kind": kind,
        "sku": { "name": sku },
        "properties": {},
    });
    eprintln!("Creating Cognitive Services account '{name}'...");
    let result = cmd.put_lro(&path, &body).await?;
    cmd.save_cache()?;
    eprintln!();
    Ok(result)
}

/// `azrs cognitiveservices account list [-g <rg>]` — list in a resource group, or across the subscription.
pub async fn account_list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = if let Some(rg) = resource_group {
        format!("/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.CognitiveServices/accounts?api-version={API_VERSION}")
    } else {
        format!("/subscriptions/{{subscriptionId}}/providers/Microsoft.CognitiveServices/accounts?api-version={API_VERSION}")
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs cognitiveservices account show -n <name> -g <rg>`
pub async fn account_show(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.CognitiveServices/accounts/{name}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs cognitiveservices account delete -n <name> -g <rg>`
pub async fn account_delete(name: &str, resource_group: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.CognitiveServices/accounts/{name}?api-version={API_VERSION}"
    );
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    eprintln!("Cognitive Services account '{name}' deleted.");
    Ok(())
}

/// `azrs cognitiveservices account update -n <name> -g <rg> [--set key=value ...] [--tags key=value ...]`
///
/// Applies a PATCH with tag updates and/or arbitrary `properties.*` values (dot-path `--set`).
pub async fn account_update(
    name: &str,
    resource_group: &str,
    tags: Option<&[String]>,
    set_props: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.CognitiveServices/accounts/{name}?api-version={API_VERSION}"
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

/// `azrs cognitiveservices account keys list -n <name> -g <rg>`
pub async fn account_keys_list(name: &str, resource_group: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.CognitiveServices/accounts/{name}/listKeys?api-version={API_VERSION}"
    );
    let result = cmd.post(&path, None).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs cognitiveservices account keys regenerate -n <name> -g <rg> --key-name <Key1|Key2>`
pub async fn account_keys_regenerate(
    name: &str,
    resource_group: &str,
    key_name: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.CognitiveServices/accounts/{name}/regenerateKey?api-version={API_VERSION}"
    );
    let body = serde_json::json!({ "keyName": key_name });
    let result = cmd.post(&path, Some(&body)).await?;
    cmd.save_cache()?;
    Ok(result)
}

// TODO: Add deployment and commitment-plan subgroups.

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
