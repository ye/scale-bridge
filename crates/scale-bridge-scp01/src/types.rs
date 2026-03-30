use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WeightReading {
    pub value: Decimal,
    pub unit: WeightUnit,
    pub format: WeightFormat,
    pub display: DisplayState,
    pub status: ScaleStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScaleStatus {
    pub motion: bool,
    pub at_zero: bool,
    pub under_capacity: bool,
    pub over_capacity: bool,
    pub ram_error: bool,
    pub rom_error: bool,
    pub eeprom_error: bool,
    pub faulty_calibration: bool,
    pub net_weight: bool,
    pub initial_zero_error: bool,
    pub range: WeightRange,
    #[serde(skip_serializing)]
    pub raw_status: Option<String>,
}

impl ScaleStatus {
    pub fn has_error(&self) -> bool {
        self.ram_error
            || self.rom_error
            || self.eeprom_error
            || self.faulty_calibration
            || self.initial_zero_error
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum WeightUnit {
    #[serde(rename = "lb")]
    Lb,
    #[serde(rename = "kg")]
    Kg,
    #[serde(rename = "oz")]
    Oz,
    #[serde(rename = "g")]
    G,
    #[serde(rename = "lb oz")]
    LbOz,
}

impl WeightUnit {
    pub fn as_str(&self) -> &'static str {
        match self {
            WeightUnit::Lb => "lb",
            WeightUnit::Kg => "kg",
            WeightUnit::Oz => "oz",
            WeightUnit::G => "g",
            WeightUnit::LbOz => "lb oz",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum WeightFormat {
    Decimal,
    PoundsOunces,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum DisplayState {
    Normal,
    OverCapacity,
    UnderCapacity,
    ZeroError,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum WeightRange {
    Low,
    High,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MetrologyReading {
    pub raw_counts: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AboutInfo {
    pub model: String,
    pub version: String,
    pub capacity: String,
    pub load_cell_serial: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DiagnosticInfo {
    pub power_on_starts: u32,
    pub calibrations: u32,
    pub overcapacity_events: u32,
    pub normalized_counts: u32,
    pub span_counts: u32,
    pub zero_counts: u32,
    pub cal_gravity: Decimal,
    pub span_weight: String,
}
