use std::{
    io::{Read, Result, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
};

use rast::{
    protocols::{tcp::*, *},
    settings::*,
};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello from client!");

    // TODO: add embedding during compile
    let conf = match Settings::new() {
        Ok(conf) => {
            println!("{:?}", conf);
            conf
        },
        Err(e) => {
            panic!("{:?}", e);
        },
    };

    let address = SocketAddr::new(conf.server.ip, conf.server.port);
    let mut client: Box<dyn ProtoClient<Conf = TcpConf>> =
        Box::new(TcpClient::new_client(address, None).await?);

    let msg_s = "Ping!";
    let _ = client.send(msg_s.to_string()).await?;
    println!("Message sent: {}", msg_s);

    let msg_r = client.recv().await?;
    println!("Message received: {}", msg_r);

    Ok(())
}
