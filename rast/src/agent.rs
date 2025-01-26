use std::{net::SocketAddr, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use ulid::Ulid;

use crate::{
    encoding::{JsonPackager, Packager},
    messages::{AgentInit, Message},
    protocols::{Messager, ProtoConnection},
    RastError, Result,
};

#[derive(Debug)]
pub struct Agent {
    ulid: Ulid,
    connection: Option<Arc<Mutex<dyn ProtoConnection>>>,
}

impl Agent {
    pub async fn with_connection(connection: Arc<Mutex<dyn ProtoConnection>>) -> Result<Agent> {
        let ulid = {
            let mut conn = connection.lock().await;
            let mut messager = Messager::with_packager(&mut *conn, JsonPackager);
            let msg: Message = messager.receive().await?;
            let decoded: AgentInit = JsonPackager::decode(&msg.data)?;
            decoded.ulid
        };

        let agent = Agent {
            ulid,
            connection: Some(connection),
        };

        Ok(agent)
    }

    pub fn get_ulid(&self) -> Ulid {
        self.ulid
    }

    pub fn get_connection(&self) -> Option<Arc<Mutex<dyn ProtoConnection>>> {
        self.connection.clone()
    }

    pub async fn get_details(&self) -> Result<AgentData> {
        let conn = self
            .connection
            .as_ref()
            .ok_or(RastError::TODO("Missing agent connection".to_string()))?
            .lock()
            .await;
        let data = AgentData {
            ulid: self.ulid,
            local_ip: conn.local_addr().ok(),
        };
        Ok(data)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AgentData {
    pub ulid: Ulid,
    pub local_ip: Option<SocketAddr>,
}
