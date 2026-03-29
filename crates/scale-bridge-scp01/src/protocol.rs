use crate::command::NciCommand;
use crate::response::NciResponse;
use scale_bridge_core::{Command, Protocol, ScaleError};

pub struct NciProtocol;

impl Protocol for NciProtocol {
    type Command = NciCommand;
    type Response = NciResponse;

    fn encode_command(&self, cmd: &NciCommand) -> Vec<u8> {
        vec![cmd.command_byte()]
    }

    fn decode_response(&self, cmd: &NciCommand, frame: &[u8]) -> Result<NciResponse, ScaleError> {
        if frame.starts_with(b"?") {
            return Ok(NciResponse::UnrecognizedCommand);
        }
        crate::parser::parse_frame(cmd, frame)
    }
}
