pub mod status;
pub mod weight;

use crate::command::NciCommand;
use crate::response::NciResponse;
use scale_bridge_core::ScaleError;

fn parse_control_response(frame: &[u8]) -> Result<NciResponse, ScaleError> {
    let is_standalone_status_frame =
        frame.first() == Some(&0x0A) && frame.iter().filter(|&&b| b == 0x0A).count() == 1;

    if is_standalone_status_frame {
        if let Ok(s) = status::parse_status_only(frame) {
            return Ok(NciResponse::Status(s));
        }
    }

    Ok(NciResponse::Acknowledged)
}

pub fn parse_frame(cmd: &NciCommand, frame: &[u8]) -> Result<NciResponse, ScaleError> {
    match cmd {
        NciCommand::Weight | NciCommand::HighResolution => {
            if let Ok(s) = status::parse_status_only(frame) {
                let is_standalone_status_frame = frame.first() == Some(&0x0A)
                    && frame.iter().filter(|&&b| b == 0x0A).count() == 1;
                if is_standalone_status_frame {
                    return Ok(NciResponse::Status(s));
                }
            }
            weight::parse_weight(cmd, frame)
        }
        NciCommand::Status => {
            let s = status::parse_status_only(frame)?;
            Ok(NciResponse::Status(s))
        }
        NciCommand::Zero | NciCommand::Tare | NciCommand::Units => parse_control_response(frame),
        NciCommand::Metrology => weight::parse_metrology(frame),
        NciCommand::About => weight::parse_about(frame),
        NciCommand::Diagnostic => weight::parse_diagnostic(frame),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_returns_status_when_scale_replies_with_status_frame() {
        let resp = parse_frame(&NciCommand::Zero, b"\x0aS00\x0d\x03").unwrap();
        assert!(matches!(resp, NciResponse::Status(_)));
    }

    #[test]
    fn tare_falls_back_to_acknowledged_for_non_status_reply() {
        let resp = parse_frame(&NciCommand::Tare, b"\x0aOK\x0d\x03").unwrap();
        assert!(matches!(resp, NciResponse::Acknowledged));
    }

    #[test]
    fn weight_returns_status_when_scale_replies_with_standalone_status_frame() {
        let resp = parse_frame(&NciCommand::Weight, b"\x0aS20\x0d\x03").unwrap();
        match resp {
            NciResponse::Status(s) => assert!(s.at_zero),
            other => panic!("expected Status, got {other:?}"),
        }
    }
}
