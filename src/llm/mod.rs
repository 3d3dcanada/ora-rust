//! OrA LLM Module
//!
//! Single LLM integration - handles all agentic work internally.
//! Supports OpenAI, Anthropic, MiniMax, DeepSeek, GLM, and Ollama.

pub mod client;
pub mod prompts;
pub mod providers;
pub mod tools;

pub use client::LlmClient;
pub use prompts::SystemPrompt;
pub use tools::ToolDefinition;
