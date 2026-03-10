//! OrA Validator
//!
//! Validates operations against the Constitution before execution.

use crate::kernel::constitution::Constitution;
use serde::{Deserialize, Serialize};

/// Operation validator - checks operations against the Constitution
#[derive(Debug)]
pub struct Validator {
    constitution: Constitution,
}

impl Validator {
    /// Create a new validator
    pub fn new(constitution: Constitution) -> Self {
        Self { constitution }
    }

    /// Validate an operation
    pub fn validate(&self, operation: &str, details: &str) -> ValidationResult {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        // Check prime directive
        if let Some(violation) = self.constitution.check_prime_directive(operation, details) {
            violations.push(ValidationViolation {
                violation_type: ViolationType::PrimeDirective,
                message: violation,
                severity: Severity::Critical,
            });
        }

        // Check prohibited operations
        if self.constitution.is_prohibited(operation, details) {
            violations.push(ValidationViolation {
                violation_type: ViolationType::Prohibited,
                message: format!(
                    "Operation '{}' with details '{}' is prohibited",
                    operation, details
                ),
                severity: Severity::Error,
            });
        }

        // Check for suspicious patterns (warnings)
        let suspicious = self.check_suspicious_patterns(details);
        warnings.extend(suspicious);

        ValidationResult {
            valid: violations.is_empty(),
            violations,
            warnings,
        }
    }

    /// Check for suspicious patterns (soft warnings)
    fn check_suspicious_patterns(&self, details: &str) -> Vec<ValidationViolation> {
        let mut warnings = Vec::new();
        let details_lower = details.to_lowercase();

        // Password/credential patterns
        if details_lower.contains("password") || details_lower.contains("credential") {
            warnings.push(ValidationViolation {
                violation_type: ViolationType::Suspicious,
                message: "Operation involves credentials - ensure proper authorization".to_string(),
                severity: Severity::Warning,
            });
        }

        // Network requests
        if details_lower.contains("http://") || details_lower.contains("https://") {
            warnings.push(ValidationViolation {
                violation_type: ViolationType::Suspicious,
                message: "Operation involves network request".to_string(),
                severity: Severity::Info,
            });
        }

        // System commands
        if details_lower.contains("sudo")
            || details_lower.contains("chmod")
            || details_lower.contains("chown")
        {
            warnings.push(ValidationViolation {
                violation_type: ViolationType::Suspicious,
                message: "Operation involves system-level command".to_string(),
                severity: Severity::Warning,
            });
        }

        warnings
    }
}

/// Result of validation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub violations: Vec<ValidationViolation>,
    pub warnings: Vec<ValidationViolation>,
}

impl ValidationResult {
    /// Check if result has any critical violations
    pub fn has_critical(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == Severity::Critical)
    }

    /// Get combined message
    pub fn message(&self) -> String {
        if self.valid {
            if self.warnings.is_empty() {
                "Operation validated successfully".to_string()
            } else {
                format!("Operation allowed with {} warning(s)", self.warnings.len())
            }
        } else {
            format!(
                "Operation blocked with {} violation(s)",
                self.violations.len()
            )
        }
    }
}

/// A single validation violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationViolation {
    pub violation_type: ViolationType,
    pub message: String,
    pub severity: Severity,
}

/// Type of violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    PrimeDirective,
    Prohibited,
    Authority,
    Suspicious,
}

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_operation() {
        let constitution = Constitution::new();
        let validator = Validator::new(constitution);

        let result = validator.validate("read", "read a file from workspace");
        assert!(result.valid);
    }

    #[test]
    fn test_prohibited_operation() {
        let constitution = Constitution::new();
        let validator = Validator::new(constitution);

        let result = validator.validate("delete", "delete /etc/passwd");
        assert!(!result.valid);
        assert!(result
            .violations
            .iter()
            .any(|v| v.violation_type == ViolationType::Prohibited));
    }

    #[test]
    fn test_prime_directive() {
        let constitution = Constitution::new();
        let validator = Validator::new(constitution);

        let result = validator.validate("harm", "harm humans");
        assert!(!result.valid);
        assert!(result.has_critical());
    }
}
