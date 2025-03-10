use rllm::chat::{ChatProvider, MessageType};
use rllm::{
    chat::{ChatMessage, ChatRole},
};
use crate::customer_llm_backend::AIGCWLLM;

pub async fn generate_msg(git_diff_content: &str) {
    let llm =AIGCWLLM::new("xx".into(), "xx", Some("gpt-4o-mini".into()), None, None,Some(60),  None, None);
    let prompt = format!(
        r#"Generate a concise git commit message based on the following diff changes. Follow these rules:

1. 使用行业标准前缀（feat/fix/docs/style/test/chore）
2. 只保留核心变更描述，不超过12个英文单词
3. 禁止任何技术细节、文件路径或代码引用
4. 聚焦用户/系统可见的核心影响

结构模板：
[类型]: [动词开头描述主要变更]

示例：
fix: handle null pointer in payment processing
feat: add dark mode toggle button
chore: update CI config timeout

需要处理的diff内容：
{}"#,
        git_diff_content
    );
    let messages = vec![ChatMessage {
        role: ChatRole::User,
        message_type: MessageType::default(),
        content: prompt,
    }];

    let chat_resp = llm.chat(&messages).await;
    match chat_resp {
        Ok(text) => println!("Chat response:\n{}", text),
        Err(e) => eprintln!("Chat error: {}", e),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_generate_msg() {
        let diff_content = r#"diff --git a/Cargo.lock b/Cargo.lock
index 7a18ace..dd1ebe0 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -547,8 +547,10 @@ dependencies = [
  "clap",
  "derive-new",
  "dialoguer",
+ "reqwest",
  "rllm",
  "serde",
+ "serde_json",
  "tokio",
  "toml",
 ]
diff --git a/Cargo.toml b/Cargo.toml
index aaee08d..c393134 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -10,5 +10,7 @@ anyhow = "1.0.75"
 serde = { version = "1.0", features = ["derive"] }
 toml = "0.7.0"
 derive-new = "0.7.0"
-rllm = "1.1.7"
-tokio = { version="1.43.0", features = ["test"]}
+rllm = { version="1.1.7" , features = ["ollama"]}
+tokio = { version="1.43.0", features = ["full"]}
+reqwest = "0.12.12"
+serde_json = "1.0.140"
diff --git a/src/customer_llm_backend.rs b/src/customer_llm_backend.rs
index e69de29..14d4b7c 100644
--- a/src/customer_llm_backend.rs
+++ b/src/customer_llm_backend.rs
@@ -0,0 +1,185 @@
+use reqwest::Client;
+use reqwest::header::{HeaderName, HeaderValue};
+use rllm::chat::{ChatMessage, ChatProvider, ChatResponse, ChatRole, Tool};
+use rllm::{async_trait, LLMProvider, ToolCall};
+use rllm::completion::{CompletionProvider, CompletionResponse};
+use rllm::embedding::EmbeddingProvider;
+use rllm::error::LLMError;
+use serde::{Deserialize, Serialize};
+use serde_json::Value;
+
+pub struct AIGCWLLM {
+    pub url: String,
+    pub api_key: Option<String>,
+    pub model: String,
+    pub max_tokens: Option<u32>,
+    pub temperature: Option<f32>,
+    pub system: Option<String>,
+    pub timeout_seconds: Option<u64>,
+    pub stream: Option<bool>,
+    pub top_p: Option<f32>,
+    pub top_k: Option<u32>,
+    client: Client,
+}
+
+#[derive(Serialize)]
+struct AIGCWLLMChatRequest<'a> {
+    model: String,
+    messages: Vec<AIGCWLLMChatMessage<'a>>,
+    // stream: bool,
+    // temperature: Option<f32>,
+}
+
+#[derive(Serialize, Debug)]
+struct AIGCWLLMChatMessage<'a> {
+    role: &'a str,
+    content: &'a str,
+}
+
+
+#[derive(Deserialize, Debug, Default)]
+pub struct AIGCWLLMChatResponse {
+    choices: Vec<AIGCWLLMChatChoice>,
+}
+
+impl std::fmt::Display for AIGCWLLMChatResponse {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        write!(f, "{:?}", self)
+    }
+}
+
+#[derive(Deserialize, Debug)]
+struct AIGCWLLMChatChoice {
+    message: AIGCWLLMChatMsg,
+}
+
+#[derive(Deserialize, Debug)]
+struct AIGCWLLMChatMsg {
+    content: String,
+}
+impl ChatResponse for AIGCWLLMChatResponse {
+    fn text(&self) -> Option<String> {
+        self.choices.first().and_then(|c| {
+            if c.message.content.is_empty() {
+                None
+            } else {
+                Some(c.message.content.clone())
+            }
+        })
+    }
+
+    fn tool_calls(&self) -> Option<Vec<ToolCall>> {
+        None
+    }
+}
+
+impl AIGCWLLM {
+    pub fn new(
+        url: String,
+        api_key: impl Into<String>,
+        model: Option<String>,
+        max_tokens: Option<u32>,
+        temperature: Option<f32>,
+        timeout_seconds: Option<u64>,
+        system: Option<String>,
+        stream: Option<bool>,
+    ) -> Self {
+        let mut builder = Client::builder();
+        if let Some(sec) = timeout_seconds {
+            builder = builder.timeout(std::time::Duration::from_secs(sec));
+        }
+        Self {
+            url,
+            api_key: Some(api_key.into()),
+            model: model.unwrap_or("deepseek-chat".to_string()),
+            max_tokens,
+            temperature,
+            system,
+            timeout_seconds,
+            stream,
+            top_p: None,
+            top_k: None,
+            client: builder.build().expect("Failed to build reqwest Client"),
+
+        }
+    }
+}
+
+
+#[async_trait]
+impl ChatProvider for AIGCWLLM {
+    async fn chat(&self, messages: &[rllm::chat::ChatMessage]) -> Result<Box<dyn ChatResponse>, LLMError> {
+        let mut messages: Vec<AIGCWLLMChatMessage> = messages
+            .iter()
+            .map(|m| AIGCWLLMChatMessage {
+                role: match m.role {
+                    ChatRole::User => "user",
+                    ChatRole::Assistant => "assistant",
+                },
+                content: &m.content,
+            })
+            .collect();
+
+        if let Some(system) = &self.system {
+            messages.insert(
+                0,
+                AIGCWLLMChatMessage {
+                    role: "system",
+                    content: system,
+                },
+            );
+        }
+
+        let body = AIGCWLLMChatRequest {
+            model: self.model.clone(),
+            messages,
+            // temperature: self.temperature,
+            // stream: self.stream.unwrap_or(false),
+        };
+
+        let mut request = self
+            .client
+            .post(self.url.as_str())
+            .header("Content-Type", "application/json")
+            .json(&body);
+        if let Some(api_key) = &self.api_key {
+            request = request.bearer_auth(&api_key);
+        }
+
+        if let Some(timeout) = self.timeout_seconds {
+            request = request.timeout(std::time::Duration::from_secs(timeout));
+        }
+
+        let resp = request.send().await?.error_for_status()?;
+        let json_resp: AIGCWLLMChatResponse = resp.json().await?;
+        Ok(Box::new(json_resp))
+    }
+
+    async fn chat_with_tools(
+        &self,
+        _messages: &[ChatMessage],
+        _tools: Option<&[Tool]>,
+    ) -> Result<Box<dyn ChatResponse>, LLMError> {
+        todo!()
+    }
+}
+
+#[async_trait]
+impl CompletionProvider for AIGCWLLM {
+    async fn complete(&self, _req: &rllm::completion::CompletionRequest) -> Result<CompletionResponse, LLMError> {
+        Ok(CompletionResponse {
+            text: "DeepSeek completion not implemented.".into(),
+        })
+    }
+}
+
+#[async_trait]
+impl EmbeddingProvider for AIGCWLLM {
+    async fn embed(&self, _text: Vec<String>) -> Result<Vec<Vec<f32>>, LLMError> {
+        Err(LLMError::ProviderError(
+            "Embedding not supported".to_string(),
+        ))
+    }
+}
+
+impl LLMProvider for AIGCWLLM {}
\ No newline at end of file
diff --git a/src/llm.rs b/src/llm.rs
index b6c9079..239eb5a 100644
--- a/src/llm.rs
+++ b/src/llm.rs
@@ -1,54 +1,29 @@
-use rllm::chat::MessageType;
+use rllm::chat::{ChatProvider, MessageType};
 use rllm::{
     builder::{LLMBackend, LLMBuilder},
     chat::{ChatMessage, ChatRole},
 };
+use crate::customer_llm_backend::AIGCWLLM;

 pub async fn generate_msg(git_diff_content: &str) {
-    let llm = LLMBuilder::new()
-        .backend(LLMBackend::OpenAI) // or LLMBackend::Anthropic, LLMBackend::Ollama, LLMBackend::DeepSeek, LLMBackend::XAI, LLMBackend::Phind ...
-        .api_key(std::env::var("OPENAI_API_KEY").unwrap_or("sk-TESTKEY".into()))
-        .model("gpt-4o-mini") // or model("claude-3-5-sonnet-20240620") or model("grok-2-latest") or model("deepseek-chat") or model("llama3.1") or model("Phind-70B") ...
-        .max_tokens(1000)
-        .temperature(0.7)
-        .stream(false)
-        .build()
-        .expect("Failed to build LLM");
+    let llm =AIGCWLLM::new("https://oneapi.cheftin.com/v1/chat/completions".into(), "sk-a5lVhxjEoASZ6Swf9504596f003741DfB2A609D107E6180b", Some("gpt-4o-mini".into()), None, None,Some(60),  None, None);
     let prompt = format!(
-        r#"你是一个专业的版本控制助手，需要根据提供的 git diff 内容生成符合 Conventional Commits 规范的提交信息。请按以下步骤处理：
+        r#"Generate a concise git commit message based on the following diff changes. Follow these rules:

-1. 分析代码变更内容：
-   - 识别新增/删除的代码片段
-   - 判断变更类型（feat/fix/chore/docs/style/refactor/test等）
-   - 确定影响范围（模块/组件/功能）
+1. 使用行业标准前缀（feat/fix/docs/style/test/chore）
+2. 只保留核心变更描述，不超过12个英文单词
+3. 禁止任何技术细节、文件路径或代码引用
+4. 聚焦用户/系统可见的核心影响

-2. 生成结构化信息：
-   - 类型(type): 用英文小写开头
-   - 范围(scope): 括号内的模块名称（可选）
-   - 主题(subject): 50字内的简明描述
-   - 正文(body): 说明变动背景和原因（可选）
-   - 页脚(footer): 关联issue或PR（可选）
-
-3. 遵守格式要求：
-   - 首行不超过72字符
-   - 使用命令式现在时态（"add" 而非 "added"）
-   - 正文每行72字符换行
-   - 空行分隔标题、正文和页脚
+结构模板：
+[类型]: [动词开头描述主要变更]

 示例：
-diff --git a/src/utils/date.js b/src/utils/date.js
-+ export function formatTimestamp(ts) {{
-+   return new Date(ts).toISOString().split('T')[0];
-+ }}
-
-生成：
-feat(utils): add timestamp formatting function
-
-Add standardized date formatting utility for consistent date display across UI components. Returns dates in YYYY-MM-DD format.
-
-Closes #123
+fix: handle null pointer in payment processing
+feat: add dark mode toggle button
+chore: update CI config timeout

-现在请处理以下 git diff 内容：
+需要处理的diff内容：
 {}"#;
        let result = generate_msg(diff_content).await;
    }
}