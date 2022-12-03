use crate::{
    capnp_utils::{CapnpRead, CapnpWrite},
    ui_request_capnp::*,
};

#[derive(Debug)]
pub enum UiRequest {
    Ping,
    GetIps,
    GetIpData(String),
    Command(String),
}

#[derive(Debug)]
pub enum UiResponse {
    Pong,
    GetIps(Vec<String>),
    GetIpData(IpData),
    Command(String),
}

#[derive(Debug)]
pub struct IpData {
    ip: String,
}

impl<'a> CapnpRead<'a> for UiRequest {
    type Reader = ui_request::Reader<'a>;

    fn read_capnp(reader: Self::Reader) -> Self {
        match reader.get_ui_message().which() {
            Ok(ui_request::ui_message::Ping(())) => UiRequest::Ping,
            Ok(ui_request::ui_message::GetIps(())) => UiRequest::GetIps,
            Ok(ui_request::ui_message::GetIpData(ip)) => UiRequest::GetIpData(ip.unwrap().into()),
            Ok(ui_request::ui_message::Which::Command(cmd)) => {
                UiRequest::Command(cmd.unwrap().into())
            },
            Err(_) => UiRequest::Ping,
        }
    }
}

impl<'a> CapnpWrite<'a> for UiRequest {
    type Builder = ui_request::Builder<'a>;

    fn write_capnp(&self, builder: &mut Self::Builder) {
        let mut msg = builder.reborrow().get_ui_message();

        match self {
            UiRequest::Ping => msg.set_ping(()),
            UiRequest::GetIps => msg.set_get_ips(()),
            UiRequest::GetIpData(ip) => msg.set_get_ip_data(ip),
            UiRequest::Command(cmd) => msg.set_command(cmd),
        };
    }
}

impl<'a> CapnpRead<'a> for UiResponse {
    type Reader = ui_response::Reader<'a>;

    fn read_capnp(reader: Self::Reader) -> Self {
        match reader.get_ui_message().which() {
            Ok(ui_response::ui_message::Which::Pong(())) => UiResponse::Pong,
            Ok(ui_response::ui_message::Which::GetIps(ips)) => {
                let ips = ips.unwrap().iter().map(|e| e.unwrap().into()).collect();
                UiResponse::GetIps(ips)
            },
            Ok(ui_response::ui_message::Which::GetIpData(ip_data)) => {
                let ip_data = IpData::read_capnp(ip_data);
                UiResponse::GetIpData(ip_data)
            },
            Ok(ui_response::ui_message::Which::Command(cmd)) => {
                UiResponse::Command(cmd.unwrap().into())
            },
            Err(_) => UiResponse::Pong,
        }
    }
}

impl<'a> CapnpWrite<'a> for UiResponse {
    type Builder = ui_response::Builder<'a>;

    fn write_capnp(&self, builder: &mut Self::Builder) {
        let mut msg = builder.reborrow().get_ui_message();

        match self {
            UiResponse::Pong => msg.set_pong(()),
            UiResponse::GetIps(ips) => {
                let mut ips_builder = msg.init_get_ips(ips.len().try_into().unwrap());
                for (i, ip) in (&ips).iter().enumerate() {
                    ips_builder.set(i.try_into().unwrap(), ip);
                }
            },
            UiResponse::GetIpData(ip_data) => {
                let mut ip_data_builder = msg.init_get_ip_data();
                ip_data.write_capnp(&mut ip_data_builder)
            },
            UiResponse::Command(cmd) => msg.set_command(cmd),
        };
    }
}

impl<'a> CapnpRead<'a> for IpData {
    type Reader = ui_response::ui_message::get_ip_data::Reader<'a>;

    fn read_capnp(reader: Self::Reader) -> Self {
        let msg = reader.reborrow();
        IpData {
            ip: msg.get_ip().unwrap().to_string(),
        }
    }
}

impl<'a> CapnpWrite<'a> for IpData {
    type Builder = ui_response::ui_message::get_ip_data::Builder<'a>;

    fn write_capnp(&self, builder: &mut Self::Builder) {
        let mut msg = builder.reborrow();
        msg.set_ip(&self.ip);
    }
}
