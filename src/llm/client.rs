//! OrA LLM Client
//!
//! Single LLM client that handles all agentic work.
//! The model does the agentic heavy lifting - we just proxy requests.

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// LLM Provider
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    MiniMax,
    DeepSeek,
    GLM,
    Ollama,
}

impl LlmProvider {
    pub fn from_str(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "openai" => Self::OpenAI,
            "anthropic" => Self::Anthropic,
            "minimax" => Self::MiniMax,
            "deepseek" => Self::DeepSeek,
            "glm" => Self::GLM,
            "local" | "ollama" => Self::Ollama,
            _ => Self::OpenAI,
        }
    }

    pub fn api_key_env(&self) -> &str {
        match self {
            Self::OpenAI => "OPENAI_API_KEY",
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::MiniMax => "MINIMAX_API_KEY",
            Self::DeepSeek => "DEEPSEEK_API_KEY",
            Self::GLM => "GLM_API_KEY",
            Self::Ollama => "",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Anthropic => "anthropic",
            Self::MiniMax => "minimax",
            Self::DeepSeek => "deepseek",
            Self::GLM => "glm",
            Self::Ollama => "ollama",
        }
    }
}

/// Message for LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Tool call from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
}

/// LLM Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableModel {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub selected: bool,
}

/// LLM Client
pub struct LlmClient {
    provider: LlmProvider,
    model: String,
    api_key: Option<String>,
    base_url: Option<String>,
    max_tokens: u32,
    temperature: f32,
}

impl LlmClient {
    /// Create new LLM client
    pub fn new(
        provider: LlmProvider,
        model: String,
        api_key: Option<String>,
        base_url: Option<String>,
    ) -> Self {
        Self {
            provider,
            model,
            api_key,
            base_url,
            max_tokens: 8192,
            temperature: 0.7,
        }
    }

    /// Create from config
    pub fn from_config(
        provider: &str,
        model: &str,
        api_key: Option<String>,
        base_url: Option<String>,
    ) -> Self {
        let provider = LlmProvider::from_str(provider);
        let api_key = api_key.or_else(|| std::env::var(provider.api_key_env()).ok());
        Self::new(provider, model.to_string(), api_key, base_url)
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn provider_name(&self) -> &'static str {
        self.provider.as_str()
    }

    pub fn configured_model(&self) -> &str {
        &self.model
    }

    pub fn fallback_model_info(&self) -> AvailableModel {
        AvailableModel {
            id: self.model.clone(),
            name: self.model.clone(),
            provider: self.provider_name().to_string(),
            selected: true,
        }
    }

    pub async fn available_models(&self) -> Result<Vec<AvailableModel>> {
        match self.provider {
            LlmProvider::Ollama => {
                let models =
                    crate::llm::providers::local::list_models(self.base_url.as_deref()).await?;
                let selected = crate::llm::providers::local::select_model(&models, &self.model);
                Ok(models
                    .into_iter()
                    .map(|model| AvailableModel {
                        id: model.id.clone(),
                        name: model.name,
                        provider: self.provider_name().to_string(),
                        selected: selected.as_deref() == Some(model.id.as_str()),
                    })
                    .collect())
            }
            _ => Ok(vec![self.fallback_model_info()]),
        }
    }

    fn resolved_model(&self) -> Result<String> {
        Ok(self.model.clone())
    }

    async fn resolve_model(&self) -> Result<String> {
        match self.provider {
            LlmProvider::Ollama => {
                crate::llm::providers::local::resolve_model(self.base_url.as_deref(), &self.model)
                    .await
            }
            _ => self.resolved_model(),
        }
    }

    /// Simple chat with system prompt and user message
    pub async fn chat(&self, system_prompt: &str, user_message: &str) -> Result<String> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_message.to_string(),
            },
        ];

        let response = self.chat_with_messages(messages, None).await?;
        Ok(response.content)
    }

    /// Full chat with messages and optional tools
    pub async fn chat_with_messages(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<crate::llm::ToolDefinition>>,
    ) -> Result<LlmResponse> {
        let provider: Box<dyn crate::llm::providers::LlmProviderTrait> = match self.provider {
            LlmProvider::Ollama => Box::new(crate::llm::providers::local::LocalProvider),
            LlmProvider::OpenAI => {
                Box::new(crate::llm::providers::openai::OpenAiCompatibleProvider {
                    default_base_url: "https://api.openai.com/v1".to_string(),
                    provider_name: "openai".to_string(),
                })
            }
            LlmProvider::DeepSeek => {
                Box::new(crate::llm::providers::openai::OpenAiCompatibleProvider {
                    default_base_url: "https://api.deepseek.com/v1".to_string(),
                    provider_name: "deepseek".to_string(),
                })
            }
            LlmProvider::GLM => Box::new(crate::llm::providers::openai::OpenAiCompatibleProvider {
                default_base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
                provider_name: "glm".to_string(),
            }),
            LlmProvider::MiniMax => {
                Box::new(crate::llm::providers::openai::OpenAiCompatibleProvider {
                    default_base_url: "https://api.minimax.chat/v1".to_string(),
                    provider_name: "minimax".to_string(),
                })
            }
            LlmProvider::Anthropic => Box::new(crate::llm::providers::anthropic::AnthropicProvider),
        };

        let model = self.resolve_model().await?;

        provider
            .chat(
                &model,
                self.api_key.as_deref(),
                self.base_url.as_deref(),
                self.max_tokens,
                self.temperature,
                messages,
                tools,
            )
            .await
    }

    /// Get API key
    pub fn get_api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// Check if configured
    pub fn is_configured(&self) -> bool {
        self.api_key.is_some() || self.provider == LlmProvider::Ollama
    }
}
