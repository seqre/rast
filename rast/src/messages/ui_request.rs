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
    Command(Ip, String),
}

/// Task response from C2 to UI.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum UiResponse {
    Pong,
    GetIps(Vec<String>),
    GetIpData(IpData),
    Command(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct IpData {
    pub ip: Ip,
}
