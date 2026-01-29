# Native Git Passthrough Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `gcw` behave like native git by removing `--` separator requirement, auto-detecting non-TTY environments, and forwarding all non-commit commands transparently.

**Architecture:** Replace clap's subcommand parsing with manual argument inspection. Only intercept `commit -m` in TTY environments for interactive type selection. All other commands pass through directly to git.

**Tech Stack:** Rust 1.70+, std::io::IsTerminal for TTY detection, manual argument parsing

---

## Task 1: Add TTY Detection Utility

**Files:**
- Modify: `src/main.rs:1-10` (add imports and utility function)

**Step 1: Add imports for TTY detection**

At the top of `src/main.rs`, add after existing imports:

```rust
use std::io::IsTerminal;
```

**Step 2: Add TTY detection function**

After the imports in `src/main.rs`, before the `Cli` struct definition (around line 10), add:

```rust
/// Check if running in an interactive terminal
fn is_tty() -> bool {
    std::io::stdin().is_terminal()
}
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Successful compilation with existing warnings

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: add TTY detection utility"
```

---

## Task 2: Add Commit Detection Logic

**Files:**
- Modify: `src/main.rs:10-20` (add helper function)

**Step 1: Add commit detection function**

After the `is_tty()` function in `src/main.rs`, add:

```rust
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

    args.iter().any(|arg| arg == "-m" || arg == "--message")
}
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Successful compilation

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: add commit detection logic"
```

---

## Task 3: Add Argument Extraction Functions

**Files:**
- Modify: `src/main.rs:20-80` (add helper functions)

**Step 1: Add message extraction function**

After `should_intercept_commit()` in `src/main.rs`, add:

```rust
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
```

**Step 2: Add flags extraction function**

After `extract_message()` in `src/main.rs`, add:

```rust
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

        if i == message_idx + 1 && (args[message_idx] == "-m" || args[message_idx] == "--message") {
            // Skip the message value
            continue;
        }

        flags.push(arg.clone());
    }

    flags
}
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Successful compilation

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: add argument extraction helpers"
```

---

## Task 4: Add Interactive Commit Handler

**Files:**
- Modify: `src/main.rs:80-120` (add handler function)

**Step 1: Add interactive commit handler**

After the extraction functions in `src/main.rs`, add:

```rust
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
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Successful compilation (may have warnings about unused function)

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: add interactive commit handler"
```

---

## Task 5: Replace Main Function with New Logic

**Files:**
- Modify: `src/main.rs:11-105` (replace Cli struct and main function)

**Step 1: Remove old Cli struct and GitCommand enum**

Delete lines 11-44 in `src/main.rs` (the entire `Cli` struct and `GitCommand` enum definitions).

**Step 2: Replace main function**

Replace the existing `main()` function (lines 46-94) with:

```rust
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if should_intercept_commit(&args) {
        handle_interactive_commit(args)
    } else {
        execute_git(&args)
    }
}
```

**Step 3: Remove unused clap import**

At the top of `src/main.rs`, remove or comment out:

```rust
use clap::Parser;
```

**Step 4: Verify it compiles**

Run: `cargo build`
Expected: Successful compilation

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "refactor: replace clap parsing with manual routing"
```

---

## Task 6: Manual Testing - Passthrough Commands

**Step 1: Build release binary**

Run: `cargo build --release`
Expected: Successful build

**Step 2: Test status command**

Run: `./target/release/gcw status`
Expected: Shows git status output without errors

**Step 3: Test status with flags**

Run: `./target/release/gcw status -s`
Expected: Shows short status output (no `--` needed!)

**Step 4: Test log command**

Run: `./target/release/gcw log --oneline -5`
Expected: Shows last 5 commits in oneline format

**Step 5: Test help**

Run: `./target/release/gcw --help`
Expected: Shows native git help (not clap-generated help)

**Step 6: Test commit help**

Run: `./target/release/gcw commit --help`
Expected: Shows native git commit help

**Step 7: Document test results**

Create file `TESTING.md` with results:

```markdown
# Manual Test Results - Native Passthrough

## Passthrough Commands (✓ means working)
- [ ] `gcw status` -
- [ ] `gcw status -s` -
- [ ] `gcw log --oneline` -
- [ ] `gcw --help` -
- [ ] `gcw commit --help` -
```

**Step 8: Commit test documentation**

```bash
git add TESTING.md
git commit -m "docs: add manual testing checklist"
```

---

## Task 7: Manual Testing - Interactive Commit in TTY

**Step 1: Create test changes**

Run: `echo "test" >> TESTING.md && git add TESTING.md`
Expected: File staged

**Step 2: Test commit with -m in terminal**

Run: `./target/release/gcw commit -m "test message"`
Expected: Shows interactive type selection menu

**Step 3: Select a type and verify**

Action: Use arrow keys to select "feat", press Enter
Expected: Commit succeeds with message like "feat: ✨ test message"

**Step 4: Verify commit was created**

Run: `git log -1 --oneline`
Expected: Shows the commit with prefixed type

**Step 5: Reset the test commit**

Run: `git reset HEAD~1`
Expected: Commit removed, changes back in staging

**Step 6: Update test documentation**

Update `TESTING.md`:

```markdown
## Interactive Commit (TTY)
- [ ] `gcw commit -m "msg"` shows type selection -
- [ ] Selected type is prefixed to message -
```

**Step 7: Commit**

```bash
git add TESTING.md
git commit -m "docs: add TTY commit test results"
```

---

## Task 8: Manual Testing - Non-TTY Passthrough

**Step 1: Test commit in non-TTY environment**

Run: `echo "" | ./target/release/gcw commit -m "non-tty test"`
Expected: Commits directly without interactive menu (passthrough to git)

**Step 2: Verify commit message**

Run: `git log -1 --pretty=%s`
Expected: Shows "non-tty test" (no type prefix)

**Step 3: Reset test commit**

Run: `git reset HEAD~1`

**Step 4: Test with combined flags**

Run: `echo "more test" >> TESTING.md && git add TESTING.md && echo "" | ./target/release/gcw commit -am "combined flags test"`
Expected: Commits with -a flag working

**Step 5: Verify and reset**

Run: `git log -1 --pretty=%s && git reset HEAD~1`
Expected: Shows commit message, then resets

**Step 6: Update test documentation**

Update `TESTING.md`:

```markdown
## Non-TTY Passthrough
- [ ] Piped input skips interactive menu -
- [ ] Message passed through unchanged -
- [ ] Combined flags (-am) work correctly -
```

**Step 7: Commit**

```bash
git add TESTING.md
git commit -m "docs: add non-TTY test results"
```

---

## Task 9: Manual Testing - Edge Cases

**Step 1: Test commit without -m**

Run: `echo "test" >> TESTING.md && git add TESTING.md && GIT_EDITOR=true ./target/release/gcw commit`
Expected: Opens editor (or exits immediately with GIT_EDITOR=true)

**Step 2: Test commit with --amend**

Run: `./target/release/gcw commit --amend -m "amended message"`
Expected: In TTY: shows type selection; creates amended commit

**Step 3: Reset**

Run: `git reset HEAD~1 && git restore --staged TESTING.md && git restore TESTING.md`

**Step 4: Test commit with extra flags**

Run: `echo "test" >> TESTING.md && git add TESTING.md && ./target/release/gcw commit -m "test" --no-verify`
Expected: Shows type selection, includes --no-verify flag

**Step 5: Verify and clean up**

Run: `git log -1 && git reset HEAD~1`

**Step 6: Update test documentation**

Update `TESTING.md`:

```markdown
## Edge Cases
- [ ] `gcw commit` without -m opens editor -
- [ ] `gcw commit --amend -m "msg"` works -
- [ ] Extra flags (--no-verify) preserved -
```

**Step 7: Commit**

```bash
git add TESTING.md
git commit -m "docs: add edge case test results"
```

---

## Task 10: Remove Unused Clap Dependency (Optional Cleanup)

**Files:**
- Modify: `Cargo.toml:8`

**Step 1: Check if clap is still used**

Run: `grep -r "use clap" src/`
Expected: No matches (all clap usage removed)

**Step 2: Comment out clap dependency**

In `Cargo.toml`, change line 8 from:

```toml
clap = { version = "4.4.0", features = ["derive"] }
```

To:

```toml
# clap = { version = "4.4.0", features = ["derive"] }  # Removed - using manual arg parsing
```

**Step 3: Verify it still builds**

Run: `cargo build`
Expected: Successful build, faster compile time

**Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "chore: remove unused clap dependency"
```

---

## Task 11: Update Main Branch Files

**Files:**
- Modify: `Cargo.toml:16` (sync serde_json to main branch)

**Step 1: Check main branch Cargo.toml**

Note: Main branch is missing serde_json dependency that was added in worktree

**Step 2: Prepare change for main branch**

This will be included when merging the branch back to main.

Document in commit message:

```bash
git add Cargo.toml
git commit -m "build: add serde_json dependency (was missing in main)"
```

---

## Task 12: Final Verification

**Step 1: Run full build**

Run: `cargo build --release`
Expected: Clean build with no errors

**Step 2: Check binary size**

Run: `ls -lh target/release/gcw`
Expected: Binary exists, reasonable size

**Step 3: Final test run**

Run all test commands from TESTING.md checklist and mark results

**Step 4: Update TESTING.md with final results**

Mark all checkboxes as complete if passing

**Step 5: Commit final test results**

```bash
git add TESTING.md
git commit -m "docs: mark all tests passing"
```

**Step 6: Review all commits**

Run: `git log --oneline feature/native-git-passthrough ^main`
Expected: See clean, logical commit history

---

## Completion

After all tasks are complete:

1. All tests in TESTING.md should be passing
2. Binary works without `--` separator
3. TTY detection working correctly
4. All edge cases handled

Next steps (use superpowers:finishing-a-development-branch):
- Decide whether to merge directly or create PR
- Clean up worktree after integration
