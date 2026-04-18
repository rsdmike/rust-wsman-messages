use alloc::string::String;

pub struct SetupInput {
    pub admin_password_hash: String,
    pub encryption_type: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupOutput {
    pub return_value: u32,
}
