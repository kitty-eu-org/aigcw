# Native Git Passthrough Design

**Date:** 2026-01-29
**Status:** Approved

## Problem Statement

The current `gcw` git wrapper has three issues that prevent it from behaving like native git:

1. **Parameter passing requires `--` separator** - Commands like `gcw status -s` or `gcw commit -m "message"` require `gcw -- status -s` or `gcw -- commit -- -m "message"`
2. **Non-TTY environment failures** - In non-interactive environments (Claude Code, CI/CD), `dialoguer` throws "not a terminal" error, breaking automation workflows
3. **Help text doesn't show git commands** - `gcw --help` shows clap-generated wrapper help instead of native git help

## Goals

1. Make `gcw` behave exactly like native git for all commands except interactive `commit -m`
2. Automatically detect non-TTY environments and fall back to native git behavior
3. Maintain current interactive commit type selection when in TTY environment
4. Forward `--help` and all other flags directly to native git

## Design

### Architecture Changes

**Current Flow:**
```
User input → clap parser (with external_subcommands) → requires `--` → execute_git()
```

**New Flow:**
```
User input → Manual arg inspection → Route to handler → execute_git()
            ↓
            ├─ commit -m + TTY → interactive selection
            ├─ commit -m + non-TTY → direct passthrough
            └─ all other commands → direct passthrough
```

### Implementation Strategy

#### 1. Remove clap subcommand parsing

Replace the current `allow_external_subcommands = true` approach with manual argument inspection:

```rust
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if should_intercept_commit(&args) {
        handle_interactive_commit(args)
    } else {
        execute_git(&args)
    }
}
```

#### 2. Smart commit detection

```rust
fn should_intercept_commit(args: &[String]) -> bool {
    // Check if first arg is "commit" and "-m" flag is present
    if args.is_empty() || args[0] != "commit" {
        return false;
    }

    // Check if we're in a TTY environment
    if !is_tty() {
        return false;
    }

    // Check if -m or --message is present
    args.iter().any(|arg| arg == "-m" || arg == "--message")
}
```

#### 3. TTY Detection

Use `std::io::IsTerminal` trait (Rust 1.70+):

```rust
use std::io::IsTerminal;

fn is_tty() -> bool {
    std::io::stdin().is_terminal()
}
```

For older Rust versions, fallback to `atty` crate.

#### 4. Argument parsing for commit

Only parse commit-specific flags when intercepting:

```rust
fn handle_interactive_commit(args: Vec<String>) -> anyhow::Result<()> {
    // Parse flags manually or use clap::Parser on remaining args
    let (message_idx, message) = extract_message_flag(&args)?;
    let other_flags = extract_other_flags(&args, message_idx);

    // Show interactive type selection
    let commit_type = select_commit_type()?;

    // Build final git command
    let mut git_args = vec!["commit".to_string()];
    git_args.extend(other_flags);
    git_args.push("-m".to_string());
    git_args.push(format!("{} {}", commit_type, message));

    execute_git(&git_args)
}
```

### Edge Cases

1. **`git commit` without `-m`** → Always passthrough (opens editor)
2. **`git commit --amend -m "msg"`** → Intercept only in TTY
3. **`git commit -am "msg"`** → Parse combined flags correctly
4. **`git commit -m "msg" --no-verify`** → Preserve all extra flags
5. **`git --help`** → Direct passthrough to show native git help
6. **`git commit --help`** → Direct passthrough to show native git commit help
7. **Unknown flags** → Always passthrough (forward compatibility)

### Benefits

- ✅ No more `--` separator needed
- ✅ Works in automation/CI environments
- ✅ Maintains interactive UX when running manually
- ✅ Fully compatible with all git flags and future git versions
- ✅ Zero breaking changes to existing behavior

### Migration

No user migration needed - this is a pure enhancement. All existing workflows continue to work, and new usage patterns are now supported.

## Testing Checklist

- [ ] `gcw status -s` works without `--`
- [ ] `gcw log --oneline` works without `--`
- [ ] `gcw commit -m "message"` shows type selection in terminal
- [ ] `gcw commit -m "message"` in non-TTY (piped input) passes through directly
- [ ] `gcw commit -am "message"` handles combined flags
- [ ] `gcw commit --amend -m "message"` works correctly
- [ ] `gcw commit` without `-m` opens editor normally
- [ ] `gcw --help` shows native git help (not wrapper help)
- [ ] `gcw commit --help` shows native git commit help
