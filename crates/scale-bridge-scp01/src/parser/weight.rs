// stub — implemented in Task 8
use scale_bridge_core::ScaleError;
use crate::command::NciCommand;
use crate::response::NciResponse;

pub fn parse_weight(_cmd: &NciCommand, _frame: &[u8]) -> Result<NciResponse, ScaleError> {
    Err(ScaleError::ParseError("not yet implemented".into()))
}

pub fn parse_metrology(_frame: &[u8]) -> Result<NciResponse, ScaleError> {
    Err(ScaleError::ParseError("not yet implemented".into()))
}

pub fn parse_about(_frame: &[u8]) -> Result<NciResponse, ScaleError> {
    Err(ScaleError::ParseError("not yet implemented".into()))
}

pub fn parse_diagnostic(_frame: &[u8]) -> Result<NciResponse, ScaleError> {
    Err(ScaleError::ParseError("not yet implemented".into()))
}
