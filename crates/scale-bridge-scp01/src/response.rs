use serde::Serialize;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum NciResponse {
    Weight(WeightReading),
    HighResolution(WeightReading),
    Status(ScaleStatus),
    /// Returned for Zero, Tare, Units commands
    Acknowledged,
    Metrology(MetrologyReading),
    About(AboutInfo),
    Diagnostic(DiagnosticInfo),
    /// Scale replied '?' — command not recognized
    UnrecognizedCommand,
}
