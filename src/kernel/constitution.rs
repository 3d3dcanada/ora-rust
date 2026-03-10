//! OrA Constitution
//!
//! The Constitution is the supreme law that governs all operations in OrA.
//! It defines:
//! - Prime Directive (absolute rules that cannot be violated)
//! - Prohibited operations
//! - Authority hierarchy
//!
//! This is NOT a suggestion system - these are HARD boundaries enforced by the kernel.

use serde::{Deserialize, Serialize};

/// The Prime Directive - absolute rules that CANNOT be violated
/// These rules are enforced at the kernel level before any operation executes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimeDirective {
    /// Cannot harm humans or cause physical/psychological injury
    pub no_harm: bool,

    /// Cannot facilitate illegal activities
    pub no_illegal: bool,

    /// Cannot bypass security controls
    pub no_security_bypass: bool,

    /// Cannot exfiltrate data unauthorized
    pub no_data_exfiltration: bool,
}

impl Default for PrimeDirective {
    fn default() -> Self {
        Self {
            no_harm: true,
            no_illegal: true,
            no_security_bypass: true,
            no_data_exfiltration: true,
        }
    }
}

/// Types of operations that are explicitly prohibited
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProhibitedOperation {
    /// Delete system files
    DeleteSystemFiles,

    /// Attempt privilege escalation
    PrivilegeEscalation,

    /// Modify the constitution itself
    ModifyConstitution,

    /// Disable security controls
    DisableSecurity,

    /// Exfiltrate data
    DataExfiltration,

    /// Execute malware/ransomware
    Malware,

    /// Social engineering attacks
    SocialEngineering,

    /// Cryptocurrency mining
    CryptoMining,

    /// Botnet creation
    BotnetCreation,

    /// Any custom prohibited operation
    Custom(String),
}

impl ProhibitedOperation {
    /// Get the description of this prohibited operation
    pub fn description(&self) -> &str {
        match self {
            Self::DeleteSystemFiles => "Cannot delete system files",
            Self::PrivilegeEscalation => "Cannot attempt privilege escalation",
            Self::ModifyConstitution => "Cannot modify the constitution",
            Self::DisableSecurity => "Cannot disable security controls",
            Self::DataExfiltration => "Cannot exfiltrate data",
            Self::Malware => "Cannot create or execute malware",
            Self::SocialEngineering => "Cannot perform social engineering",
            Self::CryptoMining => "Cannot perform cryptocurrency mining",
            Self::BotnetCreation => "Cannot create botnets",
            Self::Custom(s) => s,
        }
    }

    /// Check if an operation matches this prohibited operation
    pub fn matches(&self, operation: &str, details: &str) -> bool {
        let op = operation.to_lowercase();
        let det = details.to_lowercase();
        match self {
            Self::DeleteSystemFiles => {
                op.contains("delete")
                    && (det.contains("system")
                        || det.contains("/etc")
                        || det.contains("/bin")
                        || det.contains("/var"))
            }
            Self::PrivilegeEscalation => {
                op.contains("su")
                    || op.contains("sudo")
                    || det.contains("root")
                    || det.contains("privilege")
            }
            Self::ModifyConstitution => op.contains("modify") && det.contains("constitution"),
            Self::DisableSecurity => {
                (op.contains("disable") || op.contains("stop"))
                    && (det.contains("security") || det.contains("firewall"))
            }
            Self::DataExfiltration => {
                op.contains("exfiltrate")
                    || (op.contains("upload") && det.contains("sensitive"))
                    || op.contains("export")
            }
            Self::Malware => {
                op.contains("malware") || op.contains("virus") || op.contains("ransomware")
            }
            Self::SocialEngineering => op.contains("phish") || op.contains("social engineering"),
            Self::CryptoMining => {
                op.contains("mine") || op.contains("crypto") || op.contains("bitcoin")
            }
            Self::BotnetCreation => op.contains("botnet"),
            Self::Custom(s) => op.contains(&s.to_lowercase()) || det.contains(&s.to_lowercase()),
        }
    }
}

/// The Constitution - supreme governing document for OrA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    /// Version identifier
    pub version: String,

    /// Prime Directive rules
    pub prime_directive: PrimeDirective,

    /// Prohibited operations
    pub prohibited_operations: Vec<ProhibitedOperation>,

    /// Authority hierarchy (A0 = lowest, A5 = highest)
    pub authority_hierarchy: Vec<AuthorityRequirement>,

    /// Custom rules (JSON-like for extensibility)
    pub custom_rules: Vec<CustomRule>,

    /// Immutable hash for verification
    #[serde(skip)]
    pub immutable_hash: String,
}

impl Default for Constitution {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            prime_directive: PrimeDirective::default(),
            prohibited_operations: Self::default_prohibited(),
            authority_hierarchy: Self::default_authority_hierarchy(),
            custom_rules: Vec::new(),
            immutable_hash: String::new(),
        }
    }
}

impl Constitution {
    /// Default prohibited operations
    fn default_prohibited() -> Vec<ProhibitedOperation> {
        vec![
            ProhibitedOperation::DeleteSystemFiles,
            ProhibitedOperation::PrivilegeEscalation,
            ProhibitedOperation::ModifyConstitution,
            ProhibitedOperation::DisableSecurity,
            ProhibitedOperation::DataExfiltration,
            ProhibitedOperation::Malware,
            ProhibitedOperation::SocialEngineering,
            ProhibitedOperation::CryptoMining,
            ProhibitedOperation::BotnetCreation,
        ]
    }

    /// Default authority hierarchy
    fn default_authority_hierarchy() -> Vec<AuthorityRequirement> {
        vec![
            AuthorityRequirement {
                level: 0,
                name: "Guest".to_string(),
                description: "Read-only, no network access".to_string(),
                requires_approval: false,
                allowed_operations: vec!["read".to_string()],
            },
            AuthorityRequirement {
                level: 1,
                name: "User".to_string(),
                description: "Read-write workspace, limited network".to_string(),
                requires_approval: false,
                allowed_operations: vec![
                    "read".to_string(),
                    "write".to_string(),
                    "search".to_string(),
                ],
            },
            AuthorityRequirement {
                level: 2,
                name: "Developer".to_string(),
                description: "Sandboxed shell, full network".to_string(),
                requires_approval: false,
                allowed_operations: vec![
                    "read".to_string(),
                    "write".to_string(),
                    "search".to_string(),
                    "execute".to_string(),
                ],
            },
            AuthorityRequirement {
                level: 3,
                name: "Senior".to_string(),
                description: "Unsandboxed shell with approval".to_string(),
                requires_approval: true,
                allowed_operations: vec![
                    "read".to_string(),
                    "write".to_string(),
                    "search".to_string(),
                    "execute".to_string(),
                    "admin".to_string(),
                ],
            },
            AuthorityRequirement {
                level: 4,
                name: "Admin".to_string(),
                description: "System-wide access, credential modification".to_string(),
                requires_approval: true,
                allowed_operations: vec![
                    "read".to_string(),
                    "write".to_string(),
                    "search".to_string(),
                    "execute".to_string(),
                    "admin".to_string(),
                    "credential".to_string(),
                ],
            },
            AuthorityRequirement {
                level: 5,
                name: "Root".to_string(),
                description: "Disable gates temporarily (requires 2FA/hardware key)".to_string(),
                requires_approval: true,
                allowed_operations: vec!["*".to_string()],
            },
        ]
    }

    /// Create a new Constitution
    pub fn new() -> Self {
        Self::default()
    }

    /// Verify the constitution's integrity
    pub fn verify_immutability(&self) -> bool {
        // Compute a hash of the immutable parts
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        // Hash version
        hasher.update(&self.version);

        // Hash prime directive
        hasher.update(self.prime_directive.no_harm.to_string());
        hasher.update(self.prime_directive.no_illegal.to_string());
        hasher.update(self.prime_directive.no_security_bypass.to_string());
        hasher.update(self.prime_directive.no_data_exfiltration.to_string());

        // Hash prohibited operations
        for op in &self.prohibited_operations {
            hasher.update(op.description());
        }

        // Compute final hash
        let result = hasher.finalize();
        let computed_hash = hex::encode(result);

        // For first run, set the hash
        if self.immutable_hash.is_empty() {
            return true;
        }

        // Verify against stored hash
        computed_hash == self.immutable_hash
    }

    /// Check if an operation is prohibited
    pub fn is_prohibited(&self, operation: &str, details: &str) -> bool {
        // Check explicit prohibitions
        for prohibited in &self.prohibited_operations {
            if prohibited.matches(operation, details) {
                return true;
            }
        }

        false
    }

    /// Check prime directive violations
    pub fn check_prime_directive(&self, _operation: &str, details: &str) -> Option<String> {
        let details_lower = details.to_lowercase();

        // Check no harm
        if self.prime_directive.no_harm {
            let harmful_keywords = ["harm", "injure", "kill", "attack", "violence"];
            for keyword in harmful_keywords {
                if details_lower.contains(keyword) {
                    return Some(format!(
                        "Prime Directive violation: Operation involves harm - '{}'",
                        details
                    ));
                }
            }
        }

        // Check no illegal
        if self.prime_directive.no_illegal {
            let illegal_keywords = ["illegal", "crime", "theft", "fraud", "hack"];
            for keyword in illegal_keywords {
                if details_lower.contains(keyword) {
                    return Some(format!(
                        "Prime Directive violation: Operation involves illegal activity - '{}'",
                        details
                    ));
                }
            }
        }

        // Check no security bypass
        if self.prime_directive.no_security_bypass {
            let bypass_keywords = ["bypass", "exploit", "vulnerability", "backdoor"];
            for keyword in bypass_keywords {
                if details_lower.contains(keyword) {
                    return Some(format!(
                        "Prime Directive violation: Operation attempts security bypass - '{}'",
                        details
                    ));
                }
            }
        }

        // Check no data exfiltration
        if self.prime_directive.no_data_exfiltration {
            let exfil_keywords = ["exfiltrate", "steal", "extract", "dump"];
            for keyword in exfil_keywords {
                if details_lower.contains(keyword) {
                    return Some(format!(
                        "Prime Directive violation: Operation attempts data exfiltration - '{}'",
                        details
                    ));
                }
            }
        }

        None
    }

    /// Get authority requirement for a level
    pub fn get_authority_requirement(&self, level: u8) -> Option<&AuthorityRequirement> {
        self.authority_hierarchy.iter().find(|r| r.level == level)
    }

    /// Check if operation is allowed at authority level
    pub fn can_execute(&self, level: u8, operation: &str) -> bool {
        if let Some(requirement) = self.get_authority_requirement(level) {
            requirement.allowed_operations.contains(&"*".to_string())
                || requirement
                    .allowed_operations
                    .iter()
                    .any(|op| operation.starts_with(op))
        } else {
            false
        }
    }
}

/// Authority requirement definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityRequirement {
    /// Authority level (0-5)
    pub level: u8,

    /// Name (Guest, User, Developer, Senior, Admin, Root)
    pub name: String,

    /// Description
    pub description: String,

    /// Whether this level requires approval for operations
    pub requires_approval: bool,

    /// List of allowed operation types
    pub allowed_operations: Vec<String>,
}

/// Custom rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    /// Rule name
    pub name: String,

    /// Rule description
    pub description: String,

    /// Pattern to match (regex)
    pub pattern: String,

    /// Action when matched (allow, block, flag)
    pub action: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_constitution() {
        let constitution = Constitution::new();
        assert_eq!(constitution.version, "1.0.0");
        assert!(constitution.verify_immutability());
    }

    #[test]
    fn test_prohibited_operations() {
        let constitution = Constitution::new();
        assert!(constitution.is_prohibited("delete", "/etc/passwd"));
        assert!(!constitution.is_prohibited("read", "/home/user/file.txt"));
    }

    #[test]
    fn test_prime_directive() {
        let constitution = Constitution::new();
        assert!(constitution
            .check_prime_directive("harm", "harm humans")
            .is_some());
        assert!(constitution
            .check_prime_directive("read", "read a file")
            .is_none());
    }

    #[test]
    fn test_authority_requirements() {
        let constitution = Constitution::new();
        assert!(constitution.can_execute(0, "read"));
        assert!(!constitution.can_execute(0, "execute"));
        assert!(constitution.can_execute(5, "anything"));
    }
}
