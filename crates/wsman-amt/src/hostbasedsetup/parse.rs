use wsman_core::WsmanError;
use wsman_core::parse::{extract_text, extract_u32, has_fault};

use super::types::SetupOutput;

pub fn parse_setup(xml: &[u8]) -> Result<SetupOutput, WsmanError> {
    if has_fault(xml) {
        return Err(WsmanError::SoapFault(
            extract_text(xml, "Text").unwrap_or_default(),
        ));
    }
    let return_value =
        extract_u32(xml, "ReturnValue").ok_or(WsmanError::Parse("missing ReturnValue"))?;
    Ok(SetupOutput { return_value })
}
