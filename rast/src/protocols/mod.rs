//! Implementations of [ProtoConnection] for specific protocols.

use std::{fmt::Debug, net::SocketAddr, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::sink::SinkExt;
use serde::Serialize;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};
use tokio_util::codec::{BytesCodec, Framed};

pub mod tcp;
// pub mod websocket;

#[async_trait]
pub trait ProtoConnection: Send + Sync + Unpin + AsyncRead + AsyncWrite {
    fn get_ip(&self) -> Result<SocketAddr>;
}

impl Debug for dyn ProtoConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProtoConnection")
            .field("ip", &self.get_ip())
            .finish()
    }
}

#[async_trait]
pub trait ProtoServer: Send + Sync {
    async fn get_conn(&self) -> Result<Arc<Mutex<dyn ProtoConnection>>>;
}

#[async_trait]
pub trait ProtoFactory {
    type Conf;

    async fn new_server(conf: &Self::Conf) -> Result<Arc<dyn ProtoServer>>;
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

    pub async fn send(&mut self, msg: impl Serialize) -> Result<()> {
        let request = serde_json::to_vec(&msg)?;
        self.frame.send(Bytes::from(request)).await?;
        Ok(())
    }

    // pub async fn recv<'a, M: Deserialize<'a>>(&mut self) -> Result<M> {
    //    let bytes = self.frame.next().await.unwrap().unwrap();
    //    Ok(serde_json::from_slice(&bytes.clone)?)
    //}
}

pub fn get_rw_frame<S, C>(stream: S, codec: C) -> Framed<S, C>
where
    S: AsyncWrite + AsyncRead + Unpin,
{
    Framed::new(stream, codec)
}

//     let conn = self.connections.get(ip).unwrap();
//     let mut conn = conn.lock().await;
//
//    // TODO: put all of that into struct and do abstractions
//     let mut frame = get_rw_frame(conn.deref_mut(), BytesCodec::new());
//
//     let request =
//     AgentMessage::C2Request(C2Request::ExecCommand(cmd.to_string())); let
// request     = serde_json::to_vec(&request)?;
//
//     frame.send(Bytes::from(request)).await?;
//     let bytes = frame.next().await.unwrap().unwrap();
//
//     let output = serde_json::from_slice(&bytes)?;
//     let AgentResponse::CommandResponse(output) = output;
//
