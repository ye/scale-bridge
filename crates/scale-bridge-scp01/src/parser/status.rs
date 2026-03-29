// stub — implemented in Task 7
use scale_bridge_core::ScaleError;
use crate::types::ScaleStatus;

pub fn parse_status_bytes(_bytes: &[u8]) -> Result<ScaleStatus, ScaleError> {
    Err(ScaleError::ParseError("not yet implemented".into()))
}

pub fn extract_status_bytes(_frame: &[u8]) -> Result<(Vec<u8>, Vec<u8>), ScaleError> {
    Err(ScaleError::ParseError("not yet implemented".into()))
}

pub fn parse_status_only(_frame: &[u8]) -> Result<ScaleStatus, ScaleError> {
    Err(ScaleError::ParseError("not yet implemented".into()))
}
