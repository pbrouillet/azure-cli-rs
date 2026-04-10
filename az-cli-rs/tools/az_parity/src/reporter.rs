use crate::differ::{Difference, DiffKind};
use colored::*;
use std::time::Duration;

#[derive(Debug)]
pub struct TestResult {
    pub name: String,
    pub command: String,
    pub status: TestStatus,
    pub az_time: Option<Duration>,
    pub azrs_time: Option<Duration>,
    pub diffs: Vec<Difference>,
    pub az_stdout: Option<String>,
    pub azrs_stdout: Option<String>,
}

#[derive(Debug)]
pub enum TestStatus {
    Pass,
    Diff,
    Skip(String),
    Error(String),
}

fn format_duration(d: Duration) -> String {
    if d.as_secs() > 0 {
        format!("{:.1}s", d.as_secs_f64())
    } else {
        format!("{}ms", d.as_millis())
    }
}

fn format_value_short(v: &serde_json::Value) -> String {
    let s = v.to_string();
    if s.len() > 60 {
        format!("{}…", &s[..57])
    } else {
        s
    }
}

pub fn print_report(results: &[TestResult], verbose: bool, json_output: bool) {
    if json_output {
        print_json_report(results);
        return;
    }

    println!();
    println!("{}", "═══ az-parity report ═══".bold());
    println!();

    for result in results {
        let timing = match (result.az_time, result.azrs_time) {
            (Some(az), Some(azrs)) => {
                format!(" (az: {}, azrs: {})", format_duration(az), format_duration(azrs))
            }
            (Some(az), None) => format!(" (az: {})", format_duration(az)),
            (None, Some(azrs)) => format!(" (azrs: {})", format_duration(azrs)),
            _ => String::new(),
        };

        match &result.status {
            TestStatus::Pass => {
                println!(
                    "{} {:<45} {}{}",
                    "✓".green().bold(),
                    result.name,
                    "PASS".green().bold(),
                    timing.dimmed()
                );
            }
            TestStatus::Diff => {
                println!(
                    "{} {:<45} {}{}",
                    "✗".red().bold(),
                    result.name,
                    "DIFF".red().bold(),
                    timing.dimmed()
                );
                for diff in &result.diffs {
                    print_diff(diff);
                }
            }
            TestStatus::Skip(reason) => {
                println!(
                    "{} {:<45} {}",
                    "⚠".yellow().bold(),
                    result.name,
                    "SKIP".yellow().bold()
                );
                println!("  {} {}", "→".dimmed(), reason.dimmed());
            }
            TestStatus::Error(err) => {
                println!(
                    "{} {:<45} {}",
                    "✗".red().bold(),
                    result.name,
                    "ERROR".red().bold()
                );
                println!("  {} {}", "→".dimmed(), err.red());
            }
        }

        if verbose {
            if let TestStatus::Diff | TestStatus::Error(_) = &result.status {
                if let Some(ref az_out) = result.az_stdout {
                    println!("  {} {}", "az stdout:".dimmed(), truncate(az_out.trim(), 200).dimmed());
                }
                if let Some(ref azrs_out) = result.azrs_stdout {
                    println!("  {} {}", "azrs stdout:".dimmed(), truncate(azrs_out.trim(), 200).dimmed());
                }
                println!();
            }
        }
    }

    // Summary
    let pass = results.iter().filter(|r| matches!(r.status, TestStatus::Pass)).count();
    let diff = results.iter().filter(|r| matches!(r.status, TestStatus::Diff)).count();
    let skip = results.iter().filter(|r| matches!(r.status, TestStatus::Skip(_))).count();
    let err = results.iter().filter(|r| matches!(r.status, TestStatus::Error(_))).count();

    println!();
    print!("Summary: ");
    print!("{}", format!("{} passed", pass).green());
    if diff > 0 {
        print!(", {}", format!("{} diff", diff).red());
    }
    if skip > 0 {
        print!(", {}", format!("{} skipped", skip).yellow());
    }
    if err > 0 {
        print!(", {}", format!("{} errors", err).red());
    }
    println!();
}

fn print_diff(diff: &Difference) {
    let arrow = "→".dimmed();
    match &diff.kind {
        DiffKind::AzOnly(val) => {
            println!(
                "  {} {} az has {} (not in azrs): {}",
                arrow,
                diff.path.cyan(),
                diff.path.split('.').last().unwrap_or("?"),
                format_value_short(val).yellow()
            );
        }
        DiffKind::AzrsOnly(val) => {
            println!(
                "  {} {} azrs has extra field: {}",
                arrow,
                diff.path.cyan(),
                format_value_short(val).yellow()
            );
        }
        DiffKind::ValueMismatch { az, azrs } => {
            println!(
                "  {} {} az={} azrs={}",
                arrow,
                diff.path.cyan(),
                format_value_short(az).green(),
                format_value_short(azrs).red()
            );
        }
        DiffKind::TypeMismatch { az, azrs } => {
            println!(
                "  {} {} type mismatch: az={} azrs={}",
                arrow,
                diff.path.cyan(),
                az.green(),
                azrs.red()
            );
        }
        DiffKind::ArrayLengthMismatch { az, azrs } => {
            println!(
                "  {} {} array length: az={} azrs={}",
                arrow,
                diff.path.cyan(),
                az.to_string().green(),
                azrs.to_string().red()
            );
        }
        DiffKind::ExitCode { az, azrs } => {
            println!(
                "  {} exit code: az={} azrs={}",
                arrow,
                az.to_string().green(),
                azrs.to_string().red()
            );
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}…", &s[..max])
    } else {
        s.to_string()
    }
}

fn print_json_report(results: &[TestResult]) {
    let report: Vec<serde_json::Value> = results.iter().map(|r| {
        let status = match &r.status {
            TestStatus::Pass => "pass",
            TestStatus::Diff => "diff",
            TestStatus::Skip(_) => "skip",
            TestStatus::Error(_) => "error",
        };

        let diffs: Vec<serde_json::Value> = r.diffs.iter().map(|d| {
            let (kind, detail) = match &d.kind {
                DiffKind::AzOnly(v) => ("az_only", serde_json::json!({"value": v})),
                DiffKind::AzrsOnly(v) => ("azrs_only", serde_json::json!({"value": v})),
                DiffKind::ValueMismatch { az, azrs } => ("value_mismatch", serde_json::json!({"az": az, "azrs": azrs})),
                DiffKind::TypeMismatch { az, azrs } => ("type_mismatch", serde_json::json!({"az": az, "azrs": azrs})),
                DiffKind::ArrayLengthMismatch { az, azrs } => ("array_length", serde_json::json!({"az": az, "azrs": azrs})),
                DiffKind::ExitCode { az, azrs } => ("exit_code", serde_json::json!({"az": az, "azrs": azrs})),
            };
            serde_json::json!({
                "path": d.path,
                "kind": kind,
                "detail": detail,
            })
        }).collect();

        serde_json::json!({
            "name": r.name,
            "command": r.command,
            "status": status,
            "az_time_ms": r.az_time.map(|d| d.as_millis()),
            "azrs_time_ms": r.azrs_time.map(|d| d.as_millis()),
            "diffs": diffs,
        })
    }).collect();

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
