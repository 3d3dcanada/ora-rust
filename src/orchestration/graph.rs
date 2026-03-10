//! OrA Directed Acyclic Graph (DAG) Router
//!
//! Manages the execution flow between specialized Agent nodes using petgraph.

use super::agent::{AgentContext, AgentType, NodeAgent};
use crate::error::{OraError, Result};
use petgraph::graph::{DiGraph, NodeIndex};
use std::sync::Arc;

/// The finalized state after graph execution completes
pub struct RouteResult {
    pub final_context: AgentContext,
    pub path_taken: Vec<AgentType>,
}

pub struct DagRouter {
    graph: DiGraph<Arc<dyn NodeAgent>, ()>,
    nodes: std::collections::HashMap<AgentType, NodeIndex>,
}

impl Default for DagRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl DagRouter {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            nodes: std::collections::HashMap::new(),
        }
    }

    /// Add an agent node to the graph
    pub fn add_agent(&mut self, agent: Arc<dyn NodeAgent>) {
        let agent_type = agent.agent_type();
        let idx = self.graph.add_node(agent);
        self.nodes.insert(agent_type, idx);
    }

    /// Add a directional route from one agent to another
    pub fn add_route(&mut self, from: AgentType, to: AgentType) -> Result<()> {
        let from_idx = self
            .nodes
            .get(&from)
            .cloned()
            .ok_or_else(|| OraError::AgentGraphError {
                message: format!("Missing agent in graph: {:?}", from),
            })?;

        let to_idx = self
            .nodes
            .get(&to)
            .cloned()
            .ok_or_else(|| OraError::AgentGraphError {
                message: format!("Missing agent in graph: {:?}", to),
            })?;

        self.graph.add_edge(from_idx, to_idx, ());
        Ok(())
    }

    /// Execute the graph starting from a specific node
    pub async fn execute(
        &self,
        start_node: AgentType,
        initial_context: AgentContext,
    ) -> Result<RouteResult> {
        let start_idx = *self
            .nodes
            .get(&start_node)
            .ok_or_else(|| OraError::AgentGraphError {
                message: format!("Missing start agent: {:?}", start_node),
            })?;

        let mut context = initial_context;
        let mut path_taken = Vec::new();

        let mut current_idx = start_idx;

        // Linear path traversal following outbound edges
        loop {
            let agent = &self.graph[current_idx];
            path_taken.push(agent.agent_type());

            // Execute the agent logic (this could invoke LLM calls, run commands, etc)
            context = agent.execute(context).await?;

            // For a simple pipeline DAG, we take the first outbound neighbor.
            // A more complex router would evaluate state conditions to pick an edge.
            let mut neighbors = self.graph.neighbors(current_idx);
            if let Some(next_idx) = neighbors.next() {
                current_idx = next_idx;
            } else {
                break; // We reached a terminal node
            }
        }

        Ok(RouteResult {
            final_context: context,
            path_taken,
        })
    }
}
