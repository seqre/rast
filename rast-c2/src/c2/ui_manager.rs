use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    vec,
};

use anyhow::Result;
use bidirectional_channel::Requester;
use rast::{
    messages::ui_request::*,
    protocols::{tcp::TcpFactory, *},
    settings,
};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
    task::JoinHandle,
};

use crate::c2::{C2Notification, Dummy};

pub struct UiManager {
    c2_tx: Requester<UiRequest, UiResponse>,
    servers: Vec<JoinHandle<()>>,
    connections: Vec<(SocketAddr, Arc<Mutex<ProtoConnectionType>>)>,
    connections_rx: UnboundedReceiver<Arc<Mutex<ProtoConnectionType>>>,
}

impl UiManager {
    pub async fn with_settings(
        conf: &settings::Ui,
        c2_tx: Requester<UiRequest, UiResponse>,
    ) -> Result<Self> {
        let (tx, rx) = unbounded_channel();
        let mut servers = vec![];

        if let Some(conf) = &conf.tcp {
            let server = TcpFactory::new_server(conf).await?;
            let cloned = tx.clone();
            let task = tokio::spawn(async move {
                loop {
                    if let Ok(conn) = server.get_conn().await {
                        cloned.send(conn);
                    }
                }
            });
            servers.push(task);
        }

        let ui = UiManager {
            c2_tx,
            servers,
            connections: vec![],
            connections_rx: rx,
        };

        Ok(ui)
    }

    pub async fn start(&mut self) -> Result<()> {
        loop {
            while let Ok(conn) = self.connections_rx.try_recv() {
                self.add_connection(conn).await?;
            }

            for (ip, conn) in &self.connections {
                if let Ok(Some(msg)) = conn.lock().unwrap().try_recv().await {
                    // do something
                }
                // self.handle_connection_message(conn).await?;
                // cannot borrow self as mutable
            }
        }
    }

    pub async fn message(&self, notification: C2Notification) {}

    async fn add_connection(&mut self, conn: Arc<Mutex<ProtoConnectionType>>) -> Result<()> {
        let ip = conn.lock().unwrap().get_ip()?;
        self.connections.push((ip, conn));
        Ok(())
    }

    // async fn handle_connection_message(
    //    &mut self,
    //    conn: &Arc<Mutex<ProtoConnectionType>>,
    //) -> Result<()> {
    //    let dupa = conn.lock().unwrap().try_recv().await?;
    //    // let dupa = conn.try_recv().await?;
    //    Ok(())
    //}
}
