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
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Json,
    Jsonc,
    Table,
    Tsv,
    Yaml,
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

    /// Manage storage accounts
    #[command(subcommand)]
    Storage(StorageCommands),

    /// Manage Key Vault resources
    #[command(subcommand)]
    Keyvault(KeyvaultCommands),

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
    /// List virtual machines
    List(VmListArgs),
    /// Show a virtual machine
    Show(VmShowArgs),
    /// Start a virtual machine
    Start(VmActionArgs),
    /// Stop (power off) a virtual machine
    Stop(VmActionArgs),
    /// Restart a virtual machine
    Restart(VmActionArgs),
    /// Deallocate a virtual machine
    Deallocate(VmActionArgs),
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
