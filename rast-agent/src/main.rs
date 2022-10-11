use std::{
    io::{Read, Result, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
};

use rast::protocols::{tcp::*, *};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello from client!");

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 42069);
    let mut client: Box<dyn ProtoClient<Conf = TcpConf>> =
        Box::new(TcpClient::new_client(address, None).await?);

    let msg_s = "Ping!";
    let _ = client.send(msg_s.to_string()).await?;
    println!("Message sent: {}", msg_s);

    let msg_r = client.recv().await?;
    println!("Message received: {}", msg_r);

    Ok(())
}
