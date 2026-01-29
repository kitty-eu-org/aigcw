pub mod commit_types;
mod git_utils;
mod llm;
mod customer_llm_backend;

use crate::commit_types::load_config;
use dialoguer::{theme::ColorfulTheme, Select};
use std::io::IsTerminal;
use std::process::Command;

/// Check if running in an interactive terminal with both stdin and stdout connected to a TTY.
/// Used to determine whether to show interactive commit type selection.
fn is_tty() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

/// Determine if we should intercept this commit command
/// Only intercept if:
/// 1. First arg is "commit"
/// 2. -m or --message flag is present
/// 3. We're in a TTY environment
fn should_intercept_commit(args: &[String]) -> bool {
    if args.is_empty() || args[0] != "commit" {
        return false;
    }

    if !is_tty() {
        return false;
    }

    args.iter().any(|arg|
        arg == "-m" ||
        arg == "--message" ||
        (arg.starts_with("-m") && arg.len() > 2) ||
        arg.starts_with("--message=")
    )
}

/// Extract the message value from commit args
/// Handles both -m "msg" and -m"msg" formats
fn extract_message(args: &[String]) -> Option<(usize, String)> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "-m" || arg == "--message" {
            // Next arg is the message
            if i + 1 < args.len() {
                return Some((i, args[i + 1].clone()));
            }
        } else if arg.starts_with("-m") && arg.len() > 2 {
            // Combined format: -m"message"
            return Some((i, arg[2..].to_string()));
        } else if arg.starts_with("--message=") {
            // Long format: --message=value
            return Some((i, arg[10..].to_string()));
        }
    }
    None
}

/// Extract all flags except the message flag
fn extract_other_flags(args: &[String], message_idx: usize) -> Vec<String> {
    let mut flags = Vec::new();
    let mut skip_next = false;

    for (i, arg) in args.iter().enumerate() {
        if i == 0 {
            // Skip "commit" command
            continue;
        }

        if skip_next {
            skip_next = false;
            continue;
        }

        if i == message_idx {
            // Skip -m or --message
            if arg == "-m" || arg == "--message" {
                skip_next = true;
            }
            continue;
        }

        flags.push(arg.clone());
    }

    flags
}

/// Handle interactive commit with type selection
fn handle_interactive_commit(args: Vec<String>) -> anyhow::Result<()> {
    let (message_idx, message) = match extract_message(&args) {
        Some(result) => result,
        None => {
            // No -m flag found, shouldn't happen due to should_intercept_commit
            // but pass through anyway
            return execute_git(&args);
        }
    };

    let other_flags = extract_other_flags(&args, message_idx);

    // Load config and show type selection
    let config = load_config()?;
    let selects: Vec<String> = config.types.iter().map(|x| x.show_string()).collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select commit type")
        .items(&selects)
        .interact()?;

    let prefixed_msg = format!("{} {}", selects[selection], message);

    // Build final git command
    let mut git_args = vec!["commit".to_string()];
    git_args.extend(other_flags);
    git_args.push("-m".to_string());
    git_args.push(prefixed_msg);

    execute_git(&git_args)
}


fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if should_intercept_commit(&args) {
        handle_interactive_commit(args)
    } else {
        execute_git(&args)
    }
}

fn execute_git(args: &[String]) -> anyhow::Result<()> {
    let mut binding = Command::new("git");
    let command = binding.args(args);
    let status = command.status()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
