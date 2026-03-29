use scale_bridge_core::ScaleError;
use scale_bridge_scp01::{NciResponse, DisplayState};
use chrono::Utc;

pub fn print(response: &NciResponse) -> Result<(), ScaleError> {
    let ts = Utc::now().to_rfc3339();
    match response {
        NciResponse::Weight(w) | NciResponse::HighResolution(w) => {
            let state = match w.display {
                DisplayState::Normal        => "normal",
                DisplayState::OverCapacity  => "over_capacity",
                DisplayState::UnderCapacity => "under_capacity",
                DisplayState::ZeroError     => "zero_error",
            };
            let motion = if w.status.motion { "motion" } else { "stable" };
            println!("{},{},{},{},{}", ts, w.value, w.unit.as_str(), state, motion);
        }
        other => {
            let json = serde_json::to_string(other)
                .map_err(|e| ScaleError::ParseError(e.to_string()))?;
            println!("{ts},{json}");
        }
    }
    Ok(())
}
