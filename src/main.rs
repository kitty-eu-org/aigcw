mod app_config;
pub mod commit_types;
mod customer_llm_backend;
mod git_utils;
mod llm;

use crate::app_config::load_app_config;
use crate::commit_types::load_config;
use crate::git_utils::get_diff_content;
use crate::llm::generate_msg;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Select};
use std::io::IsTerminal;
use std::process::Command;

/// Check if running in an interactive terminal with both stdin and stdout connected to a TTY.
/// Used to determine whether to show interactive commit type selection.
fn is_tty() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

#[derive(Parser, Debug)]
#[command(name = "gcw")]
#[command(about = "AI-powered Git Commit Wrapper", long_about = None)]
enum Cli {
    #[command(external_subcommand)]
    Git(Vec<String>),
}

#[derive(Debug)]
enum GitCommand {
    Commit {
        all: bool,
        message: Option<String>,
        patch: bool,
        amend: bool,
        extra_args: Vec<String>,
    },
    Other(Vec<String>),
}

impl GitCommand {
    fn parse(args: Vec<String>) -> Self {
        if args.is_empty() || args[0] != "commit" {
            return GitCommand::Other(args);
        }

        let mut all = false;
        let mut patch = false;
        let mut amend = false;
        let mut message = None;
        let mut extra_args = Vec::new();
        let mut skip_next = false;

        for (i, arg) in args.iter().enumerate().skip(1) {
            if skip_next {
                skip_next = false;
                continue;
            }

            match arg.as_str() {
                "--all" | "-a" => all = true,
                "--patch" | "-p" => patch = true,
                "--amend" => amend = true,
                "-m" | "--message" => {
                    if i + 1 < args.len() {
                        message = Some(args[i + 1].clone());
                        skip_next = true;
                    }
                }
                _ if arg.starts_with("-m") => {
                    message = Some(arg[2..].to_string());
                }
                _ if arg.starts_with("--message=") => {
                    message = Some(arg[10..].to_string());
                }
                _ => extra_args.push(arg.clone()),
            }
        }

        GitCommand::Commit {
            all,
            message,
            patch,
            amend,
            extra_args,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = GitCommand::parse(args);

    match cli {
        GitCommand::Commit {
            all,
            message,
            patch,
            amend,
            extra_args,
        } => {
            let mut base_args = Vec::new();
            if all {
                base_args.push("--all".to_string());
            }
            if patch {
                base_args.push("--patch".to_string());
            }
            if amend {
                base_args.push("--amend".to_string());
            }

            if let Some(msg) = message {
                let config = load_config()?;
                let selects: Vec<String> = config.types.iter().map(|x| x.show_string()).collect();
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select commit type")
                    .items(&selects)
                    .interact()?;
                let msg = if msg.is_empty() {
                    let git_diff_content = get_diff_content()?;
                    if git_diff_content.is_empty() {
                        println!("No changes to commit.");
                        return Ok(());
                    }
                    let app_config = load_app_config()?;
                    generate_msg(&selects[selection], &git_diff_content, &app_config.llm_config).await?
                } else {
                    msg
                };


                let prefixed_msg = format!("{} {} ", selects[selection], msg);

                let mut args = vec!["commit".to_string()];
                args.extend(base_args);
                args.push("-m".to_string());
                args.push(prefixed_msg);
                args.extend(extra_args);

                execute_git(&args)
            } else {
                let mut args = vec!["commit".to_string()];
                args.extend(base_args);
                args.extend(extra_args);
                execute_git(&args)
            }
        }
        GitCommand::Other(args) => execute_git(&args),
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
