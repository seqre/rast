//! Types for C2 <-> UI communication.


use serde::{Deserialize, Serialize};
use ulid::Ulid;
use rast_agent::messages::{AgentResponse, C2Request};

/// Task request from UI to C2.
#[derive(Debug, Serialize, Deserialize)]
pub enum UiRequest {
    Ping,
    GetAgents,
    AgentRequest(Ulid, C2Request),
    // GetAgentData(Ulid),
    // ShellRequest(Ulid, String),
    // GetCommands(Ulid),
    // ExecCommand(Ulid, String, Vec<String>),
}

/// Task response from C2 to UI.
#[derive(Debug, Serialize, Deserialize)]
pub enum UiResponse {
    Pong,
    Agents(Vec<Ulid>),
    AgentResponse(AgentResponse),
    // AgentData(AgentData),
    // ShellOutput(String),
    // Commands(Vec<(String, String)>),
    // CommandOutput(String),
}
