use wsman_core::WsmanError;
use wsman_core::client::Client;
use wsman_core::envelope::build_get;
use wsman_core::schema::Namespace;
use wsman_core::transport::{ResponseBuf, Transport};

use super::parse;
use super::types::GeneralSettings;

const REQUEST_BUF: usize = 2048;
const RESPONSE_BUF: usize = 4096;
const AUTH_BUF: usize = 1024;

pub struct Settings<'c, T: Transport> {
    client: &'c mut Client<T>,
}

impl<'c, T: Transport> Settings<'c, T> {
    pub fn new(client: &'c mut Client<T>) -> Self {
        Self { client }
    }

    pub fn get(&mut self) -> Result<GeneralSettings, WsmanError> {
        let uri = Namespace::Amt.resource_uri("AMT_GeneralSettings");
        let id = self.client.next_message_id();

        let mut req = [0u8; REQUEST_BUF];
        let req_len = build_get(&uri, &[], id, None, &mut req)?;

        let mut body = [0u8; RESPONSE_BUF];
        let mut auth = [0u8; AUTH_BUF];
        let mut rb = ResponseBuf::new(&mut body, &mut auth);
        let n = self.client.execute(&req[..req_len], &mut rb)?;
        parse::parse(&body[..n])
    }
}
