//! Types for C2 <-> Agent communication.

use serde::{Deserialize, Serialize};

// TODO: refactor to use identifier and json-serialized message

/// Message send to agent from C2 server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    C2Request(C2Request),
    AgentResponse(AgentResponse),
}

/// Task request from C2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum C2Request {
    ExecShell(String),
    GetCommands,
    ExecCommand(String, Vec<String>),
}

/// Task response from agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentResponse {
    ShellResponse(String),
    Commands(Vec<(String, String)>),
    CommandOutput(String),
    Error(String),
}
