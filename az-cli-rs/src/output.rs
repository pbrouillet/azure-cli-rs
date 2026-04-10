/// Output formatting — json, jsonc, table, tsv, yaml, none.
///
/// Pipeline: command result → --query JMESPath → output format → stdout
use crate::cli::OutputFormat;
use crate::error::Result;
use crate::profile::Profile;
use colored::Colorize;

/// Apply JMESPath query (if any) and format the output.
pub fn format_and_print(
    value: &serde_json::Value,
    format: OutputFormat,
    query: Option<&str>,
) -> Result<()> {
    // Step 1: Apply JMESPath query
    let result = if let Some(q) = query {
        apply_jmespath(value, q)?
    } else {
        value.clone()
    };

    // Step 2: Format output
    match format {
        OutputFormat::Json => print_json(&result),
        OutputFormat::Jsonc => print_jsonc(&result),
        OutputFormat::Table => print_table(&result),
        OutputFormat::Tsv => print_tsv(&result),
        OutputFormat::Yaml => print_yaml(&result),
        OutputFormat::Yamlc => print_yamlc(&result),
        OutputFormat::None => Ok(()),
    }
}

/// Apply a JMESPath expression to a JSON value.
/// Evaluate a JMESPath expression against a JSON value.
pub fn jmespath_eval(value: &serde_json::Value, expr: &str) -> serde_json::Value {
    apply_jmespath(value, expr).unwrap_or(serde_json::Value::Null)
}

fn apply_jmespath(value: &serde_json::Value, expr: &str) -> Result<serde_json::Value> {
    let expression = jmespath::compile(expr).map_err(|e| {
        crate::error::AzrsError::General(format!("Invalid JMESPath expression: {e}"))
    })?;

    let json_str = serde_json::to_string(value)?;
    let jmes_value = jmespath::Variable::from_json(&json_str).map_err(|e| {
        crate::error::AzrsError::General(format!("JMESPath conversion failed: {e}"))
    })?;

    let result = expression.search(&jmes_value).map_err(|e| {
        crate::error::AzrsError::General(format!("JMESPath evaluation failed: {e}"))
    })?;

    // Convert back to serde_json::Value
    let result_str = serde_json::to_string(&*result)?;
    Ok(serde_json::from_str(&result_str)?)
}

/// Pretty-printed JSON.
fn print_json(value: &serde_json::Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

/// Colorized JSON.
fn print_jsonc(value: &serde_json::Value) -> Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    for line in json.lines() {
        println!("{}", colorize_json_line(line));
    }
    Ok(())
}

/// Table output — auto-detect columns from top-level scalar fields.
/// Skips `id`, `type`, `etag` (matching az Python _TableOutput._auto_table_item).
fn print_table(value: &serde_json::Value) -> Result<()> {
    let items = match value {
        serde_json::Value::Array(arr) => arr.clone(),
        serde_json::Value::Null => return Ok(()),
        other => vec![other.clone()],
    };

    if items.is_empty() {
        return Ok(());
    }

    // Collect columns from the first item (top-level scalar fields only)
    let skip_keys = ["id", "type", "etag"];
    let columns: Vec<String> = if let Some(serde_json::Value::Object(first)) = items.first() {
        first
            .keys()
            .filter(|k| {
                !skip_keys.contains(&k.to_lowercase().as_str())
                    && matches!(
                        first.get(k.as_str()),
                        Some(serde_json::Value::String(_))
                            | Some(serde_json::Value::Number(_))
                            | Some(serde_json::Value::Bool(_))
                            | Some(serde_json::Value::Null)
                    )
            })
            .cloned()
            .collect()
    } else {
        // Not an object — just print values
        for item in &items {
            println!("{}", value_to_str(item));
        }
        return Ok(());
    };

    if columns.is_empty() {
        // No scalar columns found — fall back to JSON
        return print_json(value);
    }

    // Calculate column widths
    let mut widths: Vec<usize> = columns.iter().map(|c| capitalize(c).len()).collect();
    let rows: Vec<Vec<String>> = items
        .iter()
        .map(|item| {
            columns
                .iter()
                .enumerate()
                .map(|(i, col)| {
                    let val = item
                        .get(col.as_str())
                        .map(|v| value_to_str(v))
                        .unwrap_or_default();
                    if val.len() > widths[i] {
                        widths[i] = val.len();
                    }
                    val
                })
                .collect()
        })
        .collect();

    // Cap widths at 50 chars
    for w in &mut widths {
        if *w > 50 {
            *w = 50;
        }
    }

    // Print header
    let header: Vec<String> = columns
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{:<width$}", capitalize(c), width = widths[i]))
        .collect();
    println!("{}", header.join("  "));

    let sep: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
    println!("{}", sep.join("  "));

    // Print rows
    for row in &rows {
        let formatted: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, val)| format!("{:<width$}", truncate(val, widths[i]), width = widths[i]))
            .collect();
        println!("{}", formatted.join("  "));
    }

    Ok(())
}

/// TSV output — shallow tab-separated values.
fn print_tsv(value: &serde_json::Value) -> Result<()> {
    let items = match value {
        serde_json::Value::Array(arr) => arr.clone(),
        serde_json::Value::Null => return Ok(()),
        other => vec![other.clone()],
    };

    for item in &items {
        match item {
            serde_json::Value::Object(map) => {
                let vals: Vec<String> = map.values().map(|v| value_to_str(v)).collect();
                println!("{}", vals.join("\t"));
            }
            _ => println!("{}", value_to_str(item)),
        }
    }
    Ok(())
}

/// YAML output.
fn print_yaml(value: &serde_json::Value) -> Result<()> {
    let yaml = serde_yaml::to_string(value)
        .map_err(|e| crate::error::AzrsError::General(format!("YAML error: {e}")))?;
    print!("{yaml}");
    Ok(())
}

/// Colorized YAML output.
fn print_yamlc(value: &serde_json::Value) -> Result<()> {
    let yaml = serde_yaml::to_string(value)
        .map_err(|e| crate::error::AzrsError::General(format!("YAML error: {e}")))?;
    for line in yaml.lines() {
        println!("{}", colorize_yaml_line(line));
    }
    Ok(())
}

/// Print a summary after login (unchanged — uses stderr, not affected by --output).
pub fn print_login_summary(profile: &Profile) {
    let subs = &profile.subscriptions;
    if subs.is_empty() {
        eprintln!("No subscriptions found.");
        return;
    }

    eprintln!(
        "\nLogged in. {} subscription(s) found.\n",
        subs.len()
    );

    eprintln!(
        "  {:<40} {:<36} {}",
        "Name", "SubscriptionId", "Default"
    );
    eprintln!("  {}", "-".repeat(90));

    for sub in subs {
        let default_marker = if sub.is_default { " *" } else { "" };
        eprintln!(
            "  {:<40} {:<36}{}",
            truncate(&sub.name, 38),
            sub.id,
            default_marker
        );
    }
    eprintln!();
}

// --- Helpers ---

fn value_to_str(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => String::new(),
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

fn colorize_json_line(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with('"') && trimmed.contains(':') {
        // Key-value line: colorize key
        if let Some(colon_pos) = line.find(':') {
            let key_part = &line[..colon_pos + 1];
            let val_part = &line[colon_pos + 1..];
            let val_trimmed = val_part.trim();
            let colored_val = if val_trimmed.starts_with('"') {
                val_part.green().to_string()
            } else if val_trimmed == "true" || val_trimmed == "false"
                || val_trimmed == "null"
                || val_trimmed.trim_end_matches(',').starts_with('"')
            {
                val_part.green().to_string()
            } else {
                val_part.cyan().to_string()
            };
            format!("{}{}", key_part.blue(), colored_val)
        } else {
            line.to_string()
        }
    } else {
        line.to_string()
    }
}

fn colorize_yaml_line(line: &str) -> String {
    let trimmed = line.trim_start();
    // YAML key: value lines
    if let Some(colon_pos) = trimmed.find(": ") {
        let indent = &line[..line.len() - trimmed.len()];
        let key = &trimmed[..colon_pos + 1]; // includes the colon
        let val = &trimmed[colon_pos + 2..]; // after ": "
        let colored_val = if val == "true" || val == "false" || val == "null" || val == "~" {
            val.cyan().to_string()
        } else if val.starts_with('\'') || val.starts_with('"') {
            val.green().to_string()
        } else if val.parse::<f64>().is_ok() {
            val.cyan().to_string()
        } else {
            val.green().to_string()
        };
        format!("{}{} {}", indent, key.blue(), colored_val)
    } else if trimmed.starts_with("- ") {
        // List items
        line.to_string()
    } else if trimmed.ends_with(':') {
        // Section headers (key with no value)
        let indent = &line[..line.len() - trimmed.len()];
        format!("{}{}", indent, trimmed.blue())
    } else {
        line.to_string()
    }
}
