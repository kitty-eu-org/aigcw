pub mod commit_types;
mod git_utils;
mod llm;
mod customer_llm_backend;

use crate::commit_types::load_config;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Select};
use std::process::Command;

#[derive(Parser)]
#[command(
    name = "gcw",
    disable_help_flag = false,
    disable_version_flag = false,
    allow_external_subcommands = true
)]
struct Cli {
    #[command(subcommand)]
    command: GitCommand,
}

#[derive(clap::Subcommand)]
enum GitCommand {
    #[command(name = "commit")]
    Commit {
        #[arg(short = 'a', long)]
        all: bool,

        #[arg(short = 'm', long)]
        message: Option<String>,

        #[arg(short = 'p', long)]
        patch: bool,

        #[arg(long = "amend")]
        amend: bool,

        #[arg(last = true)]
        extra_args: Vec<String>,
    },
    #[command(external_subcommand)]
    Other(Vec<String>),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
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

                let prefixed_msg = format!("{} {}", selects[selection], msg);

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
