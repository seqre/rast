//! Types for C2 <-> UI communication.

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

type Ip = SocketAddr;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum UiRequest {
    Ping,
    GetIps,
    GetIpData(Ip),
    Command(Ip, String),
}

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
