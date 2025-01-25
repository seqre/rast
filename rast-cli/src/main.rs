use std::sync::Arc;

use anyhow::{bail, Result};
use rast::{
    protocols::{quic::QuicFactory, tcp::TcpFactory, ProtoConnection, ProtoFactory},
    settings::{Connection, Settings},
    RastError,
};
use rast_cli::{get_shell, ShellState};
use tokio::sync::Mutex;
use tracing::info;

async fn get_connection(settings: &Settings) -> Result<Arc<Mutex<dyn ProtoConnection>>> {
    for conf in settings.server.ui_listeners.iter() {
        let conn = match conf {
            Connection::Tcp(tcp_conf) => TcpFactory::new_connection(tcp_conf).await,
            Connection::Quic(quic_conf) => QuicFactory::new_connection(quic_conf).await,
            _ => bail!(RastError::Unknown),
        };

        if let Ok(conn) = conn {
            info!("Connected to C2 at: {:?}", conn.lock().await.remote_addr()?);
            return Ok(conn);
        }
    }

    Err(RastError::Unknown.into())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), RastError> {
    // tracing_subscriber::fmt()
    //     .with_max_level(LevelFilter::DEBUG)
    //     .init();

    let conf = match Settings::new() {
        Ok(conf) => {
            println!("Config:");
            println!("{conf:?}");
            conf
        },
        Err(e) => {
            panic!("{e:?}");
        },
    };

    let connection = get_connection(&conf).await?;
    let state = ShellState::new(connection);
    let mut shell = get_shell(state);

    shell.run_async().await?;

    Ok(())
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
