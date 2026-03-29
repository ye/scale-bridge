use crate::command::NciCommand;
use crate::response::NciResponse;
use scale_bridge_core::{Command, Protocol, ScaleError};

pub struct NciProtocol;

fn is_unrecognized_command_frame(frame: &[u8]) -> bool {
    let trimmed = frame
        .iter()
        .copied()
        .filter(|b| !matches!(b, b'\n' | b'\r' | 0x03))
        .collect::<Vec<_>>();
    trimmed == [b'?']
}

impl Protocol for NciProtocol {
    type Command = NciCommand;
    type Response = NciResponse;

    fn encode_command(&self, cmd: &NciCommand) -> Vec<u8> {
        vec![cmd.command_byte()]
    }

    fn decode_response(&self, cmd: &NciCommand, frame: &[u8]) -> Result<NciResponse, ScaleError> {
        if is_unrecognized_command_frame(frame) {
            return Ok(NciResponse::UnrecognizedCommand);
        }
        crate::parser::parse_frame(cmd, frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_bare_unrecognized_command_frame() {
        let protocol = NciProtocol;
        let resp = protocol.decode_response(&NciCommand::About, b"?").unwrap();
        assert!(matches!(resp, NciResponse::UnrecognizedCommand));
    }

    #[test]
    fn recognizes_framed_unrecognized_command_response() {
        let protocol = NciProtocol;
        let resp = protocol
            .decode_response(&NciCommand::About, b"\x0a?\x0d\x03")
            .unwrap();
        assert!(matches!(resp, NciResponse::UnrecognizedCommand));
    }
}
