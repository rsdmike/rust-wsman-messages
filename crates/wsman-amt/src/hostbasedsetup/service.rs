use alloc::format;

use wsman_core::WsmanError;
use wsman_core::client::Client;
use wsman_core::envelope::build_invoke;
use wsman_core::schema::Namespace;
use wsman_core::transport::{ResponseBuf, Transport};

use super::parse::parse_setup;
use super::types::{SetupInput, SetupOutput};

const REQUEST_BUF: usize = 2048;
const RESPONSE_BUF: usize = 4096;
const AUTH_BUF: usize = 1024;

const RESOURCE_CLASS: &str = "IPS_HostBasedSetupService";
const NAMESPACE_URI: &str = "http://intel.com/wbem/wscim/1/ips-schema/1/IPS_HostBasedSetupService";
const ACTION_SETUP: &str =
    "http://intel.com/wbem/wscim/1/ips-schema/1/IPS_HostBasedSetupService/Setup";

pub struct HostBasedSetupService<'c, T: Transport> {
    client: &'c mut Client<T>,
}

impl<'c, T: Transport> HostBasedSetupService<'c, T> {
    pub fn new(client: &'c mut Client<T>) -> Self {
        Self { client }
    }

    pub fn setup(&mut self, input: SetupInput) -> Result<SetupOutput, WsmanError> {
        let uri = Namespace::Ips.resource_uri(RESOURCE_CLASS);
        let id = self.client.next_message_id();

        let input_xml = format!(
            "<h:Setup_INPUT xmlns:h=\"{NAMESPACE_URI}\">\
<h:NetAdminPassEncryptionType>{}</h:NetAdminPassEncryptionType>\
<h:NetworkAdminPassword>{}</h:NetworkAdminPassword>\
</h:Setup_INPUT>",
            input.encryption_type, input.admin_password_hash
        );

        let mut req = [0u8; REQUEST_BUF];
        let req_len = build_invoke(
            ACTION_SETUP,
            &uri,
            input_xml.as_bytes(),
            &[],
            id,
            None,
            &mut req,
        )?;

        let mut body = [0u8; RESPONSE_BUF];
        let mut auth = [0u8; AUTH_BUF];
        let mut rb = ResponseBuf::new(&mut body, &mut auth);
        let n = self.client.execute(&req[..req_len], &mut rb)?;
        parse_setup(&body[..n])
    }
}
