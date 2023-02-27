//! Context data of agent shell.

use std::{env::current_dir, path::PathBuf};

/// Context for [RastAgent](crate::RastAgent) built-in commands.
#[derive(Default)]
pub struct Context {
    current_dir: PathBuf,
}

impl Context {
    pub fn new() -> Self {
        Context {
            current_dir: current_dir().unwrap(),
        }
    }

    pub fn get_dir(&self) -> PathBuf {
        self.current_dir.clone()
    }

    pub fn change_dir(&mut self, path: PathBuf) {
        self.current_dir = path;
    }
}
