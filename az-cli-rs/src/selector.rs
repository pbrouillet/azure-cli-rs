/// Interactive subscription selector — shown after login when running in a TTY.
///
/// Mirrors the Python CLI `_subscription_selector.py` behavior:
/// - Shows a numbered list of subscriptions with tenant info
/// - Marks the suggested default with `*`
/// - Lets the user pick by number or press Enter for default
use crate::profile::Subscription;
use std::io::{self, BufRead, Write};

/// Returns true if both stdin and stdout are connected to a terminal.
pub fn is_interactive() -> bool {
    atty_stdin() && atty_stdout()
}

fn atty_stdin() -> bool {
    unsafe { libc_isatty(0) }
}

fn atty_stdout() -> bool {
    unsafe { libc_isatty(1) }
}

#[cfg(unix)]
unsafe fn libc_isatty(fd: i32) -> bool {
    unsafe { libc::isatty(fd) != 0 }
}

#[cfg(not(unix))]
unsafe fn libc_isatty(_fd: i32) -> bool {
    false
}

/// Present an interactive subscription selector and return the selected index.
///
/// If the user presses Enter without typing, returns the default index.
/// Returns `None` if no subscriptions are available.
pub fn select_subscription(subs: &[Subscription]) -> Option<usize> {
    if subs.is_empty() {
        return None;
    }

    // Find the default (first Enabled, or first overall)
    let default_idx = subs
        .iter()
        .position(|s| s.state == "Enabled")
        .unwrap_or(0);

    eprintln!("\n[Tenant and subscription selection]\n");

    // Print header
    eprintln!(
        "{:<6} {:<40} {:<38} {}",
        "No", "Subscription name", "Subscription ID", "Tenant"
    );
    eprintln!(
        "{:<6} {:<40} {:<38} {}",
        "-----",
        "----------------------------------------",
        "--------------------------------------",
        "--------"
    );

    // Print rows
    for (i, sub) in subs.iter().enumerate() {
        let marker = if i == default_idx { " *" } else { "" };
        let tenant_label = sub
            .tenant_display_name
            .as_deref()
            .or(sub.tenant_default_domain.as_deref())
            .unwrap_or(&sub.tenant_id);

        eprintln!(
            "[{:>2}]{:<2} {:<40} {:<38} {}",
            i + 1,
            marker,
            truncate(&sub.name, 38),
            &sub.id,
            truncate(tenant_label, 20),
        );
    }

    // Show default info
    let default_sub = &subs[default_idx];
    let default_tenant = default_sub
        .tenant_display_name
        .as_deref()
        .or(default_sub.tenant_default_domain.as_deref())
        .unwrap_or(&default_sub.tenant_id);
    eprintln!(
        "\nThe default is marked with an *; the default tenant is '{}' and subscription is '{}' ({}).",
        default_tenant, default_sub.name, default_sub.id
    );

    // Prompt
    eprint!("Select a subscription and tenant (Type a number or Enter for no changes): ");
    io::stderr().flush().ok();

    let stdin = io::stdin();
    let line = stdin.lock().lines().next();

    match line {
        Some(Ok(input)) => {
            let input = input.trim();
            if input.is_empty() {
                Some(default_idx)
            } else if let Ok(num) = input.parse::<usize>() {
                if num >= 1 && num <= subs.len() {
                    Some(num - 1)
                } else {
                    eprintln!("Invalid selection. Using default.");
                    Some(default_idx)
                }
            } else {
                eprintln!("Invalid input. Using default.");
                Some(default_idx)
            }
        }
        _ => Some(default_idx),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
