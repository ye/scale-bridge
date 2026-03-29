use scale_bridge_core::ScaleError;
use scale_bridge_scp01::NciResponse;

pub fn print(response: &NciResponse) -> Result<(), ScaleError> {
    let json =
        serde_json::to_string(response).map_err(|e| ScaleError::ParseError(e.to_string()))?;
    println!("{json}");
    Ok(())
}
