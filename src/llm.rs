use crate::app_config::{LLMConfig, LLMProvider};
use crate::customer_llm_backend::AIGCWLLM;
use rllm::builder::{LLMBackend, LLMBuilder};
use rllm::chat::{ChatMessage, ChatRole};
use rllm::chat::{ChatProvider, MessageType};
use std::str::FromStr;
use crate::commit_types::CommitTypeConfig;

impl From<&LLMProvider> for LLMBackend {
    fn from(provider: &LLMProvider) -> Self {
        match provider {
            LLMProvider::OpenAI => LLMBackend::OpenAI,
            LLMProvider::Anthropic => LLMBackend::Anthropic,
            LLMProvider::Ollama => LLMBackend::Ollama,
            LLMProvider::DeepSeek => LLMBackend::DeepSeek,
            LLMProvider::XAI => LLMBackend::XAI,
            LLMProvider::Phind => LLMBackend::Phind,
            _ => panic!("Invalid LLMProvider"),
        }
    }
}
pub async fn generate_msg(
    commit_type_str: &str,
    git_diff_content: &str,
    llm_config: &LLMConfig,
) -> anyhow::Result<String> {
    if !llm_config.enable {
        return Ok("".into());
    }
    let llm = if llm_config.is_custom() {
        let llm = AIGCWLLM::new(
            llm_config.url.clone().unwrap(),
            llm_config.api_key.clone().unwrap(),
            llm_config.model.clone(),
            None,
            None,
            Some(60),
            None,
            None,
        );
        Box::new(llm)
    } else {
        LLMBuilder::new()
            .backend(LLMBackend::from(&llm_config.provider)) // or LLMBackend::Anthropic, LLMBackend::Ollama, LLMBackend::DeepSeek, LLMBackend::XAI, LLMBackend::Phind ...
            .api_key(llm_config.api_key.clone().unwrap())
            .model(llm_config.model.clone().unwrap()) // or model("claude-3-5-sonnet-20240620") or model("grok-2-latest") or model("deepseek-chat") or model("llama3.1") or model("Phind-70B") ...
            .stream(false)
            .build()
            .expect("Failed to build LLM")
    };

    let prompt = format!(
        r#"Generate a concise git commit message based on the selected commit type and diff. Requirements:

1. Commit type [{}] defines the message's intent, but should NOT appear in output
2. Start with a strong action verb aligned with the type's purpose (add/fix/improve/etc)
3. Focus on user-facing value specific to the commit type
4. Maximum 12 words, no technical details/paths/code

Type-verb mapping guidance:
• feat: add, introduce, implement, enable
• fix: resolve, prevent, avoid, repair
• perf: optimize, reduce, accelerate, speed up
• docs: document, describe, clarify
• test: verify, validate, check

Examples (type in brackets for reference only):
[feat] → Add quick filters to report dashboard
[fix] → Retain form data after network errors
[perf] → Reduce PDF generation memory usage
[docs] → Clarify multi-factor auth setup steps

Diff to analyze:
{}"#,
        commit_type_str
        ,git_diff_content
    );
    let messages = vec![ChatMessage {
        role: ChatRole::User,
        message_type: MessageType::default(),
        content: prompt,
    }];

    let chat_resp = llm.chat(&messages).await?;
    Ok(chat_resp.text().expect("call llm error"))
}
