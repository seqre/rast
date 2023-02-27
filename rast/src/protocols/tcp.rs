//! TCP implementation of [ProtoConnection].
use std::{
    net::IpAddr,
    pin::Pin,
    task::{Context, Poll},
};

use serde::Deserialize;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
};

use crate::protocols::*;

pub struct TcpFactory {}

pub struct TcpServer {
    listener: TcpListener,
}

#[pin_project::pin_project]
pub struct TcpConnection {
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct TcpConf {
    pub ip: IpAddr,
    pub port: u16,
}

impl TcpConnection {
    pub fn new(stream: TcpStream) -> Self {
        let (reader, writer) = stream.into_split();
        TcpConnection { reader, writer }
    }
}

#[async_trait]
impl ProtoConnection for TcpConnection {
    fn get_ip(&self) -> Result<SocketAddr> {
        let ip = self.reader.peer_addr()?;
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
        Pin::new(self.project().reader).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpConnection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(self.project().writer).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Pin::new(self.project().writer).poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(self.project().writer).poll_shutdown(cx)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[futures_io::IoSlice<'_>],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(self.project().writer).poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.writer.is_write_vectored()
    }
}
