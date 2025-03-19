# aigcw (AI Git Commit Wrapper) 🦀🤖

[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/aigcw)](https://crates.io/crates/aigcw)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

**AI-Powered Conventional Commits Generator for Git**

## Features ✨

- 🤖 Generate [Conventional Commits](https://www.conventionalcommits.org/) via LLMs
- 🦀 Rust-native performance with <1ms overhead
- 🔌 Zero-config Git hook integration
- 🌐 Multi-LLM support: OpenAI/Claude/Ollama/Azure
- ⚡ Smart diff filtering with regex rules
- 📝 Interactive message editing

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

```bash
# Initialize git hook (run once)
aigcw install-hook

# Commit with AI (requires API key)
git add .
git commit  # Auto-generates: feat(parser): add diff filtering logic
```

## Configuration

Create `~/.aigcw/config.toml`:

```toml
[core]
model = "gpt-4-turbo"  # gpt-3.5-turbo/claude-3/ollama/azure
lang = "en"            # en|zh|ja|es

[openai]
api_key = "${ENV_OPENAI_KEY}"  # Env var substitution

[template]
style = "conventional"  # conventional|angular|semantic
max_diff_lines = 200     # Truncate large diffs
```

## Supported Models

| Provider  | Example Models   | Required Env Vars        |
| --------- | ---------------- | ------------------------ |
| OpenAI    | GPT-4/GPT-3.5    | `OPENAI_API_KEY`         |
| Anthropic | Claude 3         | `ANTHROPIC_API_KEY`      |
| Ollama    | CodeLlama/Llama2 | `OLLAMA_HOST` (optional) |
| Azure     | GPT series       | `AZURE_OPENAI_ENDPOINT`  |

## Development

```bash
# Build
cargo build --release

# Install from local
cargo install --path .
```

## License

MIT © 2025 aigcw
