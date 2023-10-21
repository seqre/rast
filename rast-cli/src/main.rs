

use anyhow::{anyhow, Result};
use rast::{
    protocols::{tcp::TcpFactory, ProtoFactory},
    settings::Settings,
};
use rast_cli::{get_shell, ShellState};


#[tokio::main]
async fn main() -> Result<()> {
    // tracing_subscriber::fmt()
    //    .with_max_level(LevelFilter::INFO)
    //    .init();

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

    let connection = if let Some(conf) = &conf.server.ui {
        let conf = &conf.tcp.unwrap();
        TcpFactory::new_client(conf).await
    } else {
        Err(anyhow!("Can't connect to C2 using TCP."))
    };

    let state = ShellState::new(connection.unwrap());

    let mut shell = get_shell(state);

    shell.run_async().await?;

    Ok(())
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
