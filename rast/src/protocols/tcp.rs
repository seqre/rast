use std::{
    net::IpAddr,
    pin::Pin,
    task::{Context, Poll},
};

use capnp::message::ReaderOptions;
use capnp_futures::serialize::{read_message, write_message};
use futures_io::{AsyncRead, AsyncWrite};
use serde_derive::Deserialize;
use tokio::{
    io::ReadBuf,
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

#[derive(Debug, Deserialize)]
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

/// Absolutely disgusting
impl AsyncWrite for TcpConnection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        tokio::io::AsyncWrite::poll_write(Pin::new(self.project().writer), cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        tokio::io::AsyncWrite::poll_flush(Pin::new(self.project().writer), cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        tokio::io::AsyncWrite::poll_shutdown(Pin::new(self.project().writer), cx)
    }
}

/// Absolutely disgusting
impl AsyncRead for TcpConnection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let readbuf = &mut ReadBuf::new(buf);
        match tokio::io::AsyncRead::poll_read(Pin::new(self.project().reader), cx, readbuf) {
            Poll::Ready(_) => Poll::Ready(std::io::Result::Ok(readbuf.filled().len())),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[async_trait]
impl ProtoConnection for TcpConnection {
    async fn recv(&mut self) -> Result<ConnectionOutput> {
        // self.stream.readable().await?;
        let read = read_message(&mut self, ReaderOptions::new()).await?;
        Ok(read)
    }

    async fn send(&mut self, msg: Message) -> Result<()> {
        let write = write_message(&mut self, msg).await?;
        Ok(write)
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

    async fn new_client(conf: &Self::Conf) -> Result<Arc<ProtoConnectionType>> {
        let address = SocketAddr::new(conf.ip, conf.port);
        let stream = TcpStream::connect(address).await;

        match stream {
            Ok(stream) => Ok(Arc::new(TcpConnection::new(stream))),
            Err(e) => Err(e.into()),
        }
    }
}

#[async_trait]
impl ProtoServer for TcpServer {
    async fn get_conn(&self) -> Result<Arc<ProtoConnectionType>> {
        match self.listener.accept().await {
            Ok((stream, _address)) => Ok(Arc::new(TcpConnection::new(stream))),
            Err(e) => Err(e.into()),
        }
    }
}
