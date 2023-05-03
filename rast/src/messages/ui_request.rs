//! Types for C2 <-> UI communication.

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

// TODO: redo this shit

type Ip = SocketAddr;

/// Task request from UI to C2.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum UiRequest {
    Ping,
    GetIps,
    GetIpData(Ip),
    ShellRequest(Ip, String),
    GetCommands(Ip),
    ExecCommand(Ip, String, Vec<String>),
}

/// Task response from C2 to UI.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum UiResponse {
    Pong,
    GetIps(Vec<String>),
    GetIpData(IpData),
    ShellOutput(String),
    Commands(Vec<(String, String)>),
    CommandOutput(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct IpData {
    pub ip: Ip,
}
