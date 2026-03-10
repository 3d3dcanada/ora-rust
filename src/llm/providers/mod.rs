//! LLM Providers module
use crate::error::Result;
use crate::llm::client::{LlmResponse, Message};
use crate::llm::ToolDefinition;
use async_trait::async_trait;

pub mod anthropic;
pub mod local;
pub mod openai;

#[async_trait]
pub trait LlmProviderTrait: Send + Sync {
    async fn chat(
        &self,
        model: &str,
        api_key: Option<&str>,
        base_url: Option<&str>,
        max_tokens: u32,
        temperature: f32,
        messages: Vec<Message>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<LlmResponse>;
}
