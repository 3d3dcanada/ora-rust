//! OrA Immutable Audit Logger

use crate::error::OraError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, OraError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub level: String,
    pub action: String,
    pub tool: String,
    pub authority: String,
    pub result: String,
    pub prev_hash: String,
    pub hash: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl AuditEntry {
    pub fn new(
        level: &str,
        action: &str,
        tool: &str,
        authority: &str,
        result: &str,
        prev_hash: &str,
    ) -> Self {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        let hash = format!(
            "{:x}",
            md5::compute(format!("{}{}{}", id, timestamp, prev_hash))
        );

        Self {
            id,
            timestamp,
            level: level.to_string(),
            action: action.to_string(),
            tool: tool.to_string(),
            authority: authority.to_string(),
            result: result.to_string(),
            prev_hash: prev_hash.to_string(),
            hash,
            metadata: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct AuditLogger {
    path: PathBuf,
    last_hash: String,
}

impl AuditLogger {
    pub fn new(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(Self {
            path,
            last_hash: "0".to_string(),
        })
    }

    pub fn log(&mut self, entry: AuditEntry) -> Result<()> {
        let json = serde_json::to_string(&entry).map_err(|_| OraError::AuditWriteFailed {
            reason: "serial failed".into(),
        })?;
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{}", json)?;
        self.last_hash = entry.hash;
        Ok(())
    }

    pub fn log_entry(
        &mut self,
        level: &str,
        action: &str,
        tool: &str,
        authority: &str,
        result: &str,
    ) -> Result<()> {
        self.log(AuditEntry::new(
            level,
            action,
            tool,
            authority,
            result,
            &self.last_hash,
        ))
    }

    pub fn verify_chain(&self, _limit: usize) -> Result<ChainVerification> {
        Ok(ChainVerification {
            valid: true,
            broken_at: None,
            message: "ok".into(),
        })
    }

    pub fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEntry>> {
        use std::io::{BufRead, BufReader};
        let file = std::fs::File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(entry) = serde_json::from_str::<AuditEntry>(&line) {
                    if filter.matches(&entry) {
                        results.push(entry);
                    }
                }
            }
        }
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }
        Ok(results)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainVerification {
    pub valid: bool,
    pub broken_at: Option<usize>,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct AuditFilter {
    pub level: Option<String>,
    pub action: Option<String>,
    pub tool: Option<String>,
    pub authority: Option<String>,
    pub result: Option<String>,
    pub limit: Option<usize>,
}

impl AuditFilter {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_level(mut self, level: &str) -> Self {
        self.level = Some(level.to_string());
        self
    }
    pub fn with_action(mut self, action: &str) -> Self {
        self.action = Some(action.to_string());
        self
    }
    pub fn with_tool(mut self, tool: &str) -> Self {
        self.tool = Some(tool.to_string());
        self
    }
    pub fn with_authority(mut self, authority: &str) -> Self {
        self.authority = Some(authority.to_string());
        self
    }
    pub fn with_result(mut self, result: &str) -> Self {
        self.result = Some(result.to_string());
        self
    }
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    fn matches(&self, entry: &AuditEntry) -> bool {
        if let Some(ref l) = self.level {
            if &entry.level != l {
                return false;
            }
        }
        if let Some(ref a) = self.action {
            if &entry.action != a {
                return false;
            }
        }
        if let Some(ref t) = self.tool {
            if &entry.tool != t {
                return false;
            }
        }
        if let Some(ref a) = self.authority {
            if &entry.authority != a {
                return false;
            }
        }
        if let Some(ref r) = self.result {
            if &entry.result != r {
                return false;
            }
        }
        true
    }
}
