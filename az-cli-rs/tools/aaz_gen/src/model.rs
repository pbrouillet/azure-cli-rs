/// IR model — intermediate representation for AAZ commands.
///
/// These structs capture everything needed to generate Rust code
/// from a parsed Python AAZ command file. JSON-serializable for
/// --dump-ir debugging and future alternative parsers.
use serde::{Deserialize, Serialize};

/// A complete service module (e.g. "network") with all its command groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceModule {
    /// Service name (e.g. "network", "compute", "storage")
    pub name: String,
    /// All command groups within this service
    pub groups: Vec<CommandGroup>,
}

/// A command group (e.g. "network asg", "network vnet")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandGroup {
    /// Group name (e.g. "asg", "vnet", "nsg")
    pub name: String,
    /// Full CLI path (e.g. "network asg")
    pub cli_path: String,
    /// Help text
    pub help: Option<String>,
    /// Commands within this group
    pub commands: Vec<CommandDef>,
    /// Nested subgroups (e.g. nsg → rule, vnet → subnet)
    #[serde(default)]
    pub subgroups: Vec<CommandGroup>,
}

/// A single CLI command (e.g. "network asg show").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDef {
    /// Command verb (e.g. "show", "create", "list", "delete")
    pub verb: String,
    /// Full CLI path (e.g. "network asg show")
    pub cli_path: String,
    /// Help text (first line of docstring)
    pub help: Option<String>,
    /// Source file this was parsed from
    pub source_file: String,
    /// HTTP operation
    pub operation: OperationDef,
    /// CLI arguments
    pub args: Vec<ArgDef>,
    /// Whether this is a long-running operation
    pub is_lro: bool,
    /// Whether this supports pagination
    pub is_paged: bool,
    /// Whether the operation has a request body
    pub has_body: bool,
    /// Mappings from CLI args to JSON body fields (for create/update commands)
    #[serde(default)]
    pub body_mappings: Vec<BodyMapping>,
    /// Explicit URL parameter mappings parsed from serialize_url_param().
    /// Maps URL placeholder name → CLI arg name (e.g. "vmName" → "vm_name").
    #[serde(default)]
    pub url_param_map: Vec<UrlParamMapping>,
}

/// Explicit mapping from a URL placeholder to a CLI argument.
/// Parsed from `serialize_url_param("placeholderName", self.ctx.args.arg_name)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlParamMapping {
    /// URL placeholder (e.g. "vmName", "loadBalancerName")
    pub placeholder: String,
    /// CLI arg name (e.g. "vm_name", "lb_name")
    pub arg_name: String,
}

/// Maps a CLI argument to a JSON body field path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyMapping {
    /// JSON path in the body (e.g. "location", "properties.addressSpace.addressPrefixes")
    pub json_path: String,
    /// CLI arg name (e.g. "location", "tags", "address_prefixes")
    pub arg_name: String,
    /// Value type (AAZStrType, AAZDictType, AAZBoolType, AAZIntType, AAZListType)
    pub value_type: String,
}

/// An HTTP operation definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationDef {
    /// HTTP method (GET, PUT, POST, DELETE, PATCH)
    pub method: String,
    /// URL template with ARM placeholders
    /// e.g. "/subscriptions/{subscriptionId}/resourceGroups/{resourceGroupName}/providers/..."
    pub url_template: String,
    /// API version (e.g. "2021-08-01")
    pub api_version: String,
}

/// A CLI argument definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgDef {
    /// Internal name (snake_case, e.g. "resource_group")
    pub name: String,
    /// CLI option strings (e.g. ["-n", "--name"])
    pub options: Vec<String>,
    /// Argument type
    pub arg_type: ArgType,
    /// Whether the arg is required
    pub required: bool,
    /// Help text
    pub help: Option<String>,
}

/// Argument type — maps to clap arg types and body field serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgType {
    Str,
    Int,
    Float,
    Bool,
    ResourceGroup,
    Location,
    SubscriptionId,
    Dict,
    List,
}

impl ArgType {
    /// Rust type string for this arg type.
    pub fn rust_type(&self) -> &str {
        match self {
            ArgType::Str | ArgType::ResourceGroup | ArgType::Location | ArgType::SubscriptionId => "&str",
            ArgType::Int => "i64",
            ArgType::Float => "f64",
            ArgType::Bool => "bool",
            ArgType::Dict | ArgType::List => "&str", // JSON string passthrough
        }
    }

    /// clap value_parser hint
    pub fn clap_type(&self) -> &str {
        match self {
            ArgType::Str | ArgType::ResourceGroup | ArgType::Location | ArgType::SubscriptionId => "String",
            ArgType::Int => "i64",
            ArgType::Float => "f64",
            ArgType::Bool => "bool",
            ArgType::Dict | ArgType::List => "String",
        }
    }
}
