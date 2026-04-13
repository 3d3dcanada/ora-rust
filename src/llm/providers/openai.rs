//! OpenAI and OpenAI-Compatible Provider (DeepSeek, GLM, MiniMax)
use super::LlmProviderTrait;
use crate::error::{OraError, Result};
use crate::llm::client::{LlmResponse, Message, ToolCall, Usage};
use crate::llm::ToolDefinition;
use async_trait::async_trait;
use serde::Deserialize;

const AUTO_MODEL: &str = "auto";
const PREFERRED_MODEL_FAMILIES: &[&str] = &[
    "claude-sonnet",
    "claude-opus",
    "qwen",
    "deepseek",
    "llama",
    "nvidia/default",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibleModelInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    #[serde(default)]
    data: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    id: String,
}

pub async fn list_models(
    base_url: Option<&str>,
    api_key: Option<&str>,
) -> Result<Vec<CompatibleModelInfo>> {
    let api_key = api_key.ok_or_else(|| OraError::CredentialNotFound {
        provider: "litellm".to_string(),
    })?;

    let client = reqwest::Client::new();
    let base = base_url
        .unwrap_or("http://127.0.0.1:4000")
        .trim_end_matches('/');
    let url = format!("{}/models", base);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|error| OraError::NetworkError {
            message: format!("OpenAI-compatible model discovery failed: {}", error),
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(OraError::LlmError {
            provider: "litellm".to_string(),
            message: format!(
                "OpenAI-compatible model discovery error {}: {}",
                status, text
            ),
            status_code: Some(status.as_u16()),
        });
    }

    let payload: ModelsResponse =
        response
            .json()
            .await
            .map_err(|error| OraError::NetworkError {
                message: format!("Failed to parse OpenAI-compatible model catalog: {}", error),
            })?;

    let mut models = payload
        .data
        .into_iter()
        .map(|model| CompatibleModelInfo {
            id: model.id.clone(),
            name: model.id,
        })
        .collect::<Vec<_>>();

    models.sort_by(|left, right| left.id.cmp(&right.id));
    models.dedup_by(|left, right| left.id == right.id);
    Ok(models)
}

pub fn select_model(models: &[CompatibleModelInfo], requested_model: &str) -> Option<String> {
    if models.is_empty() {
        return None;
    }

    let requested = requested_model.trim();
    if requested.is_empty() || requested.eq_ignore_ascii_case(AUTO_MODEL) {
        return select_preferred_model(models);
    }

    if let Some(model) = models.iter().find(|model| {
        model.id.eq_ignore_ascii_case(requested) || model.name.eq_ignore_ascii_case(requested)
    }) {
        return Some(model.id.clone());
    }

    let requested_lower = requested.to_ascii_lowercase();
    models
        .iter()
        .find(|model| {
            model.id.to_ascii_lowercase().starts_with(&requested_lower)
                || model
                    .name
                    .to_ascii_lowercase()
                    .starts_with(&requested_lower)
        })
        .map(|model| model.id.clone())
}

pub async fn resolve_model(
    base_url: Option<&str>,
    api_key: Option<&str>,
    requested_model: &str,
) -> Result<String> {
    let models = list_models(base_url, api_key).await?;
    if let Some(model) = select_model(&models, requested_model) {
        return Ok(model);
    }

    let available = models
        .iter()
        .map(|model| model.id.clone())
        .collect::<Vec<_>>()
        .join(", ");

    Err(OraError::ModelNotAvailable {
        model: if available.is_empty() {
            requested_model.to_string()
        } else {
            format!("{} (available: {})", requested_model, available)
        },
    })
}

fn select_preferred_model(models: &[CompatibleModelInfo]) -> Option<String> {
    for family in PREFERRED_MODEL_FAMILIES {
        let family_match = family.to_ascii_lowercase();
        if let Some(model) = models.iter().find(|model| {
            let id = model.id.to_ascii_lowercase();
            let name = model.name.to_ascii_lowercase();
            id.starts_with(&family_match)
                || name.starts_with(&family_match)
                || id.contains(&format!("/{}", family_match))
                || name.contains(&format!("/{}", family_match))
        }) {
            return Some(model.id.clone());
        }
    }

    models.first().map(|model| model.id.clone())
}

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

        let base = base_url.unwrap_or(&self.default_base_url);
        let client = reqwest::Client::new();
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
