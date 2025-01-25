//! Types for C2 <-> Agent communication.

use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Task request from C2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum C2Request {
    GetUlid,
    GetCommands,
    ExecShell(String),
    ExecCommand(String, Vec<String>),
}

/// Task response from agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentResponse {
    Ulid(Ulid),
    Error(String),
    Commands(Vec<(String, String)>),
    ShellResponse(String),
    CommandOutput(String),
}
