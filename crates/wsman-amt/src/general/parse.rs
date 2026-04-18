use wsman_core::WsmanError;
use wsman_core::parse::extract_text;

use super::types::GeneralSettings;

pub fn parse(xml: &[u8]) -> Result<GeneralSettings, WsmanError> {
    if wsman_core::parse::has_fault(xml) {
        return Err(WsmanError::SoapFault(
            extract_text(xml, "Text").unwrap_or_default(),
        ));
    }
    Ok(GeneralSettings {
        digest_realm: extract_text(xml, "DigestRealm").unwrap_or_default(),
        instance_id: extract_text(xml, "InstanceID").unwrap_or_default(),
        host_name: extract_text(xml, "HostName").unwrap_or_default(),
        domain_name: extract_text(xml, "DomainName").unwrap_or_default(),
        network_interface_enabled: extract_text(xml, "NetworkInterfaceEnabled")
            .map(|s| s == "true")
            .unwrap_or(false),
    })
}
