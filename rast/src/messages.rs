//! Types used for inter-binaries communication.

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::encoding::Encoding;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInit {
    pub ulid: Ulid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageZone {
    Internal,
    External(Ulid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Ulid,
    pub response_to: Option<Ulid>,
    pub zone: MessageZone,
    pub encoding: Encoding,
    pub data: Vec<u8>,
}

impl Message {
    #[must_use]
    pub fn new(zone: MessageZone, encoding: Encoding, data: Vec<u8>) -> Self {
        Self {
            id: Ulid::new(),
            response_to: None,
            zone,
            encoding,
            data,
        }
    }

    #[must_use]
    pub fn respond(&self, data: Vec<u8>) -> Self {
        Self {
            id: Ulid::new(),
            response_to: Some(self.id),
            zone: self.zone.clone(),
            encoding: self.encoding,
            data,
        }
    }
}
