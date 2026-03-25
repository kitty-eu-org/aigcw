# aigcw (AI Git Commit Wrapper) 🦀🤖

[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/aigcw)](https://crates.io/crates/aigcw)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

**AI-Powered Conventional Commits Generator for Git**

## Features ✨

- 🤖 AI-generated commit messages from staged diff via LLM
- 🎨 Interactive commit type selector with emoji (feat, fix, docs, ...)
- 🔢 Optional issue number embedding — `feat(#123): ✨ message`
- 🌐 Multi-LLM support: OpenAI / Anthropic / Ollama / DeepSeek / XAI / Phind / Google / Groq / Custom
- 🦀 Rust-native, minimal overhead

## Installation

### Quick Install (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/kitty-eu-org/aigcw/main/scripts/install.sh | bash
```

### From Cargo

```bash
cargo install aigcw
```

### From Source

```bash
git clone https://github.com/kitty-eu-org/aigcw.git
cd aigcw
cargo install --path .
```

## Usage

`gcw` is a drop-in wrapper around `git`. Use it in place of `git`:

```bash
# Stage changes and commit interactively
git add .
gcw commit

# Interactive flow:
#  1. Select commit type  →  feat / fix / docs / ...
#  2. Enter issue number  →  123  (or press Enter to skip)
#  3. Enter message       →  or leave blank to generate via LLM
#
# Result: feat(#123): ✨ add new feature
#    or:  feat: ✨ add new feature  (if issue skipped)

# Pass a message directly (skips LLM generation)
gcw commit -m "initial setup"

# All other git commands pass through unchanged
gcw push
gcw pull
gcw status
```

## Configuration

On first run, `gcw` creates a config file at `~/.config/aigcw/config.toml`.

Example config:

```toml
config_version = 1

[llm_config]
provider = "OpenAI"   # OpenAI | Anthropic | Ollama | DeepSeek | XAI | Phind | Google | Groq | CUSTOM
enable = true
api_key = "sk-..."
model = "gpt-4o"
# url = "https://custom-endpoint/v1"  # required for CUSTOM provider
```

You can also customise commit types by creating `.commitconfig.toml` in your project root:

```toml
[emoji]
enable = true

[[types]]
name = "feat"
emoji = "✨"
desc = "A new feature"

[[types]]
name = "fix"
emoji = "🐛"
desc = "A bug fix"
```

## Development

```bash
# Build
cargo build --release

# Install from local
cargo install --path .
```

## License

MIT © 2025 aigcw
