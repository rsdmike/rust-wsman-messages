#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvisioningMode {
    None = 1,
    AdminControlMode = 2,
}

impl ProvisioningMode {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

pub struct UnprovisionInput {
    pub mode: ProvisioningMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnprovisionOutput {
    pub return_value: u32,
}
