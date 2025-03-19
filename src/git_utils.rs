use anyhow::Context;
use std::process::Command;

pub fn get_diff_content() -> anyhow::Result<String> {
    let diff_output = Command::new("git")
        .args(["diff", "--staged"])
        .output()
        .context("Failed to execute git diff --staged")?;

    if !diff_output.status.success() {
        anyhow::bail!("Git diff command failed: {:?}", diff_output.status);
    }

    let diff_content = String::from_utf8_lossy(&diff_output.stdout);
    if diff_content.trim().is_empty() {
        anyhow::bail!("No staged changes detected.");
    }
    Ok(diff_content.to_string())
}
