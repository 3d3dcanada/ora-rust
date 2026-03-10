//! OrA Graph Node Agent
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    Planner,
    Researcher,
    Coder,
    Reviewer,
    TerminalExec,
}

/// The context passed between nodes in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub session_id: String,
    pub original_objective: String,
    pub current_payload: String,
    pub metadata: std::collections::HashMap<String, String>,
}

#[async_trait::async_trait]
pub trait NodeAgent: Send + Sync {
    /// Return the type/role of this agent
    fn agent_type(&self) -> AgentType;

    /// Execute the agent's specific logic on the given context.
    /// Returns the mutated/updated context to be passed to the next node.
    async fn execute(&self, context: AgentContext) -> Result<AgentContext>;
}
