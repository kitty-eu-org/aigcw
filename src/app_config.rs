use anyhow::{Context, Result};
use derive_new::new;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_config_version() -> u32 {
    1
}

#[derive(Debug, Deserialize, Serialize, new)]
pub struct AppConfig {
    #[serde(default = "default_config_version")]
    pub config_version: u32,
    pub llm_config: LLMConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LLMProvider {
    OpenAI,
    /// Anthropic API provider (Claude models)
    Anthropic,
    /// Ollama local LLM provider for self-hosted models
    Ollama,
    /// DeepSeek API provider for their LLM models
    DeepSeek,
    /// X.AI (formerly Twitter) API provider
    XAI,
    /// Phind API provider for code-specialized models
    Phind,
    /// Google Gemini API provider
    Google,
    /// Groq API provider
    Groq,

    CUSTOM,
}

#[derive(Debug, Deserialize, Serialize, new)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub enable: bool,
    pub api_key: Option<String>,
    pub url: Option<String>,
    pub model: Option<String>,
}

impl LLMConfig {
    pub fn is_custom(&self) -> bool {
        matches!(self.provider, LLMProvider::CUSTOM)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new(
            1,
            LLMConfig::new(LLMProvider::OpenAI, false, None, None, None),
        )
    }
}

// 核心逻辑：自定义 macOS 的配置目录
fn get_config_dir() -> Result<PathBuf> {
    let app_name = "aigcw";
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        Ok(home.join(".config").join(app_name))
    }

    #[cfg(not(target_os = "macos"))]
    {
        let proj_dirs = directories::ProjectDirs::from("", "", app_name)
            .context("Failed to get project directories")?;
        Ok(proj_dirs.config_dir().to_path_buf())
    }
}

fn get_config_path() -> anyhow::Result<PathBuf> {
    let config_dir = get_config_dir()?;

    fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("config.toml"))
}

pub fn load_app_config() -> anyhow::Result<AppConfig> {
    let config_path = get_config_path()?;

    let config = if config_path.exists() {
        let config_str = fs::read_to_string(&config_path)?;
        config::Config::builder()
            .add_source(config::File::from_str(
                &config_str,
                config::FileFormat::Toml,
            ))
            .build()?
            .try_deserialize()?
    } else {
        // 创建默认配置
        let default_config = AppConfig::default();
        fs::write(config_path, toml::to_string(&default_config)?)?;
        default_config
    };

    Ok(config)
}

fn init_default_config(config_path: &PathBuf) -> Result<()> {
    if config_path.exists() {
        // 已有配置文件时检查版本
        let config_file = config::File::from(config_path.clone());
        let existing: AppConfig = config::Config::builder()
            .add_source(config_file)
            .build()?
            .try_deserialize()?;

        if existing.config_version < AppConfig::default().config_version {
            println!(
                "发现旧版配置文件 (v{})，正在升级到 v{}...",
                existing.config_version,
                AppConfig::default().config_version
            );
            // add version migrate logic
        }
        return Ok(());
    }

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建配置目录: {}", parent.display()))?;
    }

    let default_config = toml::to_string_pretty(&AppConfig::default())?;
    fs::write(config_path, default_config)
        .with_context(|| format!("无法写入配置文件: {}", config_path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_get_config_path() {
        let config_path = get_config_path().unwrap();
        println!("{:?}", config_path);
    }
}
