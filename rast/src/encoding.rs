use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::Result;

pub trait Packager {
    fn encode<T: Serialize>(&self, data: &T) -> Result<Bytes>;
    fn decode<'de, T: Deserialize<'de>>(&self, bytes: &'de Bytes) -> Result<T>;
}

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
