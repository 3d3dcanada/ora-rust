//! OrA Error Types
//!
//! Centralized error handling with domain-specific error types.

use serde::{Deserialize, Serialize};

/// Main Result type for OrA
pub type Result<T> = std::result::Result<T, OraError>;

/// Core error types for OrA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OraError {
    // Constitution Errors
    PrimeDirectiveViolation {
        message: String,
        operation: String,
    },
    ProhibitedOperation {
        operation: String,
        reason: String,
    },
    ConstitutionalViolation {
        rule: String,
        details: String,
    },

    // Authority Errors
    AuthorityInsufficient {
        required: u8,
        current: u8,
        operation: String,
    },
    AuthorityEscalationFailed {
        reason: String,
        required_level: u8,
    },

    // Vault Errors
    VaultLocked,
    VaultCorrupted,
    CredentialNotFound {
        provider: String,
    },
    EncryptionError {
        message: String,
    },

    // Agent/LLM Errors
    LlmError {
        provider: String,
        message: String,
        status_code: Option<u16>,
    },
    ModelNotAvailable {
        model: String,
    },
    ToolExecutionFailed {
        tool: String,
        message: String,
    },
    RateLimitExceeded {
        provider: String,
        retry_after: Option<u64>,
    },
    AgentError {
        message: String,
    },
    AgentGraphError {
        message: String,
    },

    // Security Errors
    SecurityBlocked {
        reason: String,
    },
    SecurityGateBlocked {
        gate: String,
        reason: String,
        pattern: Option<String>,
    },
    InvalidSession {
        session_id: String,
    },
    AuthenticationFailed {
        reason: String,
    },

    // Audit Errors
    AuditChainBroken {
        expected_hash: String,
        found_hash: String,
    },
    AuditWriteFailed {
        reason: String,
    },

    // Infrastructure Errors
    NetworkError {
        message: String,
    },
    FileSystemError {
        path: String,
        message: String,
    },
    ConfigError {
        field: String,
        message: String,
    },
    WebSocketError {
        message: String,
    },
    InvalidMessage {
        message_type: String,
        reason: String,
    },
    ClientNotFound {
        client_id: String,
    },

    // Process Errors
    ProcessError {
        message: String,
    },
}

impl OraError {
    pub fn user_message(&self) -> String {
        match self {
            OraError::PrimeDirectiveViolation { message, .. } => {
                format!("Operation blocked by Prime Directive: {}", message)
            }
            OraError::ProhibitedOperation { operation, .. } => {
                format!("Operation '{}' is prohibited", operation)
            }
            OraError::AuthorityInsufficient {
                required,
                current,
                operation,
            } => {
                format!(
                    "Insufficient authority for '{}'. Required: A{}, Current: A{}",
                    operation, required, current
                )
            }
            OraError::VaultLocked => "Vault is locked".to_string(),
            OraError::CredentialNotFound { provider } => {
                format!("No credentials for: {}", provider)
            }
            OraError::LlmError {
                provider, message, ..
            } => {
                format!("LLM error from {}: {}", provider, message)
            }
            OraError::SecurityGateBlocked { reason, .. } => {
                format!("Security blocked: {}", reason)
            }
            _ => "An error occurred".to_string(),
        }
    }

    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            OraError::VaultCorrupted
                | OraError::AuditChainBroken { .. }
                | OraError::PrimeDirectiveViolation { .. }
        )
    }
}

impl std::fmt::Display for OraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for OraError {}

impl From<std::io::Error> for OraError {
    fn from(e: std::io::Error) -> Self {
        OraError::FileSystemError {
            path: "unknown".to_string(),
            message: e.to_string(),
        }
    }
}

impl From<serde_json::Error> for OraError {
    fn from(e: serde_json::Error) -> Self {
        OraError::ConfigError {
            field: "json".to_string(),
            message: e.to_string(),
        }
    }
}

impl From<toml::de::Error> for OraError {
    fn from(e: toml::de::Error) -> Self {
        OraError::ConfigError {
            field: "toml".to_string(),
            message: e.to_string(),
        }
    }
}

impl From<reqwest::Error> for OraError {
    fn from(e: reqwest::Error) -> Self {
        OraError::NetworkError {
            message: e.to_string(),
        }
    }
}
