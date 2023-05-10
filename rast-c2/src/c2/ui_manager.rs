//! UI incoming connections handler.

use std::{fmt::Debug, net::SocketAddr, sync::Arc, time::Duration, vec};

use anyhow::{bail, Result};
use bidirectional_channel::{bounded, ReceivedRequest, Requester, Responder};
use futures_util::{sink::SinkExt, stream::StreamExt};
use rast::{
    encoding::{JsonPackager, Packager},
    messages::ui_request::{UiRequest, UiResponse},
    protocols::{
        quic::QuicFactory, tcp::TcpFactory, Messager, ProtoConnection, ProtoFactory, ProtoServer,
    },
    settings::{self, Connection, Settings},
    RastError,
};
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver},
        Mutex,
    },
    task::JoinHandle,
};
use tracing::{debug, info, trace};

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
        let inner = tokio::spawn(async move {
            if let Err(e) = inner.run().await {
                debug!("InnerUiManager exited with error: {:?}", e);
            };
        });

        let ui = UiManager {
            inner_thread: inner,
            requests: rx,
        };

        info!("UiManager instance created");

        Ok(ui)
    }

    // pub async fn message(&self, notification: C2Notification) {
    //    todo!()
    //}

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

        for conf in settings.server.ui_listeners.iter() {
            let server = match conf {
                Connection::Tcp(tcp_conf) => TcpFactory::new_server(tcp_conf).await,
                Connection::Quic(quic_conf) => QuicFactory::new_server(quic_conf).await,
                _ => bail!(RastError::Unknown),
            };

            if let Ok(server) = server {
                debug!("Creating UI listener: {server:?}");
                let cloned = tx.clone();
                let task = tokio::spawn(async move {
                    loop {
                        if let Ok(conn) = server.get_conn().await {
                            info!(
                                "UI Server got connection from: {:?}",
                                conn.lock().await.remote_addr()
                            );
                            match cloned.send(conn) {
                                Ok(_) => debug!("UI connection sent"),
                                Err(e) => debug!("Failed to send UI connection: {:?}", e),
                            };
                        }
                    }
                });
                servers.push(task);
            }
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

    async fn run(&mut self) -> Result<()> {
        loop {
            while let Ok(conn) = self.connections_rx.try_recv() {
                info!("UI received connection");
                self.add_connection(conn).await?;
                debug!("UI added connection");
            }
        }
    }

    async fn add_connection(&mut self, conn: Arc<Mutex<dyn ProtoConnection>>) -> Result<()> {
        let ip = conn.lock().await.remote_addr()?;
        let requester = self.requester.clone();
        let task = tokio::spawn(async move {
            let mut conn = conn.lock().await;
            let mut messager = Messager::new(&mut *conn);
            let packager = JsonPackager::default();
            loop {
                if let Some(msg) = messager.next().await {
                    let msg: UiRequest = packager.decode(&msg.unwrap().into()).unwrap();
                    trace!("Request: {:?}", msg);
                    let response = requester.send(msg).await;
                    trace!("Response: {:?}", response);
                    let response = response.unwrap();
                    let response = packager.encode(&response).unwrap();
                    if let Err(e) = messager.send(response).await {
                        debug!("Failed to send UI response: {:?}", e);
                    };
                }
            }
        });
        let task = task.await?;
        self.connections.push((ip, task));
        Ok(())
    }
}
