//! Abstraction over \[en|de]coding of data

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::Result;

/// Abstraction over \[en|de]coding of data
pub trait Packager {
    fn encode<T: Serialize>(&self, data: &T) -> Result<Bytes>;
    fn decode<'de, T: Deserialize<'de>>(&self, bytes: &'de Bytes) -> Result<T>;
}

/// JSON \[en|de]coder for data
#[derive(Debug, Default)]
pub struct JsonPackager;

impl Packager for JsonPackager {
    fn encode<T: Serialize>(&self, data: &T) -> Result<Bytes> {
        Ok(serde_json::to_vec(data)?.into())
    }

    fn decode<'de, T: Deserialize<'de>>(&self, bytes: &'de Bytes) -> Result<T> {
        let decoded = serde_json::from_slice(bytes)?;
        Ok(decoded)
    }
}
