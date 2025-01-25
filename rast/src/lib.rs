//! The Rast project commonly used functionalities.

use crate::protocols::NetworkError;

pub mod agent;
pub mod encoding;
pub mod messages;
pub mod protocols;
pub mod settings;

pub type Result<T> = std::result::Result<T, RastError>;

#[derive(thiserror::Error, Debug)]
pub enum RastError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Network(#[from] NetworkError),

    #[error(transparent)]
    Conversion(#[from] serde_json::Error),

    #[error(transparent)]
    Runtime(#[from] anyhow::Error),

    #[error(transparent)]
    RustTls(#[from] rustls::Error),

    #[error(transparent)]
    X509(#[from] rcgen::Error),

    #[error("TODO: {0}")]
    TODO(String),

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
