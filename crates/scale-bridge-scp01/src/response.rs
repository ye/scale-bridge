use crate::types::*;
use serde::Serialize;

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
    /// Scale replied with framed '?' response — command not recognized
    UnrecognizedCommand,
}
