//! Configuration for binaries.

use config::{Config, ConfigError, Environment, File};
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

#[cfg(not(debug_assertions))]
static CONFIG: &'static str = include_str!("../../config/default.yaml");

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let conf = Config::builder();

        #[cfg(debug_assertions)]
        let conf = conf.add_source(File::with_name("config/default"));

        #[cfg(not(debug_assertions))]
        let conf = conf.add_source(File::from_str(CONFIG, config::FileFormat::Yaml));

        let conf = conf.add_source(Environment::with_prefix("RAST")).build()?;

        conf.try_deserialize()
    }
}
