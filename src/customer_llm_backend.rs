use reqwest::Client;
use reqwest::header::{HeaderName, HeaderValue};
use rllm::chat::{ChatMessage, ChatProvider, ChatResponse, ChatRole, Tool};
use rllm::{async_trait, LLMProvider, ToolCall};
use rllm::completion::{CompletionProvider, CompletionResponse};
use rllm::embedding::EmbeddingProvider;
use rllm::error::LLMError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct AIGCWLLM {
    pub url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub system: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub stream: Option<bool>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    client: Client,
}

#[derive(Serialize)]
struct AIGCWLLMChatRequest<'a> {
    model: String,
    messages: Vec<AIGCWLLMChatMessage<'a>>,
    // stream: bool,
    // temperature: Option<f32>,
}

#[derive(Serialize, Debug)]
struct AIGCWLLMChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}


#[derive(Deserialize, Debug, Default)]
pub struct AIGCWLLMChatResponse {
    choices: Vec<AIGCWLLMChatChoice>,
}

impl std::fmt::Display for AIGCWLLMChatResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Debug)]
struct AIGCWLLMChatChoice {
    message: AIGCWLLMChatMsg,
}

#[derive(Deserialize, Debug)]
struct AIGCWLLMChatMsg {
    content: String,
}
impl ChatResponse for AIGCWLLMChatResponse {
    fn text(&self) -> Option<String> {
        self.choices.first().and_then(|c| {
            if c.message.content.is_empty() {
                None
            } else {
                Some(c.message.content.clone())
            }
        })
    }

    fn tool_calls(&self) -> Option<Vec<ToolCall>> {
        None
    }
}

impl AIGCWLLM {
    pub fn new(
        url: String,
        api_key: impl Into<String>,
        model: Option<String>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
        timeout_seconds: Option<u64>,
        system: Option<String>,
        stream: Option<bool>,
    ) -> Self {
        let mut builder = Client::builder();
        if let Some(sec) = timeout_seconds {
            builder = builder.timeout(std::time::Duration::from_secs(sec));
        }
        Self {
            url,
            api_key: Some(api_key.into()),
            model: model.unwrap_or("deepseek-chat".to_string()),
            max_tokens,
            temperature,
            system,
            timeout_seconds,
            stream,
            top_p: None,
            top_k: None,
            client: builder.build().expect("Failed to build reqwest Client"),

        }
    }
}


#[async_trait]
impl ChatProvider for AIGCWLLM {
    async fn chat(&self, messages: &[rllm::chat::ChatMessage]) -> Result<Box<dyn ChatResponse>, LLMError> {
        let mut messages: Vec<AIGCWLLMChatMessage> = messages
            .iter()
            .map(|m| AIGCWLLMChatMessage {
                role: match m.role {
                    ChatRole::User => "user",
                    ChatRole::Assistant => "assistant",
                },
                content: &m.content,
            })
            .collect();

        if let Some(system) = &self.system {
            messages.insert(
                0,
                AIGCWLLMChatMessage {
                    role: "system",
                    content: system,
                },
            );
        }

        let body = AIGCWLLMChatRequest {
            model: self.model.clone(),
            messages,
            // temperature: self.temperature,
            // stream: self.stream.unwrap_or(false),
        };

        let mut request = self
            .client
            .post(self.url.as_str())
            .header("Content-Type", "application/json")
            .json(&body);
        if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(&api_key);
        }

        if let Some(timeout) = self.timeout_seconds {
            request = request.timeout(std::time::Duration::from_secs(timeout));
        }

        let resp = request.send().await?.error_for_status()?;
        let json_resp: AIGCWLLMChatResponse = resp.json().await?;
        Ok(Box::new(json_resp))
    }

    async fn chat_with_tools(
        &self,
        _messages: &[ChatMessage],
        _tools: Option<&[Tool]>,
    ) -> Result<Box<dyn ChatResponse>, LLMError> {
        todo!()
    }
}

#[async_trait]
impl CompletionProvider for AIGCWLLM {
    async fn complete(&self, _req: &rllm::completion::CompletionRequest) -> Result<CompletionResponse, LLMError> {
        Ok(CompletionResponse {
            text: "DeepSeek completion not implemented.".into(),
        })
    }
}

#[async_trait]
impl EmbeddingProvider for AIGCWLLM {
    async fn embed(&self, _text: Vec<String>) -> Result<Vec<Vec<f32>>, LLMError> {
        Err(LLMError::ProviderError(
            "Embedding not supported".to_string(),
        ))
    }
}

impl LLMProvider for AIGCWLLM {}