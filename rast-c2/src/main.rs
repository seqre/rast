use std::{
    io::{Read, Result, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    thread,
};

use rast::protocols::{tcp::*, *};
use tokio;

async fn handle_client(mut conn: Box<dyn ProtoConnection>) -> Result<()> {
    let msg_r = conn.recv().await?;
    println!("Message received: {}", msg_r);

    let msg_s = "Pong!";
    let _ = conn.send(msg_s.to_string()).await?;
    println!("Message sent: {}", msg_s);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello from server!");

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 42069);
    let server: Box<dyn ProtoServer<Conf = TcpConf>> =
        Box::new(TcpServer::new_server(address, None).await?);

    loop {
        let conn = server.get_conn().await?;
        handle_client(conn).await?;
    }
}
