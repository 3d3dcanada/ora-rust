//! OrA Constitution
//!
//! The Constitution is the supreme law that governs all operations in OrA.
//! It defines:
//! - Prime Directive (absolute rules that cannot be violated)
//! - Prohibited operations
//! - Authority hierarchy
//! - Odin policy articles loaded from YAML

use crate::error::{OraError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The Prime Directive - absolute rules that CANNOT be violated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimeDirective {
    /// Cannot harm humans or cause physical/psychological injury.
    pub no_harm: bool,
    /// Cannot facilitate illegal activities.
    pub no_illegal: bool,
    /// Cannot bypass security controls.
    pub no_security_bypass: bool,
    /// Cannot exfiltrate data unauthorized.
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

/// Types of operations that are explicitly prohibited.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProhibitedOperation {
    DeleteSystemFiles,
    PrivilegeEscalation,
    ModifyConstitution,
    DisableSecurity,
    DataExfiltration,
    Malware,
    SocialEngineering,
    CryptoMining,
    BotnetCreation,
    Custom(String),
}

impl ProhibitedOperation {
    /// Get the description of this prohibited operation.
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
            Self::Custom(value) => value,
        }
    }

    /// Check if an operation matches this prohibited operation.
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
            Self::Custom(pattern) => {
                let pattern = pattern.to_lowercase();
                op.contains(&pattern) || det.contains(&pattern)
            }
        }
    }
}

/// Authority requirement definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityRequirement {
    pub level: u8,
    pub name: String,
    pub description: String,
    pub requires_approval: bool,
    pub allowed_operations: Vec<String>,
}

/// Custom rule definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    pub name: String,
    pub description: String,
    pub pattern: String,
    pub action: String,
}

/// A structured Odin article loaded from YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyArticle {
    pub article: u16,
    pub title: String,
    pub chapter: String,
    pub enforcement: String,
    pub ora_constraint: Option<String>,
    pub technical_meaning: Option<String>,
    pub checks: Vec<String>,
    pub patterns: Vec<String>,
}

/// The Constitution - supreme governing document for OrA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    /// Version identifier.
    pub version: String,
    /// Prime Directive rules.
    pub prime_directive: PrimeDirective,
    /// Prohibited operations.
    pub prohibited_operations: Vec<ProhibitedOperation>,
    /// Authority hierarchy (A0 = lowest, A5 = highest).
    pub authority_hierarchy: Vec<AuthorityRequirement>,
    /// Custom rules compiled from Odin policy.
    pub custom_rules: Vec<CustomRule>,
    /// Odin policy articles for explainability and validation.
    #[serde(default)]
    pub policy_articles: Vec<PolicyArticle>,
    /// Source YAML path, if loaded from disk.
    #[serde(default)]
    pub source_path: Option<String>,
    /// Immutable hash for verification.
    #[serde(skip)]
    pub immutable_hash: String,
}

impl Default for Constitution {
    fn default() -> Self {
        let mut constitution = Self {
            version: "1.0.0".to_string(),
            prime_directive: PrimeDirective::default(),
            prohibited_operations: Self::default_prohibited(),
            authority_hierarchy: Self::default_authority_hierarchy(),
            custom_rules: Vec::new(),
            policy_articles: Vec::new(),
            source_path: None,
            immutable_hash: String::new(),
        };
        constitution.immutable_hash = constitution.compute_immutable_hash();
        constitution
    }
}

impl Constitution {
    /// Create a new Constitution with defaults only.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load the constitution from the Odin YAML file and translate it into runtime rules.
    pub fn load_from_yaml(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(|error| OraError::FileSystemError {
            path: path.to_string_lossy().to_string(),
            message: error.to_string(),
        })?;

        let odin: OdinConstitutionFile =
            serde_yaml::from_str(&contents).map_err(|error| OraError::ConfigError {
                field: "constitution".to_string(),
                message: error.to_string(),
            })?;

        let mut constitution = Self::default();
        constitution.version = odin.protocol_version.clone();
        constitution.source_path = Some(path.to_string_lossy().to_string());
        constitution.policy_articles = odin.into_articles();
        constitution.custom_rules = constitution
            .policy_articles
            .iter()
            .flat_map(|article| {
                article.patterns.iter().map(move |pattern| CustomRule {
                    name: article
                        .ora_constraint
                        .clone()
                        .unwrap_or_else(|| format!("ARTICLE_{}", article.article)),
                    description: format!("{} (Article {})", article.title, article.article),
                    pattern: pattern.clone(),
                    action: if article.enforcement.eq_ignore_ascii_case("immutable") {
                        "block".to_string()
                    } else {
                        "flag".to_string()
                    },
                })
            })
            .collect();

        constitution.immutable_hash = constitution.compute_immutable_hash();
        Ok(constitution)
    }

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

    fn compute_immutable_hash(&self) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(&self.version);
        hasher.update(self.prime_directive.no_harm.to_string());
        hasher.update(self.prime_directive.no_illegal.to_string());
        hasher.update(self.prime_directive.no_security_bypass.to_string());
        hasher.update(self.prime_directive.no_data_exfiltration.to_string());

        for prohibited in &self.prohibited_operations {
            hasher.update(prohibited.description());
        }

        for article in &self.policy_articles {
            hasher.update(article.article.to_string());
            hasher.update(&article.title);
            hasher.update(&article.chapter);
            hasher.update(&article.enforcement);
            for check in &article.checks {
                hasher.update(check);
            }
        }

        hex::encode(hasher.finalize())
    }

    /// Verify the constitution's integrity.
    pub fn verify_immutability(&self) -> bool {
        self.compute_immutable_hash() == self.immutable_hash
    }

    /// Check if an operation is prohibited.
    pub fn is_prohibited(&self, operation: &str, details: &str) -> bool {
        self.prohibited_operations
            .iter()
            .any(|prohibited| prohibited.matches(operation, details))
    }

    /// Check prime directive violations.
    pub fn check_prime_directive(&self, _operation: &str, details: &str) -> Option<String> {
        let details_lower = details.to_lowercase();

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

    /// Get authority requirement for a level.
    pub fn get_authority_requirement(&self, level: u8) -> Option<&AuthorityRequirement> {
        self.authority_hierarchy.iter().find(|requirement| requirement.level == level)
    }

    /// Check if operation is allowed at authority level.
    pub fn can_execute(&self, level: u8, operation: &str) -> bool {
        if let Some(requirement) = self.get_authority_requirement(level) {
            requirement.allowed_operations.contains(&"*".to_string())
                || requirement
                    .allowed_operations
                    .iter()
                    .any(|allowed| operation.starts_with(allowed))
        } else {
            false
        }
    }
}

#[derive(Debug, Deserialize)]
struct OdinConstitutionFile {
    protocol_version: String,
    #[serde(default)]
    immutable_principles: Vec<OdinRule>,
    #[serde(default)]
    economic_rules: Vec<OdinRule>,
    #[serde(default)]
    node_rules: Vec<OdinRule>,
    #[serde(default)]
    governance_rules: Vec<OdinRule>,
    #[serde(default)]
    participant_rights: Vec<OdinRule>,
    #[serde(default)]
    enforcement_rules: Vec<OdinRule>,
    #[serde(default)]
    final_provisions: Vec<OdinRule>,
}

impl OdinConstitutionFile {
    fn into_articles(self) -> Vec<PolicyArticle> {
        self.immutable_principles
            .into_iter()
            .chain(self.economic_rules)
            .chain(self.node_rules)
            .chain(self.governance_rules)
            .chain(self.participant_rights)
            .chain(self.enforcement_rules)
            .chain(self.final_provisions)
            .map(PolicyArticle::from)
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct OdinRule {
    article: u16,
    title: String,
    chapter: String,
    #[serde(default)]
    technical_meaning: Option<String>,
    enforcement: String,
    #[serde(default)]
    ora_constraint: Option<String>,
    #[serde(default)]
    checks: Vec<String>,
}

impl From<OdinRule> for PolicyArticle {
    fn from(rule: OdinRule) -> Self {
        let patterns = patterns_for_rule(&rule);
        Self {
            article: rule.article,
            title: rule.title,
            chapter: rule.chapter,
            enforcement: rule.enforcement,
            ora_constraint: rule.ora_constraint,
            technical_meaning: rule.technical_meaning,
            checks: rule.checks,
            patterns,
        }
    }
}

fn patterns_for_rule(rule: &OdinRule) -> Vec<String> {
    let key = rule
        .ora_constraint
        .as_deref()
        .unwrap_or_default()
        .to_ascii_uppercase();

    let pattern = match key.as_str() {
        "ARTICLE_III_DATA_OWNERSHIP" => {
            Some(r"(?i)\b(exfiltrat(e|ion)|dump\s+(database|credentials|customer)|upload\s+.*(secret|credential|database)|scp\b|rsync\b.*@)\b")
        }
        "ARTICLE_IV_BRAIN_OWNERSHIP" => {
            Some(r"(?i)\b(transfer ownership|reassign owner|reuse.+third part(y|ies)|silent transfer)\b")
        }
        "ARTICLE_VI_PROTOCOL_ABOVE_PROVIDERS" => Some(r"(?i)\b(auth0|okta|firebase|clerk)\b"),
        "ARTICLE_VII_PORTABILITY" => Some(r"(?i)\b(lock[- ]?in|disable export|block export|proprietary trap)\b"),
        "ARTICLE_VIII_NO_PROTOCOL_CAPTURE" => Some(r"(?i)\b(hidden favoritism|tracking script|third-party telemetry|bias ranking)\b"),
        "ARTICLE_IX_QUALITY_OVER_VOLUME" => Some(r"(?i)\b(likes|views|marketing volume)\b"),
        "ARTICLE_XIII_NODE_UNIQUENESS" => Some(r"(?i)\b(clone|cosmetic copy|duplicate node)\b"),
        "ARTICLE_XVI_CLAIMS_TRANSPARENCY" => Some(r"(?i)\b(unverified claim|misleading claim|false benchmark|fabricated result)\b"),
        "ARTICLE_XVIII_GOVERNANCE_EXPLAINABILITY" => Some(r"(?i)\b(opaque routing|opaque ranking|unlogged sanction|silent rejection)\b"),
        _ => None,
    };

    pattern.map(|value| vec![value.to_string()]).unwrap_or_default()
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

    #[test]
    fn test_load_from_yaml() {
        let path = Path::new("/home/wess/ora/config/odin-constitution.yaml");
        let constitution = Constitution::load_from_yaml(path).expect("constitution should load");
        assert_eq!(constitution.version, "0.3.0");
        assert!(!constitution.policy_articles.is_empty());
        assert!(constitution.verify_immutability());
    }
}
