//! The Rast project commonly used functionalities.
use thiserror::Error;

pub mod encoding;
pub mod messages;
pub mod protocols;
pub mod settings;

pub(crate) type Result<T> = std::result::Result<T, RastError>;

#[derive(Error, Debug)]
pub enum RastError {
    #[error("IO")]
    IO(#[from] std::io::Error),

    #[error("network")]
    Network(String),

    #[error("conversion")]
    Conversion(#[from] serde_json::Error),

    #[error(transparent)]
    Runtime(#[from] anyhow::Error),

    #[error("catch-all")]
    Unknown,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
