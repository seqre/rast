//! TCP implementation of [`ProtoConnection`].
use std::{
    net::{IpAddr, Ipv4Addr},
    pin::{pin, Pin},
    task::{Context, Poll},
};

use quinn::{
    crypto::ServerConfig as CryptoServerConfig, Connection, Endpoint, RecvStream, SendStream,
    ServerConfig,
};
use rustls::server::ServerConfig as RustlsServerConfig;
use serde::Deserialize;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use super::ProtoConnectionType;
use crate::{
    protocols::{
        async_trait, Arc, Debug, Mutex, ProtoConnection, ProtoFactory, ProtoServer, Result,
        SocketAddr,
    },
    RastError,
};

/// Creates [`ProtoServer`] and [`ProtoConnection`] for TCP communication.
pub struct QuicFactory {}

impl QuicFactory {
    fn get_server_config(conf: &QuicConf) -> Result<ServerConfig> {
        let subject_alt_names = vec!["localhost".into(), conf.server_name.to_string()];
        let cert = rcgen::generate_simple_self_signed(subject_alt_names).unwrap();
        let key = cert.serialize_private_key_der();
        let cert = cert.serialize_der()?;

        let key = rustls::PrivateKey(key);
        let cert = vec![rustls::Certificate(cert)];

        let rustls_crypto_config: RustlsServerConfig = RustlsServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert, key)?;
        let rustls_crypto_config = Arc::new(rustls_crypto_config);

        Ok(ServerConfig::with_crypto(rustls_crypto_config))
    }
}

struct QuicServer {
    endpoint: Endpoint,
}

impl QuicServer {
    pub fn new(endpoint: Endpoint) -> Self {
        QuicServer { endpoint }
    }
}

#[async_trait]
impl ProtoServer for QuicServer {
    async fn get_conn(&self) -> Result<Arc<Mutex<dyn ProtoConnection>>> {
        if let Some(conn) = self.endpoint.accept().await {
            let conn = conn.await?;
            let (send, recv) = conn.accept_bi().await?;

            Ok(Arc::new(Mutex::new(QuicConnection::new(conn, recv, send))))
        } else {
            Err(RastError::Unknown)
        }
    }
}

#[pin_project::pin_project]
struct QuicConnection {
    connection: Connection,
    recv: RecvStream,
    send: SendStream,
}

impl QuicConnection {
    pub fn new(connection: Connection, recv: RecvStream, send: SendStream) -> Self {
        QuicConnection {
            connection,
            recv,
            send,
        }
    }
}

/// TCP connection related configuration values.
#[derive(Debug, Deserialize, Clone)]
pub struct QuicConf {
    pub ip: IpAddr,
    pub port: u16,
    pub server_name: String,
}

#[async_trait]
impl ProtoConnection for QuicConnection {
    fn get_type(&self) -> ProtoConnectionType {
        ProtoConnectionType::TwoWay
    }

    fn local_addr(&self) -> Result<SocketAddr> {
        let ip = self.connection.local_ip().ok_or(RastError::Unknown)?;
        let ip = SocketAddr::new(ip, 0);
        Ok(ip)
    }

    fn remote_addr(&self) -> Result<SocketAddr> {
        let ip = self.connection.remote_address();
        Ok(ip)
    }
}

#[async_trait]
impl ProtoFactory for QuicFactory {
    type Conf = QuicConf;

    async fn new_server(conf: &Self::Conf) -> Result<Arc<dyn ProtoServer>> {
        let config = QuicFactory::get_server_config(&conf)?;
        let server = SocketAddr::new(conf.ip, conf.port);
        let endpoint = Endpoint::server(config, server)?;

        Ok(Arc::new(QuicServer::new(endpoint)))
    }

    async fn new_client(conf: &Self::Conf) -> Result<Arc<Mutex<dyn ProtoConnection>>> {
        let local = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
        let endpoint = Endpoint::client(local)?;
        let server = SocketAddr::new(conf.ip, conf.port);
        let conn = endpoint.connect(server, &conf.server_name)?;
        let conn = conn.await?;
        let (send, recv) = conn.open_bi().await?;

        Ok(Arc::new(Mutex::new(QuicConnection::new(conn, recv, send))))
    }
}

impl AsyncRead for QuicConnection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        pin!(self.project().recv).poll_read(cx, buf)
    }
}

impl AsyncWrite for QuicConnection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        pin!(self.project().send).poll_write(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        pin!(self.project().send).poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        pin!(self.project().send).poll_shutdown(cx)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[futures_io::IoSlice<'_>],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        pin!(self.project().send).poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.send.is_write_vectored()
    }
}
