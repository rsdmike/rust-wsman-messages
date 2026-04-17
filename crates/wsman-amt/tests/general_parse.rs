use pretty_assertions::assert_eq;
use wsman_amt::general::types::{
    AMTNetwork, DHCPSyncRequiresHostname, GeneralSettings, GetResponse, PowerSource,
    PreferredAddressFamily, PrivacyLevel, PullResponseEnvelope, ThunderboltDock,
};
use wsman_core::response::EnumerateResponse;

fn load(name: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{name}"))
        .unwrap_or_else(|_| panic!("missing fixture: {name}"))
}

#[test]
fn parses_get_response() {
    let xml = load("get.xml");
    let resp: GetResponse = quick_xml::de::from_str(&xml).expect("deserialize");

    // Values mirror pkg/wsman/amt/general/settings_test.go:74-100 in the Go repo
    let expected = GeneralSettings {
        element_name: "Intel(r) AMT: General Settings".into(),
        instance_id: "Intel(r) AMT: General Settings".into(),
        network_interface_enabled: true,
        digest_realm: "Digest:F3EB554784E729164447A89F60B641C5".into(),
        idle_wake_timeout: 1,
        host_name: "Test Host Name".into(),
        domain_name: "Test Domain Name".into(),
        ping_response_enabled: true,
        wsman_only_mode: false,
        preferred_address_family: PreferredAddressFamily::Ipv4,
        dhcpv6_configuration_timeout: 0,
        ddns_update_enabled: false,
        ddns_update_by_dhcp_server_enabled: true,
        shared_fqdn: true,
        host_os_fqdn: "Test Host OS FQDN".into(),
        ddns_ttl: 900,
        amt_network_enabled: AMTNetwork::Enabled,
        rmcp_ping_response_enabled: true,
        ddns_periodic_update_interval: 1440,
        presence_notification_interval: 0,
        privacy_level: PrivacyLevel::Default,
        power_source: PowerSource::Ac,
        thunderbolt_dock_enabled: ThunderboltDock::Disabled,
        oem_id: 0,
        dhcp_sync_requires_hostname: DHCPSyncRequiresHostname::Enabled,
    };

    assert_eq!(resp.body.settings, expected);
}

#[test]
fn parses_enumerate_response() {
    let xml = load("enumerate.xml");
    let resp = EnumerateResponse::from_envelope(&xml).unwrap();
    assert_eq!(resp.context, "14000000-0000-0000-0000-000000000000");
}

#[test]
fn parses_pull_response() {
    let xml = load("pull.xml");
    let resp: PullResponseEnvelope = quick_xml::de::from_str(&xml).expect("deserialize");
    let items = resp.body.pull.items.unwrap().settings;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].instance_id, "Intel(r) AMT: General Settings");
    assert_eq!(items[0].idle_wake_timeout, 65535);
    assert_eq!(items[0].ddns_ttl, 900);
}
