//! OrA Orchestration Module
//!
//! Provides the Directed Acyclic Graph (DAG) for routing tasks between specialized agents.

pub mod agent;
pub mod graph;

pub use agent::{AgentContext, AgentType, NodeAgent};
pub use graph::{DagRouter, RouteResult};
