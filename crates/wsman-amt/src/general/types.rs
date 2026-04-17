// crates/wsman-amt/src/general/types.rs
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename = "Envelope")]
pub struct GetResponse {
    #[serde(rename = "Body")]
    pub body: GetBody,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GetBody {
    #[serde(rename = "AMT_GeneralSettings")]
    pub settings: GeneralSettings,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename = "Envelope")]
pub struct PullResponseEnvelope {
    #[serde(rename = "Body")]
    pub body: PullBody,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PullBody {
    #[serde(rename = "PullResponse")]
    pub pull: PullPayload,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PullPayload {
    #[serde(rename = "Items", default)]
    pub items: Option<Items>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Items {
    #[serde(rename = "AMT_GeneralSettings", default)]
    pub settings: Vec<GeneralSettings>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct GeneralSettings {
    #[serde(rename = "ElementName", default)]
    pub element_name: String,
    #[serde(rename = "InstanceID", default)]
    pub instance_id: String,
    #[serde(rename = "NetworkInterfaceEnabled", default)]
    pub network_interface_enabled: bool,
    #[serde(rename = "DigestRealm", default)]
    pub digest_realm: String,
    #[serde(rename = "IdleWakeTimeout", default)]
    pub idle_wake_timeout: i64,
    #[serde(rename = "HostName", default)]
    pub host_name: String,
    #[serde(rename = "DomainName", default)]
    pub domain_name: String,
    #[serde(rename = "PingResponseEnabled", default)]
    pub ping_response_enabled: bool,
    #[serde(rename = "WsmanOnlyMode", default)]
    pub wsman_only_mode: bool,
    #[serde(rename = "PreferredAddressFamily", default)]
    pub preferred_address_family: PreferredAddressFamily,
    #[serde(rename = "DHCPv6ConfigurationTimeout", default)]
    pub dhcpv6_configuration_timeout: i64,
    #[serde(rename = "DDNSUpdateEnabled", default)]
    pub ddns_update_enabled: bool,
    #[serde(rename = "DDNSUpdateByDHCPServerEnabled", default)]
    pub ddns_update_by_dhcp_server_enabled: bool,
    #[serde(rename = "SharedFQDN", default)]
    pub shared_fqdn: bool,
    #[serde(rename = "HostOSFQDN", default)]
    pub host_os_fqdn: String,
    #[serde(rename = "DDNSTTL", default)]
    pub ddns_ttl: i64,
    #[serde(rename = "AMTNetworkEnabled", default)]
    pub amt_network_enabled: AMTNetwork,
    #[serde(rename = "RmcpPingResponseEnabled", default)]
    pub rmcp_ping_response_enabled: bool,
    #[serde(rename = "DDNSPeriodicUpdateInterval", default)]
    pub ddns_periodic_update_interval: i64,
    #[serde(rename = "PresenceNotificationInterval", default)]
    pub presence_notification_interval: i64,
    #[serde(rename = "PrivacyLevel", default)]
    pub privacy_level: PrivacyLevel,
    #[serde(rename = "PowerSource", default)]
    pub power_source: PowerSource,
    #[serde(rename = "ThunderboltDockEnabled", default)]
    pub thunderbolt_dock_enabled: ThunderboltDock,
    #[serde(rename = "OemID", default)]
    pub oem_id: i64,
    #[serde(rename = "DHCPSyncRequiresHostname", default)]
    pub dhcp_sync_requires_hostname: DHCPSyncRequiresHostname,
}

// Request sent on Put — all fields Option<_> to respect presence semantics.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename = "h:AMT_GeneralSettings")]
pub struct GeneralSettingsRequest {
    #[serde(rename = "@xmlns:h")]
    pub xmlns_h: String,

    #[serde(rename = "h:ElementName", skip_serializing_if = "Option::is_none")]
    pub element_name: Option<String>,
    #[serde(rename = "h:InstanceID", skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(rename = "h:IdleWakeTimeout", skip_serializing_if = "Option::is_none")]
    pub idle_wake_timeout: Option<i64>,
    #[serde(rename = "h:HostName", skip_serializing_if = "Option::is_none")]
    pub host_name: Option<String>,
    #[serde(rename = "h:DomainName", skip_serializing_if = "Option::is_none")]
    pub domain_name: Option<String>,
    #[serde(
        rename = "h:PingResponseEnabled",
        skip_serializing_if = "Option::is_none"
    )]
    pub ping_response_enabled: Option<bool>,
    #[serde(rename = "h:WsmanOnlyMode", skip_serializing_if = "Option::is_none")]
    pub wsman_only_mode: Option<bool>,
    // POC ships only these 7 Put fields; full port is deferred.
}

// --- Enums ---------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum PreferredAddressFamily {
    #[default]
    Ipv4 = 0,
    Ipv6 = 1,
    Reserved = 2,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum AMTNetwork {
    #[default]
    Disabled = 0,
    Enabled = 1,
    Reserved = 2,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum ThunderboltDock {
    #[default]
    Disabled = 0,
    Enabled = 1,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum PrivacyLevel {
    #[default]
    Default = 0,
    Enhanced = 1,
    Extreme = 2,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum PowerSource {
    #[default]
    Ac = 0,
    Dc = 1,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum DHCPSyncRequiresHostname {
    #[default]
    Disabled = 0,
    Enabled = 1,
}
