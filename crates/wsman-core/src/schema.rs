use alloc::string::String;

pub const DEFAULT_TIMEOUT: &str = "PT60S";
pub const WSA_ANONYMOUS: &str = "http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    Amt,
    Ips,
    Cim,
}

impl Namespace {
    pub fn base(self) -> &'static str {
        match self {
            Namespace::Amt => "http://intel.com/wbem/wscim/1/amt-schema/1",
            Namespace::Ips => "http://intel.com/wbem/wscim/1/ips-schema/1",
            Namespace::Cim => "http://schemas.dmtf.org/wbem/wscim/1/cim-schema/2",
        }
    }
    pub fn resource_uri(self, class: &str) -> String {
        let mut s = String::with_capacity(self.base().len() + class.len() + 1);
        s.push_str(self.base());
        s.push('/');
        s.push_str(class);
        s
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Get,
    Put,
    Enumerate,
    Pull,
    Invoke(&'static str), // fully qualified URI
}

impl Action {
    pub fn uri(self) -> &'static str {
        match self {
            Action::Get => "http://schemas.xmlsoap.org/ws/2004/09/transfer/Get",
            Action::Put => "http://schemas.xmlsoap.org/ws/2004/09/transfer/Put",
            Action::Enumerate => "http://schemas.xmlsoap.org/ws/2004/09/enumeration/Enumerate",
            Action::Pull => "http://schemas.xmlsoap.org/ws/2004/09/enumeration/Pull",
            Action::Invoke(s) => s,
        }
    }
}
