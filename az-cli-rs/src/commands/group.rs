/// Resource group commands — `azrs group create/list/show/delete/exists/update/export/wait`.
///
/// ARM API: Microsoft.Resources/resourceGroups (api-version 2024-03-01)
use super::ArmCommand;
use crate::error::Result;

const API_VERSION: &str = "2024-03-01";

/// Parse `key=value` tag pairs into a map.
pub fn parse_tags(tags: &[String]) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for tag in tags {
        if let Some((k, v)) = tag.split_once('=') {
            map.insert(k.to_string(), v.to_string());
        } else {
            map.insert(tag.to_string(), String::new());
        }
    }
    map
}

/// `azrs group create -n <name> -l <location> [--tags key=value ...]`
pub async fn create(name: &str, location: &str, tags: Option<&[String]>) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourcegroups/{name}?api-version={API_VERSION}");
    let mut body = serde_json::json!({ "location": location });
    if let Some(tag_list) = tags {
        body["tags"] = serde_json::to_value(parse_tags(tag_list))?;
    }
    let result = cmd.put(&path, &body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs group list [--tag key=value]`
pub async fn list(tag_filter: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let mut path = format!("/subscriptions/{{subscriptionId}}/resourcegroups?api-version={API_VERSION}");
    if let Some(tag) = tag_filter {
        path.push_str(&format!("&$filter={}", urlencoding_tag_filter(tag)));
    }
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs group show -n <name>`
pub async fn show(name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourcegroups/{name}?api-version={API_VERSION}");
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs group delete -n <name>`
pub async fn delete(name: &str) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourcegroups/{name}?api-version={API_VERSION}");
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    eprintln!("Resource group '{name}' deleted.");
    Ok(())
}

/// `azrs group exists -n <name>`
pub async fn exists(name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourcegroups/{name}?api-version={API_VERSION}");
    let result = cmd.exists(&path).await?;
    cmd.save_cache()?;
    Ok(serde_json::Value::Bool(result))
}

fn urlencoding_tag_filter(tag: &str) -> String {
    // ARM filter: tagName eq 'key' and tagValue eq 'value'
    if let Some((k, v)) = tag.split_once('=') {
        format!(
            "tagName%20eq%20'{}'%20and%20tagValue%20eq%20'{}'",
            k, v
        )
    } else {
        format!("tagName%20eq%20'{}'", tag)
    }
}

/// `azrs group update -n <name> [--tags key=value ...]`
pub async fn update(name: &str, tags: Option<&[String]>) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourcegroups/{name}?api-version={API_VERSION}");

    let mut body = serde_json::json!({});
    if let Some(tag_list) = tags {
        body["tags"] = serde_json::to_value(parse_tags(tag_list))?;
    }

    let resp = cmd.request("PATCH", &path, Some(&body)).await?;
    cmd.save_cache()?;

    if !resp.is_success() {
        return Err(crate::error::AzrsError::General(
            format!("HTTP {}: {}", resp.status, resp.text()),
        ));
    }
    Ok(serde_json::from_str(&resp.text())?)
}

/// `azrs group export -n <name>`
///
/// Exports the resource group as an ARM template.
pub async fn export(name: &str) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourcegroups/{name}/exportTemplate?api-version={API_VERSION}"
    );

    let body = serde_json::json!({
        "resources": ["*"],
        "options": "IncludeParameterDefaultValue,IncludeComments"
    });
    let result = cmd.post(&path, Some(&body)).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs group wait -n <name> --created|--updated|--deleted|--exists|--custom <condition>`
///
/// Polls the resource group until it reaches the desired provisioning state.
pub async fn wait(
    name: &str,
    created: bool,
    updated: bool,
    deleted: bool,
    exists_flag: bool,
    custom: Option<&str>,
    interval: u64,
    timeout: u64,
) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = format!("/subscriptions/{{subscriptionId}}/resourcegroups/{name}?api-version={API_VERSION}");
    let start = std::time::Instant::now();

    loop {
        if start.elapsed().as_secs() >= timeout {
            return Err(crate::error::AzrsError::General(
                format!("Timed out waiting for resource group '{name}'"),
            ));
        }

        let resp = cmd.request("GET", &path, None).await?;
        let status = resp.status;

        if deleted {
            if status == 404 {
                cmd.save_cache()?;
                return Ok(());
            }
        } else if exists_flag {
            if status == 200 {
                cmd.save_cache()?;
                return Ok(());
            }
        } else if created || updated {
            if status == 200 {
                let body: serde_json::Value = serde_json::from_str(&resp.text())?;
                let prov_state = body
                    .pointer("/properties/provisioningState")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if prov_state == "Succeeded" {
                    cmd.save_cache()?;
                    return Ok(());
                }
            }
        } else if let Some(jmespath) = custom {
            if status == 200 {
                let body: serde_json::Value = serde_json::from_str(&resp.text())?;
                // Simple custom condition: check if a JMESPath-ish dot-path is truthy
                let result = crate::output::jmespath_eval(&body, jmespath);
                if result_is_truthy(&result) {
                    cmd.save_cache()?;
                    return Ok(());
                }
            }
        }

        eprint!(".");
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
    }
}

fn result_is_truthy(val: &serde_json::Value) -> bool {
    match val {
        serde_json::Value::Null => false,
        serde_json::Value::Bool(b) => *b,
        serde_json::Value::String(s) => !s.is_empty(),
        serde_json::Value::Number(_) => true,
        serde_json::Value::Array(a) => !a.is_empty(),
        serde_json::Value::Object(o) => !o.is_empty(),
    }
}
