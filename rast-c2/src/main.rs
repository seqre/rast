use std::{io, net::SocketAddr};

use anyhow::Result;
use rast::{
    protocols::{tcp::*, *},
    settings::*,
};
use rast_c2::RastC2;

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

    let mut c2 = RastC2::with_settings(conf);
    c2.setup().await;
    c2.start().await;

    Ok(())
}
