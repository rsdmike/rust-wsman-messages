pub const AMT_BASE: &str = "http://intel.com/wbem/wscim/1/amt-schema/1/";
pub const CIM_BASE: &str = "http://schemas.dmtf.org/wbem/wscim/1/cim-schema/2/";
pub const IPS_BASE: &str = "http://intel.com/wbem/wscim/1/ips-schema/1/";

pub const WSA_ANONYMOUS: &str = "http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous";
pub const DEFAULT_TIMEOUT: &str = "PT60S";

pub const TRANSFER_GET: &str = "http://schemas.xmlsoap.org/ws/2004/09/transfer/Get";
pub const TRANSFER_PUT: &str = "http://schemas.xmlsoap.org/ws/2004/09/transfer/Put";
pub const ENUMERATION_ENUMERATE: &str =
    "http://schemas.xmlsoap.org/ws/2004/09/enumeration/Enumerate";
pub const ENUMERATION_PULL: &str = "http://schemas.xmlsoap.org/ws/2004/09/enumeration/Pull";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    Amt,
    Cim,
    Ips,
}

impl Namespace {
    pub fn base(self) -> &'static str {
        match self {
            Namespace::Amt => AMT_BASE,
            Namespace::Cim => CIM_BASE,
            Namespace::Ips => IPS_BASE,
        }
    }

    pub fn resource_uri(self, class: &str) -> String {
        format!("{}{}", self.base(), class)
    }

    pub fn for_class(class: &str) -> Option<Namespace> {
        if class.starts_with("AMT_") {
            Some(Namespace::Amt)
        } else if class.starts_with("CIM_") {
            Some(Namespace::Cim)
        } else if class.starts_with("IPS_") {
            Some(Namespace::Ips)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Get,
    Put,
    Enumerate,
    Pull,
}

impl Action {
    pub fn uri(self) -> &'static str {
        match self {
            Action::Get => TRANSFER_GET,
            Action::Put => TRANSFER_PUT,
            Action::Enumerate => ENUMERATION_ENUMERATE,
            Action::Pull => ENUMERATION_PULL,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amt_resource_uri_matches_go() {
        assert_eq!(
            Namespace::Amt.resource_uri("AMT_GeneralSettings"),
            "http://intel.com/wbem/wscim/1/amt-schema/1/AMT_GeneralSettings"
        );
    }

    #[test]
    fn action_uris_match_go() {
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
    fn namespace_auto_selects_by_class_prefix() {
        assert_eq!(
            Namespace::for_class("AMT_GeneralSettings"),
            Some(Namespace::Amt)
        );
        assert_eq!(Namespace::for_class("CIM_Processor"), Some(Namespace::Cim));
        assert_eq!(
            Namespace::for_class("IPS_OptInService"),
            Some(Namespace::Ips)
        );
        assert_eq!(Namespace::for_class("Nonsense"), None);
    }
}
