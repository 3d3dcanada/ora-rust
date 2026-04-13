//! Shared runtime operations for governed answers, memory, missions, and browser tasks.

use crate::error::{OraError, Result};
use crate::kernel::{
    content_hash, make_id, workspace_uri, ApprovalRequest, ApprovalState, AuditEvent,
    BrowserApprovalPolicy, BrowserAuthPolicy, BrowserTask, BrowserTaskStatus, EvidenceBurden,
    EvidenceItem, EvidenceSourceType, FreshnessRequirement, MemoryRecord, MissionSpec, RiskClass,
    RouteClass, RouteDecision, TaskClass, VerifiedAnswerArtifact, WebSearchService,
};
use crate::kernel::web_search::SearchResult;
use crate::state::AppState;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const MAX_LOCAL_FILE_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedAnswerResult {
    pub response: String,
    pub route_decision: RouteDecision,
    pub evidence_bundle_id: String,
    pub evidence: Vec<EvidenceItem>,
    pub artifact_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTaskResult {
    pub task: BrowserTask,
    pub approval_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTaskRequest {
    pub task: String,
    pub url: Option<String>,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeOutcome {
    VerifiedAnswer(VerifiedAnswerResult),
    BrowserTask(BrowserTaskResult),
}

pub async fn handle_runtime_query(state: &AppState, query: &str) -> Result<RuntimeOutcome> {
    let query = query.trim();
    if query.is_empty() {
        return Err(OraError::AgentError {
            message: "Query cannot be empty".to_string(),
        });
    }

    if looks_like_browser_task(query) {
        let outcome = create_browser_task(
            state,
            BrowserTaskRequest {
                task: query.to_string(),
                url: extract_url(query),
                allowed_domains: extract_allowed_domains(query),
            },
        )?;
        return Ok(RuntimeOutcome::BrowserTask(outcome));
    }

    generate_verified_answer(state, query)
        .await
        .map(RuntimeOutcome::VerifiedAnswer)
}

pub async fn generate_verified_answer(
    state: &AppState,
    query: &str,
) -> Result<VerifiedAnswerResult> {
    let memory_matches = state.search_memory(query, 5);
    let local_matches = search_local_workspace(query, &state.config.workspace_root, 5);
    let cached_matches = state
        .control_plane
        .search_evidence(query, 5)
        .unwrap_or_default();

    let freshness_requirement = classify_freshness(query);
    let risk_class = classify_risk(query);
    let evidence_burden = classify_evidence_burden(&risk_class, &freshness_requirement);

    let (selected_route, requires_live_retrieval, route_reason) = if freshness_requirement
        == FreshnessRequirement::Live
    {
        (
            RouteClass::LiveWeb,
            true,
            "The query asks for fresh or time-sensitive information, so live retrieval is required."
                .to_string(),
        )
    } else if !local_matches.is_empty() {
        (
            RouteClass::LocalDocuments,
            false,
            "Relevant local workspace documents exist, so ORA grounded the answer in local artifacts first."
                .to_string(),
        )
    } else if !memory_matches.is_empty() {
        (
            RouteClass::Memory,
            false,
            "Existing memory records match the query with usable confidence.".to_string(),
        )
    } else if !cached_matches.is_empty() {
        (
            RouteClass::CachedEvidence,
            false,
            "Cached evidence already exists for this topic, so ORA reused that bundle before going live."
                .to_string(),
        )
    } else {
        (
            RouteClass::LiveWeb,
            true,
            "No sufficient local or cached evidence was available, so ORA retrieved live web evidence."
                .to_string(),
        )
    };

    let route_decision = RouteDecision {
        id: make_id(),
        query: query.to_string(),
        task_class: TaskClass::GeneralQuestion,
        freshness_requirement,
        risk_class,
        evidence_burden,
        selected_route: selected_route.clone(),
        route_reason,
        requires_live_retrieval,
        created_at: now_rfc3339(),
    };
    state.control_plane.record_route_decision(&route_decision)?;

    let bundle_id = make_id();
    let live_matches = if selected_route == RouteClass::LiveWeb {
        search_live_web(query).await.unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut evidence = match selected_route {
        RouteClass::Memory => memory_to_evidence(&memory_matches, &bundle_id),
        RouteClass::LocalDocuments => local_matches.clone(),
        RouteClass::CachedEvidence => cached_matches.clone(),
        RouteClass::LiveWeb => live_matches,
        RouteClass::BrowserMission => Vec::new(),
    };

    if evidence.is_empty() {
        evidence.extend(local_matches.clone());
    }
    if evidence.is_empty() {
        evidence.extend(cached_matches.clone());
    }
    if evidence.is_empty() {
        evidence.extend(memory_to_evidence(&memory_matches, &bundle_id));
    }

    for item in &mut evidence {
        item.bundle_id = bundle_id.clone();
    }

    state.control_plane.save_evidence_bundle(&bundle_id, &evidence)?;
    let artifact = VerifiedAnswerArtifact {
        route_decision: route_decision.clone(),
        evidence_bundle_id: bundle_id.clone(),
        evidence: evidence.clone(),
    };
    let artifact_path = state
        .control_plane
        .save_json_artifact("verified_answers", &bundle_id, &artifact)
        .ok()
        .map(|path| path.to_string_lossy().to_string());

    let response = grounded_answer_text(state, query, &route_decision, &evidence).await?;

    let memory_record = MemoryRecord {
        id: make_id(),
        kind: "verified_answer".to_string(),
        title: query.to_string(),
        content: response.clone(),
        provenance: format!("evidence_bundle:{bundle_id}"),
        confidence: memory_confidence(&route_decision),
        freshness_expires_at: evidence.iter().find_map(|item| item.freshness_expires_at.clone()),
        superseded_by: None,
        metadata: json!({
            "route_decision_id": route_decision.id,
            "evidence_bundle_id": bundle_id,
            "route": route_decision.selected_route,
        }),
        created_at: now_rfc3339(),
        updated_at: now_rfc3339(),
    };
    state.control_plane.save_memory_record(&memory_record)?;
    state.record_audit_event(AuditEvent {
        id: make_id(),
        event_type: "verified_answer".to_string(),
        actor: "ora".to_string(),
        summary: format!("Generated governed answer for '{}'", query),
        subject_id: Some(route_decision.id.clone()),
        metadata: json!({
            "query": query,
            "bundle_id": bundle_id,
            "route": route_decision.selected_route,
            "artifact_path": artifact_path,
        }),
        created_at: now_rfc3339(),
    });

    Ok(VerifiedAnswerResult {
        response,
        route_decision,
        evidence_bundle_id: bundle_id,
        evidence,
        artifact_path,
    })
}

pub async fn grounded_summarize(
    state: &AppState,
    title: &str,
    text: &str,
) -> Result<VerifiedAnswerResult> {
    let route_decision = RouteDecision {
        id: make_id(),
        query: title.to_string(),
        task_class: TaskClass::WorkspaceResearch,
        freshness_requirement: FreshnessRequirement::Historical,
        risk_class: RiskClass::Low,
        evidence_burden: EvidenceBurden::Grounded,
        selected_route: RouteClass::LocalDocuments,
        route_reason: "The summary was grounded in provided local content.".to_string(),
        requires_live_retrieval: false,
        created_at: now_rfc3339(),
    };
    state.control_plane.record_route_decision(&route_decision)?;

    let bundle_id = make_id();
    let evidence = vec![EvidenceItem {
        id: make_id(),
        bundle_id: bundle_id.clone(),
        source_type: EvidenceSourceType::LocalDocument,
        title: title.to_string(),
        uri: "inline://grounded-summary".to_string(),
        snippet: truncate(text, 420),
        content_hash: content_hash(text),
        observed_at: now_rfc3339(),
        freshness_expires_at: None,
        confidence: 0.92,
        provenance: "inline_text".to_string(),
        metadata: json!({ "title": title }),
        superseded_by: None,
    }];
    state.control_plane.save_evidence_bundle(&bundle_id, &evidence)?;

    let response = if state.llm.is_configured() {
        match state
            .llm
            .chat(
                "You are ORA. Produce a concise summary grounded only in the provided evidence.",
                &format!("Title: {title}\nEvidence:\n{}", evidence[0].snippet),
            )
            .await
        {
            Ok(summary) => summary,
            Err(_) => format!("Grounded summary for {title}: {}", evidence[0].snippet),
        }
    } else {
        format!("Grounded summary for {title}: {}", evidence[0].snippet)
    };

    Ok(VerifiedAnswerResult {
        response,
        route_decision,
        evidence_bundle_id: bundle_id,
        evidence,
        artifact_path: None,
    })
}

pub fn create_browser_task(
    state: &AppState,
    request: BrowserTaskRequest,
) -> Result<BrowserTaskResult> {
    let action_classes = classify_browser_action_classes(&request.task);
    let mut domains = request.allowed_domains;
    if domains.is_empty() {
        domains = extract_allowed_domains(&request.task);
    }

    let auth_policy = if contains_any(
        &request.task.to_ascii_lowercase(),
        &["login", "sign in", "authenticate", "password", "otp"],
    ) {
        BrowserAuthPolicy::ManualOnly
    } else {
        BrowserAuthPolicy::ExistingSessionOnly
    };

    let approval_policy = if action_classes.iter().any(|action| {
        matches!(
            action.as_str(),
            "login" | "submit" | "purchase" | "permission_change" | "destructive"
        )
    }) {
        BrowserApprovalPolicy::AlwaysApprove
    } else {
        BrowserApprovalPolicy::SensitiveActionApproval
    };

    let mut task = BrowserTask {
        id: make_id(),
        task: request.task.clone(),
        url: request.url,
        allowed_domains: domains,
        allowed_action_classes: action_classes.clone(),
        auth_policy,
        approval_policy,
        status: BrowserTaskStatus::Ready,
        artifact_path: None,
        created_at: now_rfc3339(),
    };

    let approval_id = if needs_browser_approval(&action_classes) {
        task.status = BrowserTaskStatus::PendingApproval;
        let approval = ApprovalRequest {
            id: make_id(),
            operation: "browser_task".to_string(),
            action_class: action_classes.join(","),
            risk_class: RiskClass::High,
            authority_required: "A3".to_string(),
            description: format!("Browser task requires approval: {}", request.task),
            request_payload: json!({
                "task_id": task.id,
                "task": request.task,
                "url": task.url,
                "allowed_domains": task.allowed_domains,
                "action_classes": action_classes,
            }),
            status: ApprovalState::Pending,
            created_at: now_rfc3339(),
            resolved_at: None,
            approver: None,
            resolution_reason: None,
        };
        let approval_id = approval.id.clone();
        state.queue_approval_request(approval);
        Some(approval_id)
    } else {
        None
    };

    state.control_plane.save_browser_task(&task)?;
    state.record_audit_event(AuditEvent {
        id: make_id(),
        event_type: "browser_task".to_string(),
        actor: "ora".to_string(),
        summary: format!("Registered browser task '{}'", task.task),
        subject_id: Some(task.id.clone()),
        metadata: json!({
            "approval_id": approval_id,
            "status": task.status,
            "allowed_domains": task.allowed_domains,
            "action_classes": task.allowed_action_classes,
        }),
        created_at: now_rfc3339(),
    });

    Ok(BrowserTaskResult {
        message: if approval_id.is_some() {
            "Browser task queued for approval. Approve it before executing sensitive actions."
                .to_string()
        } else {
            "Browser task registered in governed mode.".to_string()
        },
        task,
        approval_id,
    })
}

pub fn create_mission(
    state: &AppState,
    name: &str,
    query: &str,
    sources: Vec<String>,
    extraction_rules: Vec<String>,
    freshness_policy: &str,
) -> Result<MissionSpec> {
    let mission = MissionSpec {
        id: make_id(),
        name: name.to_string(),
        query: query.to_string(),
        sources,
        freshness_policy: freshness_policy.to_string(),
        extraction_rules,
        storage_policy: "append_only_artifacts".to_string(),
        status: "planned".to_string(),
        artifact_path: None,
        created_at: now_rfc3339(),
        last_run_at: None,
    };
    state.control_plane.save_mission(&mission)?;
    state.record_audit_event(AuditEvent {
        id: make_id(),
        event_type: "mission_created".to_string(),
        actor: "ora".to_string(),
        summary: format!("Created mission '{}'", mission.name),
        subject_id: Some(mission.id.clone()),
        metadata: json!({
            "query": mission.query,
            "sources": mission.sources,
            "freshness_policy": mission.freshness_policy,
        }),
        created_at: now_rfc3339(),
    });
    Ok(mission)
}

async fn grounded_answer_text(
    state: &AppState,
    query: &str,
    route_decision: &RouteDecision,
    evidence: &[EvidenceItem],
) -> Result<String> {
    if !state.llm.is_configured() {
        return Ok(deterministic_grounded_answer(query, route_decision, evidence));
    }

    let evidence_block = evidence
        .iter()
        .enumerate()
        .map(|(index, item)| {
            format!(
                "[E{}] {} | {} | {}\n{}",
                index + 1,
                item.title,
                item.uri,
                item.provenance,
                item.snippet
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let system_prompt = "You are ORA, a governed local runtime for MCP. Answer using only the provided evidence. Cite evidence with [E1], [E2], etc. If the evidence is incomplete or stale, say so plainly.";
    let user_prompt = format!(
        "Query: {query}\nRoute: {:?}\nEvidence:\n{evidence_block}",
        route_decision.selected_route
    );

    match state.llm.chat(system_prompt, &user_prompt).await {
        Ok(answer) => Ok(answer),
        Err(_) => Ok(deterministic_grounded_answer(query, route_decision, evidence)),
    }
}

fn deterministic_grounded_answer(
    query: &str,
    route_decision: &RouteDecision,
    evidence: &[EvidenceItem],
) -> String {
    if evidence.is_empty() {
        return format!(
            "ORA could not ground an answer for '{}' because no evidence was available. Route decision: {:?}.",
            query, route_decision.selected_route
        );
    }

    let mut response = format!(
        "Grounded answer for '{}':\nRoute: {:?}\n",
        query, route_decision.selected_route
    );
    for (index, item) in evidence.iter().take(5).enumerate() {
        response.push_str(&format!(
            "[E{}] {} ({})\n{}\n",
            index + 1,
            item.title,
            item.uri,
            item.snippet
        ));
    }
    response
}

fn search_local_workspace(query: &str, workspace_root: &Path, limit: usize) -> Vec<EvidenceItem> {
    if !workspace_root.exists() {
        return Vec::new();
    }

    let tokens = tokenize(query);
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::<(usize, PathBuf, String, String)>::new();
    for entry in WalkDir::new(workspace_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !entry.file_type().is_file() || should_skip_path(path) {
            continue;
        }

        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.len() > MAX_LOCAL_FILE_BYTES {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let lowered = content.to_ascii_lowercase();
        let score = tokens
            .iter()
            .map(|token| lowered.matches(token).count().min(5))
            .sum::<usize>();
        if score == 0 {
            continue;
        }

        let snippet = extract_snippet(&content, &tokens);
        let title = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("workspace_file")
            .to_string();
        results.push((score, path.to_path_buf(), title, snippet));
    }

    results.sort_by(|left, right| right.0.cmp(&left.0));
    results.truncate(limit);

    results
        .into_iter()
        .map(|(score, path, title, snippet)| EvidenceItem {
            id: make_id(),
            bundle_id: String::new(),
            source_type: EvidenceSourceType::LocalDocument,
            title,
            uri: workspace_uri(&path),
            snippet,
            content_hash: content_hash(&format!("{score}:{}", path.display())),
            observed_at: now_rfc3339(),
            freshness_expires_at: None,
            confidence: (0.5 + (score as f32 / 20.0)).min(0.95),
            provenance: "local_workspace".to_string(),
            metadata: json!({ "score": score }),
            superseded_by: None,
        })
        .collect()
}

async fn search_live_web(query: &str) -> Result<Vec<EvidenceItem>> {
    let service = WebSearchService::from_env();
    let results = service.search(query, 5).await?;
    Ok(results_to_evidence(results))
}

fn results_to_evidence(results: Vec<SearchResult>) -> Vec<EvidenceItem> {
    results
        .into_iter()
        .map(|result| EvidenceItem {
            id: make_id(),
            bundle_id: String::new(),
            source_type: EvidenceSourceType::LiveWeb,
            title: result.title,
            uri: result.url.clone(),
            snippet: result.snippet,
            content_hash: content_hash(&result.url),
            observed_at: now_rfc3339(),
            freshness_expires_at: Some(now_plus_hours(12)),
            confidence: 0.7,
            provenance: result.source,
            metadata: json!({ "source": "web_search" }),
            superseded_by: None,
        })
        .collect()
}

fn memory_to_evidence(records: &[MemoryRecord], bundle_id: &str) -> Vec<EvidenceItem> {
    records
        .iter()
        .map(|record| EvidenceItem {
            id: make_id(),
            bundle_id: bundle_id.to_string(),
            source_type: EvidenceSourceType::Memory,
            title: record.title.clone(),
            uri: format!("memory://{}", record.id),
            snippet: truncate(&record.content, 280),
            content_hash: content_hash(&record.content),
            observed_at: record.updated_at.clone(),
            freshness_expires_at: record.freshness_expires_at.clone(),
            confidence: record.confidence,
            provenance: record.provenance.clone(),
            metadata: record.metadata.clone(),
            superseded_by: record.superseded_by.clone(),
        })
        .collect()
}

fn classify_freshness(query: &str) -> FreshnessRequirement {
    let lowered = query.to_ascii_lowercase();
    if contains_any(
        &lowered,
        &["latest", "today", "current", "recent", "right now", "status", "price"],
    ) {
        FreshnessRequirement::Live
    } else if contains_any(&lowered, &["this week", "new", "updated"]) {
        FreshnessRequirement::Recent
    } else {
        FreshnessRequirement::Historical
    }
}

fn classify_risk(query: &str) -> RiskClass {
    let lowered = query.to_ascii_lowercase();
    if contains_any(
        &lowered,
        &[
            "production",
            "billing",
            "money",
            "purchase",
            "delete",
            "customer",
            "security",
            "credential",
        ],
    ) {
        RiskClass::High
    } else if contains_any(&lowered, &["deploy", "account", "auth", "provider"]) {
        RiskClass::Medium
    } else {
        RiskClass::Low
    }
}

fn classify_evidence_burden(
    risk_class: &RiskClass,
    freshness_requirement: &FreshnessRequirement,
) -> EvidenceBurden {
    if *risk_class == RiskClass::High || *freshness_requirement == FreshnessRequirement::Live {
        EvidenceBurden::Strict
    } else if *risk_class == RiskClass::Medium {
        EvidenceBurden::Grounded
    } else {
        EvidenceBurden::Minimal
    }
}

fn memory_confidence(route_decision: &RouteDecision) -> f32 {
    match route_decision.evidence_burden {
        EvidenceBurden::Strict => 0.88,
        EvidenceBurden::Grounded => 0.8,
        EvidenceBurden::Minimal => 0.72,
    }
}

fn looks_like_browser_task(query: &str) -> bool {
    contains_any(
        &query.to_ascii_lowercase(),
        &[
            "open browser",
            "visit ",
            "click ",
            "log in",
            "login",
            "sign in",
            "submit",
            "fill out",
            "checkout",
            "buy ",
            "browser task",
            "website",
        ],
    )
}

fn classify_browser_action_classes(task: &str) -> Vec<String> {
    let lowered = task.to_ascii_lowercase();
    let mut actions = Vec::new();
    if contains_any(
        &lowered,
        &["login", "log in", "sign in", "authenticate", "password", "otp"],
    ) {
        actions.push("login".to_string());
    }
    if contains_any(&lowered, &["submit", "post", "send", "upload"]) {
        actions.push("submit".to_string());
    }
    if contains_any(&lowered, &["buy", "purchase", "checkout", "pay"]) {
        actions.push("purchase".to_string());
    }
    if contains_any(&lowered, &["permission", "allow", "grant", "settings"]) {
        actions.push("permission_change".to_string());
    }
    if contains_any(&lowered, &["delete", "remove", "destroy", "close account"]) {
        actions.push("destructive".to_string());
    }
    if actions.is_empty() {
        actions.push("read_only".to_string());
    }
    actions
}

fn needs_browser_approval(action_classes: &[String]) -> bool {
    action_classes.iter().any(|action| action != "read_only")
}

fn extract_allowed_domains(task: &str) -> Vec<String> {
    let regex = Regex::new(r"https?://([^/\s]+)").expect("url regex should compile");
    let mut domains = HashSet::new();
    for captures in regex.captures_iter(task) {
        if let Some(domain) = captures.get(1) {
            domains.insert(domain.as_str().to_string());
        }
    }
    domains.into_iter().collect()
}

fn extract_url(task: &str) -> Option<String> {
    let regex = Regex::new(r"https?://[^\s]+").expect("url regex should compile");
    regex.find(task).map(|value| value.as_str().to_string())
}

fn tokenize(query: &str) -> Vec<String> {
    query.to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| token.len() >= 3)
        .map(|token| token.to_string())
        .collect()
}

fn extract_snippet(content: &str, tokens: &[String]) -> String {
    for line in content.lines() {
        let lowered = line.to_ascii_lowercase();
        if tokens.iter().any(|token| lowered.contains(token)) {
            return truncate(line.trim(), 280);
        }
    }
    truncate(content.trim(), 280)
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn should_skip_path(path: &Path) -> bool {
    const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules", ".next", "dist", "build"];
    path.components().any(|component| {
        let value = component.as_os_str().to_string_lossy();
        SKIP_DIRS.iter().any(|skip| value == *skip)
    })
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn now_plus_hours(hours: i64) -> String {
    (chrono::Utc::now() + chrono::Duration::hours(hours)).to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_action_classification() {
        let actions = classify_browser_action_classes("Log in and purchase a plan");
        assert!(actions.iter().any(|action| action == "login"));
        assert!(actions.iter().any(|action| action == "purchase"));
    }

    #[test]
    fn test_extract_allowed_domains() {
        let domains = extract_allowed_domains("Visit https://example.com/docs and inspect it");
        assert_eq!(domains, vec!["example.com".to_string()]);
    }

    #[test]
    fn test_classify_freshness() {
        assert_eq!(
            classify_freshness("What is the latest status?"),
            FreshnessRequirement::Live
        );
        assert_eq!(
            classify_freshness("Summarize this architecture"),
            FreshnessRequirement::Historical
        );
    }

    #[test]
    fn test_should_skip_path() {
        assert!(should_skip_path(Path::new("/tmp/project/.git/config")));
        assert!(!should_skip_path(Path::new("/tmp/project/src/main.rs")));
    }
}
