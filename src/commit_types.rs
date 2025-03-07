use derive_new::new;
use std::path::Path;
#[derive(serde::Deserialize, new)]
pub struct Config {
    pub emoji: EmojiConfig,
    pub types: Vec<CommitTypeConfig>,
}

#[derive(serde::Deserialize, new)]
pub struct EmojiConfig {
    enable: bool,
}

#[derive(serde::Deserialize, Clone, new)]
pub struct CommitTypeConfig {
    name: String,  // 类型名称
    emoji: String, // 对应 emoji
    desc: String,  // 类型描述
}

impl CommitTypeConfig {
    pub fn show_string(&self) -> String {
        format!("{}: {}", self.name, self.emoji)
    }
}

pub fn load_config() -> anyhow::Result<Config> {
    let commit_config_path = Path::new("example.txt");
    if commit_config_path.exists() {
        let content = std::fs::read_to_string(".commitconfig.toml")?;
        Ok(toml::from_str(&content)?)
    } else {
        let types = vec![
            CommitTypeConfig {
                name: "feat".into(),
                emoji: "✨".into(),
                desc: "新增功能".into(),
            },
            CommitTypeConfig {
                name: "fix".into(),
                emoji: "🐛".into(),
                desc: "Bug修复".into(),
            },
            CommitTypeConfig {
                name: "docs".into(),
                emoji: "📚".into(),
                desc: "文档更新".into(),
            },
            CommitTypeConfig {
                name: "style".into(),
                emoji: "🎨".into(),
                desc: "代码样式调整".into(),
            },
            CommitTypeConfig {
                name: "refactor".into(),
                emoji: "♻️".into(),
                desc: "代码重构".into(),
            },
            CommitTypeConfig {
                name: "perf".into(),
                emoji: "⚡️".into(),
                desc: "性能优化".into(),
            },
            CommitTypeConfig {
                name: "test".into(),
                emoji: "✅".into(),
                desc: "测试相关".into(),
            },
            CommitTypeConfig {
                name: "build".into(),
                emoji: "📦️".into(),
                desc: "构建系统变更".into(),
            },
            CommitTypeConfig {
                name: "ci".into(),
                emoji: "👷".into(),
                desc: "CI配置变更".into(),
            },
            CommitTypeConfig {
                name: "chore".into(),
                emoji: "🔧".into(),
                desc: "其他杂项".into(),
            },
            CommitTypeConfig {
                name: "revert".into(),
                emoji: "⏪️".into(),
                desc: "提交回滚".into(),
            },
        ];
        Ok(Config::new(EmojiConfig::new(true), types))
    }
}
