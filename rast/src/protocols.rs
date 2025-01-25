//! Implementations of [`ProtoConnection`] for specific protocols.

use std::{
    fmt::Debug,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};
use tokio_util::codec::{BytesCodec, Framed};

use crate::{encoding::Packager, Result};

pub mod quic;
pub mod tcp;
// pub mod websocket;

#[derive(thiserror::Error, Debug)]
pub enum NetworkError {
    #[error("Generic network error: {0}")]
    Generic(String),

    #[error("New Quic connection error: {0}")]
    QuicNewConnection(#[from] quinn::ConnectError),

    #[error("Existing Quic connection error: {0}")]
    QuicExistingConnection(#[from] quinn::ConnectionError),
}

/// Established connection over any network protocol.
#[async_trait]
pub trait ProtoConnection: Debug + Send + Sync + Unpin + AsyncRead + AsyncWrite {
    /// Gets IP of the remote agent.
    fn local_addr(&self) -> Result<SocketAddr>;
    fn remote_addr(&self) -> Result<SocketAddr>;
}

// impl Debug for dyn ProtoConnection {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("ProtoConnection")
//             .field("local_addr", &self.local_addr())
//             .field("remote_addr", &self.remote_addr())
//             .finish()
//     }
// }

/// Server-side of any network protocol.
#[async_trait]
pub trait ProtoServer: Debug + Send + Sync {
    /// Waits for a new connection from the agent.
    async fn get_conn(&self) -> Result<Arc<Mutex<dyn ProtoConnection>>>;
}

/// Creates new [ProtoConnection] and [ProtoServer] for network
/// protocol.
#[async_trait]
pub trait ProtoFactory: Debug {
    type Conf;

    /// Returns new server for network protocol.
    async fn new_server(conf: &Self::Conf) -> Result<Arc<dyn ProtoServer>>;

    /// Returns new connection for network protocol.
    async fn new_connection(conf: &Self::Conf) -> Result<Arc<Mutex<dyn ProtoConnection>>>;
}

pub struct Messager<S, P, M>
where
    S: AsyncRead + AsyncWrite,
    P: Packager,
    M: Serialize + DeserializeOwned,
{
    frame: Framed<S, BytesCodec>,
    packager: P,
    _marker: std::marker::PhantomData<M>,
}

impl<S, P, M> Messager<S, P, M>
where
    S: AsyncRead + AsyncWrite + Unpin,
    P: Packager + Default,
    M: Serialize + DeserializeOwned,
{
    pub fn new(stream: S) -> Self {
        Messager {
            frame: Framed::new(stream, BytesCodec::new()),
            packager: P::default(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S, P, M> Messager<S, P, M>
where
    S: AsyncRead + AsyncWrite + Unpin,
    P: Packager,
    M: Serialize + DeserializeOwned,
{
    pub fn with_packager(stream: S, packager: P) -> Self {
        Messager {
            frame: Framed::new(stream, BytesCodec::new()),
            packager,
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn send(&mut self, msg: &M) -> Result<()> {
        let request = P::encode(msg)?;
        futures_util::SinkExt::send(&mut self.frame, request).await?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<M> {
        let bytes = futures_util::StreamExt::next(&mut self.frame)
            .await
            .ok_or_else(|| NetworkError::Generic("No data received".to_string()))??;
        let response: M = P::decode(&bytes)?;
        Ok(response)
    }
}

impl<S, P, M> Deref for Messager<S, P, M>
where
    S: AsyncRead + AsyncWrite + Unpin,
    P: Packager,
    M: Serialize + DeserializeOwned,
{
    type Target = Framed<S, BytesCodec>;

    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl<S, P, M> DerefMut for Messager<S, P, M>
where
    S: AsyncRead + AsyncWrite + Unpin,
    P: Packager,
    M: Serialize + DeserializeOwned,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.frame
    }
}
