use std::{io::Result, net::SocketAddr};

use async_trait::async_trait;

use crate::Msg;

pub mod tcp;

#[async_trait]
pub trait ProtoConnection {
    async fn send(&mut self, msg: Msg) -> Result<()>;
    async fn recv(&mut self) -> Result<Msg>;
}

#[async_trait]
pub trait ProtoClient: ProtoConnection {
    type Conf;

    async fn new_client(server_address: SocketAddr, conf: Option<Self::Conf>) -> Result<Self>
    where
        Self: Sized;
}

#[async_trait]
pub trait ProtoServer {
    type Conf;

    async fn new_server(listening_address: SocketAddr, conf: Option<Self::Conf>) -> Result<Self>
    where
        Self: Sized;

    async fn get_conn(&self) -> Result<Box<dyn ProtoConnection>>;
}
