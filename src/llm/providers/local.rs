//! Local Provider (Ollama / vLLM)
use async_trait::async_trait;
use serde::Deserialize;

use super::LlmProviderTrait;
use crate::error::{OraError, Result};
use crate::llm::client::{LlmResponse, Message, Usage};
use crate::llm::ToolDefinition;

pub const DEFAULT_OLLAMA_BASE_URL: &str = "http://localhost:11434";
const AUTO_MODEL: &str = "auto";
const PREFERRED_MODEL_FAMILIES: &[&str] =
    &["qwen2.5-coder", "qwen3", "deepseek", "llama3", "llama"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalModelInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    #[serde(default)]
    model: Option<String>,
}

pub fn normalize_base_url(base_url: Option<&str>) -> String {
    base_url
        .unwrap_or(DEFAULT_OLLAMA_BASE_URL)
        .trim_end_matches('/')
        .to_string()
}

pub fn select_model(models: &[LocalModelInfo], requested_model: &str) -> Option<String> {
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

pub async fn resolve_model(base_url: Option<&str>, requested_model: &str) -> Result<String> {
    let models = list_models(base_url).await?;
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

pub async fn list_models(base_url: Option<&str>) -> Result<Vec<LocalModelInfo>> {
    let client = reqwest::Client::new();
    let base = normalize_base_url(base_url);
    let url = format!("{}/api/tags", base);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|error| OraError::NetworkError {
            message: format!("Ollama model discovery failed: {}", error),
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(OraError::LlmError {
            provider: "ollama".to_string(),
            message: format!("Ollama model discovery error {}: {}", status, text),
            status_code: Some(status.as_u16()),
        });
    }

    let payload: TagsResponse = response
        .json()
        .await
        .map_err(|error| OraError::NetworkError {
            message: format!("Failed to parse Ollama model catalog: {}", error),
        })?;

    let mut models = payload
        .models
        .into_iter()
        .map(|model| {
            let id = model.model.unwrap_or_else(|| model.name.clone());
            LocalModelInfo {
                id: id.clone(),
                name: model.name,
            }
        })
        .collect::<Vec<_>>();

    models.sort_by(|left, right| left.id.cmp(&right.id));
    models.dedup_by(|left, right| left.id == right.id);
    Ok(models)
}

fn select_preferred_model(models: &[LocalModelInfo]) -> Option<String> {
    for family in PREFERRED_MODEL_FAMILIES {
        let family_match = family.to_ascii_lowercase();
        if let Some(model) = models.iter().find(|model| {
            let id = model.id.to_ascii_lowercase();
            let name = model.name.to_ascii_lowercase();
            id.starts_with(&family_match)
                || name.starts_with(&family_match)
                || id.contains(&format!("{}:", family_match))
                || name.contains(&format!("{}:", family_match))
        }) {
            return Some(model.id.clone());
        }
    }

    models.first().map(|model| model.id.clone())
}

pub struct LocalProvider;

#[async_trait]
impl LlmProviderTrait for LocalProvider {
    async fn chat(
        &self,
        model: &str,
        _api_key: Option<&str>,
        base_url: Option<&str>,
        max_tokens: u32,
        temperature: f32,
        messages: Vec<Message>,
        _tools: Option<Vec<ToolDefinition>>,
    ) -> Result<LlmResponse> {
        let client = reqwest::Client::new();
        let base = normalize_base_url(base_url);
        let model = resolve_model(Some(&base), model).await?;
        let url = format!("{}/api/chat", base);

        let ollama_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|message| {
                serde_json::json!({
                    "role": message.role,
                    "content": message.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": model,
            "messages": ollama_messages,
            "stream": false,
            "options": {
                "temperature": temperature,
                "num_predict": max_tokens,
            }
        });

        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|error| OraError::NetworkError {
                message: format!("Ollama request failed: {}", error),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(OraError::LlmError {
                provider: "ollama".to_string(),
                message: format!("Ollama error {}: {}", status, text),
                status_code: Some(status.as_u16()),
            });
        }

        let json: serde_json::Value =
            response
                .json()
                .await
                .map_err(|error| OraError::NetworkError {
                    message: format!("Failed to parse Ollama response: {}", error),
                })?;

        let content = json
            .get("message")
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .unwrap_or_default()
            .to_string();

        let usage = Usage {
            input_tokens: json
                .get("prompt_eval_count")
                .and_then(|value| value.as_u64())
                .unwrap_or(0) as u32,
            output_tokens: json
                .get("eval_count")
                .and_then(|value| value.as_u64())
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

#[cfg(test)]
mod tests {
    use super::{select_model, LocalModelInfo};

    fn sample_models() -> Vec<LocalModelInfo> {
        vec![
            LocalModelInfo {
                id: "deepseek-r1:8b".to_string(),
                name: "deepseek-r1:8b".to_string(),
            },
            LocalModelInfo {
                id: "qwen3:8b".to_string(),
                name: "qwen3:8b".to_string(),
            },
            LocalModelInfo {
                id: "qwen2.5-coder:1.5b".to_string(),
                name: "qwen2.5-coder:1.5b".to_string(),
            },
        ]
    }

    #[test]
    fn auto_selection_prefers_known_model_families() {
        let selected = select_model(&sample_models(), "auto");
        assert_eq!(selected.as_deref(), Some("qwen2.5-coder:1.5b"));
    }

    #[test]
    fn prefix_selection_matches_installed_models() {
        let selected = select_model(&sample_models(), "deepseek-r1");
        assert_eq!(selected.as_deref(), Some("deepseek-r1:8b"));
    }
}
