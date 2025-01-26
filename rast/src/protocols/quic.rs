//! TCP implementation of [`ProtoConnection`].
use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
    pin::{pin, Pin},
    task::{Context, Poll},
    time::Duration,
};
#[cfg(feature = "embed-cert")]
use std::ops::Deref;
#[cfg(feature = "embed-cert")]
use include_flate::flate;
use quinn::{ClientConfig, Connection, Endpoint, RecvStream, SendStream, ServerConfig};
use rcgen::{CertifiedKey, PublicKeyData};
use rustls::pki_types::pem::PemObject;
use serde::Deserialize;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tracing::{debug, info, trace};

use crate::{
    protocols::{
        async_trait, Arc, Debug, Mutex, NetworkError, ProtoConnection, ProtoFactory, ProtoServer,
        Result, SocketAddr,
    },
    RastError,
};

// const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-29"];

#[cfg(feature = "embed-cert")]
flate!(static CERT: [u8] from "../cert.der");

/// Creates [`ProtoServer`] and [`ProtoConnection`] for TCP communication.
#[derive(Debug)]
pub struct QuicFactory {}

impl QuicFactory {
    fn get_server_config(conf: &QuicConf) -> Result<ServerConfig> {
        let cwd = std::env::current_dir()?;
        let cert_path = cwd.join("cert.der");
        let key_path = cwd.join("key.der");

        let subject_alt_names = vec!["localhost".into(), conf.server_name.to_string()];

        let CertifiedKey { cert, key_pair } =
            rcgen::generate_simple_self_signed(subject_alt_names)?;

        fs::write(&cert_path, cert.der())?;
        info!("Created cert at {cert_path:?}");
        fs::write(&key_path, key_pair.der_bytes())?;
        info!("Created key at {key_path:?}");

        let key =
            rustls::pki_types::PrivateKeyDer::from_pem_slice(key_pair.serialize_pem().as_ref())
                .map_err(|e| RastError::TODO(e.to_string()))?;
        let certs = vec![rustls::pki_types::CertificateDer::from(cert)];

        // let mut server_config = ServerConfig::with_crypto(Arc::new(server_crypto));
        let server_config = ServerConfig::with_single_cert(certs, key)?;
        
        Ok(server_config)
    }

    fn get_client_config(conf: &QuicConf) -> Result<ClientConfig> {
        #[cfg(not(feature = "embed-cert"))]
        let cert = {
            let cwd = std::env::current_dir()?;
            let cert_path = cwd.join("cert.der");
            info!("Loading cert from {cert_path:?}");

            fs::read(&cert_path)?
        };

        #[cfg(feature = "embed-cert")]
        let cert: Vec<u8> = CERT.deref().clone();

        let cert = rustls::pki_types::CertificateDer::from(cert);

        let mut roots = rustls::RootCertStore::empty();
        roots.add(cert)?;

        let mut client_config = ClientConfig::with_root_certificates(Arc::new(roots))
            .map_err(|e| RastError::TODO(e.to_string()))?;

        let mut transport_config = quinn::TransportConfig::default();
        
        transport_config.keep_alive_interval(Some(Duration::from_secs(5)));
        client_config.transport_config(Arc::new(transport_config));

        Ok(client_config)
    }
}

#[async_trait]
impl ProtoFactory for QuicFactory {
    type Conf = QuicConf;

    async fn new_server(conf: &Self::Conf) -> Result<Arc<dyn ProtoServer>> {
        let config = QuicFactory::get_server_config(conf)?;
        let socket = SocketAddr::new(conf.ip, conf.port);
        let endpoint = Endpoint::server(config, socket)?;

        Ok(Arc::new(QuicServer::new(endpoint)))
    }

    async fn new_connection(conf: &Self::Conf) -> Result<Arc<Mutex<dyn ProtoConnection>>> {
        let local = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
        let mut endpoint = Endpoint::client(local)?;

        let client_config = QuicFactory::get_client_config(conf)?;
        endpoint.set_default_client_config(client_config);

        let server = SocketAddr::new(conf.ip, conf.port);
        let conn = endpoint
            .connect(server, &conf.server_name)
            .map_err(NetworkError::QuicNewConnection)?;
        let conn = conn.await.map_err(NetworkError::QuicExistingConnection)?;
        let (mut send, recv) = conn
            .open_bi()
            .await
            .map_err(NetworkError::QuicExistingConnection)?;

        trace!("Sending single byte to open connection");
        let _ = send.write(&[0u8]).await;

        Ok(Arc::new(Mutex::new(QuicConnection::new(conn, recv, send))))
    }
}

#[derive(Debug)]
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
            let conn = conn.await.map_err(NetworkError::QuicExistingConnection)?;
            debug!("Got connection from {:?}", conn.remote_address());
            let (send, mut recv) = conn
                .accept_bi()
                .await
                .map_err(NetworkError::QuicExistingConnection)?;

            trace!("Receiving single byte to open connection");
            let mut buf: [u8; 1] = [0];
            let _out = recv.read(&mut buf).await;

            Ok(Arc::new(Mutex::new(QuicConnection::new(conn, recv, send))))
        } else {
            Err(RastError::Unknown)
        }
    }
}

#[pin_project::pin_project]
#[derive(Debug)]
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
