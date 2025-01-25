//! Abstraction over \[en|de]coding of data

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::{RastError, Result};

/// Abstraction over \[en|de]coding of data
pub trait Packager {
    const ENCODING: Encoding;
    fn encode<T: Serialize>(data: &T) -> Result<Bytes>;
    fn decode<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Encoding {
    Json,
}

/// JSON \[en|de]coder for data
#[derive(Debug, Default)]
pub struct JsonPackager;

impl Packager for JsonPackager {
    const ENCODING: Encoding = Encoding::Json;
    fn encode<T: Serialize>(data: &T) -> Result<Bytes> {
        Ok(serde_json::to_vec(data)
            .map_err(RastError::Conversion)?
            .into())
    }

    fn decode<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T> {
        serde_json::from_slice(bytes).map_err(RastError::Conversion)
    }
}
