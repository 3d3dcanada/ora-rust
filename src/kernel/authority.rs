//! OrA Authority System
//!
//! Authority levels (A0-A5) control what operations can be performed.
//! This provides fine-grained access control over the system.

use crate::error::OraError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// -----------------------------------------------------------------------------
// TYPE-STATE CLEARANCE HIERARCHY (COMPILE-TIME SECURITY)
// -----------------------------------------------------------------------------

/// Trait indicating A0 Guest clearance or higher
pub trait A0Clearance {}
/// Trait indicating A1 User clearance or higher
pub trait A1Clearance: A0Clearance {}
/// Trait indicating A2 Developer clearance or higher
pub trait A2Clearance: A1Clearance {}
/// Trait indicating A3 Senior clearance or higher
pub trait A3Clearance: A2Clearance {}
/// Trait indicating A4 Admin clearance or higher
pub trait A4Clearance: A3Clearance {}
/// Trait indicating A5 Root clearance
pub trait A5Clearance: A4Clearance {}

/// A0 Guest Marker
pub struct A0;
impl A0Clearance for A0 {}

/// A1 User Marker
pub struct A1;
impl A0Clearance for A1 {}
impl A1Clearance for A1 {}

/// A2 Developer Marker
pub struct A2;
impl A0Clearance for A2 {}
impl A1Clearance for A2 {}
impl A2Clearance for A2 {}

/// A3 Senior Marker
pub struct A3;
impl A0Clearance for A3 {}
impl A1Clearance for A3 {}
impl A2Clearance for A3 {}
impl A3Clearance for A3 {}

/// A4 Admin Marker
pub struct A4;
impl A0Clearance for A4 {}
impl A1Clearance for A4 {}
impl A2Clearance for A4 {}
impl A3Clearance for A4 {}
impl A4Clearance for A4 {}

/// A5 Root Marker
pub struct A5;
impl A0Clearance for A5 {}
impl A1Clearance for A5 {}
impl A2Clearance for A5 {}
impl A3Clearance for A5 {}
impl A4Clearance for A5 {}
impl A5Clearance for A5 {}

/// An explicitly secured context bounded by a clearance level.
/// This wrapper ensures that functions executing within it have been compile-time
/// verified to possess the required 'C' clearance.
pub struct SecureContext<C> {
    _marker: std::marker::PhantomData<C>,
    pub session_id: String,
}

impl<C> SecureContext<C> {
    /// Extremely unsafe: bypasses runtime checks to create a compile-time context.
    /// In a real scenario, this is only instantiated internally after validating a token.
    pub fn new_unchecked(session_id: String) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            session_id,
        }
    }
}

// -----------------------------------------------------------------------------
// RUNTIME DYNAMIC AUTHORITY (SESSIONS)
// -----------------------------------------------------------------------------

/// Authority levels - higher levels have more privileges
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum AuthorityLevel {
    /// Guest - Read-only, no network
    A0 = 0,

    /// User - Read-write workspace, limited network
    A1 = 1,

    /// Developer - Sandboxed shell, full network
    A2 = 2,

    /// Senior - Unsandboxed shell (with approval)
    A3 = 3,

    /// Admin - System-wide access, credential modification
    A4 = 4,

    /// Root - Disable gates temporarily (requires 2FA/hardware key)
    A5 = 5,
}

impl Default for AuthorityLevel {
    fn default() -> Self {
        Self::A0
    }
}

impl std::fmt::Display for AuthorityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A0 => write!(f, "A0-Guest"),
            Self::A1 => write!(f, "A1-User"),
            Self::A2 => write!(f, "A2-Developer"),
            Self::A3 => write!(f, "A3-Senior"),
            Self::A4 => write!(f, "A4-Admin"),
            Self::A5 => write!(f, "A5-Root"),
        }
    }
}

impl AuthorityLevel {
    /// Get the numeric value
    pub fn value(&self) -> u8 {
        *self as u8
    }

    /// Get from numeric value
    pub fn from_value(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::A0),
            1 => Some(Self::A1),
            2 => Some(Self::A2),
            3 => Some(Self::A3),
            4 => Some(Self::A4),
            5 => Some(Self::A5),
            _ => None,
        }
    }

    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::A0 => "Guest",
            Self::A1 => "User",
            Self::A2 => "Developer",
            Self::A3 => "Senior",
            Self::A4 => "Admin",
            Self::A5 => "Root",
        }
    }

    /// Get the description
    pub fn description(&self) -> &'static str {
        match self {
            Self::A0 => "Read-only, no network access",
            Self::A1 => "Read-write workspace, limited network",
            Self::A2 => "Sandboxed shell, full network",
            Self::A3 => "Unsandboxed shell (with approval)",
            Self::A4 => "System-wide access, credential modification",
            Self::A5 => "Disable gates temporarily (requires 2FA/hardware key)",
        }
    }

    /// Check if this level requires approval for operations
    pub fn requires_approval(&self) -> bool {
        matches!(self, Self::A3 | Self::A4 | Self::A5)
    }

    /// Check if this level can escalate to another
    pub fn can_escalate_to(&self, target: AuthorityLevel) -> bool {
        // Can only escalate one level at a time
        target.value() > self.value() && target.value() <= self.value() + 1
    }
}

/// Session - represents a user session with authority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID
    pub id: String,

    /// User identifier
    pub user_id: String,

    /// Current authority level
    pub authority_level: AuthorityLevel,

    /// Session created at
    pub created_at: DateTime<Utc>,

    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,

    /// Session expires at
    pub expires_at: DateTime<Utc>,

    /// Whether session is active
    pub active: bool,

    /// Session metadata
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionMetadata {
    /// Session IP address
    pub ip_address: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// Connection type (websocket/http)
    pub connection_type: Option<String>,

    /// Custom metadata
    pub extra: std::collections::HashMap<String, String>,
}

impl Session {
    /// Create a new session
    pub fn new(user_id: String, authority_level: AuthorityLevel, timeout_seconds: u64) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            authority_level,
            created_at: now,
            last_activity: now,
            expires_at: now + chrono::Duration::seconds(timeout_seconds as i64),
            active: true,
            metadata: SessionMetadata::default(),
        }
    }

    /// Check if session is valid
    pub fn is_valid(&self) -> bool {
        self.active && Utc::now() < self.expires_at
    }

    /// Refresh session activity
    pub fn refresh(&mut self, timeout_seconds: u64) {
        self.last_activity = Utc::now();
        self.expires_at = Utc::now() + chrono::Duration::seconds(timeout_seconds as i64);
    }

    /// Check if can execute operation requiring given authority
    pub fn can_execute(&self, required_level: AuthorityLevel) -> bool {
        self.is_valid() && self.authority_level >= required_level
    }

    /// Get session info for audit
    pub fn audit_info(&self) -> SessionAuditInfo {
        SessionAuditInfo {
            session_id: self.id.clone(),
            user_id: self.user_id.clone(),
            authority_level: self.authority_level.to_string(),
            created_at: self.created_at.to_rfc3339(),
            last_activity: self.last_activity.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAuditInfo {
    pub session_id: String,
    pub user_id: String,
    pub authority_level: String,
    pub created_at: String,
    pub last_activity: String,
}

/// Authority kernel - manages sessions and authority escalation
#[derive(Debug)]
pub struct AuthorityKernel {
    /// Active sessions
    sessions: std::collections::HashMap<String, Session>,

    /// Maximum authority level without escalation
    max_level: AuthorityLevel,

    /// Session timeout in seconds
    session_timeout: u64,
}

impl Default for AuthorityKernel {
    fn default() -> Self {
        Self::new(3, 3600) // Default max A3, 1 hour timeout
    }
}

impl AuthorityKernel {
    /// Create new authority kernel
    pub fn new(max_level: u8, session_timeout: u64) -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
            max_level: AuthorityLevel::from_value(max_level).unwrap_or(AuthorityLevel::A3),
            session_timeout,
        }
    }

    /// Create a new session
    pub fn create_session(&mut self, user_id: String, initial_level: AuthorityLevel) -> Session {
        let level = initial_level.min(self.max_level);
        let session = Session::new(user_id, level, self.session_timeout);
        let id = session.id.clone();
        self.sessions.insert(id, session.clone());
        session
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }

    /// Get mutable session
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(session_id)
    }

    /// Refresh session
    pub fn refresh_session(&mut self, session_id: &str) -> Result<(), OraError> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            if !session.is_valid() {
                return Err(OraError::InvalidSession {
                    session_id: session_id.to_string(),
                });
            }
            session.refresh(self.session_timeout);
            Ok(())
        } else {
            Err(OraError::ClientNotFound {
                client_id: session_id.to_string(),
            })
        }
    }

    /// End session
    pub fn end_session(&mut self, session_id: &str) -> bool {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.active = false;
            true
        } else {
            false
        }
    }

    /// Escalate authority level
    pub fn escalate(
        &mut self,
        session_id: &str,
        target_level: AuthorityLevel,
        _reason: &str,
    ) -> Result<Session, OraError> {
        let session =
            self.sessions
                .get_mut(session_id)
                .ok_or_else(|| OraError::InvalidSession {
                    session_id: session_id.to_string(),
                })?;

        if !session.is_valid() {
            return Err(OraError::InvalidSession {
                session_id: session_id.to_string(),
            });
        }

        // Check if escalation is allowed
        if target_level.value() > self.max_level.value() {
            return Err(OraError::AuthorityEscalationFailed {
                reason: "Exceeds maximum authority level".to_string(),
                required_level: target_level.value(),
            });
        }

        if !session.authority_level.can_escalate_to(target_level) {
            return Err(OraError::AuthorityEscalationFailed {
                reason: "Can only escalate one level at a time".to_string(),
                required_level: target_level.value(),
            });
        }

        session.authority_level = target_level;
        session.refresh(self.session_timeout);

        Ok(session.clone())
    }

    /// Get current authority for session
    pub fn get_authority(&self, session_id: &str) -> Result<AuthorityLevel, OraError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| OraError::InvalidSession {
                session_id: session_id.to_string(),
            })?;

        if !session.is_valid() {
            return Err(OraError::InvalidSession {
                session_id: session_id.to_string(),
            });
        }

        Ok(session.authority_level)
    }

    /// List all active sessions
    pub fn list_sessions(&self) -> Vec<SessionAuditInfo> {
        self.sessions
            .values()
            .filter(|s| s.is_valid())
            .map(|s| s.audit_info())
            .collect()
    }

    /// Clean up expired sessions
    pub fn cleanup(&mut self) -> usize {
        let now = Utc::now();
        let before = self.sessions.len();
        self.sessions
            .retain(|_, session| session.expires_at > now && session.active);
        before - self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authority_levels() {
        assert_eq!(AuthorityLevel::A0.value(), 0);
        assert_eq!(AuthorityLevel::A5.value(), 5);
        assert!(AuthorityLevel::A5 > AuthorityLevel::A0);
    }

    #[test]
    fn test_session() {
        let session = Session::new("user1".to_string(), AuthorityLevel::A2, 3600);
        assert!(session.is_valid());
        assert!(session.can_execute(AuthorityLevel::A1));
        assert!(!session.can_execute(AuthorityLevel::A3));
    }

    #[test]
    fn test_authority_kernel() {
        let mut kernel = AuthorityKernel::new(3, 3600);
        let session = kernel.create_session("user1".to_string(), AuthorityLevel::A1);

        assert_eq!(session.authority_level, AuthorityLevel::A1);

        // Can escalate one level
        let escalated = kernel.escalate(&session.id, AuthorityLevel::A2, "Need more access");
        assert!(escalated.is_ok());

        // Cannot skip levels
        let failed = kernel.escalate(&session.id, AuthorityLevel::A4, "Want root");
        assert!(failed.is_err());
    }

    // Example functions requiring specific compile-time clearances
    fn read_public_data<C: A0Clearance>(_ctx: &SecureContext<C>) -> &'static str {
        "public data"
    }

    fn write_workspace_data<C: A1Clearance>(_ctx: &SecureContext<C>) -> &'static str {
        "workspace updated"
    }

    fn access_vault<C: A5Clearance>(_ctx: &SecureContext<C>) -> &'static str {
        "vault accessed"
    }

    #[test]
    fn test_compile_time_type_state_security() {
        // We create contexts with specific markers
        let a0_ctx = SecureContext::<A0>::new_unchecked("guest".to_string());
        let a1_ctx = SecureContext::<A1>::new_unchecked("user".to_string());
        let a5_ctx = SecureContext::<A5>::new_unchecked("root".to_string());

        // A0 can read public data
        assert_eq!(read_public_data(&a0_ctx), "public data");

        // A1 can write workspace AND read public data (trait inheritance)
        assert_eq!(read_public_data(&a1_ctx), "public data");
        assert_eq!(write_workspace_data(&a1_ctx), "workspace updated");

        // A5 can do everything
        assert_eq!(read_public_data(&a5_ctx), "public data");
        assert_eq!(write_workspace_data(&a5_ctx), "workspace updated");
        assert_eq!(access_vault(&a5_ctx), "vault accessed");

        // Note: The following lines would completely fail to compile if uncommented,
        // proving our Type-State kernel works at compile time!
        // write_workspace_data(&a0_ctx); // ERROR: traits `A1Clearance` not satisfied
        // access_vault(&a1_ctx); // ERROR: trait `A5Clearance` not satisfied
    }
}
