use wsman_core::parse;

const GS_RESPONSE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<a:Envelope xmlns:a="http://www.w3.org/2003/05/soap-envelope"
            xmlns:g="http://intel.com/wbem/wscim/1/amt-schema/1/AMT_GeneralSettings">
  <a:Header><a:Action>...</a:Action></a:Header>
  <a:Body>
    <g:AMT_GeneralSettings>
      <g:DigestRealm>Digest:1234567890ABCDEF1234567890ABCDEF</g:DigestRealm>
      <g:HostName>test-host</g:HostName>
      <g:NetworkInterfaceEnabled>true</g:NetworkInterfaceEnabled>
    </g:AMT_GeneralSettings>
  </a:Body>
</a:Envelope>"#;

const SETUP_RESPONSE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<a:Envelope xmlns:a="http://www.w3.org/2003/05/soap-envelope"
            xmlns:h="http://intel.com/wbem/wscim/1/ips-schema/1/IPS_HostBasedSetupService">
  <a:Body>
    <h:Setup_OUTPUT>
      <h:ReturnValue>0</h:ReturnValue>
    </h:Setup_OUTPUT>
  </a:Body>
</a:Envelope>"#;

const FAULT_RESPONSE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<a:Envelope xmlns:a="http://www.w3.org/2003/05/soap-envelope">
  <a:Body>
    <a:Fault>
      <a:Code><a:Value>a:Sender</a:Value></a:Code>
      <a:Reason><a:Text>Access denied</a:Text></a:Reason>
    </a:Fault>
  </a:Body>
</a:Envelope>"#;

#[test]
fn extract_text_reads_first_match_by_local_name() {
    let realm = parse::extract_text(GS_RESPONSE.as_bytes(), "DigestRealm").unwrap();
    assert_eq!(realm, "Digest:1234567890ABCDEF1234567890ABCDEF");

    let host = parse::extract_text(GS_RESPONSE.as_bytes(), "HostName").unwrap();
    assert_eq!(host, "test-host");
}

#[test]
fn extract_u32_parses_decimal_text() {
    let rv = parse::extract_u32(SETUP_RESPONSE.as_bytes(), "ReturnValue").unwrap();
    assert_eq!(rv, 0);
}

#[test]
fn extract_missing_element_returns_none() {
    assert!(parse::extract_text(SETUP_RESPONSE.as_bytes(), "NoSuchElement").is_none());
}

#[test]
fn has_fault_detects_soap_fault() {
    assert!(parse::has_fault(FAULT_RESPONSE.as_bytes()));
    assert!(!parse::has_fault(GS_RESPONSE.as_bytes()));
    assert!(!parse::has_fault(SETUP_RESPONSE.as_bytes()));
}
