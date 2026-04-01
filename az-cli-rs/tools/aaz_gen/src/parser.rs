/// Python AAZ file parser — extracts command metadata via regex pattern matching.
use crate::model::*;
use regex::Regex;
use std::path::Path;

/// Parse all AAZ command files in a directory tree (recursive).
pub fn parse_aaz_directory(dir: &Path, service_name: &str) -> ServiceModule {
    let group = parse_group_recursive(dir);
    ServiceModule {
        name: service_name.to_string(),
        groups: if group.commands.is_empty() && group.subgroups.is_empty() {
            Vec::new()
        } else if group.commands.is_empty() {
            group.subgroups
        } else {
            vec![group]
        },
    }
}

fn parse_group_recursive(dir: &Path) -> CommandGroup {
    let dir_name = dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let help = dir.join("__cmd_group.py").exists()
        .then(|| std::fs::read_to_string(dir.join("__cmd_group.py")).ok())
        .flatten()
        .and_then(|c| parse_cmd_group_help(&c));

    let mut commands = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.starts_with('_') && fname.ends_with(".py")
                && fname != "__init__.py" && fname != "__cmd_group.py" {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Some(cmd) = parse_command_file(&content, &entry.path()) {
                        commands.push(cmd);
                    }
                }
            }
        }
    }
    commands.sort_by(|a, b| a.verb.cmp(&b.verb));

    let mut subgroups = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('_') || name.starts_with('.') { continue; }
                let sub = parse_group_recursive(&path);
                if !sub.commands.is_empty() || !sub.subgroups.is_empty() {
                    subgroups.push(sub);
                }
            }
        }
    }
    subgroups.sort_by(|a, b| a.name.cmp(&b.name));

    let cli_path = commands.first()
        .map(|c| c.cli_path.rsplitn(2, ' ').last().unwrap_or("").to_string())
        .unwrap_or_else(|| dir_name.clone());

    CommandGroup { name: dir_name, cli_path, help, commands, subgroups }
}

fn parse_cmd_group_help(content: &str) -> Option<String> {
    let re = Regex::new(r#"class __CMDGroup.*:\s*"""([^"]+)"#).ok()?;
    re.captures(content).map(|c| c.get(1).unwrap().as_str().trim().to_string())
}

fn parse_command_file(content: &str, path: &Path) -> Option<CommandDef> {
    let cli_path = extract_register_command(content)?;
    let verb = cli_path.split(' ').last()?.to_string();
    let help = extract_docstring(content);
    let method = extract_method(content)?;
    let url_template = extract_url(content)?;
    let api_version = extract_api_version(content).unwrap_or_else(|| "2024-01-01".to_string());
    let args = extract_args(content);
    let is_lro = content.contains("build_lro_poller") || content.contains("AZ_SUPPORT_NO_WAIT = True");
    let is_paged = content.contains("AZ_SUPPORT_PAGINATION = True") || content.contains("build_paging");
    let has_body = content.contains("def content(self)") || content.contains("def form_content");
    let all_body_mappings = if has_body { extract_body_mappings(content) } else { Vec::new() };
    let arg_names: Vec<&str> = args.iter().map(|a| a.name.as_str()).collect();
    let body_mappings: Vec<BodyMapping> = all_body_mappings.into_iter()
        .filter(|m| arg_names.contains(&m.arg_name.as_str()))
        .collect();
    let url_param_map = extract_url_param_map(content);
    Some(CommandDef { verb, cli_path, help, source_file: path.to_string_lossy().to_string(),
        operation: OperationDef { method, url_template, api_version }, args, is_lro, is_paged, has_body, body_mappings, url_param_map })
}

fn extract_register_command(content: &str) -> Option<String> {
    let re = Regex::new(r#"@register_command\(\s*"([^"]+)""#).ok()?;
    re.captures(content).map(|c| c.get(1).unwrap().as_str().to_string())
}
fn extract_docstring(content: &str) -> Option<String> {
    let re = Regex::new(r#"class \w+\(AAZCommand\):\s*"""([^"\n]+)"#).ok()?;
    re.captures(content).map(|c| c.get(1).unwrap().as_str().trim().to_string())
}
fn extract_method(content: &str) -> Option<String> {
    let re = Regex::new(r#"def method\(self\):\s*\n\s*return "(\w+)""#).ok()?;
    re.captures(content).map(|c| c.get(1).unwrap().as_str().to_string())
}
fn extract_url(content: &str) -> Option<String> {
    let re = Regex::new(r#"format_url\(\s*\n?\s*"([^"]+)""#).ok()?;
    re.captures(content).map(|c| c.get(1).unwrap().as_str().to_string())
}
fn extract_api_version(content: &str) -> Option<String> {
    let re = Regex::new(r#""api-version",\s*"([^"]+)""#).ok()?;
    re.captures(content).map(|c| c.get(1).unwrap().as_str().to_string())
}
fn extract_args(content: &str) -> Vec<ArgDef> {
    let mut args = Vec::new();
    let re_str = Regex::new(r#"_args_schema\.(\w+)\s*=\s*AAZStrArg\(\s*options=\[([^\]]+)\]"#).unwrap();
    for cap in re_str.captures_iter(content) {
        let name = cap.get(1).unwrap().as_str().to_string();
        let options = parse_options(cap.get(2).unwrap().as_str());
        let required = is_required_after(&content[cap.get(0).unwrap().end()..]);
        let help = extract_help_after(&content[cap.get(0).unwrap().end()..]);
        args.push(ArgDef { name, options, arg_type: ArgType::Str, required, help });
    }
    let re_rg = Regex::new(r#"_args_schema\.(\w+)\s*=\s*AAZResourceGroupNameArg\("#).unwrap();
    for cap in re_rg.captures_iter(content) {
        let name = cap.get(1).unwrap().as_str().to_string();
        let required = is_required_after(&content[cap.get(0).unwrap().end()..]);
        args.push(ArgDef { name, options: vec!["-g".into(), "--resource-group".into()],
            arg_type: ArgType::ResourceGroup, required, help: Some("Name of resource group.".into()) });
    }
    let re_loc = Regex::new(r#"_args_schema\.(\w+)\s*=\s*AAZResourceLocationArg\("#).unwrap();
    for cap in re_loc.captures_iter(content) {
        let name = cap.get(1).unwrap().as_str().to_string();
        let help = extract_help_after(&content[cap.get(0).unwrap().end()..]);
        args.push(ArgDef { name, options: vec!["-l".into(), "--location".into()],
            arg_type: ArgType::Location, required: false, help });
    }
    let re_dict = Regex::new(r#"_args_schema\.(\w+)\s*=\s*AAZDictArg\("#).unwrap();
    for cap in re_dict.captures_iter(content) {
        let name = cap.get(1).unwrap().as_str().to_string();
        let opts = extract_options_after(&content[cap.get(0).unwrap().end()..]);
        let options = opts.map(|o| parse_options(&o)).unwrap_or_else(|| vec![format!("--{}", name.replace('_', "-"))]);
        let help = extract_help_after(&content[cap.get(0).unwrap().end()..]);
        args.push(ArgDef { name, options, arg_type: ArgType::Dict, required: false, help });
    }
    args
}
fn parse_options(s: &str) -> Vec<String> {
    let re = Regex::new(r#""([^"]+)""#).unwrap();
    re.captures_iter(s).map(|c| c.get(1).unwrap().as_str().to_string()).collect()
}
/// Return a safe UTF-8 prefix of at most `max_bytes` bytes.
fn safe_prefix(s: &str, max_bytes: usize) -> &str {
    if max_bytes >= s.len() {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}
fn is_required_after(s: &str) -> bool {
    safe_prefix(s, 300).contains("required=True")
}
fn extract_help_after(s: &str) -> Option<String> {
    let re = Regex::new(r#"help="([^"]+)""#).ok()?;
    re.captures(safe_prefix(s, 500)).map(|c| c.get(1).unwrap().as_str().to_string())
}
fn extract_options_after(s: &str) -> Option<String> {
    let re = Regex::new(r#"options=\[([^\]]+)\]"#).ok()?;
    re.captures(safe_prefix(s, 300)).map(|c| c.get(1).unwrap().as_str().to_string())
}

/// Extract body field mappings from the `content` property builder.
fn extract_body_mappings(content: &str) -> Vec<BodyMapping> {
    let mut mappings = Vec::new();
    // Match top-level: _builder.set_prop("location", AAZStrType, ".location")
    let re = Regex::new(r#"_builder\.set_prop\("(\w+)",\s*(AAZ\w+Type),\s*"\.(\w+)"\)"#).unwrap();
    for cap in re.captures_iter(content) {
        let json_field = cap.get(1).unwrap().as_str();
        // Skip intermediate objects without arg mapping (e.g. "properties" with no .arg)
        if json_field == "properties" || json_field == "extendedLocation" {
            continue;
        }
        mappings.push(BodyMapping {
            json_path: json_field.to_string(),
            value_type: cap.get(2).unwrap().as_str().to_string(),
            arg_name: cap.get(3).unwrap().as_str().to_string(),
        });
    }

    // Match nested under properties: properties.set_prop("field", Type, ".arg")
    let re_prop = Regex::new(r#"properties\.set_prop\("(\w+)",\s*(AAZ\w+Type),\s*"\.(\w+)"\)"#).unwrap();
    for cap in re_prop.captures_iter(content) {
        mappings.push(BodyMapping {
            json_path: format!("properties.{}", cap.get(1).unwrap().as_str()),
            value_type: cap.get(2).unwrap().as_str().to_string(),
            arg_name: cap.get(3).unwrap().as_str().to_string(),
        });
    }

    mappings
}

/// Extract URL parameter mappings from serialize_url_param() calls.
/// Matches: `serialize_url_param(\n    "placeholderName", self.ctx.args.arg_name,`
fn extract_url_param_map(content: &str) -> Vec<UrlParamMapping> {
    let mut mappings = Vec::new();
    let re = Regex::new(r#"serialize_url_param\(\s*\n?\s*"(\w+)",\s*self\.ctx\.args\.(\w+)"#).unwrap();
    for cap in re.captures_iter(content) {
        let placeholder = cap.get(1).unwrap().as_str().to_string();
        let arg_name = cap.get(2).unwrap().as_str().to_string();
        // Skip subscriptionId — handled by the framework
        if placeholder == "subscriptionId" { continue; }
        mappings.push(UrlParamMapping { placeholder, arg_name });
    }
    mappings
}
