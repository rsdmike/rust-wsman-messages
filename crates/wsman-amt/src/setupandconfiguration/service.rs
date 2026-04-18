use alloc::format;

use wsman_core::client::Client;
use wsman_core::envelope::build_invoke;
use wsman_core::schema::Namespace;
use wsman_core::transport::{ResponseBuf, Transport};
use wsman_core::WsmanError;

use super::parse::parse_unprovision;
use super::types::{UnprovisionInput, UnprovisionOutput};

const REQUEST_BUF: usize = 2048;
const RESPONSE_BUF: usize = 4096;
const AUTH_BUF: usize = 1024;

const RESOURCE_CLASS: &str = "AMT_SetupAndConfigurationService";
const NAMESPACE_URI: &str =
    "http://intel.com/wbem/wscim/1/amt-schema/1/AMT_SetupAndConfigurationService";
const ACTION_UNPROVISION: &str =
    "http://intel.com/wbem/wscim/1/amt-schema/1/AMT_SetupAndConfigurationService/Unprovision";

pub struct SetupAndConfigurationService<'c, T: Transport> {
    client: &'c mut Client<T>,
}

impl<'c, T: Transport> SetupAndConfigurationService<'c, T> {
    pub fn new(client: &'c mut Client<T>) -> Self {
        Self { client }
    }

    pub fn unprovision(
        &mut self,
        input: UnprovisionInput,
    ) -> Result<UnprovisionOutput, WsmanError> {
        let uri = Namespace::Amt.resource_uri(RESOURCE_CLASS);
        let id = self.client.next_message_id();

        let input_xml = format!(
            "<s:Unprovision_INPUT xmlns:s=\"{NAMESPACE_URI}\">\
<s:ProvisioningMode>{}</s:ProvisioningMode>\
</s:Unprovision_INPUT>",
            input.mode.as_u32()
        );

        let mut req = [0u8; REQUEST_BUF];
        let req_len = build_invoke(
            ACTION_UNPROVISION,
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
        parse_unprovision(&body[..n])
    }
}
