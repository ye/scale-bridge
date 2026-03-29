pub mod csv;
pub mod json;
pub mod text;

use crate::args::OutputFormat;
use scale_bridge_core::ScaleError;
use scale_bridge_scp01::NciResponse;

pub fn print_response(response: &NciResponse, format: &OutputFormat) -> Result<(), ScaleError> {
    match format {
        OutputFormat::Text => text::print(response),
        OutputFormat::Json => json::print(response),
        OutputFormat::Csv => csv::print(response),
    }
}
