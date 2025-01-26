use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use crate::{
    commands::{Command, CommandCategory, CommandOutput},
    context::Context,
};

/// Get username of owner of the process.
#[derive(Default)]
pub struct Whoami;

#[async_trait]
impl Command for Whoami {
    fn get_name(&self) -> &'static str {
        "whoami"
    }

    fn get_short_desc(&self) -> &'static str {
        "get process owner's username"
    }

    fn get_options(&self) -> Vec<(&'static str, &'static str)> {
        vec![]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::Permissions
    }

    async fn execute(
        &self,
        _ctx: Arc<RwLock<Context>>,
        _args: Vec<String>,
    ) -> anyhow::Result<CommandOutput> {
        let output = format!("{}@{:?}", whoami::username(), whoami::fallible::hostname());
        Ok(CommandOutput::Text(output))
    }
}
