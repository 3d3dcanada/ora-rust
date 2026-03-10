//! OrA Memory Module
//!
//! Simple context memory - no external dependencies.
//! Stores conversation history and learns user preferences.

use std::collections::VecDeque;

/// A single context entry
#[derive(Debug, Clone)]
pub struct ContextEntry {
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: u64,
}

/// User preferences learned over time
#[derive(Debug, Clone, Default)]
pub struct UserPreferences {
    /// Preferred coding style
    pub style: Option<String>,
    /// Preferred language
    pub language: Option<String>,
    /// Preferred framework
    pub framework: Option<String>,
    /// Other preferences
    pub extras: std::collections::HashMap<String, String>,
}

/// Ora Memory - simple context for the agent
#[derive(Debug)]
pub struct OraMemory {
    /// Conversation history (FIFO, limited size)
    history: VecDeque<ContextEntry>,

    /// Maximum history entries
    max_history: usize,

    /// Maximum token estimate (rough)
    max_tokens: usize,

    /// Learned user preferences
    preferences: UserPreferences,
}

impl Default for OraMemory {
    fn default() -> Self {
        Self::new(50, 8000)
    }
}

impl OraMemory {
    /// Create new memory with limits
    pub fn new(max_history: usize, max_tokens: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
            max_tokens,
            preferences: UserPreferences::default(),
        }
    }

    /// Create new memory with defaults
    pub fn new_default() -> Self {
        Self::default()
    }

    /// Add user message to context
    pub fn add_user(&mut self, content: &str) {
        let entry = ContextEntry {
            role: "user".to_string(),
            content: content.to_string(),
            timestamp: Self::timestamp_now(),
        };

        self.history.push_back(entry);
        self.prune();
    }

    /// Add assistant response to context
    pub fn add_assistant(&mut self, content: &str) {
        let entry = ContextEntry {
            role: "assistant".to_string(),
            content: content.to_string(),
            timestamp: Self::timestamp_now(),
        };

        self.history.push_back(entry);
        self.prune();
    }

    /// Add system message to context
    pub fn add_system(&mut self, content: &str) {
        let entry = ContextEntry {
            role: "system".to_string(),
            content: content.to_string(),
            timestamp: Self::timestamp_now(),
        };

        self.history.push_back(entry);
        self.prune();
    }

    /// Get current timestamp
    fn timestamp_now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Prune history if too large
    fn prune(&mut self) {
        // Prune by count
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }

        // Rough token estimate pruning (4 chars per token)
        let mut token_count = 0;
        let mut to_remove = Vec::new();

        for (i, entry) in self.history.iter().enumerate() {
            token_count += entry.content.len() / 4;
            if token_count > self.max_tokens {
                to_remove.push(i);
            }
        }

        // Remove oldest entries that exceed limit
        for i in to_remove {
            if i < self.history.len() {
                self.history.remove(i);
            }
        }
    }

    /// Get formatted context for LLM
    pub fn get_context(&self) -> String {
        if self.history.is_empty() {
            return String::new();
        }

        let mut lines = Vec::new();

        // Add preferences if we have them
        if let Some(ref style) = self.preferences.style {
            lines.push(format!("User preference: {} style", style));
        }
        if let Some(ref lang) = self.preferences.language {
            lines.push(format!("User preference: {} language", lang));
        }

        if !lines.is_empty() {
            lines.push(String::new());
        }

        // Add conversation history
        for entry in self.history.iter() {
            let prefix = match entry.role.as_str() {
                "user" => "User",
                "assistant" => "Assistant",
                "system" => "System",
                _ => "Unknown",
            };
            lines.push(format!("{}: {}", prefix, entry.content));
        }

        lines.join("\n")
    }

    /// Get context as messages for LLM API
    pub fn get_messages(&self) -> Vec<crate::llm::client::Message> {
        self.history
            .iter()
            .map(|e| crate::llm::client::Message {
                role: e.role.clone(),
                content: e.content.clone(),
            })
            .collect()
    }

    /// Learn a preference from context
    pub fn learn(&mut self, key: &str, value: &str) {
        match key.to_lowercase().as_str() {
            "style" => self.preferences.style = Some(value.to_string()),
            "language" => self.preferences.language = Some(value.to_string()),
            "framework" => self.preferences.framework = Some(value.to_string()),
            _ => {
                self.preferences
                    .extras
                    .insert(key.to_string(), value.to_string());
            }
        }
    }

    /// Extract preferences from conversation
    pub fn extract_preferences(&mut self) {
        // Look for preference signals in recent history
        for entry in self.history.iter().rev().take(10) {
            let content = entry.content.to_lowercase();

            // Detect programming language preferences
            if content.contains("prefer") || content.contains("like") || content.contains("use") {
                if content.contains("python") {
                    self.preferences.language = Some("Python".to_string());
                } else if content.contains("javascript") || content.contains("typescript") {
                    self.preferences.language = Some("JavaScript/TypeScript".to_string());
                } else if content.contains("rust") {
                    self.preferences.language = Some("Rust".to_string());
                } else if content.contains("go") {
                    self.preferences.language = Some("Go".to_string());
                }

                // Detect style preferences
                if content.contains("functional") {
                    self.preferences.style = Some("Functional".to_string());
                } else if content.contains("oop") || content.contains("object oriented") {
                    self.preferences.style = Some("OOP".to_string());
                } else if content.contains("clean code") {
                    self.preferences.style = Some("Clean Code".to_string());
                }
            }
        }
    }

    /// Get a specific preference
    pub fn get_preference(&self, key: &str) -> Option<String> {
        match key.to_lowercase().as_str() {
            "style" => self.preferences.style.clone(),
            "language" => self.preferences.language.clone(),
            "framework" => self.preferences.framework.clone(),
            _ => self.preferences.extras.get(key).cloned(),
        }
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// Clear preferences
    pub fn clear_preferences(&mut self) {
        self.preferences = UserPreferences::default();
    }

    /// Get history count
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_basic() {
        let mut mem = OraMemory::new_default();

        mem.add_user("Hello");
        mem.add_assistant("Hi there!");

        assert_eq!(mem.len(), 2);

        let context = mem.get_context();
        assert!(context.contains("User: Hello"));
        assert!(context.contains("Assistant: Hi"));
    }

    #[test]
    fn test_preferences() {
        let mut mem = OraMemory::new_default();

        mem.learn("language", "Python");
        mem.learn("style", "functional");

        assert_eq!(mem.get_preference("language"), Some("Python".to_string()));
        assert_eq!(mem.get_preference("style"), Some("functional".to_string()));
    }
}
