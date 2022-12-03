pub mod capnp_utils;
pub mod messages;
pub mod protocols;
pub mod settings;

mod ui_request_capnp {
    include!(concat!(env!("OUT_DIR"), "/ui_request_capnp.rs"));
}

mod c2_agent_capnp {
    include!(concat!(env!("OUT_DIR"), "/c2_agent_capnp.rs"));
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
