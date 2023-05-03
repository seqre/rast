//! Implementations of commands used in the agent.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::Result;
use async_trait::async_trait;

use crate::{
    commands::filesystem::{Cd, Ls, Pwd},
    context::Context,
};

pub mod filesystem;

/// Command categories used for pretty-printing.
#[derive(Debug)]
pub enum CommandCategory {
    Misc,
    FilesystemManipulation,
}

/// Possible output types of the command execution.
#[derive(Debug)]
pub enum CommandOutput {
    Nothing,
    Text(String),
    ListText(Vec<String>),
}

/// General interface for built-in commands.
#[async_trait]
pub trait Command: Send + Sync {
    /// Gets name of the command used to specify keyword to be used inside the
    /// shell.
    fn get_name(&self) -> &'static str;

    /// Gets one line description of the command.
    fn get_short_desc(&self) -> &'static str;

    /// Gets possible flags and options for the command.
    fn get_options(&self) -> Vec<(&'static str, &'static str)>;

    /// Full-length description of the command and its flags and options.
    fn get_help(&self) -> String {
        String::new()
    }

    /// Gets category of the command.
    fn get_category(&self) -> CommandCategory;

    /// Executes the command.
    async fn execute(&self, ctx: Arc<RwLock<Context>>, args: Vec<String>) -> Result<CommandOutput>;
}

/// Command manager.
#[derive(Default)]
pub struct Commands {
    commands: HashMap<String, Box<dyn Command>>,
}

impl Commands {
    pub fn new() -> Self {
        Self {
            commands: Self::get_commands(),
        }
    }

    fn get_commands() -> HashMap<String, Box<dyn Command>> {
        let commands: Vec<Box<dyn Command>> = vec![
            Box::new(Cd::default()),
            Box::new(Ls::default()),
            Box::new(Pwd::default()),
        ];

        commands
            .into_iter()
            .map(|c| (c.get_name().into(), c))
            .collect()
    }
}
