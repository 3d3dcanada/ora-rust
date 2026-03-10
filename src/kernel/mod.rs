//! OrA Kernel Module
//!
//! Core kernel containing Constitution enforcement and Authority level management.
//! This is the heart of OrA's governance system.

pub mod agent;
pub mod authority;
pub mod constitution;
pub mod memory;
pub mod tools;
pub mod validator;
pub mod web_search;

pub use agent::{
    Agent, AgentResult, ExecutionPlan, ExecutionStatus, ExecutionStep, VerificationResult,
};
pub use authority::{AuthorityKernel, AuthorityLevel, Session};
pub use constitution::Constitution;
pub use memory::{ContextEntry, OraMemory, UserPreferences};
pub use tools::{ToolExecutor, ToolResult};
pub use validator::Validator;
pub use web_search::WebSearchService;

use crate::error::Result;
use crate::security::vault::Vault;
use std::path::PathBuf;
use std::sync::Arc;

/// The main Kernel - orchestrates all operations
pub struct Kernel {
    /// Workspace root directory
    workspace_root: PathBuf,

    /// The Constitution
    constitution: Constitution,

    /// Authority kernel
    authority: AuthorityKernel,

    /// Operation validator
    validator: Validator,

    /// Vault reference
    vault: Arc<Vault>,
}

impl Kernel {
    /// Create a new kernel
    pub fn new(workspace_root: PathBuf, vault: Arc<Vault>) -> Result<Self> {
        let constitution = Constitution::new();
        let validator = Validator::new(constitution.clone());
        let authority = AuthorityKernel::default();

        Ok(Self {
            workspace_root,
            constitution,
            authority,
            validator,
            vault,
        })
    }

    /// Get workspace root
    pub fn workspace_root(&self) -> &PathBuf {
        &self.workspace_root
    }

    /// Get constitution
    pub fn constitution(&self) -> &Constitution {
        &self.constitution
    }

    /// Validate an operation
    pub fn validate(
        &self,
        operation: &str,
        details: &str,
    ) -> Result<crate::kernel::validator::ValidationResult> {
        Ok(self.validator.validate(operation, details))
    }

    /// Create a new session
    pub fn create_session(&mut self, user_id: String, authority_level: AuthorityLevel) -> Session {
        self.authority.create_session(user_id, authority_level)
    }

    /// Get session
    pub fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.authority.get_session(session_id)
    }

    /// Refresh session
    pub fn refresh_session(&mut self, session_id: &str) -> Result<()> {
        self.authority.refresh_session(session_id)
    }

    /// Escalate authority
    pub fn escalate(
        &mut self,
        session_id: &str,
        target: AuthorityLevel,
        reason: &str,
    ) -> Result<Session> {
        self.authority.escalate(session_id, target, reason)
    }

    /// Check if operation is allowed
    pub fn is_allowed(&self, session: &Session, operation: &str, details: &str) -> Result<bool> {
        // First check constitution
        let validation = self.validator.validate(operation, details);

        if !validation.valid {
            return Ok(false);
        }

        // Check authority level
        let required_level = self.get_required_level(operation);
        Ok(session.can_execute(required_level))
    }

    /// Get required authority level for operation
    pub fn get_required_level(&self, operation: &str) -> AuthorityLevel {
        let op_lower = operation.to_lowercase();

        if op_lower.contains("delete") || op_lower.contains("sudo") || op_lower.contains("chmod") {
            AuthorityLevel::A3
        } else if op_lower.contains("credential") || op_lower.contains("vault") {
            AuthorityLevel::A4
        } else if op_lower.contains("root") || op_lower.contains("kernel") {
            AuthorityLevel::A5
        } else if op_lower.contains("execute") || op_lower.contains("run") {
            AuthorityLevel::A2
        } else if op_lower.contains("write") || op_lower.contains("create") {
            AuthorityLevel::A1
        } else {
            AuthorityLevel::A0
        }
    }

    /// Get vault
    pub fn vault(&self) -> &Arc<Vault> {
        &self.vault
    }
}
