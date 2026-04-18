use wsman_amt::general::{GeneralSettings, Settings};
use wsman_core::client::{Client, Credentials};
use wsman_core::transport::{ResponseBuf, ResponseMeta, Transport};
use wsman_core::WsmanError;

const GS_RESPONSE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<a:Envelope xmlns:a="http://www.w3.org/2003/05/soap-envelope"
            xmlns:g="http://intel.com/wbem/wscim/1/amt-schema/1/AMT_GeneralSettings">
  <a:Body>
    <g:AMT_GeneralSettings>
      <g:DigestRealm>Digest:AABBCCDD11223344</g:DigestRealm>
      <g:InstanceID>Intel(r) AMT: General Settings</g:InstanceID>
      <g:HostName>test-host</g:HostName>
      <g:DomainName>example.local</g:DomainName>
      <g:NetworkInterfaceEnabled>true</g:NetworkInterfaceEnabled>
    </g:AMT_GeneralSettings>
  </a:Body>
</a:Envelope>"#;

struct Canned {
    resp: &'static [u8],
}
impl Transport for Canned {
    fn post(
        &mut self,
        _headers: &[(&str, &str)],
        _body: &[u8],
        resp: &mut ResponseBuf<'_>,
    ) -> Result<ResponseMeta, WsmanError> {
        resp.body[..self.resp.len()].copy_from_slice(self.resp);
        resp.body_len = self.resp.len();
        Ok(ResponseMeta { status: 200 })
    }
}

#[test]
fn get_returns_parsed_general_settings() {
    let mut client = Client::new(Canned { resp: GS_RESPONSE }, Credentials::digest("u", "p"));
    let gs: GeneralSettings = Settings::new(&mut client).get().unwrap();

    assert_eq!(gs.digest_realm, "Digest:AABBCCDD11223344");
    assert_eq!(gs.instance_id, "Intel(r) AMT: General Settings");
    assert_eq!(gs.host_name, "test-host");
    assert_eq!(gs.domain_name, "example.local");
    assert!(gs.network_interface_enabled);
}
