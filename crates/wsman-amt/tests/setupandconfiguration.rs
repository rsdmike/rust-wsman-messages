use wsman_amt::setupandconfiguration::{
    ProvisioningMode, SetupAndConfigurationService, UnprovisionInput,
};
use wsman_core::client::{Client, Credentials};
use wsman_core::transport::{ResponseBuf, ResponseMeta, Transport};
use wsman_core::WsmanError;

const UNPROVISION_OK: &[u8] = br#"<?xml version="1.0"?>
<a:Envelope xmlns:a="http://www.w3.org/2003/05/soap-envelope"
            xmlns:s="http://intel.com/wbem/wscim/1/amt-schema/1/AMT_SetupAndConfigurationService">
  <a:Body>
    <s:Unprovision_OUTPUT><s:ReturnValue>0</s:ReturnValue></s:Unprovision_OUTPUT>
  </a:Body>
</a:Envelope>"#;

struct Spy {
    last_body: std::cell::RefCell<Vec<u8>>,
    resp: &'static [u8],
}
impl Transport for Spy {
    fn post(
        &mut self,
        _h: &[(&str, &str)],
        body: &[u8],
        resp: &mut ResponseBuf<'_>,
    ) -> Result<ResponseMeta, WsmanError> {
        *self.last_body.borrow_mut() = body.to_vec();
        resp.body[..self.resp.len()].copy_from_slice(self.resp);
        resp.body_len = self.resp.len();
        Ok(ResponseMeta { status: 200 })
    }
}

#[test]
fn unprovision_sends_mode_and_parses_return_value() {
    let spy = Spy { last_body: Default::default(), resp: UNPROVISION_OK };
    let mut client = Client::new(spy, Credentials::digest("u", "p"));
    let out = SetupAndConfigurationService::new(&mut client)
        .unprovision(UnprovisionInput {
            mode: ProvisioningMode::AdminControlMode,
        })
        .unwrap();
    assert_eq!(out.return_value, 0);

    let spy = client.into_transport();
    let body = String::from_utf8(spy.last_body.borrow().clone()).unwrap();
    assert!(body.contains("<a:Action>http://intel.com/wbem/wscim/1/amt-schema/1/AMT_SetupAndConfigurationService/Unprovision</a:Action>"));
    assert!(body.contains("<s:ProvisioningMode>2</s:ProvisioningMode>"));
}
