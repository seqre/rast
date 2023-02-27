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

pub enum CommandCategory {
    Misc,
    FilesystemManipulation,
}

pub enum CommandOutput {
    Nothing,
    Text(String),
    ListText(Vec<String>),
}

#[async_trait]
pub trait Command: Send + Sync {
    fn get_name(&self) -> &'static str;
    fn get_short_desc(&self) -> &'static str;
    fn get_options(&self) -> Vec<(&'static str, &'static str)>;
    fn get_help(&self) -> String {
        String::new()
    }
    fn get_category(&self) -> CommandCategory;
    async fn execute(&self, ctx: Arc<RwLock<Context>>, args: Vec<String>) -> Result<CommandOutput>;
}

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
            Box::new(Cd::new()),
            Box::new(Ls::new()),
            Box::new(Pwd::new()),
        ];

        commands
            .into_iter()
            .map(|c| (c.get_name().into(), c))
            .collect()
    }
}
