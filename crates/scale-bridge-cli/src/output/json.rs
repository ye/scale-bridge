use crate::output::weight_status_display;
use scale_bridge_core::ScaleError;
use scale_bridge_scp01::NciResponse;
use serde::Serialize;

#[derive(Serialize)]
struct WeightStatusDetail<'a> {
    #[serde(flatten)]
    status: &'a scale_bridge_scp01::ScaleStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    condition: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_status: Option<&'a str>,
}

#[derive(Serialize)]
struct WeightStatusError<'a> {
    error: &'static str,
    detail: WeightStatusDetail<'a>,
}

pub fn print(response: &NciResponse) -> Result<(), ScaleError> {
    let json = serde_json::to_string(response).map_err(|e| ScaleError::ParseError(e.to_string()))?;
    println!("{json}");
    Ok(())
}

pub fn print_weight_conflict(status: &scale_bridge_scp01::ScaleStatus) -> Result<(), ScaleError> {
    let display = weight_status_display(status);
    let json = serde_json::to_string(&WeightStatusError {
        error: display.error,
        detail: WeightStatusDetail {
            status,
            condition: display.condition,
            raw_status: status.raw_status.as_deref(),
        },
    })
    .map_err(|e| ScaleError::ParseError(e.to_string()))?;
    println!("{json}");
    Ok(())
}
