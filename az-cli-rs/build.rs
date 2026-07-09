/// build.rs — generates Rust command modules from Python AAZ files at build time.
///
/// Reads gen_config.toml for module configuration.
/// Writes generated code to $OUT_DIR/generated/.
/// Falls back to empty stubs if azure-cli source is not present.
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Convert a Path to a string using forward slashes so that generated
/// `#[path = "..."]` attributes are valid Rust on Windows (backslashes
/// are interpreted as escape sequences in string literals).
fn forward_slash(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

#[derive(Deserialize)]
struct GenConfig {
    azure_cli_path: String,
    modules: Vec<ModuleConfig>,
}

#[derive(Deserialize)]
struct ModuleConfig {
    service: String,
    aaz_subpath: String,
    #[serde(default)]
    cli_prefix: Option<String>,
    #[serde(default)]
    skip_top_level: bool,
    /// Additional AAZ source directories whose command groups are merged into
    /// this module. Used to nest commands that `az` re-parents under an existing
    /// top-level group (e.g. `network private-dns` lives in the `privatedns`
    /// module, `policy attestation` lives in the `policyinsights` module).
    #[serde(default)]
    extra_subpaths: Vec<String>,
}

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let gen_dir = out_dir.join("generated");
    std::fs::create_dir_all(&gen_dir).unwrap();

    // Read config
    let config_path = Path::new("gen_config.toml");
    let config: Option<GenConfig> = if config_path.exists() {
        let content = std::fs::read_to_string(config_path).unwrap();
        println!("cargo:rerun-if-changed=gen_config.toml");
        toml::from_str(&content).ok()
    } else {
        None
    };

    let mut service_modules = Vec::new();

    if let Some(config) = config {
        let cli_path = PathBuf::from(&config.azure_cli_path);

        for module in &config.modules {
            let aaz_path = cli_path.join(&module.aaz_subpath);

            if aaz_path.exists() {
                // Tell cargo to rerun if input files change
                println!("cargo:rerun-if-changed={}", aaz_path.display());

                eprintln!(
                    "aaz-gen: Parsing {} from {}",
                    module.service,
                    aaz_path.display()
                );

                let service = aaz_gen::parser::parse_aaz_directory(&aaz_path, &module.service);
                let mut service = service;

                // Merge in any extra source directories (nested-under-existing-group support).
                let cli_prefix_norm = module
                    .cli_prefix
                    .clone()
                    .unwrap_or_else(|| module.service.replace('_', "-"))
                    .replace('_', "-");
                let mut extra_groups: Vec<aaz_gen::model::CommandGroup> = Vec::new();
                for extra in &module.extra_subpaths {
                    let extra_path = cli_path.join(extra);
                    if extra_path.exists() {
                        println!("cargo:rerun-if-changed={}", extra_path.display());
                        let extra_svc =
                            aaz_gen::parser::parse_aaz_directory(&extra_path, &module.service);
                        eprintln!(
                            "aaz-gen: {} — merging {} extra group(s) from {}",
                            module.service,
                            extra_svc.groups.len(),
                            extra_path.display()
                        );
                        extra_groups.extend(extra_svc.groups);
                    } else {
                        eprintln!(
                            "aaz-gen: {} — extra path not found, skipping: {}",
                            module.service,
                            extra_path.display()
                        );
                    }
                }
                if !extra_groups.is_empty() {
                    // If the module collapses to a single wrapper group whose name matches
                    // the cli_prefix (the emitter flattens this into the top-level), the extra
                    // groups must be nested INSIDE that wrapper's subgroups — otherwise the
                    // flatten optimization breaks and a redundant nested group appears.
                    if service.groups.len() == 1
                        && service.groups[0].name.replace('_', "-") == cli_prefix_norm
                    {
                        service.groups[0].subgroups.extend(extra_groups);
                        service.groups[0]
                            .subgroups
                            .sort_by(|a, b| a.name.cmp(&b.name));
                    } else {
                        service.groups.extend(extra_groups);
                        service.groups.sort_by(|a, b| a.name.cmp(&b.name));
                    }
                }

                let total_cmds = count_commands_recursive(&service.groups);
                eprintln!(
                    "aaz-gen: {} — {} groups, {} commands",
                    module.service,
                    service.groups.len(),
                    total_cmds
                );

                // Emit service module files (recursive)
                let service_dir = gen_dir.join(&module.service);
                std::fs::create_dir_all(&service_dir).unwrap();
                emit_group_tree(&service.groups, &service_dir);

                service_modules.push((
                    module.cli_prefix.clone().unwrap_or_else(|| module.service.replace('_', "-")),
                    service,
                    module.skip_top_level,
                ));
            } else {
                eprintln!(
                    "aaz-gen: Skipping {} — path not found: {}",
                    module.service,
                    aaz_path.display()
                );
            }
        }
    }

    // Write top-level generated/mod.rs
    let mut top_mod = String::from("// @generated by aaz-gen at build time\n\n");

    for (_, svc, _) in &service_modules {
        let svc_path = gen_dir.join(&svc.name).join("mod.rs");
        top_mod.push_str(&format!(
            "#[path = \"{}\"]\npub mod {};\n\n",
            forward_slash(&svc_path),
            svc.name.replace('-', "_")
        ));
    }

    // Generate the clap types
    top_mod.push_str(&emit_clap_types(&service_modules));

    // Generate the dispatch function
    top_mod.push_str(&emit_dispatch_fn(&service_modules));

    std::fs::write(gen_dir.join("mod.rs"), top_mod).unwrap();
}

/// Recursively write group files and mod.rs for a list of groups.
fn emit_group_tree(groups: &[aaz_gen::model::CommandGroup], dir: &std::path::Path) {
    let mut mod_rs = String::from("// @generated by aaz-gen at build time\n");

    for group in groups {
        let mod_name = group.name.replace('-', "_").to_lowercase();

        if group.subgroups.is_empty() {
            // Leaf group — write as a .rs file
            let content = emit_group_file(group);
            std::fs::write(dir.join(format!("{mod_name}.rs")), content).unwrap();
            mod_rs.push_str(&format!("pub mod {mod_name};\n"));
        } else {
            // Group with subgroups — create a directory
            let sub_dir = dir.join(&mod_name);
            std::fs::create_dir_all(&sub_dir).unwrap();

            // First, write all leaf subgroup .rs files
            for sub in &group.subgroups {
                let sub_mod_name = sub.name.replace('-', "_").to_lowercase();
                if sub.subgroups.is_empty() {
                    let content = emit_group_file(sub);
                    std::fs::write(sub_dir.join(format!("{sub_mod_name}.rs")), content).unwrap();
                } else {
                    // Nested sub-subgroup — create its directory and recurse
                    let sub_sub_dir = sub_dir.join(&sub_mod_name);
                    std::fs::create_dir_all(&sub_sub_dir).unwrap();
                    write_group_dir(sub, &sub_sub_dir);
                }
            }

            // Then write mod.rs with this group's commands + submodule declarations
            let mut sub_mod = String::from("// @generated by aaz-gen at build time\n");
            if !group.commands.is_empty() {
                sub_mod.push_str("use crate::commands::ArmCommand;\n");
                sub_mod.push_str("use crate::error::Result;\n\n");
                for cmd in &group.commands {
                    sub_mod.push_str(&emit_command_fn(cmd));
                    sub_mod.push('\n');
                }
            }
            for sub in &group.subgroups {
                let sub_mod_name = sub.name.replace('-', "_").to_lowercase();
                if sub.subgroups.is_empty() {
                    sub_mod.push_str(&format!("pub mod {sub_mod_name};\n"));
                } else {
                    let sub_sub_dir = sub_dir.join(&sub_mod_name);
                    sub_mod.push_str(&format!(
                        "#[path = \"{}\"]\npub mod {sub_mod_name};\n",
                        forward_slash(&sub_sub_dir.join("mod.rs"))
                    ));
                }
            }
            std::fs::write(sub_dir.join("mod.rs"), sub_mod).unwrap();

            mod_rs.push_str(&format!(
                "#[path = \"{}\"]\npub mod {mod_name};\n",
                forward_slash(&sub_dir.join("mod.rs"))
            ));
        }
    }

    std::fs::write(dir.join("mod.rs"), mod_rs).unwrap();
}

/// Write a single group's directory with its commands and subgroups.
fn write_group_dir(group: &aaz_gen::model::CommandGroup, dir: &std::path::Path) {
    let mut mod_content = String::from("// @generated by aaz-gen at build time\n");
    if !group.commands.is_empty() {
        mod_content.push_str("use crate::commands::ArmCommand;\n");
        mod_content.push_str("use crate::error::Result;\n\n");
        for cmd in &group.commands {
            mod_content.push_str(&emit_command_fn(cmd));
            mod_content.push('\n');
        }
    }
    for sub in &group.subgroups {
        let sub_name = sub.name.replace('-', "_").to_lowercase();
        if sub.subgroups.is_empty() {
            let content = emit_group_file(sub);
            std::fs::write(dir.join(format!("{sub_name}.rs")), content).unwrap();
            mod_content.push_str(&format!("pub mod {sub_name};\n"));
        } else {
            let sub_dir = dir.join(&sub_name);
            std::fs::create_dir_all(&sub_dir).unwrap();
            write_group_dir(sub, &sub_dir);
            mod_content.push_str(&format!(
                "#[path = \"{}\"]\npub mod {sub_name};\n",
                forward_slash(&sub_dir.join("mod.rs"))
            ));
        }
    }
    std::fs::write(dir.join("mod.rs"), mod_content).unwrap();
}

/// Emit a single group .rs file with command functions.
fn emit_group_file(group: &aaz_gen::model::CommandGroup) -> String {
    let mut out = String::from("// @generated by aaz-gen at build time — do not edit\n");
    out.push_str("#![allow(unused_variables)]\n");
    out.push_str("use crate::commands::ArmCommand;\n");
    out.push_str("use crate::error::Result;\n\n");

    for cmd in &group.commands {
        out.push_str(&emit_command_fn(cmd));
        out.push('\n');
    }

    out
}

/// Emit a single command function.
fn emit_command_fn(cmd: &aaz_gen::model::CommandDef) -> String {
    let mut out = String::new();
    let fn_name = sanitize_ident(&cmd.verb);

    if let Some(ref help) = cmd.help {
        out.push_str(&format!("/// {help}\n"));
    }

    // Build param list: URL-referenced args + body-mapped args
    let url_args: Vec<&aaz_gen::model::ArgDef> = cmd
        .args
        .iter()
        .filter(|a| is_url_arg(a, cmd))
        .collect();

    // Collect body-mapped args that aren't already URL args
    let url_arg_names: Vec<&str> = url_args.iter().map(|a| a.name.as_str()).collect();
    let body_arg_names: Vec<&str> = cmd.body_mappings.iter()
        .map(|m| m.arg_name.as_str())
        .filter(|n| !url_arg_names.contains(n))
        .collect();

    // Find arg definitions for body-mapped args
    let body_args: Vec<&aaz_gen::model::ArgDef> = cmd.args.iter()
        .filter(|a| body_arg_names.contains(&a.name.as_str()))
        .collect();

    let mut params: Vec<String> = url_args
        .iter()
        .map(|a| format!("{}: &str", sanitize_ident(&a.name)))
        .collect();

    // Add body args as optional string params
    for arg in &body_args {
        let name = sanitize_ident(&arg.name);
        if arg.required {
            params.push(format!("{name}: &str"));
        } else {
            params.push(format!("{name}: Option<&str>"));
        }
    }

    let params_str = params.join(", ");

    let method = &cmd.operation.method;
    let api_ver = &cmd.operation.api_version;

    // Determine return type
    let is_list = (cmd.verb == "list" || cmd.is_paged) && method == "GET";
    let is_delete = method == "DELETE";

    let ret_type = if is_delete {
        "Result<()>"
    } else if is_list {
        "Result<Vec<serde_json::Value>>"
    } else {
        "Result<serde_json::Value>"
    };

    out.push_str(&format!("pub async fn {fn_name}({params_str}) -> {ret_type} {{\n"));
    out.push_str("    let mut cmd = ArmCommand::new()?;\n");

    // Build URL with format args
    let mut url_fmt = cmd
        .operation
        .url_template
        .replace("{subscriptionId}", "{{subscriptionId}}");
    for arg in &url_args {
        if let Some(ph) = find_url_placeholder_for_cmd(&arg.name, cmd) {
            url_fmt = url_fmt.replace(
                &format!("{{{ph}}}"),
                &format!("{{{}}}", sanitize_ident(&arg.name)),
            );
        }
    }

    // Escape any remaining {Placeholders} that weren't mapped to args
    // Build set of format arg names we've inserted
    let format_arg_names: std::collections::HashSet<String> = url_args
        .iter()
        .map(|a| sanitize_ident(&a.name))
        .collect();

    let mut final_url = String::new();
    let mut chars = url_fmt.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            if chars.peek() == Some(&'{') {
                final_url.push('{');
                final_url.push(chars.next().unwrap());
            } else {
                let mut name = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc == '}' { chars.next(); break; }
                    name.push(chars.next().unwrap());
                }
                if format_arg_names.contains(&name) {
                    // Known format arg — keep single braces
                    final_url.push('{');
                    final_url.push_str(&name);
                    final_url.push('}');
                } else {
                    // Unknown placeholder — escape as literal
                    final_url.push('{');
                    final_url.push('{');
                    final_url.push_str(&name);
                    final_url.push('}');
                    final_url.push('}');
                }
            }
        } else if c == '}' {
            if chars.peek() == Some(&'}') {
                final_url.push('}');
                final_url.push(chars.next().unwrap());
            } else {
                final_url.push('}');
            }
        } else {
            final_url.push(c);
        }
    }
    url_fmt = final_url;

    out.push_str(&format!(
        "    let arm_url = format!(\"{url_fmt}?api-version={api_ver}\");\n"
    ));

    // Emit call
    match method.as_str() {
        "GET" if is_list => {
            out.push_str("    let results = cmd.list(&arm_url).await?;\n");
            out.push_str("    cmd.save_cache()?;\n");
            out.push_str("    Ok(results)\n");
        }
        "GET" => {
            out.push_str("    let result = cmd.get(&arm_url).await?;\n");
            out.push_str("    cmd.save_cache()?;\n");
            out.push_str("    Ok(result)\n");
        }
        "PUT" if cmd.is_lro => {
            out.push_str(&emit_body_construction(cmd));
            out.push_str("    let result = cmd.put_lro(&arm_url, &body).await?;\n");
            out.push_str("    cmd.save_cache()?;\n");
            out.push_str("    Ok(result)\n");
        }
        "PUT" => {
            out.push_str(&emit_body_construction(cmd));
            out.push_str("    let result = cmd.put(&arm_url, &body).await?;\n");
            out.push_str("    cmd.save_cache()?;\n");
            out.push_str("    Ok(result)\n");
        }
        "DELETE" => {
            out.push_str("    cmd.delete(&arm_url).await?;\n");
            out.push_str("    cmd.save_cache()?;\n");
            out.push_str("    Ok(())\n");
        }
        "POST" if cmd.is_lro => {
            out.push_str("    cmd.post_lro(&arm_url, None).await?;\n");
            out.push_str("    cmd.save_cache()?;\n");
            out.push_str("    Ok(serde_json::Value::Null)\n");
        }
        _ => {
            out.push_str("    let result = cmd.get(&arm_url).await?;\n");
            out.push_str("    cmd.save_cache()?;\n");
            out.push_str("    Ok(result)\n");
        }
    }

    out.push_str("}\n");
    out
}


/// Emit all clap types for all generated services.

fn emit_clap_types(services: &[(String, aaz_gen::model::ServiceModule, bool)]) -> String {
    let mut out = String::new();

    if services.is_empty() {
        out.push_str("// No generated commands\n");
        out.push_str("#[derive(clap::Subcommand)]\npub enum GeneratedCommands {}\n");
        return out;
    }

    out.push_str("use clap::Subcommand;\n\n");

    // Top-level enum — skip services with skip_top_level
    out.push_str("#[derive(Subcommand)]\n");
    out.push_str("pub enum GeneratedCommands {\n");
    for (cli_prefix, svc, skip) in services {
        if *skip { continue; }
        let variant = capitalize(cli_prefix);
        let help = svc.groups.first()
            .and_then(|g| g.help.as_deref())
            .unwrap_or("Manage resources")
            .lines().next().unwrap_or("Manage resources");
        let cmd_name = cli_prefix.replace('_', "-");
        out.push_str(&format!("    /// {help}\n"));
        out.push_str(&format!("    #[command(subcommand, name = \"{cmd_name}\")]\n"));
        out.push_str(&format!("    {variant}({variant}Commands),\n"));
    }
    out.push_str("}\n\n");

    // Emit nested enums for each service (all services, including skip_top_level)
    for (cli_prefix, svc, _) in services {
        let prefix = capitalize(cli_prefix);
        // If the service has exactly one group whose name matches the cli_prefix, merge it
        // directly. When names differ, the group is a subcommand that must stay nested
        // (e.g. identity → federated-credential, capacity → reservation).
        let single_group_matches = svc.groups.len() == 1
            && svc.groups[0].name.replace('_', "-") == cli_prefix.replace('_', "-")
            && (!svc.groups[0].subgroups.is_empty() || !svc.groups[0].commands.is_empty());
        if single_group_matches {
            let group = &svc.groups[0];
            // Emit the service enum with the group's content directly
            out.push_str("#[derive(Subcommand)]\n");
            out.push_str(&format!("pub enum {prefix}Commands {{\n"));
            // Direct commands
            for cmd in &group.commands {
                let vcap = capitalize(&cmd.verb);
                let help = cmd.help.as_deref().unwrap_or(&cmd.verb);
                out.push_str(&format!("    /// {help}\n"));
                out.push_str(&format!("    {vcap}({prefix}{vcap}Args),\n"));
            }
            // Subgroups
            for sub in &group.subgroups {
                let scap = capitalize(&sub.name);
                let cli_sub = sub.name.replace('_', "-");
                let sub_help = sub.help.as_deref().unwrap_or("Manage resources");
                let sub_help_line = sub_help.lines().next().unwrap_or("Manage resources");
                out.push_str(&format!("    /// {sub_help_line}\n"));
                out.push_str(&format!("    #[command(subcommand, name = \"{cli_sub}\")]\n"));
                out.push_str(&format!("    {scap}({prefix}{scap}Commands),\n"));
            }
            out.push_str("}\n\n");
            // Arg structs for direct commands
            for cmd in &group.commands {
                let vcap = capitalize(&cmd.verb);
                emit_arg_struct(&mut out, &format!("{prefix}{vcap}Args"), cmd);
            }
            // Recurse for subgroups
            for sub in &group.subgroups {
                let scap = capitalize(&sub.name);
                let sp = format!("{prefix}{scap}");
                if sub.subgroups.is_empty() {
                    if sub.commands.is_empty() { continue; }
                    out.push_str("#[derive(Subcommand)]\n");
                    out.push_str(&format!("pub enum {sp}Commands {{\n"));
                    for cmd in &sub.commands {
                        let vcap = capitalize(&cmd.verb);
                        let help = cmd.help.as_deref().unwrap_or(&cmd.verb);
                        out.push_str(&format!("    /// {help}\n"));
                        out.push_str(&format!("    {vcap}({sp}{vcap}Args),\n"));
                    }
                    out.push_str("}\n\n");
                    for cmd in &sub.commands {
                        let vcap = capitalize(&cmd.verb);
                        emit_arg_struct(&mut out, &format!("{sp}{vcap}Args"), cmd);
                    }
                } else {
                    emit_deep_group(&mut out, &sp, sub);
                }
            }
        } else {
            emit_nested_enums(&mut out, &prefix, &svc.groups);
        }
    }

    out
}

/// Recursively emit nested Subcommand enums + Args structs for a group list.
fn emit_nested_enums(out: &mut String, prefix: &str, groups: &[aaz_gen::model::CommandGroup]) {
    out.push_str("#[derive(Subcommand)]\n");
    out.push_str(&format!("pub enum {prefix}Commands {{\n"));

    for group in groups {
        let gcap = capitalize(&group.name);
        let cli_group = group.name.replace('_', "-");

        if group.subgroups.is_empty() && group.commands.len() > 0 {
            // Leaf group with only commands — make it a subcommand group
            out.push_str(&format!("    /// Manage {cli_group} resources\n"));
            out.push_str(&format!("    #[command(subcommand, name = \"{cli_group}\")]\n"));
            out.push_str(&format!("    {gcap}({prefix}{gcap}Commands),\n"));
        } else if !group.subgroups.is_empty() {
            // Group with subgroups (may also have direct commands)
            let help = group.help.as_deref().unwrap_or("Manage resources");
            let help_line = help.lines().next().unwrap_or("Manage resources");
            out.push_str(&format!("    /// {help_line}\n"));
            out.push_str(&format!("    #[command(subcommand, name = \"{cli_group}\")]\n"));
            out.push_str(&format!("    {gcap}({prefix}{gcap}Commands),\n"));
        }
        // Groups with 0 commands and 0 subgroups are skipped
    }
    out.push_str("}\n\n");

    // Now emit the sub-enums
    for group in groups {
        let gcap = capitalize(&group.name);
        let sub_prefix = format!("{prefix}{gcap}");

        if group.subgroups.is_empty() {
            // Leaf group — enum of verb commands
            if group.commands.is_empty() { continue; }
            out.push_str("#[derive(Subcommand)]\n");
            out.push_str(&format!("pub enum {sub_prefix}Commands {{\n"));
            for cmd in &group.commands {
                let vcap = capitalize(&cmd.verb);
                let help = cmd.help.as_deref().unwrap_or(&cmd.verb);
                out.push_str(&format!("    /// {help}\n"));
                out.push_str(&format!("    {vcap}({sub_prefix}{vcap}Args),\n"));
            }
            out.push_str("}\n\n");
            // Arg structs
            for cmd in &group.commands {
                let vcap = capitalize(&cmd.verb);
                emit_arg_struct(out, &format!("{sub_prefix}{vcap}Args"), cmd);
            }
        } else {
            // Group with subgroups — mixed enum of direct commands + subgroup entries
            out.push_str("#[derive(Subcommand)]\n");
            out.push_str(&format!("pub enum {sub_prefix}Commands {{\n"));
            // Direct commands
            for cmd in &group.commands {
                let vcap = capitalize(&cmd.verb);
                let help = cmd.help.as_deref().unwrap_or(&cmd.verb);
                out.push_str(&format!("    /// {help}\n"));
                out.push_str(&format!("    {vcap}({sub_prefix}{vcap}Args),\n"));
            }
            // Subgroup entries
            for sub in &group.subgroups {
                let scap = capitalize(&sub.name);
                let cli_sub = sub.name.replace('_', "-");
                let sub_help = sub.help.as_deref().unwrap_or("Manage resources");
                let sub_help_line = sub_help.lines().next().unwrap_or("Manage resources");
                out.push_str(&format!("    /// {sub_help_line}\n"));
                out.push_str(&format!("    #[command(subcommand, name = \"{cli_sub}\")]\n"));
                out.push_str(&format!("    {scap}({sub_prefix}{scap}Commands),\n"));
            }
            out.push_str("}\n\n");
            // Arg structs for direct commands
            for cmd in &group.commands {
                let vcap = capitalize(&cmd.verb);
                emit_arg_struct(out, &format!("{sub_prefix}{vcap}Args"), cmd);
            }
            // Recurse for subgroups
            for sub in &group.subgroups {
                let scap = capitalize(&sub.name);
                let sub_sub_prefix = format!("{sub_prefix}{scap}");
                // Wrap sub as a single-group list if it has subgroups of its own
                if sub.subgroups.is_empty() {
                    // Leaf subgroup
                    if sub.commands.is_empty() { continue; }
                    out.push_str("#[derive(Subcommand)]\n");
                    out.push_str(&format!("pub enum {sub_sub_prefix}Commands {{\n"));
                    for cmd in &sub.commands {
                        let vcap = capitalize(&cmd.verb);
                        let help = cmd.help.as_deref().unwrap_or(&cmd.verb);
                        out.push_str(&format!("    /// {help}\n"));
                        out.push_str(&format!("    {vcap}({sub_sub_prefix}{vcap}Args),\n"));
                    }
                    out.push_str("}\n\n");
                    for cmd in &sub.commands {
                        let vcap = capitalize(&cmd.verb);
                        emit_arg_struct(out, &format!("{sub_sub_prefix}{vcap}Args"), cmd);
                    }
                } else {
                    // Sub has its own subgroups — emit mixed enum directly
                    out.push_str("#[derive(Subcommand)]\n");
                    out.push_str(&format!("pub enum {sub_sub_prefix}Commands {{\n"));
                    // Direct commands of this sub
                    for cmd in &sub.commands {
                        let vcap = capitalize(&cmd.verb);
                        let help = cmd.help.as_deref().unwrap_or(&cmd.verb);
                        out.push_str(&format!("    /// {help}\n"));
                        out.push_str(&format!("    {vcap}({sub_sub_prefix}{vcap}Args),\n"));
                    }
                    // Sub's subgroups as nested subcommands
                    for ss in &sub.subgroups {
                        let sscap = capitalize(&ss.name);
                        let cli_ss = ss.name.replace('_', "-");
                        let ss_help = ss.help.as_deref().unwrap_or("Manage resources");
                        let ss_help_line = ss_help.lines().next().unwrap_or("Manage resources");
                        out.push_str(&format!("    /// {ss_help_line}\n"));
                        out.push_str(&format!("    #[command(subcommand, name = \"{cli_ss}\")]\n"));
                        out.push_str(&format!("    {sscap}({sub_sub_prefix}{sscap}Commands),\n"));
                    }
                    out.push_str("}\n\n");
                    // Arg structs for sub's direct commands
                    for cmd in &sub.commands {
                        let vcap = capitalize(&cmd.verb);
                        emit_arg_struct(out, &format!("{sub_sub_prefix}{vcap}Args"), cmd);
                    }
                    // Recurse for sub's subgroups
                    for ss in &sub.subgroups {
                        let sscap = capitalize(&ss.name);
                        let ss_prefix = format!("{sub_sub_prefix}{sscap}");
                        if ss.subgroups.is_empty() {
                            if ss.commands.is_empty() { continue; }
                            out.push_str("#[derive(Subcommand)]\n");
                            out.push_str(&format!("pub enum {ss_prefix}Commands {{\n"));
                            for cmd in &ss.commands {
                                let vcap = capitalize(&cmd.verb);
                                let help = cmd.help.as_deref().unwrap_or(&cmd.verb);
                                out.push_str(&format!("    /// {help}\n"));
                                out.push_str(&format!("    {vcap}({ss_prefix}{vcap}Args),\n"));
                            }
                            out.push_str("}\n\n");
                            for cmd in &ss.commands {
                                let vcap = capitalize(&cmd.verb);
                                emit_arg_struct(out, &format!("{ss_prefix}{vcap}Args"), cmd);
                            }
                        } else {
                            // 4+ levels deep — emit mixed enum for this level too
                            emit_deep_group(out, &ss_prefix, ss);
                        }
                    }
                }
            }
        }
    }
}

/// Emit a deeply nested group (4+ levels) with its commands and subgroups.
fn emit_deep_group(out: &mut String, prefix: &str, group: &aaz_gen::model::CommandGroup) {
    out.push_str("#[derive(Subcommand)]\n");
    out.push_str(&format!("pub enum {prefix}Commands {{\n"));
    for cmd in &group.commands {
        let vcap = capitalize(&cmd.verb);
        let help = cmd.help.as_deref().unwrap_or(&cmd.verb);
        out.push_str(&format!("    /// {help}\n"));
        out.push_str(&format!("    {vcap}({prefix}{vcap}Args),\n"));
    }
    for sub in &group.subgroups {
        let scap = capitalize(&sub.name);
        let cli = sub.name.replace('_', "-");
        out.push_str(&format!("    /// Manage resources\n"));
        out.push_str(&format!("    #[command(subcommand, name = \"{cli}\")]\n"));
        out.push_str(&format!("    {scap}({prefix}{scap}Commands),\n"));
    }
    out.push_str("}\n\n");
    for cmd in &group.commands {
        let vcap = capitalize(&cmd.verb);
        emit_arg_struct(out, &format!("{prefix}{vcap}Args"), cmd);
    }
    for sub in &group.subgroups {
        let scap = capitalize(&sub.name);
        let sp = format!("{prefix}{scap}");
        if sub.subgroups.is_empty() {
            if sub.commands.is_empty() { continue; }
            out.push_str("#[derive(Subcommand)]\n");
            out.push_str(&format!("pub enum {sp}Commands {{\n"));
            for cmd in &sub.commands {
                let vcap = capitalize(&cmd.verb);
                let help = cmd.help.as_deref().unwrap_or(&cmd.verb);
                out.push_str(&format!("    /// {help}\n"));
                out.push_str(&format!("    {vcap}({sp}{vcap}Args),\n"));
            }
            out.push_str("}\n\n");
            for cmd in &sub.commands {
                let vcap = capitalize(&cmd.verb);
                emit_arg_struct(out, &format!("{sp}{vcap}Args"), cmd);
            }
        } else {
            emit_deep_group(out, &sp, sub);
        }
    }
}

fn emit_arg_struct(out: &mut String, struct_name: &str, cmd: &aaz_gen::model::CommandDef) {
    let relevant_args: Vec<&aaz_gen::model::ArgDef> = cmd.args.iter()
        .filter(|a| is_relevant_arg(a, cmd))
        .collect();
    out.push_str("#[derive(clap::Args)]\n");
    out.push_str(&format!("pub struct {struct_name} {{\n"));
    for arg in &relevant_args {
        let field = sanitize_ident(&arg.name);
        if let Some(ref help) = arg.help {
            out.push_str(&format!("    /// {help}\n"));
        }
        let short = arg.options.iter()
            .find(|o| o.starts_with('-') && !o.starts_with("--") && o.len() == 2)
            .map(|o| o.chars().nth(1).unwrap());
        if let Some(s) = short {
            out.push_str(&format!("    #[arg(short = '{s}', long)]\n"));
        } else {
            out.push_str("    #[arg(long)]\n");
        }
        if arg.required {
            out.push_str(&format!("    pub {field}: String,\n"));
        } else {
            out.push_str(&format!("    pub {field}: Option<String>,\n"));
        }
    }
    out.push_str("}\n\n");
}

/// Emit the dispatch function with properly nested match arms.
fn emit_dispatch_fn(services: &[(String, aaz_gen::model::ServiceModule, bool)]) -> String {
    let mut out = String::new();
    out.push_str("\npub async fn dispatch_generated(cmd: GeneratedCommands) -> crate::error::Result<Option<serde_json::Value>> {\n");
    out.push_str("    match cmd {\n");

    for (cli_prefix, svc, skip) in services {
        if *skip { continue; }
        let variant = capitalize(cli_prefix);
        let svc_mod = svc.name.replace('-', "_");
        out.push_str(&format!("        GeneratedCommands::{variant}(sub) => {{\n"));
        out.push_str(&format!("            dispatch_{svc_mod}(sub, \"{svc_mod}\").await\n"));
        out.push_str("        }\n");
    }

    out.push_str("    }\n");
    out.push_str("}\n\n");

    // Generate per-service dispatch functions (all services, including skip_top_level)
    for (cli_prefix, svc, skip) in services {
        let variant = capitalize(cli_prefix);
        let svc_mod = svc.name.replace('-', "_");
        // Make dispatch functions for skip_top_level services `pub` so they can be called from main.rs
        let vis = if *skip { "pub " } else { "" };
        out.push_str(&format!("{vis}async fn dispatch_{svc_mod}(cmd: {variant}Commands, _svc: &str) -> crate::error::Result<Option<serde_json::Value>> {{\n"));
        out.push_str("    match cmd {\n");
        let single_group_matches = svc.groups.len() == 1
            && svc.groups[0].name.replace('_', "-") == cli_prefix.replace('_', "-")
            && (!svc.groups[0].subgroups.is_empty() || !svc.groups[0].commands.is_empty());
        if single_group_matches {
            // Single-group merged: dispatch directly using the group's content
            let group = &svc.groups[0];
            let gmod = sanitize_ident(&group.name);
            // Direct commands
            for cmd in &group.commands {
                emit_dispatch_leaf_cmd(&mut out, &format!("{variant}Commands"), &svc_mod, &gmod, cmd, "        ");
            }
            // Subgroups
            for sub in &group.subgroups {
                let scap = capitalize(&sub.name);
                let sub_mod = format!("{gmod}::{}", sanitize_ident(&sub.name));
                let sub_enum = format!("{variant}{scap}Commands");
                if sub.subgroups.is_empty() {
                    out.push_str(&format!("        {variant}Commands::{scap}(sub) => match sub {{\n"));
                    for cmd in &sub.commands {
                        emit_dispatch_leaf_cmd(&mut out, &sub_enum, &svc_mod, &sub_mod, cmd, "            ");
                    }
                    out.push_str("        },\n");
                } else {
                    out.push_str(&format!("        {variant}Commands::{scap}(sub) => match sub {{\n"));
                    emit_dispatch_match_arms_inner(&mut out, &sub_enum, &svc_mod, sub, &sub_mod, "            ");
                    out.push_str("        },\n");
                }
            }
        } else {
            emit_dispatch_match_arms(&mut out, &format!("{variant}Commands"), &svc_mod, &svc.groups, "");
        }
        out.push_str("    }\n");
        out.push_str("}\n\n");
    }

    out
}

/// Recursively emit match arms for nested dispatch.
fn emit_dispatch_match_arms(
    out: &mut String,
    enum_name: &str,
    svc_mod: &str,
    groups: &[aaz_gen::model::CommandGroup],
    mod_path_prefix: &str,
) {
    let indent = "        ";

    for group in groups {
        let gcap = capitalize(&group.name);
        let gmod = sanitize_ident(&group.name);
        let cur_mod = if mod_path_prefix.is_empty() {
            gmod.clone()
        } else {
            format!("{mod_path_prefix}::{gmod}")
        };

        if group.subgroups.is_empty() {
            // Leaf group — match each command variant
            out.push_str(&format!("{indent}{enum_name}::{gcap}(sub) => match sub {{\n"));
            for cmd in &group.commands {
                let vcap = capitalize(&cmd.verb);
                let fn_name = sanitize_ident(&cmd.verb);
                let sub_enum = format!("{}{}Commands", enum_name.replace("Commands", ""), gcap);
                let call = build_call_expr(cmd, svc_mod, &cur_mod, &fn_name);
                let is_list = (cmd.verb == "list" || cmd.is_paged) && cmd.operation.method == "GET";
                let is_delete = cmd.operation.method == "DELETE";

                out.push_str(&format!("{indent}    {sub_enum}::{vcap}(args) => {{\n"));
                if is_delete {
                    out.push_str(&format!("{indent}        {call}.await?;\n"));
                    out.push_str(&format!("{indent}        Ok(None)\n"));
                } else if is_list {
                    out.push_str(&format!("{indent}        let r = {call}.await?;\n"));
                    out.push_str(&format!("{indent}        Ok(Some(serde_json::Value::Array(r)))\n"));
                } else {
                    out.push_str(&format!("{indent}        let r = {call}.await?;\n"));
                    out.push_str(&format!("{indent}        Ok(Some(r))\n"));
                }
                out.push_str(&format!("{indent}    }}\n"));
            }
            out.push_str(&format!("{indent}}},\n"));
        } else {
            // Group with subgroups — dispatch direct commands + recurse for subgroups
            let sub_enum_prefix = format!("{}{}",  enum_name.replace("Commands", ""), gcap);
            let sub_enum = format!("{sub_enum_prefix}Commands");

            out.push_str(&format!("{indent}{enum_name}::{gcap}(sub) => match sub {{\n"));

            // Direct commands
            for cmd in &group.commands {
                let vcap = capitalize(&cmd.verb);
                let fn_name = sanitize_ident(&cmd.verb);
                let call = build_call_expr(cmd, svc_mod, &cur_mod, &fn_name);
                let is_list = (cmd.verb == "list" || cmd.is_paged) && cmd.operation.method == "GET";
                let is_delete = cmd.operation.method == "DELETE";

                out.push_str(&format!("{indent}    {sub_enum}::{vcap}(args) => {{\n"));
                if is_delete {
                    out.push_str(&format!("{indent}        {call}.await?;\n"));
                    out.push_str(&format!("{indent}        Ok(None)\n"));
                } else if is_list {
                    out.push_str(&format!("{indent}        let r = {call}.await?;\n"));
                    out.push_str(&format!("{indent}        Ok(Some(serde_json::Value::Array(r)))\n"));
                } else {
                    out.push_str(&format!("{indent}        let r = {call}.await?;\n"));
                    out.push_str(&format!("{indent}        Ok(Some(r))\n"));
                }
                out.push_str(&format!("{indent}    }}\n"));
            }

            // Subgroups — recurse
            for sub in &group.subgroups {
                let scap = capitalize(&sub.name);
                let sub_sub_enum = format!("{sub_enum_prefix}{scap}Commands");
                let sub_mod = format!("{cur_mod}::{}", sanitize_ident(&sub.name));

                if sub.subgroups.is_empty() {
                    // Leaf subgroup
                    out.push_str(&format!("{indent}    {sub_enum}::{scap}(sub2) => match sub2 {{\n"));
                    for cmd in &sub.commands {
                        let vcap = capitalize(&cmd.verb);
                        let fn_name = sanitize_ident(&cmd.verb);
                        let call = build_call_expr(cmd, svc_mod, &sub_mod, &fn_name);
                        let is_list = (cmd.verb == "list" || cmd.is_paged) && cmd.operation.method == "GET";
                        let is_delete = cmd.operation.method == "DELETE";

                        out.push_str(&format!("{indent}        {sub_sub_enum}::{vcap}(args) => {{\n"));
                        if is_delete {
                            out.push_str(&format!("{indent}            {call}.await?;\n"));
                            out.push_str(&format!("{indent}            Ok(None)\n"));
                        } else if is_list {
                            out.push_str(&format!("{indent}            let r = {call}.await?;\n"));
                            out.push_str(&format!("{indent}            Ok(Some(serde_json::Value::Array(r)))\n"));
                        } else {
                            out.push_str(&format!("{indent}            let r = {call}.await?;\n"));
                            out.push_str(&format!("{indent}            Ok(Some(r))\n"));
                        }
                        out.push_str(&format!("{indent}        }}\n"));
                    }
                    out.push_str(&format!("{indent}    }},\n"));
                } else {
                    // Nested subgroup — recurse
                    out.push_str(&format!("{indent}    {sub_enum}::{scap}(sub2) => match sub2 {{\n"));
                    emit_dispatch_match_arms_inner(out, &sub_sub_enum, &svc_mod, sub, &sub_mod, &format!("{indent}        "));
                    out.push_str(&format!("{indent}    }},\n"));
                }
            }

            out.push_str(&format!("{indent}}},\n"));
        }
    }
}

/// Inner recursive dispatch for deeply nested groups.
fn emit_dispatch_match_arms_inner(
    out: &mut String,
    enum_name: &str,
    svc_mod: &str,
    group: &aaz_gen::model::CommandGroup,
    mod_path: &str,
    indent: &str,
) {
    // Direct commands
    for cmd in &group.commands {
        let vcap = capitalize(&cmd.verb);
        let fn_name = sanitize_ident(&cmd.verb);
        let call = build_call_expr(cmd, svc_mod, mod_path, &fn_name);
        let is_list = (cmd.verb == "list" || cmd.is_paged) && cmd.operation.method == "GET";
        let is_delete = cmd.operation.method == "DELETE";

        out.push_str(&format!("{indent}{enum_name}::{vcap}(args) => {{\n"));
        if is_delete {
            out.push_str(&format!("{indent}    {call}.await?;\n"));
            out.push_str(&format!("{indent}    Ok(None)\n"));
        } else if is_list {
            out.push_str(&format!("{indent}    let r = {call}.await?;\n"));
            out.push_str(&format!("{indent}    Ok(Some(serde_json::Value::Array(r)))\n"));
        } else {
            out.push_str(&format!("{indent}    let r = {call}.await?;\n"));
            out.push_str(&format!("{indent}    Ok(Some(r))\n"));
        }
        out.push_str(&format!("{indent}}}\n"));
    }

    // Subgroups
    for sub in &group.subgroups {
        let scap = capitalize(&sub.name);
        let enum_prefix = enum_name.replace("Commands", "");
        let sub_enum = format!("{enum_prefix}{scap}Commands");
        let sub_mod = format!("{mod_path}::{}", sanitize_ident(&sub.name));

        if sub.subgroups.is_empty() {
            out.push_str(&format!("{indent}{enum_name}::{scap}(sub) => match sub {{\n"));
            for cmd in &sub.commands {
                let vcap = capitalize(&cmd.verb);
                let fn_name = sanitize_ident(&cmd.verb);
                let call = build_call_expr(cmd, svc_mod, &sub_mod, &fn_name);
                let is_list = (cmd.verb == "list" || cmd.is_paged) && cmd.operation.method == "GET";
                let is_delete = cmd.operation.method == "DELETE";

                out.push_str(&format!("{indent}    {sub_enum}::{vcap}(args) => {{\n"));
                if is_delete {
                    out.push_str(&format!("{indent}        {call}.await?;\n"));
                    out.push_str(&format!("{indent}        Ok(None)\n"));
                } else if is_list {
                    out.push_str(&format!("{indent}        let r = {call}.await?;\n"));
                    out.push_str(&format!("{indent}        Ok(Some(serde_json::Value::Array(r)))\n"));
                } else {
                    out.push_str(&format!("{indent}        let r = {call}.await?;\n"));
                    out.push_str(&format!("{indent}        Ok(Some(r))\n"));
                }
                out.push_str(&format!("{indent}    }}\n"));
            }
            out.push_str(&format!("{indent}}},\n"));
        } else {
            out.push_str(&format!("{indent}{enum_name}::{scap}(sub) => match sub {{\n"));
            emit_dispatch_match_arms_inner(out, &sub_enum, &svc_mod, sub, &sub_mod, &format!("{indent}    "));
            out.push_str(&format!("{indent}}},\n"));
        }
    }
}

/// Emit a single leaf command dispatch match arm.
fn emit_dispatch_leaf_cmd(
    out: &mut String,
    enum_name: &str,
    svc_mod: &str,
    mod_path: &str,
    cmd: &aaz_gen::model::CommandDef,
    indent: &str,
) {
    let vcap = capitalize(&cmd.verb);
    let fn_name = sanitize_ident(&cmd.verb);
    let call = build_call_expr(cmd, svc_mod, mod_path, &fn_name);
    let is_list = (cmd.verb == "list" || cmd.is_paged) && cmd.operation.method == "GET";
    let is_delete = cmd.operation.method == "DELETE";

    out.push_str(&format!("{indent}{enum_name}::{vcap}(args) => {{\n"));
    if is_delete {
        out.push_str(&format!("{indent}    {call}.await?;\n"));
        out.push_str(&format!("{indent}    Ok(None)\n"));
    } else if is_list {
        out.push_str(&format!("{indent}    let r = {call}.await?;\n"));
        out.push_str(&format!("{indent}    Ok(Some(serde_json::Value::Array(r)))\n"));
    } else {
        out.push_str(&format!("{indent}    let r = {call}.await?;\n"));
        out.push_str(&format!("{indent}    Ok(Some(r))\n"));
    }
    out.push_str(&format!("{indent}}}\n"));
}

/// Build the function call expression for a command dispatch.
fn build_call_expr(
    cmd: &aaz_gen::model::CommandDef,
    svc_mod: &str,
    mod_path: &str,
    fn_name: &str,
) -> String {
    let url_args: Vec<&aaz_gen::model::ArgDef> = cmd.args.iter()
        .filter(|a| is_url_arg(a, cmd)).collect();
    let body_only: Vec<&aaz_gen::model::ArgDef> = cmd.args.iter()
        .filter(|a| !is_url_arg(a, cmd) && cmd.body_mappings.iter().any(|m| m.arg_name == a.name))
        .collect();

    let mut call_args = Vec::new();
    for a in &url_args {
        let f = sanitize_ident(&a.name);
        if a.required { call_args.push(format!("&args.{f}")); }
        else { call_args.push(format!("args.{f}.as_deref().unwrap_or_default()")); }
    }
    for a in &body_only {
        let f = sanitize_ident(&a.name);
        if a.required { call_args.push(format!("&args.{f}")); }
        else { call_args.push(format!("args.{f}.as_deref()")); }
    }
    let args_str = call_args.join(", ");

    format!("crate::generated::{svc_mod}::{mod_path}::{fn_name}({args_str})")
}

// --- Helpers ---

/// Emit body construction code from body_mappings.
fn emit_body_construction(cmd: &aaz_gen::model::CommandDef) -> String {
    let mut out = String::new();

    if cmd.body_mappings.is_empty() {
        out.push_str("    let body = serde_json::json!({});\n");
        return out;
    }

    out.push_str("    let mut body = serde_json::json!({});\n");

    for mapping in &cmd.body_mappings {
        let arg = sanitize_ident(&mapping.arg_name);
        let json_parts: Vec<&str> = mapping.json_path.split('.').collect();

        // Check if this arg is required or optional
        let is_required = cmd.args.iter()
            .find(|a| a.name == mapping.arg_name)
            .map(|a| a.required)
            .unwrap_or(false);

        // Build nested path assignment
        let json_accessor = json_parts.iter()
            .map(|p| format!("[\"{p}\"]"))
            .collect::<String>();

        let value_expr = match mapping.value_type.as_str() {
            "AAZStrType" => format!("serde_json::Value::String({arg}.to_string())"),
            "AAZBoolType" => format!("serde_json::Value::Bool({arg}.parse().unwrap_or(false))"),
            "AAZIntType" => format!("serde_json::Value::Number(serde_json::Number::from({arg}.parse::<i64>().unwrap_or(0)))"),
            "AAZDictType" => format!("serde_json::Value::String({arg}.to_string())"), // Simplified
            _ => format!("serde_json::Value::String({arg}.to_string())"),
        };

        if is_required {
            // Ensure parent objects exist
            if json_parts.len() > 1 {
                for i in 1..json_parts.len() {
                    let parent = json_parts[..i].iter()
                        .map(|p| format!("[\"{p}\"]"))
                        .collect::<String>();
                    out.push_str(&format!(
                        "    if body{parent}.is_null() {{ body{parent} = serde_json::json!({{}}); }}\n"
                    ));
                }
            }
            out.push_str(&format!("    body{json_accessor} = {value_expr};\n"));
        } else {
            out.push_str(&format!("    if let Some({arg}) = {arg} {{\n"));
            if json_parts.len() > 1 {
                for i in 1..json_parts.len() {
                    let parent = json_parts[..i].iter()
                        .map(|p| format!("[\"{p}\"]"))
                        .collect::<String>();
                    out.push_str(&format!(
                        "        if body{parent}.is_null() {{ body{parent} = serde_json::json!({{}}); }}\n"
                    ));
                }
            }
            out.push_str(&format!("        body{json_accessor} = {value_expr};\n"));
            out.push_str("    }\n");
        }
    }

    out
}

fn sanitize_ident(s: &str) -> String {
    let name = s.replace('-', "_").to_lowercase();
    match name.as_str() {
        "type" => "type_".to_string(),
        "move" => "move_".to_string(),
        "path" => "path_".to_string(),
        _ => name,
    }
}

fn capitalize(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let mut r = c.to_uppercase().to_string();
                    r.push_str(chars.as_str());
                    r
                }
            }
        })
        .collect()
}

fn has_url_placeholder(arg_name: &str, url: &str) -> bool {
    find_url_placeholder(arg_name, url).is_some()
}

/// Check if an arg is a URL parameter using the parsed explicit map.
/// Falls back to heuristic if map is empty.
fn is_url_arg(arg: &aaz_gen::model::ArgDef, cmd: &aaz_gen::model::CommandDef) -> bool {
    if !cmd.url_param_map.is_empty() {
        cmd.url_param_map.iter().any(|m| m.arg_name == arg.name)
    } else {
        has_url_placeholder(&arg.name, &cmd.operation.url_template)
    }
}

/// Find the URL placeholder name for a given arg, using the parsed map.
fn find_url_placeholder_for_cmd(arg_name: &str, cmd: &aaz_gen::model::CommandDef) -> Option<String> {
    if !cmd.url_param_map.is_empty() {
        cmd.url_param_map.iter()
            .find(|m| m.arg_name == arg_name)
            .map(|m| m.placeholder.clone())
    } else {
        find_url_placeholder(arg_name, &cmd.operation.url_template)
    }
}

/// Check if an arg should be included as a function/struct parameter.
/// True for URL-referenced args and body-mapped args.
fn is_relevant_arg(arg: &aaz_gen::model::ArgDef, cmd: &aaz_gen::model::CommandDef) -> bool {
    is_url_arg(arg, cmd)
        || cmd.body_mappings.iter().any(|m| m.arg_name == arg.name)
}

/// Count commands recursively across all groups and subgroups.
fn count_commands_recursive(groups: &[aaz_gen::model::CommandGroup]) -> usize {
    groups.iter().map(|g| g.commands.len() + count_commands_recursive(&g.subgroups)).sum()
}

/// Collect all (group_mod_path, command) pairs recursively for flat dispatch.
fn collect_all_commands<'a>(
    groups: &'a [aaz_gen::model::CommandGroup],
    prefix: &str,
) -> Vec<(String, &'a aaz_gen::model::CommandDef)> {
    let mut result = Vec::new();
    for group in groups {
        let mod_path = if prefix.is_empty() {
            sanitize_ident(&group.name)
        } else {
            format!("{}::{}", prefix, sanitize_ident(&group.name))
        };
        for cmd in &group.commands {
            result.push((mod_path.clone(), cmd));
        }
        result.extend(collect_all_commands(&group.subgroups, &mod_path));
    }
    result
}

fn find_url_placeholder(arg_name: &str, url: &str) -> Option<String> {
    let re = regex::Regex::new(r"\{(\w+)\}").unwrap();
    let placeholders: Vec<String> = re.captures_iter(url)
        .map(|c| c.get(1).unwrap().as_str().to_string())
        .filter(|p| p != "subscriptionId")
        .collect();

    // Direct exact match (case insensitive)
    for ph in &placeholders {
        if ph.to_lowercase() == arg_name.to_lowercase() {
            return Some(ph.clone());
        }
    }

    // resource_group → resourceGroupName
    if arg_name == "resource_group" {
        return placeholders.iter()
            .find(|p| p.to_lowercase().contains("resourcegroup"))
            .cloned();
    }

    // "name" → last *Name placeholder (leaf resource name)
    if arg_name == "name" {
        return placeholders.iter().rev()
            .find(|p| p.to_lowercase().ends_with("name") && !p.to_lowercase().contains("resourcegroup"))
            .cloned();
    }

    // For other *_name args (e.g. nsg_name), match intermediate *Name placeholders
    if arg_name.ends_with("_name") {
        let name_phs: Vec<&String> = placeholders.iter()
            .filter(|p| p.to_lowercase().ends_with("name") && !p.to_lowercase().contains("resourcegroup"))
            .collect();
        if name_phs.len() == 1 {
            // Only one Name placeholder — this *_name arg must map to it
            return Some(name_phs[0].clone());
        } else if name_phs.len() >= 2 {
            // Multiple — return earlier ones (last is reserved for "name")
            return Some(name_phs[name_phs.len() - 2].clone());
        }
    }

    None
}

/// Create a unique CamelCase variant name from a mod path and verb.
fn make_variant_name(mod_path: &str, verb: &str) -> String {
    let parts: Vec<&str> = mod_path.split("::").collect();
    // Always include ALL parts to avoid collisions between promoted subgroups
    let group_prefix: String = parts.iter().map(|p| capitalize(p)).collect::<Vec<_>>().join("");
    format!("{}{}", group_prefix, capitalize(verb))
}

/// Create the CLI command name from a mod path and verb.
fn make_cli_name(mod_path: &str, verb: &str, _top_groups: &[aaz_gen::model::CommandGroup]) -> String {
    let parts: Vec<&str> = mod_path.split("::").collect();
    // Always prefix with full group path for uniqueness
    let sub = parts.join("-");
    format!("{sub}-{verb}")
}
