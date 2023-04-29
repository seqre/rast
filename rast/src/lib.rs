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

    // TODO: group those 3 somehow into one?
    #[error("network")]
    Network(String),

    #[error("quic")]
    Quic(#[from] quinn::ConnectError),

    #[error("quic")]
    Quic2(#[from] quinn::ConnectionError),

    #[error("conversion")]
    Conversion(#[from] serde_json::Error),

    #[error(transparent)]
    Runtime(#[from] anyhow::Error),

    #[error("encryption")]
    Encryption(#[from] rustls::Error),

    #[error("encryption")]
    Encryptaion2(#[from] rcgen::RcgenError),

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
