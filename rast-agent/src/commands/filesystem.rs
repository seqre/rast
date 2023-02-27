//! Filesystem-related commands.

use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use super::CommandOutput;
use crate::{
    commands::{Command, CommandCategory},
    context::Context,
};

#[derive(Default)]
pub struct Cd;
#[derive(Default)]
pub struct Ls;
#[derive(Default)]
pub struct Pwd;

impl Cd {
    pub fn new() -> Self {
        Cd {}
    }
}

impl Ls {
    pub fn new() -> Self {
        Ls {}
    }

    fn get_dir_contents(path: PathBuf) -> Vec<String> {
        path.read_dir()
            .unwrap()
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect()
    }
}

impl Pwd {
    pub fn new() -> Self {
        Pwd {}
    }
}

#[async_trait]
impl Command for Cd {
    fn get_name(&self) -> &'static str {
        "cd"
    }

    fn get_short_desc(&self) -> &'static str {
        "change directory"
    }

    fn get_options(&self) -> Vec<(&'static str, &'static str)> {
        vec![]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::FilesystemManipulation
    }

    async fn execute(&self, ctx: Arc<RwLock<Context>>, args: Vec<String>) -> Result<CommandOutput> {
        if args.len() > 1 {
            anyhow!("Only one argument supported for cd");
        }

        let path = PathBuf::from(args.get(0).unwrap());

        if !path.is_dir() {
            anyhow!("No access or directory do not exist");
        }

        let mut ctx = ctx.write().unwrap();
        ctx.change_dir(path);

        Ok(CommandOutput::Nothing)
    }
}

#[async_trait]
impl Command for Ls {
    fn get_name(&self) -> &'static str {
        "ls"
    }

    fn get_short_desc(&self) -> &'static str {
        "list directory contents"
    }

    fn get_options(&self) -> Vec<(&'static str, &'static str)> {
        vec![]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::FilesystemManipulation
    }

    async fn execute(&self, ctx: Arc<RwLock<Context>>, args: Vec<String>) -> Result<CommandOutput> {
        if args.is_empty() {
            let ctx = ctx.read().unwrap();
            let output = Ls::get_dir_contents(ctx.get_dir());
            return Ok(CommandOutput::ListText(output));
        }

        if args.len() > 1 {
            anyhow!("Only one argument supported for ls");
        }

        let path = PathBuf::from(args.get(0).unwrap());

        if !path.is_dir() {
            anyhow!("No access or directory do not exist");
        }

        let output = Ls::get_dir_contents(path);

        Ok(CommandOutput::ListText(output))
    }
}

#[async_trait]
impl Command for Pwd {
    fn get_name(&self) -> &'static str {
        "pwd"
    }

    fn get_short_desc(&self) -> &'static str {
        "print working directory"
    }

    fn get_options(&self) -> Vec<(&'static str, &'static str)> {
        vec![]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::FilesystemManipulation
    }

    async fn execute(&self, ctx: Arc<RwLock<Context>>, args: Vec<String>) -> Result<CommandOutput> {
        if !args.is_empty() {
            anyhow!("No arguments supported for pwd");
        }

        let ctx = ctx.read().unwrap();
        let wd = ctx.get_dir();

        Ok(CommandOutput::Text(wd.to_string_lossy().into_owned()))
    }
}
