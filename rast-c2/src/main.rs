use std::{
    io::{Read, Result, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    thread,
};

use rast::{
    protocols::{tcp::*, *},
    settings::*,
};
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

    let conf = match Settings::new() {
        Ok(conf) => {
            println!("Config:");
            println!("{:?}", conf);
            conf
        },
        Err(e) => {
            panic!("{:?}", e);
        },
    };

    let address = SocketAddr::new(conf.server.ip, conf.server.port);
    let server: Box<dyn ProtoServer<Conf = TcpConf>> =
        Box::new(TcpServer::new_server(address, None).await?);

    loop {
        let conn = server.get_conn().await?;
        handle_client(conn).await?;
    }
}
