//! OrA Control Plane
//!
//! Durable storage for routing, evidence, memory, approvals, missions, browser tasks, and
//! operator-visible audit metadata.

use crate::error::{OraError, Result};
use parking_lot::Mutex;
use reqwest::blocking::Client as BlockingClient;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const DEFAULT_VECTOR_DIMENSIONS: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskClass {
    GeneralQuestion,
    WorkspaceResearch,
    OperationalDecision,
    BrowserMission,
    CommandExecution,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FreshnessRequirement {
    Historical,
    Recent,
    Live,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskClass {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceBurden {
    Minimal,
    Grounded,
    Strict,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RouteClass {
    Memory,
    LocalDocuments,
    CachedEvidence,
    LiveWeb,
    BrowserMission,
}

impl RouteClass {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::LocalDocuments => "local_documents",
            Self::CachedEvidence => "cached_evidence",
            Self::LiveWeb => "live_web",
            Self::BrowserMission => "browser_mission",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub id: String,
    pub query: String,
    pub task_class: TaskClass,
    pub freshness_requirement: FreshnessRequirement,
    pub risk_class: RiskClass,
    pub evidence_burden: EvidenceBurden,
    pub selected_route: RouteClass,
    pub route_reason: String,
    pub requires_live_retrieval: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSourceType {
    Memory,
    LocalDocument,
    CachedBundle,
    LiveWeb,
    BrowserArtifact,
}

impl EvidenceSourceType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::LocalDocument => "local_document",
            Self::CachedBundle => "cached_bundle",
            Self::LiveWeb => "live_web",
            Self::BrowserArtifact => "browser_artifact",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: String,
    pub bundle_id: String,
    pub source_type: EvidenceSourceType,
    pub title: String,
    pub uri: String,
    pub snippet: String,
    pub content_hash: String,
    pub observed_at: String,
    pub freshness_expires_at: Option<String>,
    pub confidence: f32,
    pub provenance: String,
    pub metadata: Value,
    pub superseded_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub content: String,
    pub provenance: String,
    pub confidence: f32,
    pub freshness_expires_at: Option<String>,
    pub superseded_by: Option<String>,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalState {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub operation: String,
    pub action_class: String,
    pub risk_class: RiskClass,
    pub authority_required: String,
    pub description: String,
    pub request_payload: Value,
    pub status: ApprovalState,
    pub created_at: String,
    pub resolved_at: Option<String>,
    pub approver: Option<String>,
    pub resolution_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionSpec {
    pub id: String,
    pub name: String,
    pub query: String,
    pub sources: Vec<String>,
    pub freshness_policy: String,
    pub extraction_rules: Vec<String>,
    pub storage_policy: String,
    pub status: String,
    pub artifact_path: Option<String>,
    pub created_at: String,
    pub last_run_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub event_type: String,
    pub actor: String,
    pub summary: String,
    pub subject_id: Option<String>,
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BrowserTaskStatus {
    PendingApproval,
    Ready,
    Running,
    Completed,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BrowserAuthPolicy {
    Deny,
    ExistingSessionOnly,
    ManualOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BrowserApprovalPolicy {
    ReadOnlyAllowed,
    SensitiveActionApproval,
    AlwaysApprove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTask {
    pub id: String,
    pub task: String,
    pub url: Option<String>,
    pub allowed_domains: Vec<String>,
    pub allowed_action_classes: Vec<String>,
    pub auth_policy: BrowserAuthPolicy,
    pub approval_policy: BrowserApprovalPolicy,
    pub status: BrowserTaskStatus,
    pub artifact_path: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedAnswerArtifact {
    pub route_decision: RouteDecision,
    pub evidence_bundle_id: String,
    pub evidence: Vec<EvidenceItem>,
}

#[derive(Debug, Clone)]
struct QdrantBackend {
    base_url: String,
    collection: String,
    client: BlockingClient,
}

#[derive(Debug)]
pub struct ControlPlane {
    db_path: PathBuf,
    artifacts_root: PathBuf,
    connection: Mutex<Connection>,
    qdrant: Option<QdrantBackend>,
}

impl ControlPlane {
    pub fn new(
        db_path: PathBuf,
        artifacts_root: PathBuf,
        qdrant_url: Option<String>,
        qdrant_collection: String,
    ) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(&artifacts_root)?;

        let connection = Connection::open(&db_path).map_err(|error| OraError::FileSystemError {
            path: db_path.to_string_lossy().to_string(),
            message: error.to_string(),
        })?;

        let control_plane = Self {
            db_path,
            artifacts_root,
            connection: Mutex::new(connection),
            qdrant: qdrant_url
                .filter(|value| !value.trim().is_empty())
                .map(|base_url| QdrantBackend {
                    base_url,
                    collection: qdrant_collection,
                    client: BlockingClient::new(),
                }),
        };

        control_plane.init_schema()?;
        control_plane.ensure_qdrant_collection();

        Ok(control_plane)
    }

    fn init_schema(&self) -> Result<()> {
        let schema = r#"
            CREATE TABLE IF NOT EXISTS route_decisions (
                id TEXT PRIMARY KEY,
                query TEXT NOT NULL,
                route_class TEXT NOT NULL,
                created_at TEXT NOT NULL,
                payload TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS evidence_items (
                id TEXT PRIMARY KEY,
                bundle_id TEXT NOT NULL,
                source_type TEXT NOT NULL,
                title TEXT NOT NULL,
                uri TEXT NOT NULL,
                created_at TEXT NOT NULL,
                payload TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS memory_records (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                payload TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS approval_requests (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                payload TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS mission_specs (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                payload TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS audit_events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                payload TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS browser_tasks (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                payload TEXT NOT NULL
            );
        "#;

        self.connection
            .lock()
            .execute_batch(schema)
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;

        Ok(())
    }

    pub fn record_route_decision(&self, decision: &RouteDecision) -> Result<()> {
        let payload = serde_json::to_string(decision)?;
        self.connection
            .lock()
            .execute(
                "INSERT OR REPLACE INTO route_decisions (id, query, route_class, created_at, payload)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    decision.id,
                    decision.query,
                    decision.selected_route.as_str(),
                    decision.created_at,
                    payload
                ],
            )
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
        Ok(())
    }

    pub fn recent_route_decisions(&self, limit: usize) -> Result<Vec<RouteDecision>> {
        let limit = limit.max(1) as i64;
        let connection = self.connection.lock();
        let mut statement = connection
            .prepare(
                "SELECT payload FROM route_decisions ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
        let rows = statement
            .query_map(params![limit], |row| row.get::<_, String>(0))
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;

        let mut results = Vec::new();
        for row in rows {
            let payload = row.map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
            if let Ok(record) = serde_json::from_str::<RouteDecision>(&payload) {
                results.push(record);
            }
        }

        Ok(results)
    }

    pub fn save_evidence_bundle(&self, bundle_id: &str, evidence: &[EvidenceItem]) -> Result<PathBuf> {
        let artifact_dir = self.artifacts_root.join("evidence");
        fs::create_dir_all(&artifact_dir)?;
        let artifact_path = artifact_dir.join(format!("{bundle_id}.json"));
        fs::write(&artifact_path, serde_json::to_vec_pretty(evidence)?)?;

        let connection = self.connection.lock();
        for item in evidence {
            let payload = serde_json::to_string(item)?;
            connection
                .execute(
                    "INSERT OR REPLACE INTO evidence_items (id, bundle_id, source_type, title, uri, created_at, payload)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        item.id,
                        item.bundle_id,
                        item.source_type.as_str(),
                        item.title,
                        item.uri,
                        item.observed_at,
                        payload
                    ],
                )
                .map_err(|error| OraError::FileSystemError {
                    path: self.db_path.to_string_lossy().to_string(),
                    message: error.to_string(),
                })?;
        }

        Ok(artifact_path)
    }

    pub fn evidence_bundle(&self, bundle_id: &str) -> Result<Vec<EvidenceItem>> {
        let connection = self.connection.lock();
        let mut statement = connection
            .prepare(
                "SELECT payload FROM evidence_items WHERE bundle_id = ?1 ORDER BY created_at ASC",
            )
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
        let rows = statement
            .query_map(params![bundle_id], |row| row.get::<_, String>(0))
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;

        let mut results = Vec::new();
        for row in rows {
            let payload = row.map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
            if let Ok(item) = serde_json::from_str::<EvidenceItem>(&payload) {
                results.push(item);
            }
        }

        Ok(results)
    }

    pub fn search_evidence(&self, query: &str, limit: usize) -> Result<Vec<EvidenceItem>> {
        let pattern = format!("%{}%", query);
        let connection = self.connection.lock();
        let mut statement = connection
            .prepare(
                "SELECT payload FROM evidence_items
                 WHERE title LIKE ?1 OR uri LIKE ?1 OR payload LIKE ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
        let rows = statement
            .query_map(params![pattern, limit.max(1) as i64], |row| row.get::<_, String>(0))
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;

        let mut results = Vec::new();
        for row in rows {
            let payload = row.map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
            if let Ok(item) = serde_json::from_str::<EvidenceItem>(&payload) {
                results.push(item);
            }
        }

        Ok(results)
    }

    pub fn save_memory_record(&self, record: &MemoryRecord) -> Result<()> {
        let payload = serde_json::to_string(record)?;
        self.connection
            .lock()
            .execute(
                "INSERT OR REPLACE INTO memory_records (id, title, content, created_at, updated_at, payload)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    record.id,
                    record.title,
                    record.content,
                    record.created_at,
                    record.updated_at,
                    payload
                ],
            )
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;

        self.best_effort_vector_upsert(record);
        Ok(())
    }

    pub fn search_memory(&self, query: &str, limit: usize) -> Result<Vec<MemoryRecord>> {
        let limit = limit.max(1);
        let pattern = format!("%{}%", query);
        let connection = self.connection.lock();
        let mut statement = connection
            .prepare(
                "SELECT payload FROM memory_records
                 WHERE title LIKE ?1 OR content LIKE ?1
                 ORDER BY updated_at DESC
                 LIMIT ?2",
            )
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
        let rows = statement
            .query_map(params![pattern, limit as i64], |row| row.get::<_, String>(0))
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;

        let mut combined = HashMap::<String, (f32, MemoryRecord)>::new();
        for row in rows {
            let payload = row.map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
            if let Ok(record) = serde_json::from_str::<MemoryRecord>(&payload) {
                combined.insert(record.id.clone(), (record.confidence, record));
            }
        }
        drop(statement);
        drop(connection);

        for record in self.search_memory_vector(query, limit) {
            let score = record.confidence;
            combined
                .entry(record.id.clone())
                .and_modify(|existing| {
                    if score > existing.0 {
                        *existing = (score, record.clone());
                    }
                })
                .or_insert((score, record));
        }

        let mut results = combined.into_values().collect::<Vec<_>>();
        results.sort_by(|left, right| right.0.partial_cmp(&left.0).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        Ok(results.into_iter().map(|(_, record)| record).collect())
    }

    pub fn save_approval_request(&self, approval: &ApprovalRequest) -> Result<()> {
        let payload = serde_json::to_string(approval)?;
        self.connection
            .lock()
            .execute(
                "INSERT OR REPLACE INTO approval_requests (id, status, created_at, payload)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    approval.id,
                    approval_state_as_str(&approval.status),
                    approval.created_at,
                    payload
                ],
            )
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
        Ok(())
    }

    pub fn resolve_approval(
        &self,
        approval_id: &str,
        status: ApprovalState,
        approver: Option<String>,
        resolution_reason: Option<String>,
    ) -> Result<Option<ApprovalRequest>> {
        let current = self.approval_request(approval_id)?;
        let Some(mut approval) = current else {
            return Ok(None);
        };

        approval.status = status;
        approval.approver = approver;
        approval.resolution_reason = resolution_reason;
        approval.resolved_at = Some(now_rfc3339());
        self.save_approval_request(&approval)?;
        Ok(Some(approval))
    }

    pub fn approval_request(&self, approval_id: &str) -> Result<Option<ApprovalRequest>> {
        let connection = self.connection.lock();
        let mut statement = connection
            .prepare("SELECT payload FROM approval_requests WHERE id = ?1")
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
        let mut rows = statement
            .query(params![approval_id])
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
        let Some(row) = rows.next().map_err(|error| OraError::FileSystemError {
            path: self.db_path.to_string_lossy().to_string(),
            message: error.to_string(),
        })? else {
            return Ok(None);
        };
        let payload: String = row.get(0).map_err(|error| OraError::FileSystemError {
            path: self.db_path.to_string_lossy().to_string(),
            message: error.to_string(),
        })?;
        Ok(serde_json::from_str(&payload).ok())
    }

    pub fn pending_approval_requests(&self) -> Result<Vec<ApprovalRequest>> {
        let connection = self.connection.lock();
        let mut statement = connection
            .prepare(
                "SELECT payload FROM approval_requests WHERE status = 'pending' ORDER BY created_at ASC",
            )
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;

        let mut approvals = Vec::new();
        for row in rows {
            let payload = row.map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
            if let Ok(approval) = serde_json::from_str::<ApprovalRequest>(&payload) {
                approvals.push(approval);
            }
        }

        Ok(approvals)
    }

    pub fn save_mission(&self, mission: &MissionSpec) -> Result<()> {
        let payload = serde_json::to_string(mission)?;
        self.connection
            .lock()
            .execute(
                "INSERT OR REPLACE INTO mission_specs (id, status, created_at, payload)
                 VALUES (?1, ?2, ?3, ?4)",
                params![mission.id, mission.status, mission.created_at, payload],
            )
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
        Ok(())
    }

    pub fn list_missions(&self, limit: usize) -> Result<Vec<MissionSpec>> {
        self.list_payloads(
            "SELECT payload FROM mission_specs ORDER BY created_at DESC LIMIT ?1",
            limit,
        )
    }

    pub fn save_browser_task(&self, task: &BrowserTask) -> Result<()> {
        let payload = serde_json::to_string(task)?;
        self.connection
            .lock()
            .execute(
                "INSERT OR REPLACE INTO browser_tasks (id, status, created_at, payload)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    task.id,
                    browser_task_status_as_str(&task.status),
                    task.created_at,
                    payload
                ],
            )
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
        Ok(())
    }

    pub fn list_browser_tasks(&self, limit: usize) -> Result<Vec<BrowserTask>> {
        self.list_payloads(
            "SELECT payload FROM browser_tasks ORDER BY created_at DESC LIMIT ?1",
            limit,
        )
    }

    pub fn save_audit_event(&self, event: &AuditEvent) -> Result<()> {
        let payload = serde_json::to_string(event)?;
        self.connection
            .lock()
            .execute(
                "INSERT OR REPLACE INTO audit_events (id, event_type, created_at, payload)
                 VALUES (?1, ?2, ?3, ?4)",
                params![event.id, event.event_type, event.created_at, payload],
            )
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
        Ok(())
    }

    pub fn recent_audit_events(&self, limit: usize) -> Result<Vec<AuditEvent>> {
        self.list_payloads(
            "SELECT payload FROM audit_events ORDER BY created_at DESC LIMIT ?1",
            limit,
        )
    }

    pub fn save_json_artifact<T: Serialize>(&self, category: &str, name: &str, value: &T) -> Result<PathBuf> {
        let directory = self.artifacts_root.join(category);
        fs::create_dir_all(&directory)?;
        let path = directory.join(format!("{name}.json"));
        fs::write(&path, serde_json::to_vec_pretty(value)?)?;
        Ok(path)
    }

    fn list_payloads<T>(&self, sql: &str, limit: usize) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let connection = self.connection.lock();
        let mut statement = connection.prepare(sql).map_err(|error| OraError::FileSystemError {
            path: self.db_path.to_string_lossy().to_string(),
            message: error.to_string(),
        })?;
        let rows = statement
            .query_map(params![limit.max(1) as i64], |row| row.get::<_, String>(0))
            .map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;

        let mut values = Vec::new();
        for row in rows {
            let payload = row.map_err(|error| OraError::FileSystemError {
                path: self.db_path.to_string_lossy().to_string(),
                message: error.to_string(),
            })?;
            if let Ok(value) = serde_json::from_str::<T>(&payload) {
                values.push(value);
            }
        }

        Ok(values)
    }

    fn ensure_qdrant_collection(&self) {
        let Some(qdrant) = &self.qdrant else {
            return;
        };

        let _ = qdrant
            .client
            .put(format!(
                "{}/collections/{}",
                qdrant.base_url.trim_end_matches('/'),
                qdrant.collection
            ))
            .json(&serde_json::json!({
                "vectors": {
                    "size": DEFAULT_VECTOR_DIMENSIONS,
                    "distance": "Cosine"
                }
            }))
            .send();
    }

    fn best_effort_vector_upsert(&self, record: &MemoryRecord) {
        let Some(qdrant) = &self.qdrant else {
            return;
        };

        let vector = embed_text(&format!("{} {}", record.title, record.content));
        let _ = qdrant
            .client
            .put(format!(
                "{}/collections/{}/points?wait=true",
                qdrant.base_url.trim_end_matches('/'),
                qdrant.collection
            ))
            .json(&serde_json::json!({
                "points": [{
                    "id": record.id,
                    "vector": vector,
                    "payload": {
                        "title": record.title,
                        "content": record.content,
                        "payload": record
                    }
                }]
            }))
            .send();
    }

    fn search_memory_vector(&self, query: &str, limit: usize) -> Vec<MemoryRecord> {
        let Some(qdrant) = &self.qdrant else {
            return Vec::new();
        };

        let response = qdrant
            .client
            .post(format!(
                "{}/collections/{}/points/search",
                qdrant.base_url.trim_end_matches('/'),
                qdrant.collection
            ))
            .json(&serde_json::json!({
                "vector": embed_text(query),
                "limit": limit,
                "with_payload": true
            }))
            .send();

        let Ok(response) = response else {
            return Vec::new();
        };
        let Ok(payload) = response.json::<Value>() else {
            return Vec::new();
        };
        let Some(results) = payload.get("result").and_then(Value::as_array) else {
            return Vec::new();
        };

        results
            .iter()
            .filter_map(|item| {
                let payload = item.get("payload")?.get("payload")?.clone();
                serde_json::from_value::<MemoryRecord>(payload).ok()
            })
            .collect()
    }
}

fn approval_state_as_str(value: &ApprovalState) -> &'static str {
    match value {
        ApprovalState::Pending => "pending",
        ApprovalState::Approved => "approved",
        ApprovalState::Rejected => "rejected",
    }
}

fn browser_task_status_as_str(value: &BrowserTaskStatus) -> &'static str {
    match value {
        BrowserTaskStatus::PendingApproval => "pending_approval",
        BrowserTaskStatus::Ready => "ready",
        BrowserTaskStatus::Running => "running",
        BrowserTaskStatus::Completed => "completed",
        BrowserTaskStatus::Rejected => "rejected",
        BrowserTaskStatus::Failed => "failed",
    }
}

fn embed_text(text: &str) -> Vec<f32> {
    let mut vector = vec![0.0f32; DEFAULT_VECTOR_DIMENSIONS];
    for token in text
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
    {
        let mut hash = 0u64;
        for byte in token.as_bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(*byte as u64);
        }
        let index = (hash as usize) % DEFAULT_VECTOR_DIMENSIONS;
        vector[index] += 1.0;
    }

    let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut vector {
            *value /= norm;
        }
    }

    vector
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn make_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn content_hash(value: &str) -> String {
    format!("{:x}", md5::compute(value))
}

pub fn workspace_uri(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

pub type SharedControlPlane = Arc<ControlPlane>;

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths(test_name: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!("ora-control-plane-{test_name}-{}", make_id()));
        (root.join("state.sqlite"), root.join("artifacts"))
    }

    #[test]
    fn control_plane_persists_memory_records() {
        let (db_path, artifacts_path) = temp_paths("memory");
        let control_plane = ControlPlane::new(db_path, artifacts_path, None, "ora_memory".to_string())
            .expect("control plane should initialize");

        let record = MemoryRecord {
            id: make_id(),
            kind: "conversation_distillation".to_string(),
            title: "Rust sidecar launch note".to_string(),
            content: "ORA is a governed MCP runtime for technical operators.".to_string(),
            provenance: "unit_test".to_string(),
            confidence: 0.9,
            freshness_expires_at: None,
            superseded_by: None,
            metadata: serde_json::json!({}),
            created_at: now_rfc3339(),
            updated_at: now_rfc3339(),
        };

        control_plane
            .save_memory_record(&record)
            .expect("memory save should succeed");
        let matches = control_plane
            .search_memory("governed MCP runtime", 5)
            .expect("memory search should succeed");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, record.id);
    }

    #[test]
    fn control_plane_round_trips_evidence_bundles() {
        let (db_path, artifacts_path) = temp_paths("evidence");
        let control_plane = ControlPlane::new(db_path, artifacts_path, None, "ora_memory".to_string())
            .expect("control plane should initialize");
        let bundle_id = make_id();
        let evidence = vec![EvidenceItem {
            id: make_id(),
            bundle_id: bundle_id.clone(),
            source_type: EvidenceSourceType::LocalDocument,
            title: "README".to_string(),
            uri: "/tmp/README.md".to_string(),
            snippet: "Local runtime truth".to_string(),
            content_hash: content_hash("Local runtime truth"),
            observed_at: now_rfc3339(),
            freshness_expires_at: None,
            confidence: 0.8,
            provenance: "unit_test".to_string(),
            metadata: serde_json::json!({}),
            superseded_by: None,
        }];

        control_plane
            .save_evidence_bundle(&bundle_id, &evidence)
            .expect("bundle save should succeed");
        let loaded = control_plane
            .evidence_bundle(&bundle_id)
            .expect("bundle load should succeed");

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "README");
    }
}
