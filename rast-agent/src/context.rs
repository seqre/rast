//! Context data of agent shell.

use std::{env::current_dir, path::PathBuf};

/// Context for [`RastAgent`](crate::RastAgent) built-in commands.
#[derive(Default)]
pub struct Context {
    current_dir: PathBuf,
}

impl Context {
    pub fn new() -> Self {
        let current_dir = current_dir().expect("Failed to get current directory, panicking!");

        Self { current_dir }
    }

    #[must_use]
    pub fn get_dir(&self) -> PathBuf {
        self.current_dir.clone()
    }

    pub fn change_dir(&mut self, path: PathBuf) {
        if path.is_relative() {
            self.current_dir = self.current_dir.join(path);
        } else {
            self.current_dir = path;
        }
    }
}
