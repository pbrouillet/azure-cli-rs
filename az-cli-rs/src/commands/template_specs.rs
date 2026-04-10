/// Template Specs commands — `azrs ts create/update/export/list/delete`.
///
/// ARM API: Microsoft.Resources/templateSpecs (api-version 2022-02-01)
use super::ArmCommand;
use crate::error::{AzrsError, Result};

const API_VERSION: &str = "2022-02-01";

/// `azrs ts list [--resource-group <rg>]`
pub async fn list(resource_group: Option<&str>) -> Result<Vec<serde_json::Value>> {
    let mut cmd = ArmCommand::new()?;
    let path = match resource_group {
        Some(rg) => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{rg}/providers/Microsoft.Resources/templateSpecs?api-version={API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/providers/Microsoft.Resources/templateSpecs?api-version={API_VERSION}"
        ),
    };
    let results = cmd.list(&path).await?;
    cmd.save_cache()?;
    Ok(results)
}

/// `azrs ts show --resource-group <rg> --name <name> [--version <version>]`
pub async fn show(resource_group: &str, name: &str, version: Option<&str>) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = match version {
        Some(v) => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Resources/templateSpecs/{name}/versions/{v}?api-version={API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Resources/templateSpecs/{name}?api-version={API_VERSION}"
        ),
    };
    let result = cmd.get(&path).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs ts create --resource-group <rg> --name <name> --version <version> --template-file <file>`
pub async fn create(
    resource_group: &str,
    name: &str,
    version: &str,
    template_file: &str,
    location: &str,
    description: Option<&str>,
    display_name: Option<&str>,
    tags: Option<&[String]>,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;

    // Ensure the template spec exists
    let spec_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Resources/templateSpecs/{name}?api-version={API_VERSION}"
    );
    let mut spec_body = serde_json::json!({
        "location": location,
        "properties": {}
    });
    if let Some(desc) = description {
        spec_body["properties"]["description"] = serde_json::Value::String(desc.to_string());
    }
    if let Some(dn) = display_name {
        spec_body["properties"]["displayName"] = serde_json::Value::String(dn.to_string());
    }
    if let Some(tag_list) = tags {
        spec_body["tags"] = serde_json::to_value(crate::commands::group::parse_tags(tag_list))?;
    }
    cmd.put(&spec_path, &spec_body).await?;

    // Create the version
    let content = std::fs::read_to_string(template_file)
        .map_err(|e| AzrsError::General(format!("Cannot read template file: {e}")))?;
    let template: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| AzrsError::General(format!("Invalid template JSON: {e}")))?;

    let ver_path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Resources/templateSpecs/{name}/versions/{version}?api-version={API_VERSION}"
    );
    let ver_body = serde_json::json!({
        "location": location,
        "properties": {
            "mainTemplate": template
        }
    });
    let result = cmd.put(&ver_path, &ver_body).await?;
    cmd.save_cache()?;
    Ok(result)
}

/// `azrs ts delete --resource-group <rg> --name <name> [--version <version>]`
pub async fn delete(resource_group: &str, name: &str, version: Option<&str>) -> Result<()> {
    let mut cmd = ArmCommand::new()?;
    let path = match version {
        Some(v) => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Resources/templateSpecs/{name}/versions/{v}?api-version={API_VERSION}"
        ),
        None => format!(
            "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Resources/templateSpecs/{name}?api-version={API_VERSION}"
        ),
    };
    cmd.delete(&path).await?;
    cmd.save_cache()?;
    Ok(())
}

/// `azrs ts export --resource-group <rg> --name <name> --version <version> --output-folder <dir>`
pub async fn export(
    resource_group: &str,
    name: &str,
    version: &str,
    output_folder: &str,
) -> Result<serde_json::Value> {
    let mut cmd = ArmCommand::new()?;
    let path = format!(
        "/subscriptions/{{subscriptionId}}/resourceGroups/{resource_group}/providers/Microsoft.Resources/templateSpecs/{name}/versions/{version}?api-version={API_VERSION}"
    );
    let result = cmd.get(&path).await?;

    // Write the template to the output folder
    std::fs::create_dir_all(output_folder)
        .map_err(|e| AzrsError::General(format!("Cannot create output folder: {e}")))?;

    if let Some(template) = result.pointer("/properties/mainTemplate") {
        let file_path = format!("{output_folder}/main.json");
        let content = serde_json::to_string_pretty(template)?;
        std::fs::write(&file_path, content)
            .map_err(|e| AzrsError::General(format!("Cannot write template: {e}")))?;
        eprintln!("Exported template to {file_path}");
    }

    cmd.save_cache()?;
    Ok(result)
}
