//! Configuration for binaries.

use std::env;

use config::{Config, ConfigError, Environment, File};
use glob::glob;
use serde::Deserialize;

use crate::protocols::{quic::QuicConf, tcp::TcpConf};

//#[derive(Debug, Deserialize, Clone)]
//#[allow(unused)]
// pub struct Dummy {}

/// UI-related configuration values.
#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Ui {
    pub tcp: Option<TcpConf>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
#[serde(rename_all = "lowercase")]
pub enum Connection {
    Tcp(TcpConf),
    Quic(QuicConf),
}

/// C2 server-related configuration values.
#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Server {
    pub ui_listeners: Vec<Connection>,
    pub agent_listeners: Vec<Connection>,
}

/// Agent-related configuration values.
#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Agent {
    pub connections: Vec<Connection>,
}

/// General configuration values.
#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Settings {
    pub server: Server,
    pub agent: Agent,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mode = env::var("RAST_RUN_MODE").unwrap_or_else(|_| "dev".into());

        let conf = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{mode}")))
            .add_source(
                glob("config/custom/*")
                    .unwrap()
                    .map(|path| File::from(path.unwrap()))
                    .collect::<Vec<_>>(),
            )
            .add_source(Environment::with_prefix("RAST"))
            .build()?;

        conf.try_deserialize()
    }
}
