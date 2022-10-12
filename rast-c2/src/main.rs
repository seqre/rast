use std::{io, io::Result, net::SocketAddr};

use rast::{
    protocols::{tcp::*, *},
    settings::*,
};

async fn handle_client(mut conn: Box<dyn ProtoConnection>) -> Result<()> {
    // let msg_r = conn.recv().await?;
    // println!("Message received: {}", msg_r);

    let stdin = io::stdin();
    loop {
        let mut cmd = String::new();
        if let Ok(n) = stdin.read_line(&mut cmd) {
            if n > 0 {
                conn.send(cmd).await?;
                let msg_r = conn.recv().await?;
                let msg_r = msg_r.trim_end_matches('\0');
                println!("{}", msg_r);
            }
        };
    }

    // Ok(())
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
