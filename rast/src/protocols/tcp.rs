//! TCP implementation of [`ProtoConnection`].
use std::{
    net::IpAddr,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use serde::Deserialize;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpListener, TcpStream},
};

use crate::protocols::{
    async_trait, Arc, Debug, Mutex, ProtoConnection, ProtoFactory, ProtoServer, Result, SocketAddr,
};

/// Creates [`ProtoServer`] and [`ProtoConnection`] for TCP communication.
pub struct TcpFactory {}

struct TcpServer {
    listener: TcpListener,
}

#[pin_project::pin_project]
struct TcpConnection {
    stream: TcpStream,
}

/// TCP connection related configuration values.
#[derive(Debug, Deserialize, Copy, Clone)]
pub struct TcpConf {
    pub ip: IpAddr,
    pub port: u16,
}

impl TcpConnection {
    pub fn new(stream: TcpStream) -> Self {
        TcpConnection { stream }
    }
}

#[async_trait]
impl ProtoConnection for TcpConnection {
    fn local_addr(&self) -> Result<SocketAddr> {
        let ip = self.stream.local_addr()?;
        Ok(ip)
    }

    fn remote_addr(&self) -> Result<SocketAddr> {
        let ip = self.stream.peer_addr()?;
        Ok(ip)
    }
}

#[async_trait]
impl ProtoFactory for TcpFactory {
    type Conf = TcpConf;

    async fn new_server(conf: &Self::Conf) -> Result<Arc<dyn ProtoServer>> {
        let address = SocketAddr::new(conf.ip, conf.port);
        let listener = TcpListener::bind(address).await;

        match listener {
            Ok(listener) => Ok(Arc::new(TcpServer { listener })),
            Err(e) => Err(e.into()),
        }
    }

    async fn new_client(conf: &Self::Conf) -> Result<Arc<Mutex<dyn ProtoConnection>>> {
        let address = SocketAddr::new(conf.ip, conf.port);
        let stream = TcpStream::connect(address).await;

        match stream {
            Ok(stream) => Ok(Arc::new(Mutex::new(TcpConnection::new(stream)))),
            Err(e) => Err(e.into()),
        }
    }
}

#[async_trait]
impl ProtoServer for TcpServer {
    async fn get_conn(&self) -> Result<Arc<Mutex<dyn ProtoConnection>>> {
        match self.listener.accept().await {
            Ok((stream, _address)) => Ok(Arc::new(Mutex::new(TcpConnection::new(stream)))),
            Err(e) => Err(e.into()),
        }
    }
}

impl AsyncRead for TcpConnection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(self.project().stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpConnection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        pin!(self.project().stream).poll_write(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        pin!(self.project().stream).poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        pin!(self.project().stream).poll_shutdown(cx)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[futures_io::IoSlice<'_>],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        pin!(self.project().stream).poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.stream.is_write_vectored()
    }
}
