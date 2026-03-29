pub mod status;
pub mod weight;

use scale_bridge_core::ScaleError;
use crate::command::NciCommand;
use crate::response::NciResponse;

pub fn parse_frame(cmd: &NciCommand, frame: &[u8]) -> Result<NciResponse, ScaleError> {
    match cmd {
        NciCommand::Weight | NciCommand::HighResolution => weight::parse_weight(cmd, frame),
        NciCommand::Status => {
            let s = status::parse_status_only(frame)?;
            Ok(NciResponse::Status(s))
        }
        NciCommand::Zero | NciCommand::Tare | NciCommand::Units => Ok(NciResponse::Acknowledged),
        NciCommand::Metrology  => weight::parse_metrology(frame),
        NciCommand::About      => weight::parse_about(frame),
        NciCommand::Diagnostic => weight::parse_diagnostic(frame),
    }
}
