//! OpenAI and OpenAI-Compatible Provider (DeepSeek, GLM, MiniMax)
use super::LlmProviderTrait;
use crate::error::{OraError, Result};
use crate::llm::client::{LlmResponse, Message, ToolCall, Usage};
use crate::llm::ToolDefinition;
use async_trait::async_trait;

pub struct OpenAiCompatibleProvider {
    pub default_base_url: String,
    pub provider_name: String,
}

#[async_trait]
impl LlmProviderTrait for OpenAiCompatibleProvider {
    async fn chat(
        &self,
        model: &str,
        api_key: Option<&str>,
        base_url: Option<&str>,
        max_tokens: u32,
        temperature: f32,
        messages: Vec<Message>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<LlmResponse> {
        let api_key = api_key.ok_or_else(|| OraError::CredentialNotFound {
            provider: self.provider_name.clone(),
        })?;

        let client = reqwest::Client::new();
        let base = base_url.unwrap_or(&self.default_base_url);
        let url = format!("{}/chat/completions", base);

        let messages_json: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages_json,
            "max_tokens": max_tokens,
            "temperature": temperature
        });

        if let Some(tools_list) = tools {
            if !tools_list.is_empty() {
                body["tools"] =
                    serde_json::to_value(tools_list).map_err(|e| OraError::ConfigError {
                        field: "tools".into(),
                        message: e.to_string(),
                    })?;
            }
        }

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| OraError::NetworkError {
                message: format!("{} request failed: {}", self.provider_name, e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(OraError::LlmError {
                provider: self.provider_name.clone(),
                message: format!("{} error {}: {}", self.provider_name, status, text),
                status_code: Some(status.as_u16()),
            });
        }

        let json: serde_json::Value =
            response.json().await.map_err(|e| OraError::NetworkError {
                message: format!("Failed to parse response: {}", e),
            })?;

        let choice = json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .ok_or_else(|| OraError::LlmError {
                provider: self.provider_name.clone(),
                message: "No choices in response".to_string(),
                status_code: None,
            })?;

        let content = choice
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        let usage = Usage {
            input_tokens: json
                .get("usage")
                .and_then(|u| u.get("prompt_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            output_tokens: json
                .get("usage")
                .and_then(|u| u.get("completion_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            total_tokens: json
                .get("usage")
                .and_then(|u| u.get("total_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        };

        let tool_calls = choice
            .get("message")
            .and_then(|m| m.get("tool_calls"))
            .and_then(|tc| tc.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|tc| {
                        Some(ToolCall {
                            id: tc.get("id")?.as_str()?.to_string(),
                            name: tc.get("function")?.get("name")?.as_str()?.to_string(),
                            arguments: tc
                                .get("function")?
                                .get("arguments")?
                                .as_str()
                                .and_then(|s| serde_json::from_str(s).ok())
                                .unwrap_or(serde_json::json!({})),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            tool_calls,
            usage,
        })
    }
}
