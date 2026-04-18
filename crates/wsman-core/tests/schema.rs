use wsman_core::schema::{Action, Namespace, DEFAULT_TIMEOUT, WSA_ANONYMOUS};

#[test]
fn action_uris_are_stable() {
    assert_eq!(
        Action::Get.uri(),
        "http://schemas.xmlsoap.org/ws/2004/09/transfer/Get"
    );
    assert_eq!(
        Action::Put.uri(),
        "http://schemas.xmlsoap.org/ws/2004/09/transfer/Put"
    );
    assert_eq!(
        Action::Enumerate.uri(),
        "http://schemas.xmlsoap.org/ws/2004/09/enumeration/Enumerate"
    );
    assert_eq!(
        Action::Pull.uri(),
        "http://schemas.xmlsoap.org/ws/2004/09/enumeration/Pull"
    );
}

#[test]
fn namespaces_resolve_resource_uris() {
    assert_eq!(
        Namespace::Amt.resource_uri("AMT_GeneralSettings"),
        "http://intel.com/wbem/wscim/1/amt-schema/1/AMT_GeneralSettings"
    );
    assert_eq!(
        Namespace::Ips.resource_uri("IPS_HostBasedSetupService"),
        "http://intel.com/wbem/wscim/1/ips-schema/1/IPS_HostBasedSetupService"
    );
    assert_eq!(
        Namespace::Cim.resource_uri("CIM_BootConfigSetting"),
        "http://schemas.dmtf.org/wbem/wscim/1/cim-schema/2/CIM_BootConfigSetting"
    );
}

#[test]
fn defaults_match_go_impl() {
    assert_eq!(DEFAULT_TIMEOUT, "PT60S");
    assert_eq!(
        WSA_ANONYMOUS,
        "http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous"
    );
}
