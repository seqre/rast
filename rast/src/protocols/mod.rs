use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use capnp::{message::Reader, serialize::OwnedSegments};

pub mod tcp;
// pub mod websocket;

pub type Message = ::capnp::message::Builder<::capnp::message::HeapAllocator>;
pub type ConnectionOutput = Reader<OwnedSegments>;
pub type ProtoConnectionType = dyn ProtoConnection + Send + Sync;

#[async_trait]
pub trait ProtoConnection {
    async fn send(&mut self, msg: Message) -> Result<()>;
    async fn recv(&mut self) -> Result<ConnectionOutput>;

    fn get_ip(&self) -> Result<SocketAddr>;
}

#[async_trait]
pub trait ProtoServer: Send + Sync {
    async fn get_conn(&self) -> Result<Arc<ProtoConnectionType>>;
}

#[async_trait]
pub trait ProtoFactory {
    type Conf;

    async fn new_server(conf: &Self::Conf) -> Result<Arc<dyn ProtoServer>>;
    async fn new_client(conf: &Self::Conf) -> Result<Arc<ProtoConnectionType>>;
}
