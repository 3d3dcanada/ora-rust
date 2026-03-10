//! Anthropic Provider (Claude 3.5 Sonnet/Opus)
use super::LlmProviderTrait;
use crate::error::{OraError, Result};
use crate::llm::client::{LlmResponse, Message, Usage};
use crate::llm::ToolDefinition;
use async_trait::async_trait;

pub struct AnthropicProvider;

#[async_trait]
impl LlmProviderTrait for AnthropicProvider {
    async fn chat(
        &self,
        model: &str,
        api_key: Option<&str>,
        _base_url: Option<&str>,
        max_tokens: u32,
        _temperature: f32, // Anthropic often handles temp differently, ignoring for simplicity here
        messages: Vec<Message>,
        _tools: Option<Vec<ToolDefinition>>, // Tools implementation for Claude requires specific mapping
    ) -> Result<LlmResponse> {
        let api_key = api_key.ok_or_else(|| OraError::CredentialNotFound {
            provider: "anthropic".to_string(),
        })?;

        let client = reqwest::Client::new();
        let url = "https://api.anthropic.com/v1/messages";

        let (system_msg, chat_messages): (Vec<_>, Vec<_>) =
            messages.into_iter().partition(|m| m.role == "system");
        let system = system_msg.first().map(|m| m.content.as_str()).unwrap_or("");

        let messages_json: Vec<serde_json::Value> = chat_messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": if m.role == "user" { "user" } else { "assistant" },
                    "content": m.content
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "system": system,
            "messages": messages_json
        });

        let response = client
            .post(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| OraError::NetworkError {
                message: format!("Anthropic request failed: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(OraError::LlmError {
                provider: "anthropic".to_string(),
                message: format!("Anthropic error {}: {}", status, text),
                status_code: Some(status.as_u16()),
            });
        }

        let json: serde_json::Value =
            response.json().await.map_err(|e| OraError::NetworkError {
                message: format!("Failed to parse Anthropic response: {}", e),
            })?;

        let content = json
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        let usage = Usage {
            input_tokens: json
                .get("usage")
                .and_then(|u| u.get("input_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            output_tokens: json
                .get("usage")
                .and_then(|u| u.get("output_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            total_tokens: 0,
        };

        Ok(LlmResponse {
            content,
            tool_calls: Vec::new(),
            usage,
        })
    }
}
