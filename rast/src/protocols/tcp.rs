use std::io::Result;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::protocols::*;

pub struct TcpClient {
    conn: TcpConnection,
}
pub struct TcpServer {
    listener: TcpListener,
}
pub struct TcpConnection {
    stream: TcpStream,
}
pub struct TcpConf {}

#[async_trait]
impl ProtoClient for TcpClient {
    type Conf = TcpConf;

    async fn new_client(server_address: SocketAddr, _conf: Option<Self::Conf>) -> Result<Self>
    where
        Self: Sized,
    {
        let stream = TcpStream::connect(server_address).await;

        match stream {
            Ok(stream) => Ok(TcpClient {
                conn: TcpConnection { stream },
            }),
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl ProtoConnection for TcpClient {
    async fn recv(&mut self) -> Result<Msg> {
        self.conn.recv().await
    }

    async fn send(&mut self, msg: Msg) -> Result<()> {
        self.conn.send(msg).await
    }
}

#[async_trait]
impl ProtoConnection for TcpConnection {
    async fn recv(&mut self) -> Result<Msg> {
        // TODO: make it better
        let mut buffer = [0; 256];
        match self.stream.read(&mut buffer).await {
            Ok(_bytes_read) => Ok(String::from_utf8_lossy(&buffer).to_string()),
            Err(e) => Err(e),
        }
    }

    async fn send(&mut self, msg: Msg) -> Result<()> {
        // TODO: make it better
        match self.stream.write_all(&msg.into_bytes()).await {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl ProtoServer for TcpServer {
    type Conf = TcpConf;

    async fn new_server(listening_address: SocketAddr, _conf: Option<Self::Conf>) -> Result<Self>
    where
        Self: Sized,
    {
        let listener = TcpListener::bind(listening_address).await;

        match listener {
            Ok(listener) => Ok(TcpServer { listener }),
            Err(e) => Err(e),
        }
    }

    async fn get_conn(&self) -> Result<Box<dyn ProtoConnection>> {
        match self.listener.accept().await {
            Ok((stream, _address)) => Ok(Box::new(TcpConnection { stream })),
            // TODO: add better error handling
            Err(e) => Err(e),
        }
    }
}
