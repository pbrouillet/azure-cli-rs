/// Azure cloud endpoint configuration.
///
/// Mirrors the cloud definitions from Python azure-cli cloud.py:364-469.

#[derive(Debug, Clone)]
pub struct CloudConfig {
    /// Display name (e.g. "AzureCloud")
    pub environment_name: String,
    /// Microsoft Entra ID login endpoint (e.g. "https://login.microsoftonline.com")
    pub active_directory: String,
    /// ARM endpoint (e.g. "https://management.azure.com/")
    pub resource_manager: String,
    /// The OAuth2 resource ID for ARM (e.g. "https://management.core.windows.net/")
    pub active_directory_resource_id: String,
}

impl CloudConfig {
    #[allow(dead_code)]
    pub fn azure_public() -> Self {
        Self {
            environment_name: "AzureCloud".into(),
            active_directory: "https://login.microsoftonline.com".into(),
            resource_manager: "https://management.azure.com/".into(),
            active_directory_resource_id: "https://management.core.windows.net/".into(),
        }
    }

    #[allow(dead_code)]
    pub fn azure_china() -> Self {
        Self {
            environment_name: "AzureChinaCloud".into(),
            active_directory: "https://login.chinacloudapi.cn".into(),
            resource_manager: "https://management.chinacloudapi.cn".into(),
            active_directory_resource_id: "https://management.core.chinacloudapi.cn/".into(),
        }
    }

    #[allow(dead_code)]
    pub fn azure_us_government() -> Self {
        Self {
            environment_name: "AzureUSGovernment".into(),
            active_directory: "https://login.microsoftonline.us".into(),
            resource_manager: "https://management.usgovcloudapi.net/".into(),
            active_directory_resource_id: "https://management.core.usgovcloudapi.net/".into(),
        }
    }

    /// Default scope for ARM operations, derived from active_directory_resource_id.
    /// Matches Python: resource_to_scopes(cloud.endpoints.active_directory_resource_id)
    pub fn default_scope(&self) -> String {
        resource_to_scope(&self.active_directory_resource_id)
    }
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self::azure_public()
    }
}

/// Convert an Azure resource URL to an OAuth2 scope by appending `/.default`.
/// Matches Python auth/util.py:92-105: `scope = resource + '/.default'`
pub fn resource_to_scope(resource: &str) -> String {
    format!("{resource}/.default")
}

/// Convert an OAuth2 scope back to a resource URL by stripping `/.default`.
#[allow(dead_code)]
pub fn scope_to_resource(scope: &str) -> String {
    scope
        .strip_suffix("/.default")
        .unwrap_or(scope)
        .to_string()
}
