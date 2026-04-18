use alloc::string::String;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeneralSettings {
    pub digest_realm: String,
    pub instance_id: String,
    pub host_name: String,
    pub domain_name: String,
    pub network_interface_enabled: bool,
}
