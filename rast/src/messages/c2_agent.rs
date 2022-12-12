use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    C2Request(C2Request),
    AgentResponse(AgentResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum C2Request {
    ExecCommand(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentResponse {
    CommandResponse(String),
}
