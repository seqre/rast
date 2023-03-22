//! Implementations of [ProtoConnection] for specific protocols.

use std::{
    fmt::Debug,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use anyhow::anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::sink::SinkExt;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};
use tokio_util::codec::{BytesCodec, Framed};

use crate::{RastError, Result};

pub mod tcp;
// pub mod websocket;

/// Established connection over any network protocol.
#[async_trait]
pub trait ProtoConnection: Send + Sync + Unpin + AsyncRead + AsyncWrite {
    /// Gets IP of the remote agent.
    fn get_ip(&self) -> Result<SocketAddr>;
}

impl Debug for dyn ProtoConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProtoConnection")
            .field("ip", &self.get_ip())
            .finish()
    }
}

/// Server-side of any network protocol.
#[async_trait]
pub trait ProtoServer: Send + Sync {
    /// Waits for a new connection from the agent.
    async fn get_conn(&self) -> Result<Arc<Mutex<dyn ProtoConnection>>>;
}

/// Creates new [ProtoConnection] and [ProtoServer] for network
/// protocol.
#[async_trait]
pub trait ProtoFactory {
    type Conf;

    /// Returns new server for network protocol.
    async fn new_server(conf: &Self::Conf) -> Result<Arc<dyn ProtoServer>>;

    /// Returns new connection for network protocol.
    async fn new_client(conf: &Self::Conf) -> Result<Arc<Mutex<dyn ProtoConnection>>>;
}

pub struct Messager<S>
where
    S: AsyncRead + AsyncWrite,
{
    frame: Framed<S, BytesCodec>,
}

impl<S> Messager<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: S) -> Self {
        Messager {
            frame: Framed::new(stream, BytesCodec::new()),
        }
    }
}

impl<S> Deref for Messager<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    type Target = Framed<S, BytesCodec>;

    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl<S> DerefMut for Messager<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.frame
    }
}
