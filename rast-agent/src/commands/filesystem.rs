//! Filesystem-related commands.

use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tracing::debug;

use super::CommandOutput;
use crate::{
    commands::{Command, CommandCategory},
    context::Context,
};

/// Change directory.
#[derive(Default)]
pub struct Cd;

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
            return Err(anyhow!("Only one argument supported for cd"));
        }

        let path = PathBuf::from(args.get(0).unwrap());

        if !path.is_dir() {
            return Err(anyhow!("No access or directory do not exist"));
        }

        // TODO: fix error handling
        let mut ctx = ctx.write().unwrap();
        ctx.change_dir(path);

        Ok(CommandOutput::Nothing)
    }
}

/// Print contents of the directory.
#[derive(Default)]
pub struct Ls;

impl Ls {
    fn get_dir_contents(path: PathBuf) -> Result<Vec<String>> {
        let dir_entries = match path.read_dir() {
            Ok(entries) => entries,
            Err(e) => {
                debug!("Failed to read directory contents: {:?}", e);
                return Err(e.into());
            },
        };

        let out = dir_entries
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap().file_name().to_string_lossy().into())
            .collect();

        Ok(out)
    }
}

/// Print current directory.
#[derive(Default)]
pub struct Pwd;

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
            // TODO: fix error handling
            let ctx = ctx.read().unwrap();
            let output = match Ls::get_dir_contents(ctx.get_dir()) {
                Ok(out) => out,
                Err(e) => return Err(e),
            };
            return Ok(CommandOutput::ListText(output));
        }

        if args.len() > 1 {
            return Err(anyhow!("Only one argument supported for ls"));
        }

        let path = PathBuf::from(args.get(0).unwrap());

        if !path.is_dir() {
            return Err(anyhow!("No access or directory do not exist"));
        }

        let output = match Ls::get_dir_contents(path) {
            Ok(out) => out,
            Err(e) => return Err(e),
        };

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
            return Err(anyhow!("No arguments supported for pwd"));
        }

        // TODO: fix error handling
        let ctx = ctx.read().unwrap();
        let wd = ctx.get_dir();

        Ok(CommandOutput::Text(wd.to_string_lossy().into_owned()))
    }
}
