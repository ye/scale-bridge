pub mod csv;
pub mod json;
pub mod text;

use crate::args::OutputFormat;
use scale_bridge_core::ScaleError;
use scale_bridge_scp01::{NciResponse, ScaleStatus};

pub struct WeightStatusDisplay<'a> {
    pub error: &'static str,
    pub condition: Option<&'static str>,
    _status: &'a ScaleStatus,
}

pub fn weight_status_display(status: &ScaleStatus) -> WeightStatusDisplay<'_> {
    let condition = match status.raw_status.as_deref() {
        Some("S10") => Some("weight not ready / unstable / dynamic-load condition"),
        Some("S20") => Some("scale is at zero"),
        _ => None,
    };
    let error = if matches!(condition, Some("weight not ready / unstable / dynamic-load condition")) {
        "weight not ready"
    } else {
        "scale returned status instead of weight"
    };

    WeightStatusDisplay {
        error,
        condition,
        _status: status,
    }
}

pub fn print_response(response: &NciResponse, format: &OutputFormat) -> Result<(), ScaleError> {
    match format {
        OutputFormat::Text => text::print(response),
        OutputFormat::Json => json::print(response),
        OutputFormat::Csv => csv::print(response),
    }
}

pub fn print_weight_conflict(status: &ScaleStatus, format: &OutputFormat) -> Result<(), ScaleError> {
    match format {
        OutputFormat::Text => text::print_weight_conflict(status),
        OutputFormat::Json => json::print_weight_conflict(status),
        OutputFormat::Csv => csv::print_weight_conflict(status),
    }
}
