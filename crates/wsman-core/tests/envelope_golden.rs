use wsman_core::envelope;
use wsman_core::schema::Namespace;
use wsman_core::selector::Selector;

fn render(f: impl FnOnce(&mut [u8]) -> Result<usize, wsman_core::WsmanError>) -> String {
    let mut buf = [0u8; 4096];
    let n = f(&mut buf).expect("build ok");
    String::from_utf8(buf[..n].to_vec()).expect("utf8")
}

#[test]
fn get_envelope_matches_expected() {
    let uri = Namespace::Amt.resource_uri("AMT_GeneralSettings");
    let xml = render(|b| envelope::build_get(&uri, &[], 1, None, b));

    assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
    assert!(xml.contains("<a:Action>http://schemas.xmlsoap.org/ws/2004/09/transfer/Get</a:Action>"));
    assert!(xml.contains("<a:To>/wsman</a:To>"));
    assert!(xml.contains("<w:ResourceURI>http://intel.com/wbem/wscim/1/amt-schema/1/AMT_GeneralSettings</w:ResourceURI>"));
    assert!(xml.contains("<a:MessageID>1</a:MessageID>"));
    assert!(xml.contains("<w:OperationTimeout>PT60S</w:OperationTimeout>"));
    assert!(xml.contains("<Body></Body>"));
    assert!(xml.ends_with("</Envelope>"));
}

#[test]
fn get_envelope_with_selector() {
    let uri = Namespace::Amt.resource_uri("AMT_GeneralSettings");
    let xml = render(|b| {
        envelope::build_get(&uri, &[Selector::new("InstanceID", "Intel(r) AMT")], 7, None, b)
    });

    assert!(xml.contains("<w:SelectorSet>"));
    assert!(xml.contains("<w:Selector Name=\"InstanceID\">Intel(r) AMT</w:Selector>"));
    assert!(xml.contains("<a:MessageID>7</a:MessageID>"));
}

#[test]
fn invoke_envelope_splices_input_body() {
    let uri = Namespace::Ips.resource_uri("IPS_HostBasedSetupService");
    let action = "http://intel.com/wbem/wscim/1/ips-schema/1/IPS_HostBasedSetupService/Setup";
    let input_xml =
        "<h:Setup_INPUT xmlns:h=\"http://intel.com/wbem/wscim/1/ips-schema/1/IPS_HostBasedSetupService\">\
<h:NetAdminPassEncryptionType>2</h:NetAdminPassEncryptionType>\
<h:NetworkAdminPassword>deadbeef</h:NetworkAdminPassword>\
</h:Setup_INPUT>";

    let xml = render(|b| envelope::build_invoke(action, &uri, input_xml.as_bytes(), &[], 42, None, b));

    assert!(xml.contains(&format!("<a:Action>{action}</a:Action>")));
    assert!(xml.contains(&format!("<w:ResourceURI>{uri}</w:ResourceURI>")));
    assert!(xml.contains("<a:MessageID>42</a:MessageID>"));
    assert!(xml.contains(input_xml));
}

#[test]
fn buffer_too_small_returns_error() {
    let uri = Namespace::Amt.resource_uri("AMT_GeneralSettings");
    let mut tiny = [0u8; 16];
    let err = envelope::build_get(&uri, &[], 1, None, &mut tiny).unwrap_err();
    assert!(matches!(err, wsman_core::WsmanError::BufferTooSmall { .. }));
}

#[test]
fn enumerate_and_pull_bodies() {
    let uri = Namespace::Amt.resource_uri("AMT_GeneralSettings");

    let e = render(|b| envelope::build_enumerate(&uri, 1, None, b));
    assert!(e.contains("<Enumerate xmlns=\"http://schemas.xmlsoap.org/ws/2004/09/enumeration\" />"));

    let p = render(|b| envelope::build_pull(&uri, "ctx-123", 2, None, b));
    assert!(p.contains("<EnumerationContext>ctx-123</EnumerationContext>"));
    assert!(p.contains("<MaxElements>999</MaxElements>"));
    assert!(p.contains("<MaxCharacters>99999</MaxCharacters>"));
}

#[test]
fn put_envelope_wraps_body_xml() {
    let uri = Namespace::Amt.resource_uri("AMT_GeneralSettings");
    let inner = "<h:GeneralSettings><h:PingResponseEnabled>true</h:PingResponseEnabled></h:GeneralSettings>";
    let xml = render(|b| envelope::build_put(&uri, inner.as_bytes(), &[], 5, None, b));
    assert!(xml.contains(&format!("<Body>{inner}</Body>")));
}
