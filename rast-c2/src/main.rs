use std::{io, net::SocketAddr};

use anyhow::Result;
use rast::{
    protocols::{tcp::*, *},
    settings::*,
};
use rast_c2::RastC2;
use tracing::{info, instrument};
use tracing_subscriber::filter::LevelFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    let conf = match Settings::new() {
        Ok(conf) => {
            info!("{:?}", conf);
            conf
        },
        Err(e) => {
            panic!("{:?}", e);
        },
    };

    let mut c2 = RastC2::with_settings(conf).await?;
    c2.run().await;

    // tui::run().await;

    Ok(())
}
