//! UI incoming connections handler.

use std::{fmt::Debug, net::SocketAddr, ops::DerefMut, sync::Arc, vec};

use anyhow::Result;
use bidirectional_channel::{bounded, ReceivedRequest, Requester, Responder};
use bytes::Bytes;
use futures_util::{sink::SinkExt, stream::StreamExt};
use rast::{
    messages::ui_request::*,
    protocols::{tcp::TcpFactory, *},
    settings,
};
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver},
        Mutex,
    },
    task::JoinHandle,
};
use tokio_util::codec::BytesCodec;
use tracing::info;

/// Manager of the UI connections.
#[derive(Debug)]
pub struct UiManager {
    inner_thread: JoinHandle<()>,
    requests: Responder<ReceivedRequest<UiRequest, UiResponse>>,
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

impl UiManager {
    /// Creates new instance using provided [UI settings](settings::Ui)
    pub async fn with_settings(conf: &settings::Ui) -> Result<Self> {
        let (tx, rx) = bounded(100);
        let mut inner = InnerUiManager::with_settings(conf, tx).await?;
        let inner = tokio::spawn(async move {
            inner.run().await;
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

impl InnerUiManager {
    async fn with_settings(
        conf: &settings::Ui,
        requester: Requester<UiRequest, UiResponse>,
    ) -> Result<Self> {
        let (tx, rx) = unbounded_channel();
        let mut servers = vec![];

        if let Some(conf) = &conf.tcp {
            let server = TcpFactory::new_server(conf).await?;
            let cloned = tx.clone();
            let task = tokio::spawn(async move {
                loop {
                    if let Ok(conn) = server.get_conn().await {
                        info!("Ui Server got connection");
                        cloned.send(conn);
                        info!("Ui connection sent")
                    }
                }
            });
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

    async fn run(&mut self) -> Result<()> {
        loop {
            while let Ok(conn) = self.connections_rx.try_recv() {
                info!("UI received connection");
                self.add_connection(conn).await?;
                info!("UI added connection");
            }
        }
    }

    async fn add_connection(&mut self, conn: Arc<Mutex<dyn ProtoConnection>>) -> Result<()> {
        let ip = conn.lock().await.get_ip()?;
        let requester = self.requester.clone();
        // info!("pre task");
        let task = tokio::spawn(async move {
            // info!("pre lock");
            let mut conn = conn.lock().await;
            // info!("post lock");
            let mut frame = get_rw_frame(conn.deref_mut(), BytesCodec::new());

            loop {
                if let Some(msg) = frame.next().await {
                    let msg: UiRequest = serde_json::from_slice(&msg.unwrap()).unwrap();
                    info!("Request: {:?}", msg);
                    let response = requester.send(msg).await;
                    info!("Response: {:?}", response);
                    let response = response.unwrap();
                    let response = serde_json::to_vec(&response).unwrap();
                    frame.send(Bytes::from(response)).await;
                }
            }
        });
        // info!("post task");
        self.connections.push((ip, task));
        // info!("{:?}", self.connections);
        todo!();
        Ok(())
    }
}
