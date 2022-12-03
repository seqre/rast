use anyhow::Result;

use crate::{
    c2_agent_capnp::*,
    protocols::{ConnectionOutput, Message},
};

pub fn create_message(content: &str) -> Message {
    let mut message = ::capnp::message::Builder::new_default();
    {
        let mut agent_message = message.init_root::<agent_message::Builder>();
        agent_message.set_content(content);
    }
    message
}

pub fn get_message(message: ConnectionOutput) -> Result<String> {
    let agent_message = message.get_root::<agent_message::Reader>()?;
    let content = agent_message.get_content()?;
    Ok(content.to_string())
}
