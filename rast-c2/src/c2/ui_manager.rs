//! UI incoming connections' handler.

use std::{fmt::Debug, net::SocketAddr, sync::Arc, vec};

use anyhow::{bail, Result};
use bidirectional_channel::{bounded, ReceivedRequest, Requester, Responder};
use rast::{
    encoding::{JsonPackager, Packager},
    messages::Message,
    protocols::{
        quic::QuicFactory, tcp::TcpFactory, Messager, ProtoConnection, ProtoFactory,
    },
    settings::{Connection, Settings},
    RastError,
};
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
    task::JoinHandle,
};
use tracing::{debug, info};

use crate::messages::{UiRequest, UiResponse};

/// Manager of the UI connections.
#[derive(Debug)]
pub struct UiManager {
    inner_thread: JoinHandle<()>,
    requests: Responder<ReceivedRequest<UiRequest, UiResponse>>,
}

impl UiManager {
    /// Creates new instance using provided [UI settings](settings::Ui)
    pub async fn with_settings(settings: &Settings) -> Result<Self> {
        let (tx, rx) = bounded(100);
        let mut inner = InnerUiManager::with_settings(settings, tx).await?;
        let inner = tokio::spawn(async move { inner.run().await });

        let ui = UiManager {
            inner_thread: inner,
            requests: rx,
        };

        info!("UiManager instance created");

        Ok(ui)
    }

    /// Returns an attempt of getting [UI request](UiRequest).
    #[tracing::instrument]
    pub fn try_recv_request(&self) -> Result<ReceivedRequest<UiRequest, UiResponse>> {
        match self.requests.try_recv() {
            Ok(req) => Ok(req),
            Err(e) => Err(e.into()),
        }
    }
}

struct InnerUiManager {
    requester: Arc<Requester<UiRequest, UiResponse>>,
    servers: Vec<JoinHandle<()>>,
    connections: Vec<(SocketAddr, JoinHandle<()>)>,
    connections_rx: UnboundedReceiver<Arc<Mutex<dyn ProtoConnection>>>,
}

impl Debug for InnerUiManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerUiManager")
            .field("servers", &self.servers)
            .field("connections", &self.connections)
            .finish()
    }
}

impl InnerUiManager {
    async fn with_settings(
        settings: &Settings,
        requester: Requester<UiRequest, UiResponse>,
    ) -> Result<Self> {
        let (tx, rx) = unbounded_channel();
        let mut servers = vec![];

        for conf in &settings.server.ui_listeners {
            let task = InnerUiManager::create_listener(conf, tx.clone()).await?;
            servers.push(task);
        }

        let ui = InnerUiManager {
            requester: Arc::new(requester),
            servers,
            connections: vec![],
            connections_rx: rx,
        };

        info!("InnerUiManager instance created");

        Ok(ui)
    }

    async fn create_listener(
        conf: &Connection,
        tx: UnboundedSender<Arc<Mutex<dyn ProtoConnection>>>,
    ) -> Result<JoinHandle<()>> {
        let server = match conf {
            Connection::Tcp(tcp_conf) => TcpFactory::new_server(tcp_conf).await,
            Connection::Quic(quic_conf) => QuicFactory::new_server(quic_conf).await,
        };

        if let Ok(server) = server {
            debug!("Creating UI listener: {server:?}");
            let task = tokio::spawn(async move {
                loop {
                    if let Ok(conn) = server.get_conn().await {
                        info!(
                            "UI Server got connection from: {:?}",
                            conn.lock().await.remote_addr()
                        );
                        match tx.send(conn) {
                            Ok(_) => debug!("UI connection sent"),
                            Err(e) => debug!("Failed to send UI connection: {:?}", e),
                        };
                    }
                }
            });
            Ok(task)
        } else {
            bail!(RastError::Unknown)
        }
    }

    async fn run(&mut self) {
        loop {
            while let Some(conn) = self.connections_rx.recv().await {
                info!("UI received connection");
                if self.add_connection(conn).await.is_ok() {
                    debug!("UI added connection");
                }
            }
        }
    }

    async fn add_connection(&mut self, conn: Arc<Mutex<dyn ProtoConnection>>) -> Result<()> {
        let ip = conn.lock().await.remote_addr()?;
        let requester = self.requester.clone();
        let task = tokio::spawn(async move {
            let mut conn = conn.lock().await;
            let mut messager = Messager::with_packager(&mut *conn, JsonPackager);
            loop {
                if let Ok(msg) = messager.receive().await {
                    let msg: Message = msg;

                    if let Ok(uireq) = JsonPackager::decode(&msg.data) {
                        match requester.send(uireq).await {
                            Ok(response) => {
                                let bytes = JsonPackager::encode(&response).unwrap();
                                let response = msg.respond(bytes.into());
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
        });
        self.connections.push((ip, task));
        Ok(())
    }
}

// trace!("Request: {:?}", msg);
// let response = requester.send(msg).await;
// trace!("Response: {:?}", response);
// let response = response.unwrap();
// if let Err(e) = messager.send(&response).await {
// debug!("Failed to send UI response: {:?}", e);
// };
