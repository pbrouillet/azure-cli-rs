use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "azrs", about = "Azure CLI (Rust)", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub global: GlobalArgs,
}

/// Global options applied to all commands.
#[derive(clap::Args)]
pub struct GlobalArgs {
    /// Output format
    #[arg(short, long, global = true, default_value = "json",
          value_enum)]
    pub output: OutputFormat,

    /// JMESPath query string (see http://jmespath.org/)
    #[arg(long, global = true)]
    pub query: Option<String>,

    /// Subscription ID or name to use (overrides active subscription)
    #[arg(long, global = true)]
    pub subscription: Option<String>,

    /// Increase logging verbosity to show HTTP requests and responses
    #[arg(long, global = true)]
    pub debug: bool,

    /// Increase logging verbosity (less than --debug)
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Only show errors, suppressing warnings and info messages
    #[arg(long, global = true)]
    pub only_show_errors: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Json,
    Jsonc,
    Table,
    Tsv,
    Yaml,
    Yamlc,
    None,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Log in to Azure
    Login(LoginArgs),

    /// Log out to remove access to Azure subscriptions
    Logout(LogoutArgs),

    /// Manage Azure subscription information
    #[command(subcommand)]
    Account(AccountCommands),

    /// Invoke a custom request to Azure REST API
    Rest(RestArgs),

    /// Manage CLI configuration
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Generate shell completions
    Completions(CompletionsArgs),

    /// Find Azure CLI commands
    Find(FindArgs),

    /// Configure Azure CLI settings interactively
    Configure,

    /// Manage registered Azure clouds
    #[command(subcommand)]
    Cloud(CloudCommands),

    /// Manage resource groups
    #[command(subcommand)]
    Group(GroupCommands),

    /// Manage Azure resources
    #[command(subcommand)]
    Resource(ResourceCommands),

    /// Manage resource providers
    #[command(subcommand)]
    Provider(ProviderCommands),

    /// Manage resource features
    #[command(subcommand)]
    Feature(FeatureCommands),

    /// Manage resource tags
    #[command(subcommand)]
    Tag(TagCommands),

    /// Manage subscription-level locks
    #[command(subcommand)]
    Lock(LockCommands),

    /// Manage ARM template deployments
    #[command(subcommand)]
    Deployment(DeploymentCommands),

    /// Manage deployment scripts
    #[command(subcommand, name = "deployment-scripts")]
    DeploymentScripts(DeploymentScriptsCommands),

    /// Manage template specs
    #[command(subcommand)]
    Ts(TsCommands),

    /// Manage deployment stacks
    #[command(subcommand)]
    Stack(StackCommands),

    /// Manage managed applications
    #[command(subcommand)]
    Managedapp(ManagedappCommands),

    /// Manage web apps
    #[command(subcommand)]
    Webapp(WebappCommands),

    /// Manage function apps
    #[command(subcommand)]
    Functionapp(FunctionappCommands),

    /// Manage App Service resources
    #[command(subcommand)]
    Appservice(AppserviceCommands),

    /// Manage Azure Static Web Apps
    #[command(subcommand)]
    Staticwebapp(StaticwebappCommands),

    /// Manage Logic App (Standard) resources
    #[command(subcommand)]
    Logicapp(LogicappCommands),

    /// Manage role-based access control (RBAC)
    #[command(subcommand)]
    Role(RoleCommands),

    /// Manage storage accounts
    #[command(subcommand)]
    Storage(StorageCommands),

    /// Manage Azure Cache for Redis
    #[command(subcommand)]
    Redis(RedisCommands),

    /// Manage Azure Container Registries
    #[command(subcommand)]
    Acr(AcrCommands),

    /// Manage App Configuration stores
    #[command(subcommand)]
    Appconfig(AppconfigCommands),

    /// Manage Azure SignalR Service
    #[command(subcommand)]
    Signalr(SignalrCommands),

    /// Manage Azure Maps accounts
    #[command(subcommand)]
    Maps(MapsCommands),

    /// Manage Cognitive Services accounts
    #[command(subcommand)]
    Cognitiveservices(CognitiveservicesCommands),

    /// Manage Azure Event Grid resources
    #[command(subcommand)]
    Eventgrid(EventgridCommands),

    /// Manage Key Vault resources
    #[command(subcommand)]
    Keyvault(KeyvaultCommands),

    /// Manage virtual machines
    #[command(subcommand)]
    Vm(VmCommands),

    /// Manage virtual machine scale sets
    #[command(subcommand)]
    Vmss(VmssCommands),

    /// Auto-generated commands from Azure CLI AAZ definitions
    #[command(flatten)]
    Generated(crate::generated::GeneratedCommands),
}

#[derive(clap::Args)]
pub struct LoginArgs {
    /// Use device code flow instead of browser-based interactive login
    #[arg(long)]
    pub use_device_code: bool,

    /// Tenant ID or domain to authenticate against
    #[arg(short, long)]
    pub tenant: Option<String>,

    /// Space-separated scopes for the access token
    #[arg(long, num_args = 1..)]
    pub scope: Option<Vec<String>>,

    /// Allow login to succeed even if no subscriptions are found
    #[arg(long)]
    pub allow_no_subscriptions: bool,

    /// Log in with a service principal
    #[arg(long)]
    pub service_principal: bool,

    /// Service principal client ID (app ID) or username
    #[arg(short, long)]
    pub username: Option<String>,

    /// Service principal secret or user password
    #[arg(short, long)]
    pub password: Option<String>,

    /// PEM certificate file path for service principal certificate auth
    #[arg(long)]
    pub certificate: Option<String>,

    /// Certificate password (for PFX/PKCS#12 files)
    #[arg(long)]
    pub certificate_password: Option<String>,

    /// Log in using a managed identity (system-assigned or user-assigned)
    #[arg(long, name = "identity")]
    pub use_identity: bool,

    /// Client ID of the user-assigned managed identity
    #[arg(long)]
    pub client_id: Option<String>,

    /// Object ID of the user-assigned managed identity
    #[arg(long)]
    pub object_id: Option<String>,

    /// Resource ID of the user-assigned managed identity
    #[arg(long)]
    pub resource_id: Option<String>,
}

#[derive(clap::Args)]
pub struct LogoutArgs {
    /// Username of the account to log out
    #[arg(short, long)]
    pub username: Option<String>,
}

#[derive(Subcommand)]
pub enum AccountCommands {
    /// Show the details of the active subscription
    Show,

    /// List all subscriptions for the logged-in account
    List(AccountListArgs),

    /// Set a subscription to be the current active subscription
    Set(AccountSetArgs),

    /// Get an access token for the active subscription
    #[command(name = "get-access-token")]
    GetAccessToken(GetAccessTokenArgs),

    /// Manage management groups
    #[command(subcommand, name = "management-group")]
    ManagementGroup(ManagementGroupCommands),

    /// List locations for the current subscription
    #[command(name = "list-locations")]
    ListLocations,
}

#[derive(clap::Args)]
pub struct AccountListArgs {
    /// List all subscriptions across all tenants (not just the active one)
    #[arg(long)]
    pub all: bool,
}

#[derive(clap::Args)]
pub struct AccountSetArgs {
    /// Subscription ID or name
    #[arg(short, long)]
    pub subscription: String,
}

#[derive(clap::Args)]
pub struct GetAccessTokenArgs {
    /// Azure resource to get the token for (e.g. https://management.azure.com/)
    #[arg(long)]
    pub resource: Option<String>,

    /// Space-separated scopes
    #[arg(long, num_args = 1..)]
    pub scope: Option<Vec<String>>,

    /// Tenant ID for the token request
    #[arg(short, long)]
    pub tenant: Option<String>,

    /// Subscription to use (defaults to current active subscription)
    #[arg(short, long)]
    pub subscription: Option<String>,
}

#[derive(clap::Args)]
pub struct RestArgs {
    /// Request URL. If it doesn't start with a host, it is treated as an ARM resource ID
    /// and prefixed with the ARM endpoint of the current cloud.
    /// {subscriptionId} is replaced with the current subscription ID.
    #[arg(long, short, alias = "uri")]
    pub url: String,

    /// HTTP request method
    #[arg(long, short, default_value = "get",
          value_parser = ["head", "get", "put", "post", "delete", "options", "patch"])]
    pub method: String,

    /// Space-separated headers in KEY=VALUE format or a JSON string
    #[arg(long, num_args = 1..)]
    pub headers: Option<Vec<String>>,

    /// Query parameters in KEY=VALUE format or a JSON string
    #[arg(long, alias = "url-parameters", num_args = 1..)]
    pub uri_parameters: Option<Vec<String>>,

    /// Request body. Use @{file} to load from a file.
    #[arg(long, short)]
    pub body: Option<String>,

    /// Do not auto-append Authorization header
    #[arg(long)]
    pub skip_authorization_header: bool,

    /// Resource URL for which CLI should acquire a token. By default, derived from --url.
    #[arg(long)]
    pub resource: Option<String>,

    /// Save response payload to a file
    #[arg(long)]
    pub output_file: Option<String>,
}

#[derive(Subcommand)]
pub enum GroupCommands {
    /// Create a new resource group
    Create(GroupCreateArgs),

    /// List resource groups
    List(GroupListArgs),

    /// Show details of a resource group
    Show(GroupShowArgs),

    /// Delete a resource group
    Delete(GroupDeleteArgs),

    /// Check if a resource group exists
    Exists(GroupExistsArgs),

    /// Update a resource group
    Update(GroupUpdateArgs),

    /// Export a resource group as an ARM template
    Export(GroupExportArgs),

    /// Wait for a resource group to reach a desired state
    Wait(GroupWaitArgs),

    /// Manage resource group-level locks
    #[command(subcommand)]
    Lock(LockCommands),
}

#[derive(clap::Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}

#[derive(clap::Args)]
pub struct GroupCreateArgs {
    /// Name of the resource group
    #[arg(short, long)]
    pub name: String,

    /// Location (e.g. eastus, westeurope)
    #[arg(short, long)]
    pub location: String,

    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct GroupListArgs {
    /// Filter by tag (key=value)
    #[arg(long)]
    pub tag: Option<String>,
}

#[derive(clap::Args)]
pub struct GroupShowArgs {
    /// Name of the resource group
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct GroupDeleteArgs {
    /// Name of the resource group
    #[arg(short, long)]
    pub name: String,

    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct GroupExistsArgs {
    /// Name of the resource group
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct GroupUpdateArgs {
    /// Name of the resource group
    #[arg(short, long)]
    pub name: String,

    /// Space-separated tags: key=value (replaces all existing tags)
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct GroupExportArgs {
    /// Name of the resource group
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct GroupWaitArgs {
    /// Name of the resource group
    #[arg(short, long)]
    pub name: String,

    /// Wait until created with provisioningState 'Succeeded'
    #[arg(long)]
    pub created: bool,

    /// Wait until updated with provisioningState 'Succeeded'
    #[arg(long)]
    pub updated: bool,

    /// Wait until deleted (404)
    #[arg(long)]
    pub deleted: bool,

    /// Wait until the resource exists
    #[arg(long)]
    pub exists: bool,

    /// Wait until the condition satisfies a custom JMESPath query
    #[arg(long)]
    pub custom: Option<String>,

    /// Polling interval in seconds
    #[arg(long, default_value = "30")]
    pub interval: u64,

    /// Maximum wait in seconds
    #[arg(long, default_value = "3600")]
    pub timeout: u64,
}

// --- Resource ---

#[derive(Subcommand)]
pub enum ResourceCommands {
    /// List resources in a subscription or resource group
    List(ResourceListArgs),

    /// Show a resource by ID or name
    Show(ResourceShowArgs),

    /// Delete a resource
    Delete(ResourceDeleteArgs),

    /// Create a resource
    Create(ResourceCreateArgs),

    /// Update a resource (GET + merge + PUT)
    Update(ResourceUpdateArgs),

    /// Tag a resource
    Tag(ResourceTagArgs),

    /// Invoke an action on a resource
    #[command(name = "invoke-action")]
    InvokeAction(ResourceInvokeActionArgs),

    /// Wait for a resource to reach a desired state
    Wait(ResourceWaitArgs),

    /// Manage resource-level locks
    #[command(subcommand, name = "lock")]
    Lock(LockCommands),

    /// Manage resource links
    #[command(subcommand)]
    Link(ResourceLinkCommands),
}

#[derive(clap::Args)]
pub struct ResourceListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,

    /// Resource type (e.g. Microsoft.Compute/virtualMachines)
    #[arg(long)]
    pub resource_type: Option<String>,

    /// Filter by tag (key[=value])
    #[arg(long)]
    pub tag: Option<String>,

    /// Filter by resource name
    #[arg(short, long)]
    pub name: Option<String>,
}

#[derive(clap::Args)]
pub struct ResourceShowArgs {
    /// Full resource ID
    #[arg(long)]
    pub ids: Option<String>,

    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,

    /// Provider namespace (e.g. Microsoft.Compute)
    #[arg(long)]
    pub namespace: Option<String>,

    /// Resource type (e.g. virtualMachines)
    #[arg(long)]
    pub resource_type: Option<String>,

    /// Resource name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Parent resource path (e.g. virtualMachineScaleSets/myVmss)
    #[arg(long)]
    pub parent: Option<String>,

    /// API version of the resource
    #[arg(long, default_value = "2024-03-01")]
    pub api_version: String,
}

#[derive(clap::Args)]
pub struct ResourceDeleteArgs {
    /// Full resource ID
    #[arg(long)]
    pub ids: Option<String>,

    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,

    /// Provider namespace
    #[arg(long)]
    pub namespace: Option<String>,

    /// Resource type
    #[arg(long)]
    pub resource_type: Option<String>,

    /// Resource name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Parent resource path
    #[arg(long)]
    pub parent: Option<String>,

    /// API version of the resource
    #[arg(long, default_value = "2024-03-01")]
    pub api_version: String,

    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct ResourceCreateArgs {
    /// Full resource ID
    #[arg(long)]
    pub ids: Option<String>,

    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,

    /// Provider namespace
    #[arg(long)]
    pub namespace: Option<String>,

    /// Resource type
    #[arg(long)]
    pub resource_type: Option<String>,

    /// Resource name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Parent resource path
    #[arg(long)]
    pub parent: Option<String>,

    /// API version of the resource
    #[arg(long, default_value = "2024-03-01")]
    pub api_version: String,

    /// JSON-formatted resource properties string or @{file}
    #[arg(short, long, alias = "properties")]
    pub properties: String,

    /// Location
    #[arg(short, long)]
    pub location: Option<String>,

    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct ResourceUpdateArgs {
    /// Full resource ID
    #[arg(long)]
    pub ids: Option<String>,

    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,

    /// Provider namespace
    #[arg(long)]
    pub namespace: Option<String>,

    /// Resource type
    #[arg(long)]
    pub resource_type: Option<String>,

    /// Resource name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Parent resource path
    #[arg(long)]
    pub parent: Option<String>,

    /// API version of the resource
    #[arg(long, default_value = "2024-03-01")]
    pub api_version: String,

    /// Property key=value pairs to set (e.g. properties.sku.name=Standard)
    #[arg(long, num_args = 1..)]
    pub set: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct ResourceTagArgs {
    /// Full resource ID
    #[arg(long)]
    pub ids: Option<String>,

    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,

    /// Provider namespace
    #[arg(long)]
    pub namespace: Option<String>,

    /// Resource type
    #[arg(long)]
    pub resource_type: Option<String>,

    /// Resource name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Parent resource path
    #[arg(long)]
    pub parent: Option<String>,

    /// API version of the resource
    #[arg(long, default_value = "2024-03-01")]
    pub api_version: String,

    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Vec<String>,

    /// Incrementally add tags (merge, don't replace)
    #[arg(long, short)]
    pub incremental: bool,
}

#[derive(clap::Args)]
pub struct ResourceInvokeActionArgs {
    /// Full resource ID
    #[arg(long)]
    pub ids: Option<String>,

    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,

    /// Provider namespace
    #[arg(long)]
    pub namespace: Option<String>,

    /// Resource type
    #[arg(long)]
    pub resource_type: Option<String>,

    /// Resource name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Parent resource path
    #[arg(long)]
    pub parent: Option<String>,

    /// API version of the resource
    #[arg(long, default_value = "2024-03-01")]
    pub api_version: String,

    /// The action to invoke
    #[arg(long)]
    pub action: String,

    /// JSON request body for the action
    #[arg(long)]
    pub request_body: Option<String>,
}

#[derive(clap::Args)]
pub struct ResourceWaitArgs {
    /// Full resource ID
    #[arg(long)]
    pub ids: Option<String>,

    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,

    /// Provider namespace
    #[arg(long)]
    pub namespace: Option<String>,

    /// Resource type
    #[arg(long)]
    pub resource_type: Option<String>,

    /// Resource name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Parent resource path
    #[arg(long)]
    pub parent: Option<String>,

    /// API version of the resource
    #[arg(long, default_value = "2024-03-01")]
    pub api_version: String,

    /// Wait until created with provisioningState 'Succeeded'
    #[arg(long)]
    pub created: bool,

    /// Wait until updated with provisioningState 'Succeeded'
    #[arg(long)]
    pub updated: bool,

    /// Wait until deleted (404)
    #[arg(long)]
    pub deleted: bool,

    /// Wait until the resource exists
    #[arg(long)]
    pub exists: bool,

    /// Wait until the condition satisfies a custom JMESPath query
    #[arg(long)]
    pub custom: Option<String>,

    /// Polling interval in seconds
    #[arg(long, default_value = "30")]
    pub interval: u64,

    /// Maximum wait in seconds
    #[arg(long, default_value = "3600")]
    pub timeout: u64,
}

// --- Resource Link ---

#[derive(Subcommand)]
pub enum ResourceLinkCommands {
    /// List resource links
    List(ResourceLinkListArgs),
    /// Show a resource link
    Show(ResourceLinkShowArgs),
    /// Create a resource link
    Create(ResourceLinkCreateArgs),
    /// Delete a resource link
    Delete(ResourceLinkDeleteArgs),
    /// Update a resource link
    Update(ResourceLinkUpdateArgs),
}

#[derive(clap::Args)]
pub struct ResourceLinkListArgs {
    /// Scope to filter links (e.g. /subscriptions/{sub}/resourceGroups/{rg})
    #[arg(long)]
    pub scope: Option<String>,
}

#[derive(clap::Args)]
pub struct ResourceLinkShowArgs {
    /// Full link ID
    #[arg(long)]
    pub link_id: String,
}

#[derive(clap::Args)]
pub struct ResourceLinkCreateArgs {
    /// Full link ID (source resource ID + /providers/Microsoft.Resources/links/{name})
    #[arg(long)]
    pub link_id: String,
    /// Target resource ID
    #[arg(long)]
    pub target_id: String,
    /// Notes
    #[arg(long)]
    pub notes: Option<String>,
}

#[derive(clap::Args)]
pub struct ResourceLinkDeleteArgs {
    /// Full link ID
    #[arg(long)]
    pub link_id: String,
}

#[derive(clap::Args)]
pub struct ResourceLinkUpdateArgs {
    /// Full link ID
    #[arg(long)]
    pub link_id: String,
    /// Target resource ID
    #[arg(long)]
    pub target_id: Option<String>,
    /// Notes
    #[arg(long)]
    pub notes: Option<String>,
}

// --- Provider ---

#[derive(Subcommand)]
pub enum ProviderCommands {
    /// List resource providers
    List(ProviderListArgs),
    /// Show details of a resource provider
    Show(ProviderShowArgs),
    /// Register a resource provider
    Register(ProviderRegisterArgs),
    /// Unregister a resource provider
    Unregister(ProviderUnregisterArgs),
    /// List provider operations
    #[command(subcommand, name = "operation")]
    Operation(ProviderOperationCommands),
    /// List provider permissions
    #[command(subcommand, name = "permission")]
    Permission(ProviderPermissionCommands),
}

#[derive(clap::Args)]
pub struct ProviderListArgs {
    /// Expand properties (e.g. resourceTypes/aliases)
    #[arg(long)]
    pub expand: Option<String>,
}

#[derive(clap::Args)]
pub struct ProviderShowArgs {
    /// Provider namespace (e.g. Microsoft.Compute)
    #[arg(short, long)]
    pub namespace: String,
    /// Expand properties
    #[arg(long)]
    pub expand: Option<String>,
}

#[derive(clap::Args)]
pub struct ProviderRegisterArgs {
    /// Provider namespace
    #[arg(short, long)]
    pub namespace: String,
}

#[derive(clap::Args)]
pub struct ProviderUnregisterArgs {
    /// Provider namespace
    #[arg(short, long)]
    pub namespace: String,
}

#[derive(Subcommand)]
pub enum ProviderOperationCommands {
    /// List provider operations
    List(ProviderOperationListArgs),
}

#[derive(clap::Args)]
pub struct ProviderOperationListArgs {
    /// Provider namespace (optional; lists all if omitted)
    #[arg(short, long)]
    pub namespace: Option<String>,
}

#[derive(Subcommand)]
pub enum ProviderPermissionCommands {
    /// List provider permissions
    List(ProviderPermissionListArgs),
}

#[derive(clap::Args)]
pub struct ProviderPermissionListArgs {
    /// Provider namespace
    #[arg(short, long)]
    pub namespace: String,
}

// --- Feature ---

#[derive(Subcommand)]
pub enum FeatureCommands {
    /// List features
    List(FeatureListArgs),
    /// Show a feature
    Show(FeatureShowArgs),
    /// Register a feature
    Register(FeatureRegisterArgs),
    /// Unregister a feature
    Unregister(FeatureUnregisterArgs),
    /// Manage feature registrations
    #[command(subcommand)]
    Registration(FeatureRegistrationCommands),
}

#[derive(clap::Args)]
pub struct FeatureListArgs {
    /// Provider namespace
    #[arg(short, long)]
    pub namespace: Option<String>,
}

#[derive(clap::Args)]
pub struct FeatureShowArgs {
    /// Provider namespace
    #[arg(short, long)]
    pub namespace: String,
    /// Feature name
    #[arg(long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FeatureRegisterArgs {
    /// Provider namespace
    #[arg(short, long)]
    pub namespace: String,
    /// Feature name
    #[arg(long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FeatureUnregisterArgs {
    /// Provider namespace
    #[arg(short, long)]
    pub namespace: String,
    /// Feature name
    #[arg(long)]
    pub name: String,
}

#[derive(Subcommand)]
pub enum FeatureRegistrationCommands {
    /// List feature registrations
    List(FeatureRegistrationListArgs),
    /// Show a feature registration
    Show(FeatureRegistrationShowArgs),
    /// Create a feature registration
    Create(FeatureRegistrationCreateArgs),
    /// Delete a feature registration
    Delete(FeatureRegistrationDeleteArgs),
}

#[derive(clap::Args)]
pub struct FeatureRegistrationListArgs {
    /// Provider namespace
    #[arg(short, long)]
    pub namespace: Option<String>,
}

#[derive(clap::Args)]
pub struct FeatureRegistrationShowArgs {
    /// Provider namespace
    #[arg(short, long)]
    pub namespace: String,
    /// Feature name
    #[arg(long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FeatureRegistrationCreateArgs {
    /// Provider namespace
    #[arg(short, long)]
    pub namespace: String,
    /// Feature name
    #[arg(long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FeatureRegistrationDeleteArgs {
    /// Provider namespace
    #[arg(short, long)]
    pub namespace: String,
    /// Feature name
    #[arg(long)]
    pub name: String,
}

// --- Tag ---

#[derive(Subcommand)]
pub enum TagCommands {
    /// List tags in a subscription
    List,
    /// Create or update tags at a scope
    Create(TagCreateArgs),
    /// Delete tags
    Delete(TagDeleteArgs),
    /// Update tags at a scope (merge/replace/delete)
    Update(TagUpdateArgs),
    /// Add a value to a tag name
    #[command(name = "add-value")]
    AddValue(TagAddValueArgs),
    /// Remove a value from a tag name
    #[command(name = "remove-value")]
    RemoveValue(TagRemoveValueArgs),
}

#[derive(clap::Args)]
pub struct TagCreateArgs {
    /// Resource ID to apply tags to
    #[arg(long)]
    pub resource_id: Option<String>,
    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
    /// Tag name (legacy tag-name creation)
    #[arg(short, long)]
    pub name: Option<String>,
}

#[derive(clap::Args)]
pub struct TagDeleteArgs {
    /// Resource ID to remove tags from
    #[arg(long)]
    pub resource_id: Option<String>,
    /// Tag name to delete
    #[arg(short, long)]
    pub name: Option<String>,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct TagUpdateArgs {
    /// Resource ID
    #[arg(long)]
    pub resource_id: String,
    /// Operation: Merge, Replace, or Delete
    #[arg(long)]
    pub operation: String,
    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Vec<String>,
}

#[derive(clap::Args)]
pub struct TagAddValueArgs {
    /// Tag name
    #[arg(short, long)]
    pub name: String,
    /// Tag value
    #[arg(long)]
    pub value: String,
}

#[derive(clap::Args)]
pub struct TagRemoveValueArgs {
    /// Tag name
    #[arg(short, long)]
    pub name: String,
    /// Tag value
    #[arg(long)]
    pub value: String,
}

// --- Lock ---

#[derive(Subcommand)]
pub enum LockCommands {
    /// Create a lock
    Create(LockCreateArgs),
    /// Delete a lock
    Delete(LockDeleteArgs),
    /// List locks
    List(LockListArgs),
    /// Update a lock
    Update(LockUpdateArgs),
}

#[derive(clap::Args)]
pub struct LockCreateArgs {
    /// Lock name
    #[arg(short, long)]
    pub name: String,
    /// Lock type
    #[arg(short = 't', long, value_parser = ["CanNotDelete", "ReadOnly"])]
    pub lock_type: String,
    /// Resource group (omit for subscription-level lock)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
    /// Notes about the lock
    #[arg(long)]
    pub notes: Option<String>,
}

#[derive(clap::Args)]
pub struct LockDeleteArgs {
    /// Lock name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct LockListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct LockUpdateArgs {
    /// Lock name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
    /// Lock type
    #[arg(short = 't', long, value_parser = ["CanNotDelete", "ReadOnly"])]
    pub lock_type: Option<String>,
    /// Notes
    #[arg(long)]
    pub notes: Option<String>,
}

// --- Storage ---

#[derive(Subcommand)]
pub enum StorageCommands {
    /// Manage storage accounts
    #[command(subcommand)]
    Account(StorageAccountCommands),

    /// Manage object storage for unstructured data (blobs)
    #[command(subcommand, name = "blob")]
    Blob(crate::generated::StorageAazBlobCommands),

    /// Manage Azure file shares
    #[command(subcommand, name = "share-rm")]
    ShareRm(crate::generated::StorageAazShareRmCommands),

    /// Show storage SKU information
    #[command(subcommand, name = "sku")]
    Sku(crate::generated::StorageAazSkuCommands),
}

#[derive(Subcommand)]
pub enum StorageAccountCommands {
    /// Create a storage account
    Create(StorageAccountCreateArgs),
    /// List storage accounts
    List(StorageAccountListArgs),
    /// Show a storage account
    Show(StorageAccountShowArgs),
    /// Delete a storage account
    Delete(StorageAccountDeleteArgs),
    /// List storage account keys
    Keys(StorageAccountKeysArgs),

    /// Get the usage of file service in storage account
    #[command(name = "file-service-usage")]
    FileServiceUsage(crate::generated::StorageAazAccountFileServiceUsageArgs),

    /// Manage Storage Account Migration
    #[command(subcommand, name = "migration")]
    Migration(crate::generated::StorageAazAccountMigrationCommands),

    /// Manage Network Security Perimeter Configuration
    #[command(subcommand, name = "network-security-perimeter-configuration")]
    NetworkSecurityPerimeterConfiguration(crate::generated::StorageAazAccountNetworkSecurityPerimeterConfigurationCommands),
}

#[derive(clap::Args)]
pub struct StorageAccountCreateArgs {
    /// Storage account name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// SKU name
    #[arg(long, default_value = "Standard_LRS")]
    pub sku: String,
    /// Account kind
    #[arg(long, default_value = "StorageV2")]
    pub kind: String,
}

#[derive(clap::Args)]
pub struct StorageAccountListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct StorageAccountShowArgs {
    /// Storage account name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct StorageAccountDeleteArgs {
    /// Storage account name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Do not prompt
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct StorageAccountKeysArgs {
    /// Storage account name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

// --- Redis ---

#[derive(Subcommand)]
pub enum RedisCommands {
    /// Create a new Redis cache
    Create(RedisCreateArgs),
    /// List Redis caches
    List(RedisListArgs),
    /// Show details of a Redis cache
    Show(RedisShowArgs),
    /// Delete a Redis cache
    Delete(RedisDeleteArgs),
    /// Retrieve the access keys of a Redis cache
    #[command(name = "list-keys")]
    ListKeys(RedisShowArgs),
    /// Regenerate a Redis cache access key
    #[command(name = "regenerate-keys")]
    RegenerateKeys(RedisRegenerateKeysArgs),
    /// Update a Redis cache
    Update(RedisUpdateArgs),
}

#[derive(clap::Args)]
pub struct RedisCreateArgs {
    /// Name of the Redis cache
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Type of Redis cache: Basic, Standard, or Premium
    #[arg(long)]
    pub sku: String,
    /// Size of the Redis cache to deploy (e.g. c0, c1, p1). C = Basic/Standard, P = Premium
    #[arg(long)]
    pub vm_size: String,
    /// Enable the non-SSL Redis port (6379)
    #[arg(long)]
    pub enable_non_ssl_port: bool,
    /// Redis version, e.g. "6.0" or "latest"
    #[arg(long)]
    pub redis_version: Option<String>,
}

#[derive(clap::Args)]
pub struct RedisListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct RedisShowArgs {
    /// Name of the Redis cache
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct RedisDeleteArgs {
    /// Name of the Redis cache
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct RedisRegenerateKeysArgs {
    /// Name of the Redis cache
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Which key to regenerate
    #[arg(long, value_parser = ["Primary", "Secondary"])]
    pub key_type: String,
}

#[derive(clap::Args)]
pub struct RedisUpdateArgs {
    /// Name of the Redis cache
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Space-separated tags in key=value form
    #[arg(long, num_args = 0..)]
    pub tags: Option<Vec<String>>,
    /// Update a property: properties.<key>=<value> given as key=value
    #[arg(long = "set", num_args = 0..)]
    pub set: Option<Vec<String>>,
}

// --- ACR ---

#[derive(Subcommand)]
pub enum AcrCommands {
    /// Create a new container registry
    Create(AcrCreateArgs),
    /// List container registries
    List(AcrListArgs),
    /// Show details of a container registry
    Show(AcrShowArgs),
    /// Delete a container registry
    Delete(AcrDeleteArgs),
    /// Update a container registry
    Update(AcrUpdateArgs),
    /// Manage registry credentials
    #[command(subcommand)]
    Credential(AcrCredentialCommands),
}

#[derive(Subcommand)]
pub enum AcrCredentialCommands {
    /// Show registry credentials
    Show(AcrShowArgs),
    /// Regenerate a registry password
    Renew(AcrCredentialRenewArgs),
}

#[derive(clap::Args)]
pub struct AcrCreateArgs {
    /// Name of the container registry
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// SKU of the container registry
    #[arg(long, value_parser = ["Basic", "Standard", "Premium"])]
    pub sku: String,
    /// Indicates whether the admin user is enabled
    #[arg(long)]
    pub admin_enabled: bool,
}

#[derive(clap::Args)]
pub struct AcrListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct AcrShowArgs {
    /// Name of the container registry
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct AcrDeleteArgs {
    /// Name of the container registry
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct AcrUpdateArgs {
    /// Name of the container registry
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Space-separated tags in key=value form
    #[arg(long, num_args = 0..)]
    pub tags: Option<Vec<String>>,
    /// Update a property: properties.<key>=<value> given as key=value
    #[arg(long = "set", num_args = 0..)]
    pub set: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct AcrCredentialRenewArgs {
    /// Name of the container registry
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// The password name to regenerate
    #[arg(long, value_parser = ["password", "password2"])]
    pub password_name: String,
}

// --- App Configuration ---

#[derive(Subcommand)]
pub enum AppconfigCommands {
    /// Create an App Configuration store
    Create(AppconfigCreateArgs),
    /// List App Configuration stores
    List(AppconfigListArgs),
    /// Show details of an App Configuration store
    Show(AppconfigShowArgs),
    /// Delete an App Configuration store
    Delete(AppconfigDeleteArgs),
    /// Manage App Configuration store credentials
    #[command(subcommand)]
    Credential(AppconfigCredentialCommands),
    /// Update an App Configuration store
    Update(AppconfigUpdateArgs),
}

#[derive(Subcommand)]
pub enum AppconfigCredentialCommands {
    /// List access keys for an App Configuration store
    List(AppconfigCredentialListArgs),
}

#[derive(clap::Args)]
pub struct AppconfigCreateArgs {
    /// Name of the App Configuration store
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// App Configuration SKU: Free, Developer, Standard, or Premium
    #[arg(long, value_parser = ["Free", "Developer", "Standard", "Premium"])]
    pub sku: String,
    /// Enable purge protection
    #[arg(short = 'p', long)]
    pub enable_purge_protection: bool,
}

#[derive(clap::Args)]
pub struct AppconfigListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct AppconfigShowArgs {
    /// Name of the App Configuration store
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct AppconfigDeleteArgs {
    /// Name of the App Configuration store
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct AppconfigCredentialListArgs {
    /// Name of the App Configuration store
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct AppconfigUpdateArgs {
    /// Name of the App Configuration store
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Space-separated tags in key=value form
    #[arg(long, num_args = 0..)]
    pub tags: Option<Vec<String>>,
    /// Update a property: properties.<key>=<value> given as key=value
    #[arg(long = "set", num_args = 0..)]
    pub set: Option<Vec<String>>,
}

// --- SignalR ---

#[derive(Subcommand)]
pub enum SignalrCommands {
    /// Create a new SignalR service
    Create(SignalrCreateArgs),
    /// List SignalR services
    List(SignalrListArgs),
    /// Show details of a SignalR service
    Show(SignalrShowArgs),
    /// Delete a SignalR service
    Delete(SignalrDeleteArgs),
    /// Update a SignalR service
    Update(SignalrUpdateArgs),
    /// Manage SignalR access keys
    #[command(subcommand)]
    Key(SignalrKeyCommands),
}

#[derive(Subcommand)]
pub enum SignalrKeyCommands {
    /// Retrieve the access keys of a SignalR service
    List(SignalrShowArgs),
    /// Regenerate a SignalR service access key
    Renew(SignalrKeyRenewArgs),
}

#[derive(clap::Args)]
pub struct SignalrCreateArgs {
    /// Name of the SignalR service
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// SKU name
    #[arg(long, value_parser = ["Free_F1", "Standard_S1", "Premium_P1"])]
    pub sku: String,
    /// The number of SignalR service units
    #[arg(long, default_value_t = 1)]
    pub unit_count: i64,
    /// The service mode
    #[arg(long, default_value = "Default", value_parser = ["Default", "Serverless", "Classic"])]
    pub service_mode: String,
}

#[derive(clap::Args)]
pub struct SignalrListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct SignalrShowArgs {
    /// Name of the SignalR service
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct SignalrDeleteArgs {
    /// Name of the SignalR service
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct SignalrKeyRenewArgs {
    /// Name of the SignalR service
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Which key to regenerate
    #[arg(long, value_parser = ["Primary", "Secondary"])]
    pub key_type: String,
}

#[derive(clap::Args)]
pub struct SignalrUpdateArgs {
    /// Name of the SignalR service
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Space-separated tags in key=value form
    #[arg(long, num_args = 0..)]
    pub tags: Option<Vec<String>>,
    /// Update a property: properties.<key>=<value> given as key=value
    #[arg(long = "set", num_args = 0..)]
    pub set: Option<Vec<String>>,
}

// --- Azure Maps ---

#[derive(Subcommand)]
pub enum MapsCommands {
    /// Manage Azure Maps accounts
    #[command(subcommand)]
    Account(MapsAccountCommands),
}

#[derive(Subcommand)]
pub enum MapsAccountCommands {
    /// Create an Azure Maps account
    Create(MapsAccountCreateArgs),
    /// List Azure Maps accounts
    List(MapsAccountListArgs),
    /// Show details of an Azure Maps account
    Show(MapsAccountShowArgs),
    /// Delete an Azure Maps account
    Delete(MapsAccountDeleteArgs),
    /// Update an Azure Maps account
    Update(MapsAccountUpdateArgs),
    /// Manage Azure Maps account keys
    #[command(subcommand)]
    Keys(MapsAccountKeysCommands),
}

#[derive(Subcommand)]
pub enum MapsAccountKeysCommands {
    /// List Azure Maps account keys
    List(MapsAccountKeysListArgs),
    /// Regenerate an Azure Maps account key
    #[command(name = "regenerate", alias = "renew")]
    Regenerate(MapsAccountKeysRegenerateArgs),
}

#[derive(clap::Args)]
pub struct MapsAccountCreateArgs {
    /// Name of the Azure Maps account
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// The name of the SKU: S0, S1, or G2
    #[arg(long, value_parser = ["S0", "S1", "G2"])]
    pub sku: String,
    /// Account kind: Gen1 or Gen2
    #[arg(long, value_parser = ["Gen1", "Gen2"])]
    pub kind: Option<String>,
    /// Location
    #[arg(short, long, default_value = "global")]
    pub location: String,
}

#[derive(clap::Args)]
pub struct MapsAccountListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct MapsAccountShowArgs {
    /// Name of the Azure Maps account
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct MapsAccountDeleteArgs {
    /// Name of the Azure Maps account
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct MapsAccountUpdateArgs {
    /// Name of the Azure Maps account
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Space-separated tags in key=value form
    #[arg(long, num_args = 0..)]
    pub tags: Option<Vec<String>>,
    /// Update a property: properties.<key>=<value> given as key=value
    #[arg(long = "set", num_args = 0..)]
    pub set: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct MapsAccountKeysListArgs {
    /// Name of the Azure Maps account
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct MapsAccountKeysRegenerateArgs {
    /// Name of the Azure Maps account
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Which key to regenerate
    #[arg(long = "key", value_parser = ["primary", "secondary"])]
    pub key_type: String,
}

// --- Cognitive Services ---

#[derive(Subcommand)]
pub enum CognitiveservicesCommands {
    /// Manage Cognitive Services accounts
    #[command(subcommand)]
    Account(CognitiveservicesAccountCommands),
}

#[derive(Subcommand)]
pub enum CognitiveservicesAccountCommands {
    /// Create a Cognitive Services account
    Create(CognitiveservicesAccountCreateArgs),
    /// List Cognitive Services accounts
    List(CognitiveservicesAccountListArgs),
    /// Show details of a Cognitive Services account
    Show(CognitiveservicesAccountShowArgs),
    /// Delete a Cognitive Services account
    Delete(CognitiveservicesAccountDeleteArgs),
    /// Update a Cognitive Services account
    Update(CognitiveservicesAccountUpdateArgs),
    /// Manage Cognitive Services account keys
    #[command(subcommand)]
    Keys(CognitiveservicesAccountKeysCommands),
}

#[derive(Subcommand)]
pub enum CognitiveservicesAccountKeysCommands {
    /// List Cognitive Services account keys
    List(CognitiveservicesAccountShowArgs),
    /// Regenerate a Cognitive Services account key
    Regenerate(CognitiveservicesAccountKeysRegenerateArgs),
}

#[derive(clap::Args)]
pub struct CognitiveservicesAccountCreateArgs {
    /// Name of the Cognitive Services account
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Kind of Cognitive Services account, e.g. OpenAI, TextAnalytics, ComputerVision
    #[arg(long)]
    pub kind: String,
    /// Name of the SKU, e.g. S0 or F0
    #[arg(long)]
    pub sku: String,
    /// Accept Responsible AI terms without prompting
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct CognitiveservicesAccountListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct CognitiveservicesAccountShowArgs {
    /// Name of the Cognitive Services account
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct CognitiveservicesAccountDeleteArgs {
    /// Name of the Cognitive Services account
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct CognitiveservicesAccountUpdateArgs {
    /// Name of the Cognitive Services account
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Space-separated tags in key=value form
    #[arg(long, num_args = 0..)]
    pub tags: Option<Vec<String>>,
    /// Update a property: properties.<key>=<value> given as key=value
    #[arg(long = "set", num_args = 0..)]
    pub set: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct CognitiveservicesAccountKeysRegenerateArgs {
    /// Name of the Cognitive Services account
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Key name to regenerate
    #[arg(long, value_parser = ["Key1", "Key2"])]
    pub key_name: String,
}

// --- Event Grid ---

#[derive(Subcommand)]
pub enum EventgridCommands {
    /// Manage Event Grid topics
    #[command(subcommand)]
    Topic(EventgridTopicCommands),
    /// Manage Event Grid domains
    #[command(subcommand)]
    Domain(EventgridDomainCommands),
}

#[derive(Subcommand)]
pub enum EventgridTopicCommands {
    /// Create an Event Grid topic
    Create(EventgridTopicCreateArgs),
    /// List Event Grid topics
    List(EventgridTopicListArgs),
    /// Show an Event Grid topic
    Show(EventgridTopicShowArgs),
    /// Delete an Event Grid topic
    Delete(EventgridTopicDeleteArgs),
    /// Update an Event Grid topic
    Update(EventgridTopicUpdateArgs),
    /// Manage Event Grid topic keys
    #[command(subcommand)]
    Key(EventgridTopicKeyCommands),
}

#[derive(Subcommand)]
pub enum EventgridTopicKeyCommands {
    /// List topic keys
    List(EventgridTopicShowArgs),
    /// Regenerate a topic key
    Regenerate(EventgridTopicKeyRegenerateArgs),
}

#[derive(Subcommand)]
pub enum EventgridDomainCommands {
    /// Create an Event Grid domain
    Create(EventgridDomainCreateArgs),
    /// List Event Grid domains
    List(EventgridDomainListArgs),
    /// Show an Event Grid domain
    Show(EventgridDomainShowArgs),
    /// Delete an Event Grid domain
    Delete(EventgridDomainDeleteArgs),
}

#[derive(clap::Args)]
pub struct EventgridTopicCreateArgs {
    /// Topic name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Input schema
    #[arg(long, value_parser = ["eventgridschema", "customeventschema", "cloudeventschemav1_0"])]
    pub input_schema: Option<String>,
    /// Public network access
    #[arg(long, value_parser = ["enabled", "disabled"])]
    pub public_network_access: Option<String>,
}

#[derive(clap::Args)]
pub struct EventgridTopicListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct EventgridTopicShowArgs {
    /// Topic name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct EventgridTopicDeleteArgs {
    /// Topic name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct EventgridTopicUpdateArgs {
    /// Topic name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Space-separated tags in key=value form
    #[arg(long, num_args = 0..)]
    pub tags: Option<Vec<String>>,
    /// Update a property: properties.<key>=<value> given as key=value
    #[arg(long = "set", num_args = 0..)]
    pub set: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct EventgridTopicKeyRegenerateArgs {
    /// Topic name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Key name to regenerate
    #[arg(long, value_parser = ["key1", "key2"])]
    pub key_name: String,
}

#[derive(clap::Args)]
pub struct EventgridDomainCreateArgs {
    /// Domain name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Input schema
    #[arg(long, value_parser = ["eventgridschema", "customeventschema", "cloudeventschemav1_0"])]
    pub input_schema: Option<String>,
    /// Public network access
    #[arg(long, value_parser = ["enabled", "disabled"])]
    pub public_network_access: Option<String>,
}

#[derive(clap::Args)]
pub struct EventgridDomainListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct EventgridDomainShowArgs {
    /// Domain name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct EventgridDomainDeleteArgs {
    /// Domain name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

// --- Network ---

#[derive(Subcommand)]
pub enum NetworkCommands {
    /// Manage virtual networks
    #[command(subcommand)]
    Vnet(NetworkVnetCommands),
    /// Manage network security groups
    #[command(subcommand)]
    Nsg(NetworkNsgCommands),
}

#[derive(Subcommand)]
pub enum NetworkVnetCommands {
    /// Create a virtual network
    Create(NetworkVnetCreateArgs),
    /// List virtual networks
    List(NetworkVnetListArgs),
    /// Show a virtual network
    Show(NetworkVnetShowArgs),
    /// Delete a virtual network
    Delete(NetworkVnetDeleteArgs),
}

#[derive(clap::Args)]
pub struct NetworkVnetCreateArgs {
    /// VNet name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Address prefixes (CIDR)
    #[arg(long = "address-prefix", num_args = 1.., default_value = "10.0.0.0/16")]
    pub address_prefixes: Vec<String>,
}

#[derive(clap::Args)]
pub struct NetworkVnetListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct NetworkVnetShowArgs {
    /// VNet name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct NetworkVnetDeleteArgs {
    /// VNet name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(Subcommand)]
pub enum NetworkNsgCommands {
    /// Create a network security group
    Create(NetworkNsgCreateArgs),
    /// List network security groups
    List(NetworkNsgListArgs),
    /// Show a network security group
    Show(NetworkNsgShowArgs),
}

#[derive(clap::Args)]
pub struct NetworkNsgCreateArgs {
    /// NSG name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
}

#[derive(clap::Args)]
pub struct NetworkNsgListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct NetworkNsgShowArgs {
    /// NSG name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

// --- VM ---

#[derive(Subcommand)]
pub enum VmCommands {
    // --- Manual commands (vm.rs + vm_ext.rs) ---
    /// List virtual machines
    List(VmListArgs),
    /// Show a virtual machine
    Show(VmShowArgs),
    /// Create a virtual machine
    Create(VmCreateArgs),
    /// Update a virtual machine
    Update(VmUpdateArgs),
    /// Start a virtual machine
    Start(VmActionArgs),
    /// Stop (power off) a virtual machine
    Stop(VmActionArgs),
    /// Restart a virtual machine
    Restart(VmActionArgs),
    /// Deallocate a virtual machine
    Deallocate(VmActionArgs),
    /// Get the instance view of a virtual machine
    #[command(name = "get-instance-view")]
    GetInstanceView(VmShowArgs),
    /// Resize a virtual machine
    Resize(VmResizeArgs),
    /// Open a specific port on a virtual machine
    #[command(name = "open-port")]
    OpenPort(VmOpenPortArgs),
    /// Configure auto-shutdown for a virtual machine
    #[command(name = "auto-shutdown")]
    AutoShutdown(VmAutoShutdownArgs),
    /// Install patches on a virtual machine
    #[command(name = "install-patches")]
    InstallPatches(VmInstallPatchesArgs),
    /// List IP addresses associated with a virtual machine
    #[command(name = "list-ip-addresses")]
    ListIpAddresses(VmShowArgs),

    // --- Manual subgroups ---
    /// Manage VM data disks
    #[command(subcommand)]
    Disk(VmDiskCommands),
    /// Manage managed identities for a VM
    #[command(subcommand)]
    Identity(VmIdentityCommands),
    /// Manage user accounts for a VM
    #[command(subcommand)]
    User(VmUserCommands),
    /// Manage VM network interfaces
    #[command(subcommand)]
    Nic(VmNicCommands),
    /// Manage VM images
    #[command(subcommand)]
    Image(VmImageCommands),
    /// Manage VM encryption
    #[command(subcommand)]
    Encryption(VmEncryptionCommands),

    // --- Generated commands ---
    /// Assess patches on a VM
    #[command(name = "assess-patches")]
    AssessPatches(crate::generated::VmAssessPatchesArgs),
    /// Capture information for a stopped VM
    Capture(crate::generated::VmCaptureArgs),
    /// Convert a VM with unmanaged disks to use managed disks
    Convert(crate::generated::VmConvertArgs),
    /// Delete a virtual machine
    Delete(crate::generated::VmDeleteArgs),
    /// Mark a VM as generalized
    Generalize(crate::generated::VmGeneralizeArgs),
    /// List available sizes for VMs
    #[command(name = "list-sizes")]
    ListSizes(crate::generated::VmListSizesArgs),
    /// List available resizing options for VMs
    #[command(name = "list-vm-resize-options")]
    ListVmResizeOptions(crate::generated::VmListVmResizeOptionsArgs),
    /// Migrate a VM to Flexible VMSS
    #[command(name = "migrate-to-vmss")]
    MigrateToVmss(crate::generated::VmMigrateToVmssArgs),
    /// Perform maintenance on a virtual machine
    #[command(name = "perform-maintenance")]
    PerformMaintenance(crate::generated::VmPerformMaintenanceArgs),
    /// Reapply VMs
    Reapply(crate::generated::VmReapplyArgs),
    /// Redeploy an existing VM
    Redeploy(crate::generated::VmRedeployArgs),
    /// Reimage a virtual machine
    Reimage(crate::generated::VmReimageArgs),
    /// Simulate the eviction of a Spot VM
    #[command(name = "simulate-eviction")]
    SimulateEviction(crate::generated::VmSimulateEvictionArgs),
    /// Wait for a VM to reach a condition
    Wait(crate::generated::VmWaitArgs),

    // --- Generated subgroups ---
    /// Group resources into availability sets
    #[command(subcommand, name = "availability-set")]
    AvailabilitySet(crate::generated::VmAvailabilitySetCommands),
    /// Troubleshoot boot failures for VMs
    #[command(subcommand, name = "boot-diagnostics")]
    BootDiagnostics(crate::generated::VmBootDiagnosticsCommands),
    /// Manage extensions on VMs
    #[command(subcommand, name = "extension")]
    Extension(crate::generated::VmExtensionCommands),
    /// Manage Dedicated Hosts for Virtual Machines
    #[command(subcommand, name = "host")]
    Host(crate::generated::VmHostCommands),
    /// Manage run commands on a Virtual Machine
    #[command(subcommand, name = "run-command")]
    RunCommand(crate::generated::VmRunCommandCommands),
}

#[derive(clap::Args)]
pub struct VmListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct VmShowArgs {
    /// VM name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct VmActionArgs {
    /// VM name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct VmCreateArgs {
    /// VM name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Image URN (publisher:offer:sku:version)
    #[arg(long)]
    pub image: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// VM size
    #[arg(long)]
    pub size: Option<String>,
    /// Admin username
    #[arg(long)]
    pub admin_username: Option<String>,
    /// Admin password
    #[arg(long)]
    pub admin_password: Option<String>,
    /// SSH key values
    #[arg(long, num_args = 1..)]
    pub ssh_key_values: Option<Vec<String>>,
    /// Generate SSH keys if not present
    #[arg(long)]
    pub generate_ssh_keys: bool,
    /// OS type (Linux or Windows)
    #[arg(long)]
    pub os_type: Option<String>,
    /// Tags (key=value pairs)
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct VmUpdateArgs {
    /// VM name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Property key=value pairs to set
    #[arg(long, num_args = 1..)]
    pub set: Vec<String>,
}

#[derive(clap::Args)]
pub struct VmResizeArgs {
    /// VM name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// New VM size
    #[arg(long)]
    pub size: String,
}

#[derive(clap::Args)]
pub struct VmOpenPortArgs {
    /// VM name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Port or port range to open
    #[arg(long)]
    pub port: String,
    /// Rule priority (100-4096)
    #[arg(long)]
    pub priority: Option<u32>,
}

#[derive(clap::Args)]
pub struct VmAutoShutdownArgs {
    /// VM name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Shutdown time (HH:MM)
    #[arg(long)]
    pub time: Option<String>,
    /// Timezone
    #[arg(long)]
    pub timezone: Option<String>,
    /// Disable auto-shutdown
    #[arg(long)]
    pub off: bool,
}

#[derive(clap::Args)]
pub struct VmInstallPatchesArgs {
    /// VM name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Maximum duration (ISO 8601)
    #[arg(long)]
    pub maximum_duration: String,
    /// Reboot setting (IfRequired, Never, Always)
    #[arg(long)]
    pub reboot_setting: String,
}

#[derive(Subcommand)]
pub enum VmDiskCommands {
    /// Attach a managed data disk to a VM
    Attach(VmDiskAttachArgs),
    /// Detach a managed data disk from a VM
    Detach(VmDiskDetachArgs),
}

#[derive(clap::Args)]
pub struct VmDiskAttachArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// VM name
    #[arg(long)]
    pub vm_name: String,
    /// Disk name
    #[arg(short, long)]
    pub name: String,
    /// LUN
    #[arg(long)]
    pub lun: Option<i64>,
    /// Disk size in GB (for new disks)
    #[arg(long)]
    pub size_gb: Option<i64>,
    /// Create a new empty managed disk
    #[arg(long)]
    pub new: bool,
}

#[derive(clap::Args)]
pub struct VmDiskDetachArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// VM name
    #[arg(long)]
    pub vm_name: String,
    /// Disk name
    #[arg(short, long)]
    pub name: String,
}

#[derive(Subcommand)]
pub enum VmIdentityCommands {
    /// Assign managed identities to a VM
    Assign(VmIdentityArgs),
    /// Remove managed identities from a VM
    Remove(VmIdentityArgs),
}

#[derive(clap::Args)]
pub struct VmIdentityArgs {
    /// VM name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// User-assigned identity resource IDs (use [system] for system-assigned)
    #[arg(long, num_args = 1..)]
    pub identities: Option<Vec<String>>,
}

#[derive(Subcommand)]
pub enum VmUserCommands {
    /// Update a user account on a VM
    Update(VmUserUpdateArgs),
    /// Delete a user account from a VM
    Delete(VmUserDeleteArgs),
    /// Reset SSH configuration on a VM
    #[command(name = "reset-ssh")]
    ResetSsh(VmUserResetSshArgs),
}

#[derive(clap::Args)]
pub struct VmUserUpdateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// VM name
    #[arg(short, long)]
    pub name: String,
    /// Username
    #[arg(short, long)]
    pub username: String,
    /// Password
    #[arg(short, long)]
    pub password: Option<String>,
    /// SSH public key value
    #[arg(long)]
    pub ssh_key_value: Option<String>,
}

#[derive(clap::Args)]
pub struct VmUserDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// VM name
    #[arg(short, long)]
    pub name: String,
    /// Username to delete
    #[arg(short, long)]
    pub username: String,
}

#[derive(clap::Args)]
pub struct VmUserResetSshArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// VM name
    #[arg(short, long)]
    pub name: String,
}

#[derive(Subcommand)]
pub enum VmNicCommands {
    /// Add NICs to a VM
    Add(VmNicModifyArgs),
    /// Remove NICs from a VM
    Remove(VmNicModifyArgs),
    /// Replace all NICs on a VM
    Set(VmNicSetArgs),
    /// List NICs on a VM
    List(VmNicListArgs),
}

#[derive(clap::Args)]
pub struct VmNicModifyArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// VM name
    #[arg(long)]
    pub vm_name: String,
    /// NIC resource IDs
    #[arg(long, num_args = 1..)]
    pub nics: Vec<String>,
}

#[derive(clap::Args)]
pub struct VmNicSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// VM name
    #[arg(long)]
    pub vm_name: String,
    /// NIC resource IDs
    #[arg(long, num_args = 1..)]
    pub nics: Vec<String>,
    /// Primary NIC resource ID
    #[arg(long)]
    pub primary_nic: Option<String>,
}

#[derive(clap::Args)]
pub struct VmNicListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// VM name
    #[arg(long)]
    pub vm_name: String,
}

#[derive(Subcommand)]
pub enum VmImageCommands {
    /// List VM image versions
    List(VmImageListArgs),
    /// List VM image offers
    #[command(name = "list-offers")]
    ListOffers(VmImageListOffersArgs),
    /// List VM image publishers
    #[command(name = "list-publishers")]
    ListPublishers(VmImageListPublishersArgs),
    /// List VM image SKUs
    #[command(name = "list-skus")]
    ListSkus(VmImageListSkusArgs),
    /// Accept Marketplace terms for a VM image
    #[command(name = "accept-terms")]
    AcceptTerms(VmImageAcceptTermsArgs),
}

#[derive(clap::Args)]
pub struct VmImageListArgs {
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Publisher
    #[arg(short, long)]
    pub publisher: String,
    /// Offer
    #[arg(short = 'f', long)]
    pub offer: String,
    /// SKU
    #[arg(short, long)]
    pub sku: String,
}

#[derive(clap::Args)]
pub struct VmImageListOffersArgs {
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Publisher
    #[arg(short, long)]
    pub publisher: String,
}

#[derive(clap::Args)]
pub struct VmImageListPublishersArgs {
    /// Location
    #[arg(short, long)]
    pub location: String,
}

#[derive(clap::Args)]
pub struct VmImageListSkusArgs {
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Publisher
    #[arg(short, long)]
    pub publisher: String,
    /// Offer
    #[arg(short = 'f', long)]
    pub offer: String,
}

#[derive(clap::Args)]
pub struct VmImageAcceptTermsArgs {
    /// Publisher
    #[arg(short, long)]
    pub publisher: String,
    /// Offer
    #[arg(short = 'f', long)]
    pub offer: String,
    /// Plan name
    #[arg(long)]
    pub plan: String,
}

#[derive(Subcommand)]
pub enum VmEncryptionCommands {
    /// Enable disk encryption on a VM
    Enable(VmEncryptionEnableArgs),
    /// Disable disk encryption on a VM
    Disable(VmEncryptionDisableArgs),
}

#[derive(clap::Args)]
pub struct VmEncryptionEnableArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// VM name
    #[arg(short, long)]
    pub name: String,
    /// Key Vault URL or resource ID
    #[arg(long)]
    pub disk_encryption_keyvault: String,
    /// Volume type (OS, Data, All)
    #[arg(long)]
    pub volume_type: Option<String>,
    /// Key encryption key URL
    #[arg(long)]
    pub key_encryption_key: Option<String>,
    /// Key encryption algorithm
    #[arg(long)]
    pub key_encryption_algorithm: Option<String>,
}

#[derive(clap::Args)]
pub struct VmEncryptionDisableArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// VM name
    #[arg(short, long)]
    pub name: String,
    /// Volume type (OS, Data, All)
    #[arg(long)]
    pub volume_type: Option<String>,
}

// --- VMSS ---

#[derive(Subcommand)]
pub enum VmssCommands {
    // --- Manual commands (vmss_ext.rs) ---
    /// Create a virtual machine scale set
    Create(VmssCreateArgs),
    /// Update a virtual machine scale set
    Update(VmssUpdateArgs),
    /// Scale a virtual machine scale set
    Scale(VmssScaleArgs),
    /// Deallocate VMs within a VMSS
    Deallocate(VmssInstanceActionArgs),
    /// Restart VMs within a VMSS
    Restart(VmssInstanceActionArgs),
    /// Stop VMs within a VMSS
    Stop(VmssInstanceActionArgs),
    /// Reimage VMs within a VMSS
    Reimage(VmssInstanceActionArgs),
    /// Get the instance view of a VMSS
    #[command(name = "get-instance-view")]
    GetInstanceView(VmssNameArgs),
    /// Update instances in a VMSS
    #[command(name = "update-instances")]
    UpdateInstances(VmssUpdateInstancesArgs),
    /// List instance connection info for a VMSS
    #[command(name = "list-instance-connection-info")]
    ListInstanceConnectionInfo(VmssNameArgs),
    /// List public IPs for all VMSS instances
    #[command(name = "list-instance-public-ips")]
    ListInstancePublicIps(VmssNameArgs),
    /// Set orchestration service state
    #[command(name = "set-orchestration-service-state")]
    SetOrchestrationServiceState(VmssOrchestrationArgs),

    // --- Manual subgroups ---
    /// Manage managed identities for a VMSS
    #[command(subcommand)]
    Identity(VmssIdentityCommands),

    // --- Generated commands ---
    /// Delete a VM scale set
    Delete(crate::generated::VmssDeleteArgs),
    /// Delete VMs within a VMSS
    #[command(name = "delete-instances")]
    DeleteInstances(crate::generated::VmssDeleteInstancesArgs),
    /// List the OS upgrades on a VM scale set instance
    #[command(name = "get-os-upgrade-history")]
    GetOsUpgradeHistory(crate::generated::VmssGetOsUpgradeHistoryArgs),
    /// List all VM scale sets under a resource group
    List(crate::generated::VmssListArgs),
    /// List all virtual machines in a VM scale set
    #[command(name = "list-instances")]
    ListInstances(crate::generated::VmssListInstancesArgs),
    /// List SKUs available for your VM scale set
    #[command(name = "list-skus")]
    ListSkus(crate::generated::VmssListSkusArgs),
    /// Perform maintenance on VMs in a scale set
    #[command(name = "perform-maintenance")]
    PerformMaintenance(crate::generated::VmssPerformMaintenanceArgs),
    /// Simulate the eviction of a Spot VM in a VMSS
    #[command(name = "simulate-eviction")]
    SimulateEviction(crate::generated::VmssSimulateEvictionArgs),
    /// Start VMs within a VMSS
    Start(crate::generated::VmssStartArgs),
    /// Manual platform update domain walk
    #[command(name = "update-domain-walk")]
    UpdateDomainWalk(crate::generated::VmssUpdateDomainWalkArgs),
    /// Wait for a VMSS to reach a condition
    Wait(crate::generated::VmssWaitArgs),

    // --- Generated subgroups ---
    /// Manage extensions on a VM scale set
    #[command(subcommand, name = "extension")]
    Extension(crate::generated::VmssExtensionCommands),
    /// Manage network interfaces of a VMSS
    #[command(subcommand, name = "nic")]
    Nic(crate::generated::VmssNicCommands),
    /// Manage rolling upgrades
    #[command(subcommand, name = "rolling-upgrade")]
    RollingUpgrade(crate::generated::VmssRollingUpgradeCommands),
    /// Manage run commands on a VMSS
    #[command(subcommand, name = "run-command")]
    RunCommand(crate::generated::VmssRunCommandCommands),
}

#[derive(clap::Args)]
pub struct VmssCreateArgs {
    /// VMSS name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Image URN (publisher:offer:sku:version)
    #[arg(long)]
    pub image: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Number of instances
    #[arg(long)]
    pub instance_count: Option<i64>,
    /// VM size
    #[arg(long)]
    pub vm_sku: Option<String>,
    /// Admin username
    #[arg(long)]
    pub admin_username: Option<String>,
    /// Admin password
    #[arg(long)]
    pub admin_password: Option<String>,
    /// Upgrade policy mode (Manual, Automatic, Rolling)
    #[arg(long)]
    pub upgrade_policy_mode: Option<String>,
    /// Tags (key=value pairs)
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct VmssUpdateArgs {
    /// VMSS name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Property key=value pairs to set
    #[arg(long, num_args = 1..)]
    pub set: Option<Vec<String>>,
    /// Tags (key=value pairs)
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct VmssScaleArgs {
    /// VMSS name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// New instance count
    #[arg(long)]
    pub new_capacity: i64,
}

#[derive(clap::Args)]
pub struct VmssInstanceActionArgs {
    /// VMSS name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Instance IDs to act on (omit for all)
    #[arg(long, num_args = 1..)]
    pub instance_ids: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct VmssNameArgs {
    /// VMSS name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct VmssUpdateInstancesArgs {
    /// VMSS name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Instance IDs to update
    #[arg(long, num_args = 1..)]
    pub instance_ids: Vec<String>,
}

#[derive(clap::Args)]
pub struct VmssOrchestrationArgs {
    /// VMSS name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Service name
    #[arg(long)]
    pub service_name: String,
    /// Action (Resume, Suspend)
    #[arg(long)]
    pub action: String,
}

#[derive(Subcommand)]
pub enum VmssIdentityCommands {
    /// Assign managed identities to a VMSS
    Assign(VmssIdentityArgs),
    /// Remove managed identities from a VMSS
    Remove(VmssIdentityArgs),
}

#[derive(clap::Args)]
pub struct VmssIdentityArgs {
    /// VMSS name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Assign system-assigned identity
    #[arg(long)]
    pub system_assigned: bool,
    /// User-assigned identity resource IDs
    #[arg(long, num_args = 1..)]
    pub user_assigned: Option<Vec<String>>,
}

// --- Config ---

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Set configuration values (e.g. `azrs config set defaults.group=MyRG`)
    Set(ConfigSetArgs),
    /// Get a configuration value (e.g. `azrs config get defaults.group`)
    Get(ConfigGetArgs),
    /// Unset a configuration value
    Unset(ConfigUnsetArgs),
}

#[derive(clap::Args)]
pub struct ConfigSetArgs {
    /// Key=value pairs (e.g. defaults.group=MyRG defaults.location=eastus)
    #[arg(num_args = 1..)]
    pub pairs: Vec<String>,
}

#[derive(clap::Args)]
pub struct ConfigGetArgs {
    /// Key to get (e.g. defaults.group)
    pub key: String,
}

#[derive(clap::Args)]
pub struct ConfigUnsetArgs {
    /// Key to unset (e.g. defaults.group)
    pub key: String,
}

// --- Find ---

#[derive(clap::Args)]
pub struct FindArgs {
    /// Search query (e.g. "create vm", "storage blob")
    pub query: String,
}

// --- Cloud ---

#[derive(Subcommand)]
pub enum CloudCommands {
    /// List registered clouds
    List,
    /// Show details of a cloud
    Show(CloudShowArgs),
    /// Set the active cloud
    Set(CloudSetArgs),
}

#[derive(clap::Args)]
pub struct CloudShowArgs {
    /// Cloud name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct CloudSetArgs {
    /// Cloud name
    #[arg(short, long)]
    pub name: String,
}

// --- Role (RBAC) ---

#[derive(Subcommand)]
pub enum RoleCommands {
    /// Manage role assignments
    #[command(subcommand)]
    Assignment(RoleAssignmentCommands),
    /// Manage role definitions
    #[command(subcommand)]
    Definition(RoleDefinitionCommands),
}

#[derive(Subcommand)]
pub enum RoleAssignmentCommands {
    /// List role assignments
    List(RoleAssignmentListArgs),
    /// Create a role assignment
    Create(RoleAssignmentCreateArgs),
    /// Delete role assignments
    Delete(RoleAssignmentDeleteArgs),
}

#[derive(clap::Args)]
pub struct RoleAssignmentListArgs {
    /// Scope (defaults to current subscription)
    #[arg(long)]
    pub scope: Option<String>,
    /// Resource group (used to build scope if --scope not set)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
    /// Filter by assignee object ID
    #[arg(long)]
    pub assignee: Option<String>,
    /// Filter by role name or ID
    #[arg(long)]
    pub role: Option<String>,
    /// Include inherited assignments
    #[arg(long)]
    pub include_inherited: bool,
    /// Show all assignments (including inherited)
    #[arg(long)]
    pub all: bool,
}

#[derive(clap::Args)]
pub struct RoleAssignmentCreateArgs {
    /// Role name or ID (e.g. "Contributor", "Reader")
    #[arg(long)]
    pub role: String,
    /// Scope for the assignment
    #[arg(long)]
    pub scope: String,
    /// Object ID of the assignee (user, group, or service principal)
    #[arg(long)]
    pub assignee_object_id: Option<String>,
    /// Display name, email, or object ID of the assignee
    #[arg(long)]
    pub assignee: Option<String>,
    /// Principal type (User, Group, ServicePrincipal, ForeignGroup)
    #[arg(long)]
    pub assignee_principal_type: Option<String>,
    /// Assignment name (GUID, auto-generated if not specified)
    #[arg(short, long)]
    pub name: Option<String>,
    /// Description of the role assignment
    #[arg(long)]
    pub description: Option<String>,
    /// Condition for the role assignment (ABAC)
    #[arg(long)]
    pub condition: Option<String>,
    /// Condition version (defaults to 2.0 if --condition is set)
    #[arg(long)]
    pub condition_version: Option<String>,
}

#[derive(clap::Args)]
pub struct RoleAssignmentDeleteArgs {
    /// Scope
    #[arg(long)]
    pub scope: Option<String>,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
    /// Filter by assignee object ID
    #[arg(long)]
    pub assignee: Option<String>,
    /// Filter by role name or ID
    #[arg(long)]
    pub role: Option<String>,
    /// Assignment resource IDs to delete directly
    #[arg(long, num_args = 1..)]
    pub ids: Option<Vec<String>>,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(Subcommand)]
pub enum RoleDefinitionCommands {
    /// List role definitions
    List(RoleDefinitionListArgs),
    /// Create a custom role definition
    Create(RoleDefinitionCreateArgs),
    /// Update a custom role definition
    Update(RoleDefinitionUpdateArgs),
    /// Delete a custom role definition
    Delete(RoleDefinitionDeleteArgs),
}

#[derive(clap::Args)]
pub struct RoleDefinitionListArgs {
    /// Scope (defaults to current subscription)
    #[arg(long)]
    pub scope: Option<String>,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
    /// Filter by role name
    #[arg(short, long)]
    pub name: Option<String>,
    /// Only show custom role definitions
    #[arg(long)]
    pub custom_role_only: bool,
}

#[derive(clap::Args)]
pub struct RoleDefinitionCreateArgs {
    /// JSON role definition (inline string or @filename)
    #[arg(long)]
    pub role_definition: String,
}

#[derive(clap::Args)]
pub struct RoleDefinitionUpdateArgs {
    /// JSON role definition (inline string or @filename)
    #[arg(long)]
    pub role_definition: String,
}

#[derive(clap::Args)]
pub struct RoleDefinitionDeleteArgs {
    /// Scope (defaults to current subscription)
    #[arg(long)]
    pub scope: Option<String>,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
    /// Role name to delete
    #[arg(short, long)]
    pub name: String,
}

// --- Key Vault ---

#[derive(Subcommand)]
pub enum KeyvaultCommands {
    /// Manage Key Vault secrets
    #[command(subcommand)]
    Secret(KeyvaultSecretCommands),
}

#[derive(Subcommand)]
pub enum KeyvaultSecretCommands {
    /// Set a secret in Key Vault
    Set(KeyvaultSecretSetArgs),
    /// Show a secret from Key Vault
    Show(KeyvaultSecretShowArgs),
    /// List secrets in Key Vault
    List(KeyvaultSecretListArgs),
    /// Delete a secret from Key Vault
    Delete(KeyvaultSecretDeleteArgs),
}

#[derive(clap::Args)]
pub struct KeyvaultSecretSetArgs {
    /// Vault name
    #[arg(long)]
    pub vault_name: String,
    /// Secret name
    #[arg(short, long)]
    pub name: String,
    /// Secret value
    #[arg(long)]
    pub value: String,
}

#[derive(clap::Args)]
pub struct KeyvaultSecretShowArgs {
    /// Vault name
    #[arg(long)]
    pub vault_name: String,
    /// Secret name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct KeyvaultSecretListArgs {
    /// Vault name
    #[arg(long)]
    pub vault_name: String,
}

#[derive(clap::Args)]
pub struct KeyvaultSecretDeleteArgs {
    /// Vault name
    #[arg(long)]
    pub vault_name: String,
    /// Secret name
    #[arg(short, long)]
    pub name: String,
}

// ── Deployment commands ─────────────────────────────────────────────

#[derive(Subcommand)]
pub enum DeploymentCommands {
    /// Manage resource-group-scoped deployments
    #[command(subcommand)]
    Group(DeploymentScopeCommands),

    /// Manage subscription-scoped deployments
    #[command(subcommand)]
    Sub(DeploymentScopeCommands),

    /// Manage management-group-scoped deployments
    #[command(subcommand)]
    Mg(DeploymentScopeCommands),

    /// Manage tenant-scoped deployments
    #[command(subcommand)]
    Tenant(DeploymentScopeCommands),
}

#[derive(Subcommand)]
pub enum DeploymentScopeCommands {
    /// List deployments
    List(DeploymentListArgs),

    /// Show a deployment
    Show(DeploymentShowArgs),

    /// Delete a deployment
    Delete(DeploymentDeleteArgs),

    /// Create a deployment
    Create(DeploymentCreateArgs),

    /// Validate a deployment template
    Validate(DeploymentValidateArgs),

    /// Export the template used for a deployment
    Export(DeploymentExportArgs),

    /// Cancel a deployment
    Cancel(DeploymentCancelArgs),

    /// Execute a deployment What-If operation
    #[command(name = "what-if")]
    WhatIf(DeploymentWhatIfArgs),

    /// Manage deployment operations
    #[command(subcommand)]
    Operation(DeploymentOperationCommands),
}

#[derive(Subcommand)]
pub enum DeploymentOperationCommands {
    /// List deployment operations
    List(DeploymentOperationListArgs),
}

impl DeploymentScopeCommands {
    /// Extract the resource_group from whichever variant is active.
    pub fn resource_group_ref(&self) -> Option<&str> {
        match self {
            Self::List(a) => a.resource_group.as_deref(),
            Self::Show(a) => a.resource_group.as_deref(),
            Self::Delete(a) => a.resource_group.as_deref(),
            Self::Create(a) => a.resource_group.as_deref(),
            Self::Validate(a) => a.resource_group.as_deref(),
            Self::Export(a) => a.resource_group.as_deref(),
            Self::Cancel(a) => a.resource_group.as_deref(),
            Self::WhatIf(a) => a.resource_group.as_deref(),
            Self::Operation(DeploymentOperationCommands::List(a)) => a.resource_group.as_deref(),
        }
    }

    /// Extract a management-group name used as scope identifier (reuses resource_group field).
    pub fn management_group_ref(&self) -> Option<&str> {
        self.resource_group_ref()
    }
}

#[derive(clap::Args)]
pub struct DeploymentListArgs {
    /// Name of the resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,

    /// OData filter expression
    #[arg(long)]
    pub filter: Option<String>,
}

#[derive(clap::Args)]
pub struct DeploymentShowArgs {
    /// Deployment name
    #[arg(short, long)]
    pub name: String,

    /// Name of the resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct DeploymentDeleteArgs {
    /// Deployment name
    #[arg(short, long)]
    pub name: String,

    /// Name of the resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct DeploymentCreateArgs {
    /// Deployment name
    #[arg(short, long)]
    pub name: String,

    /// Path to a template file
    #[arg(short = 'f', long)]
    pub template_file: Option<String>,

    /// URI of a remote template file
    #[arg(long)]
    pub template_uri: Option<String>,

    /// Supply deployment parameter values (JSON string, file path, or key=value pairs)
    #[arg(short, long)]
    pub parameters: Option<String>,

    /// Deployment mode
    #[arg(long, value_parser = ["Incremental", "Complete"])]
    pub mode: Option<String>,

    /// Do not wait for the long-running operation to finish
    #[arg(long)]
    pub no_wait: bool,

    /// Name of the resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct DeploymentValidateArgs {
    /// Deployment name
    #[arg(short, long)]
    pub name: String,

    /// Path to a template file
    #[arg(long)]
    pub template_file: Option<String>,

    /// URI of a remote template file
    #[arg(long)]
    pub template_uri: Option<String>,

    /// Supply deployment parameter values
    #[arg(long)]
    pub parameters: Option<String>,

    /// Name of the resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct DeploymentExportArgs {
    /// Deployment name
    #[arg(short, long)]
    pub name: String,

    /// Name of the resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct DeploymentCancelArgs {
    /// Deployment name
    #[arg(short, long)]
    pub name: String,

    /// Name of the resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct DeploymentWhatIfArgs {
    /// Deployment name
    #[arg(short, long)]
    pub name: String,

    /// Path to a template file
    #[arg(long)]
    pub template_file: Option<String>,

    /// URI of a remote template file
    #[arg(long)]
    pub template_uri: Option<String>,

    /// Supply deployment parameter values
    #[arg(long)]
    pub parameters: Option<String>,

    /// Name of the resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct DeploymentOperationListArgs {
    /// Deployment name
    #[arg(short, long)]
    pub name: String,

    /// Name of the resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

// ── Deployment-scripts commands ─────────────────────────────────────

#[derive(Subcommand)]
pub enum DeploymentScriptsCommands {
    /// List deployment scripts
    List(DeploymentScriptsListArgs),

    /// Show the logs for a deployment script
    #[command(name = "show-log")]
    ShowLog(DeploymentScriptsShowLogArgs),

    /// Delete a deployment script
    Delete(DeploymentScriptsDeleteArgs),
}

#[derive(clap::Args)]
pub struct DeploymentScriptsListArgs {
    /// Name of the resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct DeploymentScriptsShowLogArgs {
    /// Name of the resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,

    /// Deployment script name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct DeploymentScriptsDeleteArgs {
    /// Name of the resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,

    /// Deployment script name
    #[arg(short, long)]
    pub name: String,
}

// ── Template Specs commands ─────────────────────────────────────────

#[derive(Subcommand)]
pub enum TsCommands {
    /// List template specs
    List(TsListArgs),
    /// Show a template spec
    Show(TsShowArgs),
    /// Create a template spec
    Create(TsCreateArgs),
    /// Delete a template spec
    Delete(TsDeleteArgs),
    /// Export a template spec version
    Export(TsExportArgs),
}

#[derive(clap::Args)]
pub struct TsListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct TsShowArgs {
    /// Template spec name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Version
    #[arg(long)]
    pub version: Option<String>,
}

#[derive(clap::Args)]
pub struct TsCreateArgs {
    /// Template spec name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Version
    #[arg(long)]
    pub version: String,
    /// Path to a template file
    #[arg(short = 'f', long)]
    pub template_file: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Description
    #[arg(long)]
    pub description: Option<String>,
    /// Display name
    #[arg(long)]
    pub display_name: Option<String>,
    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct TsDeleteArgs {
    /// Template spec name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Version (omit to delete entire spec)
    #[arg(long)]
    pub version: Option<String>,
}

#[derive(clap::Args)]
pub struct TsExportArgs {
    /// Template spec name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Version
    #[arg(long)]
    pub version: String,
    /// Output folder
    #[arg(long)]
    pub output_folder: String,
}

// ── Deployment Stacks commands ─────────────────────────────────────

#[derive(Subcommand)]
pub enum StackCommands {
    /// Manage resource-group-scoped deployment stacks
    #[command(subcommand)]
    Group(StackScopeCommands),
    /// Manage subscription-scoped deployment stacks
    #[command(subcommand)]
    Sub(StackScopeCommands),
    /// Manage management-group-scoped deployment stacks
    #[command(subcommand)]
    Mg(StackScopeCommands),
}

#[derive(Subcommand)]
pub enum StackScopeCommands {
    /// List deployment stacks
    List(StackListArgs),
    /// Show a deployment stack
    Show(StackShowArgs),
    /// Delete a deployment stack
    Delete(StackDeleteArgs),
    /// Export a deployment stack template
    Export(StackExportArgs),
}

impl StackScopeCommands {
    pub fn resource_group_ref(&self) -> Option<&str> {
        match self {
            Self::List(a) => a.resource_group.as_deref(),
            Self::Show(a) => a.resource_group.as_deref(),
            Self::Delete(a) => a.resource_group.as_deref(),
            Self::Export(a) => a.resource_group.as_deref(),
        }
    }

    pub fn management_group_ref(&self) -> Option<&str> {
        match self {
            Self::List(a) => a.management_group_id.as_deref(),
            Self::Show(a) => a.management_group_id.as_deref(),
            Self::Delete(a) => a.management_group_id.as_deref(),
            Self::Export(a) => a.management_group_id.as_deref(),
        }
    }
}

#[derive(clap::Args)]
pub struct StackListArgs {
    /// Resource group (for group scope)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
    /// Management group ID (for mg scope)
    #[arg(long)]
    pub management_group_id: Option<String>,
}

#[derive(clap::Args)]
pub struct StackShowArgs {
    /// Stack name
    #[arg(short, long)]
    pub name: String,
    /// Resource group (for group scope)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
    /// Management group ID (for mg scope)
    #[arg(long)]
    pub management_group_id: Option<String>,
}

#[derive(clap::Args)]
pub struct StackDeleteArgs {
    /// Stack name
    #[arg(short, long)]
    pub name: String,
    /// Resource group (for group scope)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
    /// Management group ID (for mg scope)
    #[arg(long)]
    pub management_group_id: Option<String>,
}

#[derive(clap::Args)]
pub struct StackExportArgs {
    /// Stack name
    #[arg(short, long)]
    pub name: String,
    /// Resource group (for group scope)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
    /// Management group ID (for mg scope)
    #[arg(long)]
    pub management_group_id: Option<String>,
}

// ── Managed Application commands ───────────────────────────────────

#[derive(Subcommand)]
pub enum ManagedappCommands {
    /// List managed applications
    List(ManagedappListArgs),
    /// Show a managed application
    Show(ManagedappShowArgs),
    /// Create a managed application
    Create(ManagedappCreateArgs),
    /// Delete a managed application
    Delete(ManagedappDeleteArgs),
    /// Manage managed application definitions
    #[command(subcommand)]
    Definition(ManagedappDefinitionCommands),
}

#[derive(clap::Args)]
pub struct ManagedappListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct ManagedappShowArgs {
    /// Application name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct ManagedappCreateArgs {
    /// Application name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Application kind (e.g. MarketPlace, ServiceCatalog)
    #[arg(long)]
    pub kind: String,
    /// Managed resource group ID
    #[arg(long)]
    pub managed_rg_id: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Application definition ID
    #[arg(long)]
    pub definition_id: Option<String>,
    /// Parameters JSON string
    #[arg(long)]
    pub parameters: Option<String>,
}

#[derive(clap::Args)]
pub struct ManagedappDeleteArgs {
    /// Application name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(Subcommand)]
pub enum ManagedappDefinitionCommands {
    /// List managed application definitions
    List(ManagedappDefinitionListArgs),
    /// Create a managed application definition
    Create(ManagedappDefinitionCreateArgs),
    /// Delete a managed application definition
    Delete(ManagedappDefinitionDeleteArgs),
    /// Update a managed application definition
    Update(ManagedappDefinitionUpdateArgs),
}

#[derive(clap::Args)]
pub struct ManagedappDefinitionListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct ManagedappDefinitionCreateArgs {
    /// Definition name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Lock level (None, CanNotDelete, ReadOnly)
    #[arg(long)]
    pub lock_level: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Display name
    #[arg(long)]
    pub display_name: Option<String>,
    /// Description
    #[arg(long)]
    pub description: Option<String>,
    /// Package file URI
    #[arg(long)]
    pub package_file_uri: Option<String>,
    /// Authorizations JSON string
    #[arg(long)]
    pub authorizations: Option<String>,
}

#[derive(clap::Args)]
pub struct ManagedappDefinitionDeleteArgs {
    /// Definition name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
}

#[derive(clap::Args)]
pub struct ManagedappDefinitionUpdateArgs {
    /// Definition name
    #[arg(short, long)]
    pub name: String,
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Lock level
    #[arg(long)]
    pub lock_level: Option<String>,
    /// Display name
    #[arg(long)]
    pub display_name: Option<String>,
    /// Description
    #[arg(long)]
    pub description: Option<String>,
    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

// ── Management group commands ───────────────────────────────────────

#[derive(Subcommand)]
pub enum ManagementGroupCommands {
    /// List management groups
    List,

    /// Show a management group
    Show(ManagementGroupShowArgs),

    /// Create a management group
    Create(ManagementGroupCreateArgs),

    /// Delete a management group
    Delete(ManagementGroupDeleteArgs),

    /// Check if a management group name is available
    #[command(name = "check-name-availability")]
    CheckNameAvailability(ManagementGroupCheckNameArgs),

    /// Manage management group subscriptions
    #[command(subcommand)]
    Subscription(ManagementGroupSubscriptionCommands),

    /// Manage management group entities
    #[command(subcommand)]
    Entities(ManagementGroupEntitiesCommands),

    /// Manage hierarchy settings
    #[command(subcommand, name = "hierarchy-settings")]
    HierarchySettings(ManagementGroupHierarchySettingsCommands),

    /// Manage tenant backfill
    #[command(subcommand, name = "tenant-backfill")]
    TenantBackfill(ManagementGroupTenantBackfillCommands),
}

#[derive(clap::Args)]
pub struct ManagementGroupShowArgs {
    /// Management group name
    #[arg(short, long)]
    pub name: String,

    /// Expand child entities
    #[arg(long)]
    pub expand: Option<String>,

    /// Include child groups when expanding
    #[arg(long)]
    pub recurse: bool,
}

#[derive(clap::Args)]
pub struct ManagementGroupCreateArgs {
    /// Management group name
    #[arg(short, long)]
    pub name: String,

    /// Display name for the management group
    #[arg(short, long)]
    pub display_name: Option<String>,

    /// Parent management group ID
    #[arg(short, long)]
    pub parent: Option<String>,
}

#[derive(clap::Args)]
pub struct ManagementGroupDeleteArgs {
    /// Management group name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct ManagementGroupCheckNameArgs {
    /// Management group name to check
    #[arg(short, long)]
    pub name: String,
}

#[derive(Subcommand)]
pub enum ManagementGroupSubscriptionCommands {
    /// Add a subscription to a management group
    Add(ManagementGroupSubscriptionAddArgs),

    /// Remove a subscription from a management group
    Remove(ManagementGroupSubscriptionRemoveArgs),

    /// Show a subscription under a management group
    #[command(name = "show-sub-under-mg")]
    ShowSubUnderMg(ManagementGroupSubscriptionShowArgs),
}

#[derive(clap::Args)]
pub struct ManagementGroupSubscriptionAddArgs {
    /// Management group name
    #[arg(short, long)]
    pub name: String,

    /// Subscription ID
    #[arg(short, long)]
    pub subscription: String,
}

#[derive(clap::Args)]
pub struct ManagementGroupSubscriptionRemoveArgs {
    /// Management group name
    #[arg(short, long)]
    pub name: String,

    /// Subscription ID
    #[arg(short, long)]
    pub subscription: String,
}

#[derive(clap::Args)]
pub struct ManagementGroupSubscriptionShowArgs {
    /// Management group name
    #[arg(short, long)]
    pub name: String,

    /// Subscription ID
    #[arg(short, long)]
    pub subscription: String,
}

#[derive(Subcommand)]
pub enum ManagementGroupEntitiesCommands {
    /// List entities for management groups
    List,
}

#[derive(Subcommand)]
pub enum ManagementGroupHierarchySettingsCommands {
    /// List hierarchy settings for a management group
    List(ManagementGroupHierarchySettingsListArgs),

    /// Create hierarchy settings for a management group
    Create(ManagementGroupHierarchySettingsCreateArgs),

    /// Delete hierarchy settings for a management group
    Delete(ManagementGroupHierarchySettingsDeleteArgs),
}

#[derive(clap::Args)]
pub struct ManagementGroupHierarchySettingsListArgs {
    /// Management group name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct ManagementGroupHierarchySettingsCreateArgs {
    /// Management group name
    #[arg(short, long)]
    pub name: String,

    /// Require authorization for group creation under this management group
    #[arg(long)]
    pub require_authorization: Option<bool>,

    /// Default management group for new subscriptions
    #[arg(long)]
    pub default_management_group: Option<String>,
}

#[derive(clap::Args)]
pub struct ManagementGroupHierarchySettingsDeleteArgs {
    /// Management group name
    #[arg(short, long)]
    pub name: String,
}

#[derive(Subcommand)]
pub enum ManagementGroupTenantBackfillCommands {
    /// Get the backfill status for the tenant
    Get,

    /// Start backfilling subscriptions for the tenant
    Start,
}

// ── Webapp commands ─────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum WebappCommands {
    /// List web apps
    List(WebappListArgs),
    /// Show details of a web app
    Show(WebappShowArgs),
    /// Create a web app
    Create(WebappCreateArgs),
    /// Delete a web app
    Delete(WebappDeleteArgs),
    /// Stop a web app
    Stop(WebappStopArgs),
    /// Start a web app
    Start(WebappStartArgs),
    /// Restart a web app
    Restart(WebappRestartArgs),
    /// Update a web app
    Update(WebappUpdateArgs),
    /// List available runtimes
    #[command(name = "list-runtimes")]
    ListRuntimes(WebappListRuntimesArgs),
    /// Deploy to a web app
    Deploy(WebappDeployArgs),
    /// Manage web app identity
    #[command(subcommand)]
    Identity(WebappIdentityCommands),
    /// Configure a web app
    #[command(subcommand)]
    Config(WebappConfigCommands),
    /// Manage web app deployments
    #[command(subcommand)]
    Deployment(WebappDeploymentCommands),
    /// Manage CORS settings
    #[command(subcommand)]
    Cors(WebappCorsCommands),
    /// Manage VNet integrations
    #[command(subcommand, name = "vnet-integration")]
    VnetIntegration(WebappVnetIntegrationCommands),
    /// Manage web app logs
    #[command(subcommand)]
    Log(WebappLogCommands),
    /// Manage deleted web apps
    #[command(subcommand)]
    Deleted(WebappDeletedCommands),
    /// Manage continuous webjobs
    #[command(subcommand, name = "webjob-continuous")]
    WebjobContinuous(WebappWebjobContinuousCommands),
    /// Manage triggered webjobs
    #[command(subcommand, name = "webjob-triggered")]
    WebjobTriggered(WebappWebjobTriggeredCommands),
    /// Manage traffic routing
    #[command(subcommand, name = "traffic-routing")]
    TrafficRouting(WebappTrafficRoutingCommands),
}

#[derive(clap::Args)]
pub struct WebappListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct WebappShowArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappCreateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// App Service plan name or ID
    #[arg(short, long)]
    pub plan: String,
    /// Runtime stack (e.g. "PYTHON:3.11")
    #[arg(short, long)]
    pub runtime: Option<String>,
    /// Startup file or command
    #[arg(long)]
    pub startup_file: Option<String>,
    /// Container image name
    #[arg(long = "deployment-container-image-name")]
    pub deployment_container_image_name: Option<String>,
    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct WebappDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct WebappStopArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappStartArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappRestartArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappUpdateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Property overrides in key=value format
    #[arg(long, num_args = 1..)]
    pub set: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct WebappListRuntimesArgs {
    /// Filter by OS type (linux, windows)
    #[arg(long)]
    pub os: Option<String>,
}

#[derive(clap::Args)]
pub struct WebappDeployArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Path to the artifact to deploy
    #[arg(long)]
    pub src_path: String,
    /// Deployment type (e.g. zip, war, jar, ear, static)
    #[arg(long = "type")]
    pub deploy_type: Option<String>,
}

// Webapp Identity

#[derive(Subcommand)]
pub enum WebappIdentityCommands {
    /// Assign a managed identity to the web app
    Assign(WebappIdentityAssignArgs),
    /// Remove managed identity from the web app
    Remove(WebappIdentityRemoveArgs),
}

#[derive(clap::Args)]
pub struct WebappIdentityAssignArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Identity type (SystemAssigned, UserAssigned)
    #[arg(long)]
    pub identity_type: Option<String>,
}

#[derive(clap::Args)]
pub struct WebappIdentityRemoveArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

// Webapp Config

#[derive(Subcommand)]
pub enum WebappConfigCommands {
    /// Update web app configuration
    Set(WebappConfigSetArgs),
    /// Manage web app application settings
    #[command(subcommand)]
    Appsettings(WebappConfigAppsettingsCommands),
    /// Manage web app connection strings
    #[command(subcommand, name = "connection-string")]
    ConnectionString(WebappConfigConnstrCommands),
    /// Manage web app hostnames
    #[command(subcommand)]
    Hostname(WebappConfigHostnameCommands),
    /// Manage SSL bindings
    #[command(subcommand)]
    Ssl(WebappConfigSslCommands),
    /// Manage access restrictions
    #[command(subcommand, name = "access-restriction")]
    AccessRestriction(WebappConfigAccessRestrictionCommands),
    /// Manage container settings
    #[command(subcommand)]
    Container(WebappConfigContainerCommands),
    /// Manage backups
    #[command(subcommand)]
    Backup(WebappConfigBackupCommands),
}

#[derive(clap::Args)]
pub struct WebappConfigSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Property overrides in key=value format
    #[arg(long, num_args = 1..)]
    pub set: Vec<String>,
}

#[derive(Subcommand)]
pub enum WebappConfigAppsettingsCommands {
    /// List application settings
    List(WebappConfigAppsettingsListArgs),
    /// Set application settings
    Set(WebappConfigAppsettingsSetArgs),
    /// Delete application settings
    Delete(WebappConfigAppsettingsDeleteArgs),
}

#[derive(clap::Args)]
pub struct WebappConfigAppsettingsListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappConfigAppsettingsSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Space-separated settings in key=value format
    #[arg(long, num_args = 1..)]
    pub settings: Vec<String>,
}

#[derive(clap::Args)]
pub struct WebappConfigAppsettingsDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Space-separated setting keys to delete
    #[arg(long = "setting-names", num_args = 1..)]
    pub setting_names: Vec<String>,
}

// Webapp Config Connection Strings

#[derive(Subcommand)]
pub enum WebappConfigConnstrCommands {
    /// List connection strings
    List(WebappConfigConnstrListArgs),
    /// Set connection strings
    Set(WebappConfigConnstrSetArgs),
    /// Delete connection strings
    Delete(WebappConfigConnstrDeleteArgs),
}

#[derive(clap::Args)]
pub struct WebappConfigConnstrListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappConfigConnstrSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Connection string type (e.g. SQLAzure, SQLServer, Custom, MySql, PostgreSQL)
    #[arg(short, long = "connection-string-type")]
    pub connection_string_type: String,
    /// Space-separated connection strings in key=value format
    #[arg(long, num_args = 1..)]
    pub settings: Vec<String>,
}

#[derive(clap::Args)]
pub struct WebappConfigConnstrDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Space-separated connection string keys to delete
    #[arg(long = "setting-names", num_args = 1..)]
    pub setting_names: Vec<String>,
}

// Webapp Config Hostname

#[derive(Subcommand)]
pub enum WebappConfigHostnameCommands {
    /// List hostnames for a web app
    List(WebappConfigHostnameListArgs),
    /// Add a hostname to a web app
    Add(WebappConfigHostnameAddArgs),
    /// Delete a hostname from a web app
    Delete(WebappConfigHostnameDeleteArgs),
}

#[derive(clap::Args)]
pub struct WebappConfigHostnameListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappConfigHostnameAddArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Hostname to bind
    #[arg(long)]
    pub hostname: String,
}

#[derive(clap::Args)]
pub struct WebappConfigHostnameDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Hostname to remove
    #[arg(long)]
    pub hostname: String,
}

// Webapp Deployment

#[derive(Subcommand)]
pub enum WebappDeploymentCommands {
    /// List publishing profiles for a web app
    #[command(name = "list-publishing-profiles")]
    ListPublishingProfiles(WebappDeploymentListPublishingProfilesArgs),
    /// Manage deployment sources
    #[command(subcommand)]
    Source(WebappDeploymentSourceCommands),
    /// Manage deployment slots
    #[command(subcommand)]
    Slot(WebappDeploymentSlotCommands),
    /// Manage GitHub Actions integration
    #[command(subcommand, name = "github-actions")]
    GithubActions(WebappDeploymentGithubActionsCommands),
    /// Manage container deployment
    #[command(subcommand)]
    Container(WebappDeploymentContainerCommands),
    /// Manage deployment user
    #[command(subcommand)]
    User(WebappDeploymentUserCommands),
}

#[derive(clap::Args)]
pub struct WebappDeploymentListPublishingProfilesArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(Subcommand)]
pub enum WebappDeploymentSourceCommands {
    /// Show deployment source config
    Show(WebappDeploymentSourceShowArgs),
    /// Deploy from a zip file
    #[command(name = "config-zip")]
    ConfigZip(WebappDeploymentSourceConfigZipArgs),
    /// Delete deployment source config
    Delete(WebappDeploymentSourceDeleteArgs),
    /// Sync deployment source
    Sync(WebappDeploymentSourceSyncArgs),
}

#[derive(clap::Args)]
pub struct WebappDeploymentSourceShowArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappDeploymentSourceConfigZipArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Path to the zip file to deploy
    #[arg(long)]
    pub src: String,
}

#[derive(clap::Args)]
pub struct WebappDeploymentSourceDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappDeploymentSourceSyncArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

// Webapp CORS

#[derive(Subcommand)]
pub enum WebappCorsCommands {
    /// Add allowed origins
    Add(WebappCorsAddArgs),
    /// Remove allowed origins
    Remove(WebappCorsRemoveArgs),
}

#[derive(clap::Args)]
pub struct WebappCorsAddArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Space-separated allowed origins
    #[arg(long = "allowed-origins", num_args = 1..)]
    pub allowed_origins: Vec<String>,
}

#[derive(clap::Args)]
pub struct WebappCorsRemoveArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Space-separated allowed origins to remove
    #[arg(long = "allowed-origins", num_args = 1..)]
    pub allowed_origins: Vec<String>,
}

// Webapp Config SSL

#[derive(Subcommand)]
pub enum WebappConfigSslCommands {
    /// List SSL bindings
    List(WebappConfigSslListArgs),
    /// Bind an SSL certificate
    Bind(WebappConfigSslBindArgs),
    /// Unbind an SSL certificate
    Unbind(WebappConfigSslUnbindArgs),
}

#[derive(clap::Args)]
pub struct WebappConfigSslListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappConfigSslBindArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// SSL type (SniEnabled or IpBasedEnabled)
    #[arg(long)]
    pub ssl_type: String,
    /// Certificate thumbprint
    #[arg(long)]
    pub certificate_thumbprint: String,
    /// Hostname to bind SSL to
    #[arg(long)]
    pub hostname: String,
}

#[derive(clap::Args)]
pub struct WebappConfigSslUnbindArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Hostname to unbind SSL from
    #[arg(long)]
    pub hostname: String,
}

// Webapp Config Access Restriction

#[derive(Subcommand)]
pub enum WebappConfigAccessRestrictionCommands {
    /// Add an access restriction rule
    Add(WebappConfigAccessRestrictionAddArgs),
    /// Remove an access restriction rule
    Remove(WebappConfigAccessRestrictionRemoveArgs),
    /// Set access restriction configuration
    Set(WebappConfigAccessRestrictionSetArgs),
}

#[derive(clap::Args)]
pub struct WebappConfigAccessRestrictionAddArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Rule name
    #[arg(long)]
    pub rule_name: String,
    /// Priority (100-65000)
    #[arg(long)]
    pub priority: u32,
    /// Action (Allow or Deny)
    #[arg(long)]
    pub action: String,
    /// IP address or CIDR
    #[arg(long)]
    pub ip_address: Option<String>,
}

#[derive(clap::Args)]
pub struct WebappConfigAccessRestrictionRemoveArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Rule name to remove
    #[arg(long)]
    pub rule_name: String,
}

#[derive(clap::Args)]
pub struct WebappConfigAccessRestrictionSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Use same restrictions for SCM site
    #[arg(long)]
    pub use_same_restrictions_for_scm_site: bool,
}

// Webapp Config Container

#[derive(Subcommand)]
pub enum WebappConfigContainerCommands {
    /// Set container settings
    Set(WebappConfigContainerSetArgs),
    /// Delete container settings
    Delete(WebappConfigContainerDeleteArgs),
}

#[derive(clap::Args)]
pub struct WebappConfigContainerSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Container image name
    #[arg(long = "docker-custom-image-name")]
    pub docker_custom_image_name: String,
    /// Container registry URL
    #[arg(long = "docker-registry-server-url")]
    pub docker_registry_server_url: Option<String>,
    /// Container registry username
    #[arg(long = "docker-registry-server-user")]
    pub docker_registry_server_user: Option<String>,
    /// Container registry password
    #[arg(long = "docker-registry-server-password")]
    pub docker_registry_server_password: Option<String>,
}

#[derive(clap::Args)]
pub struct WebappConfigContainerDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

// Webapp Config Backup

#[derive(Subcommand)]
pub enum WebappConfigBackupCommands {
    /// List backups
    List(WebappConfigBackupListArgs),
    /// Create a backup
    Create(WebappConfigBackupCreateArgs),
    /// Delete a backup
    Delete(WebappConfigBackupDeleteArgs),
    /// Restore from a backup
    Restore(WebappConfigBackupRestoreArgs),
}

#[derive(clap::Args)]
pub struct WebappConfigBackupListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappConfigBackupCreateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Backup name
    #[arg(long)]
    pub backup_name: String,
    /// SAS URL for the storage container
    #[arg(long)]
    pub storage_account_url: String,
}

#[derive(clap::Args)]
pub struct WebappConfigBackupDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Backup ID
    #[arg(long)]
    pub backup_id: String,
}

#[derive(clap::Args)]
pub struct WebappConfigBackupRestoreArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Backup ID to restore
    #[arg(long)]
    pub backup_id: String,
    /// SAS URL for the storage container
    #[arg(long)]
    pub storage_account_url: String,
}

// Webapp VNet Integration

#[derive(Subcommand)]
pub enum WebappVnetIntegrationCommands {
    /// List VNet integrations
    List(WebappVnetIntegrationListArgs),
    /// Add a VNet integration
    Add(WebappVnetIntegrationAddArgs),
    /// Remove a VNet integration
    Remove(WebappVnetIntegrationRemoveArgs),
}

#[derive(clap::Args)]
pub struct WebappVnetIntegrationListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappVnetIntegrationAddArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// VNet name
    #[arg(long)]
    pub vnet: String,
    /// Subnet name
    #[arg(long)]
    pub subnet: String,
}

#[derive(clap::Args)]
pub struct WebappVnetIntegrationRemoveArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// VNet name
    #[arg(long)]
    pub vnet: String,
}

// Webapp Log

#[derive(Subcommand)]
pub enum WebappLogCommands {
    /// Configure logging
    Config(WebappLogConfigArgs),
    /// Download log files
    Download(WebappLogDownloadArgs),
    /// Start live log tail
    Tail(WebappLogTailArgs),
}

#[derive(clap::Args)]
pub struct WebappLogConfigArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Application logging (filesystem, azureblobstorage, off)
    #[arg(long)]
    pub application_logging: Option<String>,
    /// Web server logging (filesystem, off)
    #[arg(long)]
    pub web_server_logging: Option<String>,
    /// Log level (Error, Warning, Information, Verbose)
    #[arg(long)]
    pub level: Option<String>,
}

#[derive(clap::Args)]
pub struct WebappLogDownloadArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappLogTailArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

// Webapp Deleted

#[derive(Subcommand)]
pub enum WebappDeletedCommands {
    /// List deleted web apps
    List(WebappDeletedListArgs),
    /// Restore a deleted web app
    Restore(WebappDeletedRestoreArgs),
}

#[derive(clap::Args)]
pub struct WebappDeletedListArgs {
    /// Resource group (optional filter)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct WebappDeletedRestoreArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name to restore into
    #[arg(short, long)]
    pub name: String,
    /// Deleted site ID
    #[arg(long)]
    pub deleted_id: String,
}

// Webapp Webjob Continuous

#[derive(Subcommand)]
pub enum WebappWebjobContinuousCommands {
    /// List continuous webjobs
    List(WebappWebjobContinuousListArgs),
    /// Start a continuous webjob
    Start(WebappWebjobContinuousStartArgs),
    /// Stop a continuous webjob
    Stop(WebappWebjobContinuousStopArgs),
    /// Remove a continuous webjob
    Remove(WebappWebjobContinuousRemoveArgs),
}

#[derive(clap::Args)]
pub struct WebappWebjobContinuousListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappWebjobContinuousStartArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Webjob name
    #[arg(long)]
    pub webjob_name: String,
}

#[derive(clap::Args)]
pub struct WebappWebjobContinuousStopArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Webjob name
    #[arg(long)]
    pub webjob_name: String,
}

#[derive(clap::Args)]
pub struct WebappWebjobContinuousRemoveArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Webjob name
    #[arg(long)]
    pub webjob_name: String,
}

// Webapp Webjob Triggered

#[derive(Subcommand)]
pub enum WebappWebjobTriggeredCommands {
    /// List triggered webjobs
    List(WebappWebjobTriggeredListArgs),
    /// Run a triggered webjob
    Run(WebappWebjobTriggeredRunArgs),
    /// Remove a triggered webjob
    Remove(WebappWebjobTriggeredRemoveArgs),
    /// Show triggered webjob history/logs
    Log(WebappWebjobTriggeredLogArgs),
}

#[derive(clap::Args)]
pub struct WebappWebjobTriggeredListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappWebjobTriggeredRunArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Webjob name
    #[arg(long)]
    pub webjob_name: String,
}

#[derive(clap::Args)]
pub struct WebappWebjobTriggeredRemoveArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Webjob name
    #[arg(long)]
    pub webjob_name: String,
}

#[derive(clap::Args)]
pub struct WebappWebjobTriggeredLogArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Webjob name
    #[arg(long)]
    pub webjob_name: String,
}

// Webapp Traffic Routing

#[derive(Subcommand)]
pub enum WebappTrafficRoutingCommands {
    /// Set traffic routing distribution
    Set(WebappTrafficRoutingSetArgs),
    /// Clear traffic routing rules
    Clear(WebappTrafficRoutingClearArgs),
}

#[derive(clap::Args)]
pub struct WebappTrafficRoutingSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Distribution in format slot=percentage (e.g. staging=25)
    #[arg(long, num_args = 1..)]
    pub distribution: Vec<String>,
}

#[derive(clap::Args)]
pub struct WebappTrafficRoutingClearArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

// Webapp Deployment Slot

#[derive(Subcommand)]
pub enum WebappDeploymentSlotCommands {
    /// List deployment slots
    List(WebappDeploymentSlotListArgs),
    /// Create a deployment slot
    Create(WebappDeploymentSlotCreateArgs),
    /// Delete a deployment slot
    Delete(WebappDeploymentSlotDeleteArgs),
    /// Swap deployment slots
    Swap(WebappDeploymentSlotSwapArgs),
}

#[derive(clap::Args)]
pub struct WebappDeploymentSlotListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct WebappDeploymentSlotCreateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Slot name
    #[arg(short, long)]
    pub slot: String,
}

#[derive(clap::Args)]
pub struct WebappDeploymentSlotDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Slot name
    #[arg(short, long)]
    pub slot: String,
}

#[derive(clap::Args)]
pub struct WebappDeploymentSlotSwapArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Source slot name
    #[arg(short, long)]
    pub slot: String,
    /// Target slot name (defaults to production)
    #[arg(long, default_value = "production")]
    pub target_slot: String,
}

// Webapp Deployment GitHub Actions

#[derive(Subcommand)]
pub enum WebappDeploymentGithubActionsCommands {
    /// Configure GitHub Actions integration
    Add(WebappDeploymentGithubActionsAddArgs),
    /// Remove GitHub Actions integration
    Remove(WebappDeploymentGithubActionsRemoveArgs),
}

#[derive(clap::Args)]
pub struct WebappDeploymentGithubActionsAddArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// GitHub repository (owner/repo)
    #[arg(long)]
    pub repo: String,
    /// Branch name
    #[arg(long)]
    pub branch: String,
    /// GitHub personal access token
    #[arg(long)]
    pub token: String,
}

#[derive(clap::Args)]
pub struct WebappDeploymentGithubActionsRemoveArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

// Webapp Deployment Container

#[derive(Subcommand)]
pub enum WebappDeploymentContainerCommands {
    /// Configure continuous deployment for containers
    Config(WebappDeploymentContainerConfigArgs),
    /// Show the CD webhook URL
    #[command(name = "show-cd-url")]
    ShowCdUrl(WebappDeploymentContainerShowCdUrlArgs),
}

#[derive(clap::Args)]
pub struct WebappDeploymentContainerConfigArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
    /// Enable continuous deployment
    #[arg(long)]
    pub enable_cd: bool,
}

#[derive(clap::Args)]
pub struct WebappDeploymentContainerShowCdUrlArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Web app name
    #[arg(short, long)]
    pub name: String,
}

// Webapp Deployment User

#[derive(Subcommand)]
pub enum WebappDeploymentUserCommands {
    /// Set deployment user credentials
    Set(WebappDeploymentUserSetArgs),
}

#[derive(clap::Args)]
pub struct WebappDeploymentUserSetArgs {
    /// Username for deployment
    #[arg(long = "user-name")]
    pub user_name: String,
    /// Password for deployment
    #[arg(long)]
    pub password: String,
}

// ── Functionapp commands ───────────────────────────────────────────

#[derive(Subcommand)]
pub enum FunctionappCommands {
    /// List function apps
    List(FunctionappListArgs),
    /// Show details of a function app
    Show(FunctionappShowArgs),
    /// Create a function app
    Create(FunctionappCreateArgs),
    /// Delete a function app
    Delete(FunctionappDeleteArgs),
    /// Stop a function app
    Stop(FunctionappStopArgs),
    /// Start a function app
    Start(FunctionappStartArgs),
    /// Restart a function app
    Restart(FunctionappRestartArgs),
    /// Update a function app
    Update(FunctionappUpdateArgs),
    /// List available function runtimes
    #[command(name = "list-runtimes")]
    ListRuntimes,
    /// Deploy to a function app
    Deploy(FunctionappDeployArgs),
    /// Configure a function app
    #[command(subcommand)]
    Config(FunctionappConfigCommands),
    /// Manage function app host keys
    #[command(subcommand)]
    Keys(FunctionappKeysCommands),
    /// Manage individual functions
    #[command(subcommand)]
    Function(FunctionappFunctionCommands),
    /// Manage function app deployments
    #[command(subcommand)]
    Deployment(FunctionappDeploymentCommands),
    /// Manage function app plans (consumption/flex)
    #[command(subcommand)]
    Plan(FunctionappPlanCommands),
    /// Manage deployment slots
    #[command(subcommand, name = "deployment-slot")]
    DeploymentSlot(FunctionappDeploymentSlotCommands),
    /// Manage virtual network integrations
    #[command(subcommand, name = "vnet-integration")]
    VnetIntegration(FunctionappVnetIntegrationCommands),
    /// Manage scale configuration
    #[command(subcommand, name = "scale-config")]
    ScaleConfig(FunctionappScaleConfigCommands),
}

#[derive(clap::Args)]
pub struct FunctionappListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct FunctionappShowArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FunctionappCreateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// App Service plan name or ID
    #[arg(short, long)]
    pub plan: Option<String>,
    /// Use consumption plan
    #[arg(long)]
    pub consumption_plan_location: Option<String>,
    /// Runtime stack (e.g. "node", "python", "dotnet")
    #[arg(short, long)]
    pub runtime: Option<String>,
    /// OS type (linux, windows)
    #[arg(long)]
    pub os_type: Option<String>,
    /// Storage account name or connection string
    #[arg(short, long)]
    pub storage_account: Option<String>,
    /// Location
    #[arg(short, long)]
    pub location: Option<String>,
    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct FunctionappDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct FunctionappStopArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FunctionappStartArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FunctionappRestartArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FunctionappUpdateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Property overrides in key=value format
    #[arg(long, num_args = 1..)]
    pub set: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct FunctionappDeployArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Path to the artifact to deploy
    #[arg(long)]
    pub src_path: String,
    /// Deployment type (e.g. zip)
    #[arg(long = "type")]
    pub deploy_type: Option<String>,
}

// Functionapp Config

#[derive(Subcommand)]
pub enum FunctionappConfigCommands {
    /// Manage function app application settings
    #[command(subcommand)]
    Appsettings(FunctionappConfigAppsettingsCommands),
}

#[derive(Subcommand)]
pub enum FunctionappConfigAppsettingsCommands {
    /// List application settings
    List(FunctionappConfigAppsettingsListArgs),
    /// Set application settings
    Set(FunctionappConfigAppsettingsSetArgs),
    /// Delete application settings
    Delete(FunctionappConfigAppsettingsDeleteArgs),
}

#[derive(clap::Args)]
pub struct FunctionappConfigAppsettingsListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FunctionappConfigAppsettingsSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Space-separated settings in key=value format
    #[arg(long, num_args = 1..)]
    pub settings: Vec<String>,
}

#[derive(clap::Args)]
pub struct FunctionappConfigAppsettingsDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Space-separated setting keys to delete
    #[arg(long = "setting-names", num_args = 1..)]
    pub setting_names: Vec<String>,
}

// Functionapp Keys

#[derive(Subcommand)]
pub enum FunctionappKeysCommands {
    /// List host keys
    List(FunctionappKeysListArgs),
    /// Set a host function key
    Set(FunctionappKeysSetArgs),
    /// Delete a host function key
    Delete(FunctionappKeysDeleteArgs),
}

#[derive(clap::Args)]
pub struct FunctionappKeysListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FunctionappKeysSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Key name
    #[arg(long)]
    pub key_name: String,
    /// Key value (generated if not provided)
    #[arg(long)]
    pub key_value: Option<String>,
}

#[derive(clap::Args)]
pub struct FunctionappKeysDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Key name
    #[arg(long)]
    pub key_name: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

// Functionapp Function

#[derive(Subcommand)]
pub enum FunctionappFunctionCommands {
    /// List functions in a function app
    List(FunctionappFunctionListArgs),
    /// Show details of a function
    Show(FunctionappFunctionShowArgs),
    /// Delete a function
    Delete(FunctionappFunctionDeleteArgs),
    /// Manage function keys
    #[command(subcommand)]
    Keys(FunctionappFunctionKeysCommands),
}

#[derive(clap::Args)]
pub struct FunctionappFunctionListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FunctionappFunctionShowArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Function name
    #[arg(long)]
    pub function_name: String,
}

#[derive(clap::Args)]
pub struct FunctionappFunctionDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Function name
    #[arg(long)]
    pub function_name: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

// Functionapp Function Keys

#[derive(Subcommand)]
pub enum FunctionappFunctionKeysCommands {
    /// List keys for a function
    List(FunctionappFunctionKeysListArgs),
    /// Set a key for a function
    Set(FunctionappFunctionKeysSetArgs),
    /// Delete a key for a function
    Delete(FunctionappFunctionKeysDeleteArgs),
}

#[derive(clap::Args)]
pub struct FunctionappFunctionKeysListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Function name
    #[arg(long)]
    pub function_name: String,
}

#[derive(clap::Args)]
pub struct FunctionappFunctionKeysSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Function name
    #[arg(long)]
    pub function_name: String,
    /// Key name
    #[arg(long)]
    pub key_name: String,
    /// Key value (generated if not provided)
    #[arg(long)]
    pub key_value: Option<String>,
}

#[derive(clap::Args)]
pub struct FunctionappFunctionKeysDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Function name
    #[arg(long)]
    pub function_name: String,
    /// Key name
    #[arg(long)]
    pub key_name: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

// Functionapp Deployment

#[derive(Subcommand)]
pub enum FunctionappDeploymentCommands {
    /// List publishing profiles for a function app
    #[command(name = "list-publishing-profiles")]
    ListPublishingProfiles(FunctionappDeploymentListPublishingProfilesArgs),
    /// Manage deployment sources
    #[command(subcommand)]
    Source(FunctionappDeploymentSourceCommands),
}

#[derive(clap::Args)]
pub struct FunctionappDeploymentListPublishingProfilesArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(Subcommand)]
pub enum FunctionappDeploymentSourceCommands {
    /// Deploy from a zip file
    #[command(name = "config-zip")]
    ConfigZip(FunctionappDeploymentSourceConfigZipArgs),
}

#[derive(clap::Args)]
pub struct FunctionappDeploymentSourceConfigZipArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Path to the zip file to deploy
    #[arg(long)]
    pub src: String,
}

// ── Functionapp Plan (consumption/flex) ─────────────────────────────

#[derive(Subcommand)]
pub enum FunctionappPlanCommands {
    /// List function app plans
    List(FunctionappPlanListArgs),
    /// Show a function app plan
    Show(FunctionappPlanShowArgs),
    /// Create a function app plan
    Create(FunctionappPlanCreateArgs),
    /// Delete a function app plan
    Delete(FunctionappPlanDeleteArgs),
    /// Update a function app plan
    Update(FunctionappPlanUpdateArgs),
}

#[derive(clap::Args)]
pub struct FunctionappPlanListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct FunctionappPlanShowArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Plan name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FunctionappPlanCreateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Plan name
    #[arg(short, long)]
    pub name: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Pricing tier (e.g. Y1, EP1, EP2)
    #[arg(long)]
    pub sku: Option<String>,
    /// Create a Linux plan
    #[arg(long)]
    pub is_linux: bool,
    /// Maximum elastic worker count
    #[arg(long)]
    pub max_burst: Option<i64>,
    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct FunctionappPlanDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Plan name
    #[arg(short, long)]
    pub name: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct FunctionappPlanUpdateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Plan name
    #[arg(short, long)]
    pub name: String,
    /// Pricing tier
    #[arg(long)]
    pub sku: Option<String>,
    /// Maximum elastic worker count
    #[arg(long)]
    pub max_burst: Option<i64>,
    /// Number of workers
    #[arg(long)]
    pub number_of_workers: Option<i64>,
}

// ── Functionapp Deployment Slot ─────────────────────────────────────

#[derive(Subcommand)]
pub enum FunctionappDeploymentSlotCommands {
    /// List deployment slots
    List(FunctionappDeploymentSlotListArgs),
    /// Create a deployment slot
    Create(FunctionappDeploymentSlotCreateArgs),
    /// Delete a deployment slot
    Delete(FunctionappDeploymentSlotDeleteArgs),
    /// Swap deployment slots
    Swap(FunctionappDeploymentSlotSwapArgs),
}

#[derive(clap::Args)]
pub struct FunctionappDeploymentSlotListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FunctionappDeploymentSlotCreateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Slot name
    #[arg(short, long)]
    pub slot: String,
}

#[derive(clap::Args)]
pub struct FunctionappDeploymentSlotDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Slot name
    #[arg(short, long)]
    pub slot: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct FunctionappDeploymentSlotSwapArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Source slot name
    #[arg(short, long)]
    pub slot: String,
    /// Target slot (default: production)
    #[arg(long)]
    pub target_slot: Option<String>,
}

// ── Functionapp VNet Integration ────────────────────────────────────

#[derive(Subcommand)]
pub enum FunctionappVnetIntegrationCommands {
    /// List virtual network integrations
    List(FunctionappVnetIntegrationListArgs),
    /// Add a virtual network integration
    Add(FunctionappVnetIntegrationAddArgs),
    /// Remove a virtual network integration
    Remove(FunctionappVnetIntegrationRemoveArgs),
}

#[derive(clap::Args)]
pub struct FunctionappVnetIntegrationListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct FunctionappVnetIntegrationAddArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Virtual network name
    #[arg(long)]
    pub vnet: String,
    /// Subnet name or resource ID
    #[arg(long)]
    pub subnet: String,
}

#[derive(clap::Args)]
pub struct FunctionappVnetIntegrationRemoveArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
}

// ── Functionapp Scale Config ────────────────────────────────────────

#[derive(Subcommand)]
pub enum FunctionappScaleConfigCommands {
    /// Set scale configuration
    Set(FunctionappScaleConfigSetArgs),
}

#[derive(clap::Args)]
pub struct FunctionappScaleConfigSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Function app name
    #[arg(short, long)]
    pub name: String,
    /// Maximum burst (scale limit)
    #[arg(long)]
    pub max_burst: Option<i64>,
    /// Scale trigger type
    #[arg(long)]
    pub trigger_type: Option<String>,
    /// Scale trigger value
    #[arg(long)]
    pub trigger_value: Option<String>,
}

// ── App Service commands ────────────────────────────────────────────

#[derive(Subcommand)]
pub enum AppserviceCommands {
    /// List available locations for App Service
    #[command(name = "list-locations")]
    ListLocations,
    /// Manage App Service plans
    #[command(subcommand)]
    Plan(AppservicePlanCommands),
    /// Manage App Service Environments (ASE)
    #[command(subcommand)]
    Ase(AppserviceAseCommands),
    /// Manage custom domains
    #[command(subcommand)]
    Domain(AppserviceDomainCommands),
}

#[derive(Subcommand)]
pub enum AppservicePlanCommands {
    /// List App Service plans
    List(AppservicePlanListArgs),
    /// Show an App Service plan
    Show(AppservicePlanShowArgs),
    /// Create an App Service plan
    Create(AppservicePlanCreateArgs),
    /// Delete an App Service plan
    Delete(AppservicePlanDeleteArgs),
    /// Update an App Service plan
    Update(AppservicePlanUpdateArgs),
    /// Manage App Service plan identity
    #[command(subcommand)]
    Identity(AppservicePlanIdentityCommands),
}

#[derive(clap::Args)]
pub struct AppservicePlanListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct AppservicePlanShowArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Plan name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct AppservicePlanCreateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Plan name
    #[arg(short, long)]
    pub name: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Pricing tier (e.g. F1, B1, S1, P1V2)
    #[arg(long)]
    pub sku: Option<String>,
    /// Create a Linux App Service plan
    #[arg(long)]
    pub is_linux: bool,
    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct AppservicePlanDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Plan name
    #[arg(short, long)]
    pub name: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct AppservicePlanUpdateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Plan name
    #[arg(short, long)]
    pub name: String,
    /// Pricing tier
    #[arg(long)]
    pub sku: Option<String>,
    /// Number of workers
    #[arg(long)]
    pub number_of_workers: Option<i64>,
}

// ── Appservice ASE ──────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum AppserviceAseCommands {
    /// List App Service Environments
    List(AppserviceAseListArgs),
    /// Show an App Service Environment
    Show(AppserviceAseShowArgs),
    /// Create an App Service Environment
    Create(AppserviceAseCreateArgs),
    /// Delete an App Service Environment
    Delete(AppserviceAseDeleteArgs),
    /// Update an App Service Environment
    Update(AppserviceAseUpdateArgs),
    /// List VIPs and addresses for an ASE
    #[command(name = "list-addresses")]
    ListAddresses(AppserviceAseListAddressesArgs),
    /// List App Service plans in an ASE
    #[command(name = "list-plans")]
    ListPlans(AppserviceAseListPlansArgs),
    /// Upgrade an App Service Environment
    Upgrade(AppserviceAseUpgradeArgs),
}

#[derive(clap::Args)]
pub struct AppserviceAseListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct AppserviceAseShowArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// ASE name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct AppserviceAseCreateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// ASE name
    #[arg(short, long)]
    pub name: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Virtual network name
    #[arg(long)]
    pub vnet_name: String,
    /// Subnet name or ID
    #[arg(long)]
    pub subnet: String,
    /// ASE kind (e.g. ASEv3)
    #[arg(long)]
    pub kind: Option<String>,
    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct AppserviceAseDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// ASE name
    #[arg(short, long)]
    pub name: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct AppserviceAseUpdateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// ASE name
    #[arg(short, long)]
    pub name: String,
    /// Property=value pairs to set
    #[arg(long, num_args = 1..)]
    pub set: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct AppserviceAseListAddressesArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// ASE name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct AppserviceAseListPlansArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// ASE name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct AppserviceAseUpgradeArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// ASE name
    #[arg(short, long)]
    pub name: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

// ── Appservice Domain ───────────────────────────────────────────────

#[derive(Subcommand)]
pub enum AppserviceDomainCommands {
    /// Register a custom domain
    Create(AppserviceDomainCreateArgs),
    /// Show terms and agreements for domain registration
    #[command(name = "show-terms")]
    ShowTerms,
}

#[derive(clap::Args)]
pub struct AppserviceDomainCreateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Domain hostname (e.g. example.com)
    #[arg(long)]
    pub hostname: String,
    /// Contact info as JSON string
    #[arg(long)]
    pub contact_info: String,
}

// ── Appservice Plan Identity ────────────────────────────────────────

#[derive(Subcommand)]
pub enum AppservicePlanIdentityCommands {
    /// Assign a managed identity to the plan
    Assign(AppservicePlanIdentityAssignArgs),
    /// Remove managed identity from the plan
    Remove(AppservicePlanIdentityRemoveArgs),
}

#[derive(clap::Args)]
pub struct AppservicePlanIdentityAssignArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Plan name
    #[arg(short, long)]
    pub name: String,
    /// Identity type (e.g. SystemAssigned, UserAssigned)
    #[arg(long)]
    pub identity_type: Option<String>,
}

#[derive(clap::Args)]
pub struct AppservicePlanIdentityRemoveArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Plan name
    #[arg(short, long)]
    pub name: String,
}

// ── Static Web App commands ─────────────────────────────────────────

#[derive(Subcommand)]
pub enum StaticwebappCommands {
    /// List static web apps
    List(StaticwebappListArgs),
    /// Show details of a static web app
    Show(StaticwebappShowArgs),
    /// Create a static web app
    Create(StaticwebappCreateArgs),
    /// Delete a static web app
    Delete(StaticwebappDeleteArgs),
    /// Update a static web app
    Update(StaticwebappUpdateArgs),
    /// Manage function app settings
    #[command(subcommand)]
    Appsettings(StaticwebappAppsettingsCommands),
    /// Manage custom hostnames
    #[command(subcommand)]
    Hostname(StaticwebappHostnameCommands),
    /// Manage build environments
    #[command(subcommand)]
    Environment(StaticwebappEnvironmentCommands),
}

#[derive(clap::Args)]
pub struct StaticwebappListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct StaticwebappShowArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Static web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct StaticwebappCreateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Static web app name
    #[arg(short, long)]
    pub name: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// URL for the repository of the static site
    #[arg(long)]
    pub source: Option<String>,
    /// The target branch in the repository
    #[arg(long)]
    pub branch: Option<String>,
    /// A user's GitHub or Azure Dev Ops repository token
    #[arg(long)]
    pub token: Option<String>,
    /// Pricing tier (Free or Standard)
    #[arg(long)]
    pub sku: Option<String>,
    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct StaticwebappDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Static web app name
    #[arg(short, long)]
    pub name: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct StaticwebappUpdateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Static web app name
    #[arg(short, long)]
    pub name: String,
    /// Property overrides in key=value format
    #[arg(long, num_args = 1..)]
    pub set: Option<Vec<String>>,
}

// Staticwebapp Appsettings

#[derive(Subcommand)]
pub enum StaticwebappAppsettingsCommands {
    /// List function app settings
    List(StaticwebappAppsettingsListArgs),
    /// Set function app settings
    Set(StaticwebappAppsettingsSetArgs),
    /// Delete function app settings
    Delete(StaticwebappAppsettingsDeleteArgs),
}

#[derive(clap::Args)]
pub struct StaticwebappAppsettingsListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Static web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct StaticwebappAppsettingsSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Static web app name
    #[arg(short, long)]
    pub name: String,
    /// Settings in key=value format
    #[arg(long, num_args = 1..)]
    pub settings: Vec<String>,
}

#[derive(clap::Args)]
pub struct StaticwebappAppsettingsDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Static web app name
    #[arg(short, long)]
    pub name: String,
    /// Setting names to delete
    #[arg(long = "setting-names", num_args = 1..)]
    pub setting_names: Vec<String>,
}

// Staticwebapp Hostname

#[derive(Subcommand)]
pub enum StaticwebappHostnameCommands {
    /// List custom hostnames
    List(StaticwebappHostnameListArgs),
    /// Set a custom hostname
    Set(StaticwebappHostnameSetArgs),
    /// Delete a custom hostname
    Delete(StaticwebappHostnameDeleteArgs),
}

#[derive(clap::Args)]
pub struct StaticwebappHostnameListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Static web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct StaticwebappHostnameSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Static web app name
    #[arg(short, long)]
    pub name: String,
    /// Custom hostname
    #[arg(long)]
    pub hostname: String,
}

#[derive(clap::Args)]
pub struct StaticwebappHostnameDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Static web app name
    #[arg(short, long)]
    pub name: String,
    /// Custom hostname
    #[arg(long)]
    pub hostname: String,
}

// Staticwebapp Environment

#[derive(Subcommand)]
pub enum StaticwebappEnvironmentCommands {
    /// List build environments
    List(StaticwebappEnvironmentListArgs),
    /// Show a build environment
    Show(StaticwebappEnvironmentShowArgs),
    /// Delete a build environment
    Delete(StaticwebappEnvironmentDeleteArgs),
}

#[derive(clap::Args)]
pub struct StaticwebappEnvironmentListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Static web app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct StaticwebappEnvironmentShowArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Static web app name
    #[arg(short, long)]
    pub name: String,
    /// Environment name
    #[arg(long)]
    pub environment_name: String,
}

#[derive(clap::Args)]
pub struct StaticwebappEnvironmentDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Static web app name
    #[arg(short, long)]
    pub name: String,
    /// Environment name
    #[arg(long)]
    pub environment_name: String,
}

// ── Logicapp commands ──────────────────────────────────────────────

#[derive(Subcommand)]
pub enum LogicappCommands {
    /// List logic apps
    List(LogicappListArgs),
    /// Show details of a logic app
    Show(LogicappShowArgs),
    /// Create a logic app
    Create(LogicappCreateArgs),
    /// Delete a logic app
    Delete(LogicappDeleteArgs),
    /// Stop a logic app
    Stop(LogicappStopArgs),
    /// Start a logic app
    Start(LogicappStartArgs),
    /// Restart a logic app
    Restart(LogicappRestartArgs),
    /// Update a logic app
    Update(LogicappUpdateArgs),
    /// Configure a logic app
    #[command(subcommand)]
    Config(LogicappConfigCommands),
    /// Manage logic app deployments
    #[command(subcommand)]
    Deployment(LogicappDeploymentCommands),
}

#[derive(clap::Args)]
pub struct LogicappListArgs {
    /// Resource group (omit for all in subscription)
    #[arg(short = 'g', long)]
    pub resource_group: Option<String>,
}

#[derive(clap::Args)]
pub struct LogicappShowArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Logic app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct LogicappCreateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Logic app name
    #[arg(short, long)]
    pub name: String,
    /// App Service plan name or ID
    #[arg(short, long)]
    pub plan: String,
    /// Location
    #[arg(short, long)]
    pub location: String,
    /// Storage account name
    #[arg(long)]
    pub storage_account: Option<String>,
    /// Space-separated tags: key=value
    #[arg(long, num_args = 1..)]
    pub tags: Option<Vec<String>>,
}

#[derive(clap::Args)]
pub struct LogicappDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Logic app name
    #[arg(short, long)]
    pub name: String,
    /// Do not prompt for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct LogicappStopArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Logic app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct LogicappStartArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Logic app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct LogicappRestartArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Logic app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct LogicappUpdateArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Logic app name
    #[arg(short, long)]
    pub name: String,
    /// Space-separated property=value pairs
    #[arg(long, num_args = 1..)]
    pub set: Option<Vec<String>>,
}

// Logicapp Config

#[derive(Subcommand)]
pub enum LogicappConfigCommands {
    /// Manage app settings
    #[command(subcommand)]
    Appsettings(LogicappConfigAppsettingsCommands),
}

#[derive(Subcommand)]
pub enum LogicappConfigAppsettingsCommands {
    /// List app settings
    List(LogicappConfigAppsettingsListArgs),
    /// Set app settings
    Set(LogicappConfigAppsettingsSetArgs),
    /// Delete app settings
    Delete(LogicappConfigAppsettingsDeleteArgs),
}

#[derive(clap::Args)]
pub struct LogicappConfigAppsettingsListArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Logic app name
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::Args)]
pub struct LogicappConfigAppsettingsSetArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Logic app name
    #[arg(short, long)]
    pub name: String,
    /// Space-separated settings: key=value
    #[arg(long, num_args = 1..)]
    pub settings: Vec<String>,
}

#[derive(clap::Args)]
pub struct LogicappConfigAppsettingsDeleteArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Logic app name
    #[arg(short, long)]
    pub name: String,
    /// Space-separated setting names to delete
    #[arg(long = "setting-names", num_args = 1..)]
    pub setting_names: Vec<String>,
}

// Logicapp Deployment

#[derive(Subcommand)]
pub enum LogicappDeploymentCommands {
    /// Manage deployment sources
    #[command(subcommand)]
    Source(LogicappDeploymentSourceCommands),
}

#[derive(Subcommand)]
pub enum LogicappDeploymentSourceCommands {
    /// Deploy from a zip file
    #[command(name = "config-zip")]
    ConfigZip(LogicappDeploymentSourceConfigZipArgs),
}

#[derive(clap::Args)]
pub struct LogicappDeploymentSourceConfigZipArgs {
    /// Resource group
    #[arg(short = 'g', long)]
    pub resource_group: String,
    /// Logic app name
    #[arg(short, long)]
    pub name: String,
    /// Path to the zip file to deploy
    #[arg(long)]
    pub src: String,
}
