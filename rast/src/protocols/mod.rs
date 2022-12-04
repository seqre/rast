use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use async_trait::async_trait;
use capnp::{message::Reader, serialize::OwnedSegments};

use crate::capnp_utils::{CapnpRead, CapnpWrite};

pub mod tcp;
// pub mod websocket;

#[async_trait]
pub trait ProtoConnection<'a, B, R> {
    async fn send<M: CapnpWrite<'a, Builder = B>>(&mut self, msg: M) -> Result<()>;
    async fn recv<M: CapnpRead<'a, Reader = R>>(&mut self) -> Result<M>;
    async fn try_recv<M: CapnpRead<'a, Reader = R>>(&mut self) -> Result<Option<M>>;

    fn get_ip(&self) -> Result<SocketAddr>;
}

#[async_trait]
pub trait ProtoServer<B, R>: Send + Sync {
    async fn get_conn(&self) -> Result<Arc<Mutex<dyn ProtoConnection<B, R>>>>;
}

#[async_trait]
pub trait ProtoFactory<B, R> {
    type Conf;

    async fn new_server(conf: &Self::Conf) -> Result<Arc<dyn ProtoServer<B, R>>>;
    async fn new_client(conf: &Self::Conf) -> Result<Arc<Mutex<dyn ProtoConnection<B, R>>>>;
}
