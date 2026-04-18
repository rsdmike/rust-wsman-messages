use wsman_amt::hostbasedsetup::{HostBasedSetupService, SetupInput, SetupOutput};
use wsman_core::WsmanError;
use wsman_core::client::{Client, Credentials};
use wsman_core::transport::{ResponseBuf, ResponseMeta, Transport};

const SETUP_OK: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<a:Envelope xmlns:a="http://www.w3.org/2003/05/soap-envelope"
            xmlns:h="http://intel.com/wbem/wscim/1/ips-schema/1/IPS_HostBasedSetupService">
  <a:Body>
    <h:Setup_OUTPUT><h:ReturnValue>0</h:ReturnValue></h:Setup_OUTPUT>
  </a:Body>
</a:Envelope>"#;

const SETUP_FAIL: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<a:Envelope xmlns:a="http://www.w3.org/2003/05/soap-envelope"
            xmlns:h="http://intel.com/wbem/wscim/1/ips-schema/1/IPS_HostBasedSetupService">
  <a:Body>
    <h:Setup_OUTPUT><h:ReturnValue>9</h:ReturnValue></h:Setup_OUTPUT>
  </a:Body>
</a:Envelope>"#;

struct Spy {
    resp: &'static [u8],
    last_body: std::cell::RefCell<Vec<u8>>,
    last_action: std::cell::RefCell<String>,
}
impl Transport for Spy {
    fn post(
        &mut self,
        _headers: &[(&str, &str)],
        body: &[u8],
        resp: &mut ResponseBuf<'_>,
    ) -> Result<ResponseMeta, WsmanError> {
        *self.last_body.borrow_mut() = body.to_vec();
        let as_str = std::str::from_utf8(body).unwrap();
        let start = as_str.find("<a:Action>").unwrap() + "<a:Action>".len();
        let end = as_str[start..].find("</a:Action>").unwrap();
        *self.last_action.borrow_mut() = as_str[start..start + end].to_string();
        resp.body[..self.resp.len()].copy_from_slice(self.resp);
        resp.body_len = self.resp.len();
        Ok(ResponseMeta { status: 200 })
    }
}

#[test]
fn setup_returns_zero_on_success() {
    let spy = Spy {
        resp: SETUP_OK,
        last_body: Default::default(),
        last_action: Default::default(),
    };
    let mut client = Client::new(spy, Credentials::digest("u", "p"));
    let out: SetupOutput = HostBasedSetupService::new(&mut client)
        .setup(SetupInput {
            admin_password_hash: "deadbeef".into(),
            encryption_type: 2,
        })
        .unwrap();
    assert_eq!(out.return_value, 0);
}

#[test]
fn setup_body_contains_password_hash_and_encryption_type() {
    let spy = Spy {
        resp: SETUP_OK,
        last_body: Default::default(),
        last_action: Default::default(),
    };
    let mut client = Client::new(spy, Credentials::digest("u", "p"));
    let _ = HostBasedSetupService::new(&mut client)
        .setup(SetupInput {
            admin_password_hash: "abc123".into(),
            encryption_type: 2,
        })
        .unwrap();
    let body_any = client.into_transport();
    let body_s = String::from_utf8(body_any.last_body.borrow().clone()).unwrap();
    assert!(body_s.contains("<h:NetAdminPassEncryptionType>2</h:NetAdminPassEncryptionType>"));
    assert!(body_s.contains("<h:NetworkAdminPassword>abc123</h:NetworkAdminPassword>"));
    assert_eq!(
        *body_any.last_action.borrow(),
        "http://intel.com/wbem/wscim/1/ips-schema/1/IPS_HostBasedSetupService/Setup"
    );
}

#[test]
fn setup_nonzero_return_is_returned_not_errored() {
    let spy = Spy {
        resp: SETUP_FAIL,
        last_body: Default::default(),
        last_action: Default::default(),
    };
    let mut client = Client::new(spy, Credentials::digest("u", "p"));
    let out = HostBasedSetupService::new(&mut client)
        .setup(SetupInput {
            admin_password_hash: "h".into(),
            encryption_type: 2,
        })
        .unwrap();
    assert_eq!(out.return_value, 9);
}
