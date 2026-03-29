use super::status::{extract_status_bytes, parse_status_bytes};
use crate::command::NciCommand;
use crate::response::NciResponse;
use crate::types::*;
use rust_decimal::Decimal;
use scale_bridge_core::ScaleError;
use std::str::FromStr;

pub fn parse_weight(cmd: &NciCommand, frame: &[u8]) -> Result<NciResponse, ScaleError> {
    let (data_bytes, status_bytes) = extract_status_bytes(frame)?;
    let status = parse_status_bytes(&status_bytes)?;

    let raw = std::str::from_utf8(&data_bytes)
        .map_err(|e| ScaleError::ParseError(format!("non-UTF8 data bytes: {e}")))?;
    let data = raw.trim();

    let display = if data.starts_with('^') {
        DisplayState::OverCapacity
    } else if data.starts_with('_') {
        DisplayState::UnderCapacity
    } else if data.starts_with('-') {
        DisplayState::ZeroError
    } else {
        DisplayState::Normal
    };

    // Pounds-ounces format: contains both "lb" and "oz"
    if data.contains("lb") && data.contains("oz") {
        let lb_str = data.split("lb").next().unwrap_or("0").trim();
        let value = Decimal::from_str(lb_str).unwrap_or(Decimal::ZERO);
        let reading = WeightReading {
            value,
            unit: WeightUnit::LbOz,
            format: WeightFormat::PoundsOunces,
            display,
            status,
        };
        return Ok(match cmd {
            NciCommand::HighResolution => NciResponse::HighResolution(reading),
            _ => NciResponse::Weight(reading),
        });
    }

    // Decimal format — strip unit suffix to find numeric part
    // Try longest suffixes first to avoid "oz" matching inside "lboz"
    let unit_suffixes: &[(&str, WeightUnit)] = &[
        ("lb", WeightUnit::Lb),
        ("kg", WeightUnit::Kg),
        ("oz", WeightUnit::Oz),
        ("g", WeightUnit::G),
    ];

    let mut value_str = data;
    let mut unit = WeightUnit::Lb;

    for (suffix, u) in unit_suffixes {
        if let Some(stripped) = data.strip_suffix(suffix) {
            value_str = stripped.trim();
            unit = u.clone();
            break;
        }
    }

    let value = if display == DisplayState::Normal {
        Decimal::from_str(value_str).map_err(|e| {
            ScaleError::ParseError(format!("cannot parse weight value '{value_str}': {e}"))
        })?
    } else {
        Decimal::ZERO
    };

    let reading = WeightReading {
        value,
        unit,
        format: WeightFormat::Decimal,
        display,
        status,
    };

    Ok(match cmd {
        NciCommand::HighResolution => NciResponse::HighResolution(reading),
        _ => NciResponse::Weight(reading),
    })
}

pub fn parse_metrology(frame: &[u8]) -> Result<NciResponse, ScaleError> {
    let (data_bytes, _) = extract_status_bytes(frame)?;
    let s = std::str::from_utf8(&data_bytes)
        .map_err(|e| ScaleError::ParseError(e.to_string()))?
        .trim();
    let raw_counts: u32 = s
        .parse()
        .map_err(|e| ScaleError::ParseError(format!("bad metrology counts '{s}': {e}")))?;
    Ok(NciResponse::Metrology(MetrologyReading { raw_counts }))
}

pub fn parse_about(frame: &[u8]) -> Result<NciResponse, ScaleError> {
    // Frame: <LF>MMMM,VV-RR,CCCC[,xxxxxx]<CR><ETX>
    let inner: Vec<u8> = frame
        .iter()
        .skip(1) // skip leading LF
        .take_while(|&&b| b != 0x0D && b != 0x03)
        .cloned()
        .collect();
    let s = std::str::from_utf8(&inner).map_err(|e| ScaleError::ParseError(e.to_string()))?;
    let parts: Vec<&str> = s.splitn(4, ',').collect();
    if parts.len() < 3 {
        return Err(ScaleError::ParseError(format!(
            "malformed About response: '{s}' (expected at least 3 comma-separated fields)"
        )));
    }
    Ok(NciResponse::About(AboutInfo {
        model: parts[0].trim().to_string(),
        version: parts[1].trim().to_string(),
        capacity: parts[2].trim().to_string(),
        load_cell_serial: parts.get(3).map(|s| s.trim().to_string()),
    }))
}

pub fn parse_diagnostic(frame: &[u8]) -> Result<NciResponse, ScaleError> {
    // Frame: <LF>SSS,CCC,OOO,nnnnnn,ssssss,zzzzzz,x.xxxx,SWT<CR><ETX>
    let inner: Vec<u8> = frame
        .iter()
        .skip(1)
        .take_while(|&&b| b != 0x0D && b != 0x03)
        .cloned()
        .collect();
    let s = std::str::from_utf8(&inner).map_err(|e| ScaleError::ParseError(e.to_string()))?;
    let p: Vec<&str> = s.splitn(8, ',').collect();
    if p.len() < 8 {
        return Err(ScaleError::ParseError(format!(
            "malformed Diagnostic response: '{s}' (expected 8 fields, got {})",
            p.len()
        )));
    }

    let parse_u32 = |v: &str| -> Result<u32, ScaleError> {
        v.trim()
            .parse()
            .map_err(|e| ScaleError::ParseError(format!("bad u32 '{v}': {e}")))
    };
    let parse_dec = |v: &str| -> Result<Decimal, ScaleError> {
        Decimal::from_str(v.trim())
            .map_err(|e| ScaleError::ParseError(format!("bad decimal '{v}': {e}")))
    };

    Ok(NciResponse::Diagnostic(DiagnosticInfo {
        power_on_starts: parse_u32(p[0])?,
        calibrations: parse_u32(p[1])?,
        overcapacity_events: parse_u32(p[2])?,
        normalized_counts: parse_u32(p[3])?,
        span_counts: parse_u32(p[4])?,
        zero_counts: parse_u32(p[5])?,
        cal_gravity: parse_dec(p[6])?,
        span_weight: p[7].trim().to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// Build status bytes: two stable/no-error bytes with correct odd parity.
    /// b1=0xB0, b2=0xB0 for stable, no errors, last byte.
    fn stable_status() -> (u8, u8) {
        // 0x30 = bits 4,5 set; count of 1s = 2 (even) → need parity bit → 0xB0
        (0xB0, 0xB0)
    }

    fn make_weight_frame(data: &[u8]) -> Vec<u8> {
        let (b1, b2) = stable_status();
        let mut f = vec![0x0A];
        f.extend_from_slice(data);
        f.push(0x0D);
        f.push(0x0A);
        f.push(b1);
        f.push(b2);
        f.push(0x0D);
        f.push(0x03);
        f
    }

    #[test]
    fn parses_decimal_lb_weight() {
        let frame = make_weight_frame(b"  1234.56lb");
        let resp = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let NciResponse::Weight(w) = resp {
            assert_eq!(w.value, dec!(1234.56));
            assert_eq!(w.unit, WeightUnit::Lb);
            assert_eq!(w.format, WeightFormat::Decimal);
            assert_eq!(w.display, DisplayState::Normal);
            assert!(!w.status.motion);
        } else {
            panic!("expected Weight variant, got {resp:?}");
        }
    }

    #[test]
    fn parses_kg_weight() {
        let frame = make_weight_frame(b"    0.567kg");
        let resp = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let NciResponse::Weight(w) = resp {
            assert_eq!(w.value, dec!(0.567));
            assert_eq!(w.unit, WeightUnit::Kg);
        } else {
            panic!("{resp:?}");
        }
    }

    #[test]
    fn parses_gram_weight() {
        let frame = make_weight_frame(b"  500.0g");
        let resp = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let NciResponse::Weight(w) = resp {
            assert_eq!(w.value, dec!(500.0));
            assert_eq!(w.unit, WeightUnit::G);
        } else {
            panic!("{resp:?}");
        }
    }

    #[test]
    fn parses_zero_weight() {
        let frame = make_weight_frame(b"    0.00lb");
        let resp = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let NciResponse::Weight(w) = resp {
            assert_eq!(w.value, dec!(0.00));
        } else {
            panic!("{resp:?}");
        }
    }

    #[test]
    fn parses_lb_oz_format() {
        let frame = make_weight_frame(b"  10lb  2.3oz");
        let resp = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let NciResponse::Weight(w) = resp {
            assert_eq!(w.unit, WeightUnit::LbOz);
            assert_eq!(w.format, WeightFormat::PoundsOunces);
            assert_eq!(w.value, dec!(10));
        } else {
            panic!("{resp:?}");
        }
    }

    #[test]
    fn parses_over_capacity() {
        // b2 bit1=over_cap: 0x30|0x02=0x32; count of 1s in 0x32 = 3 (odd) → no parity bit → 0x32
        let mut frame = vec![0x0A];
        frame.extend_from_slice(b"^^^^^^^lb");
        frame.push(0x0D);
        frame.push(0x0A);
        frame.push(0xB0); // b1 stable
        frame.push(0x32); // b2 over_capacity, odd parity already
        frame.push(0x0D);
        frame.push(0x03);

        let resp = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let NciResponse::Weight(w) = resp {
            assert_eq!(w.display, DisplayState::OverCapacity);
            assert!(w.status.over_capacity);
            assert_eq!(w.value, Decimal::ZERO);
        } else {
            panic!("{resp:?}");
        }
    }

    #[test]
    fn parses_under_capacity() {
        // b2 bit0=under_cap: 0x31; count=3 → odd parity → 0x31
        let mut frame = vec![0x0A];
        frame.extend_from_slice(b"_______lb");
        frame.push(0x0D);
        frame.push(0x0A);
        frame.push(0xB0);
        frame.push(0x31);
        frame.push(0x0D);
        frame.push(0x03);

        let resp = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let NciResponse::Weight(w) = resp {
            assert_eq!(w.display, DisplayState::UnderCapacity);
        } else {
            panic!("{resp:?}");
        }
    }

    #[test]
    fn parses_zero_error() {
        let frame = make_weight_frame(b"-------lb");
        let resp = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let NciResponse::Weight(w) = resp {
            assert_eq!(w.display, DisplayState::ZeroError);
        } else {
            panic!("{resp:?}");
        }
    }

    #[test]
    fn high_resolution_returns_high_resolution_variant() {
        let frame = make_weight_frame(b" 1234.560lb");
        let resp = parse_weight(&NciCommand::HighResolution, &frame).unwrap();
        assert!(matches!(resp, NciResponse::HighResolution(_)));
    }

    #[test]
    fn parses_about_response() {
        let frame = b"\x0a7600,01-02,150lb,ABC123\x0d\x03".to_vec();
        let resp = parse_about(&frame).unwrap();
        if let NciResponse::About(a) = resp {
            assert_eq!(a.model, "7600");
            assert_eq!(a.version, "01-02");
            assert_eq!(a.capacity, "150lb");
            assert_eq!(a.load_cell_serial, Some("ABC123".into()));
        } else {
            panic!("{resp:?}");
        }
    }

    #[test]
    fn parses_about_without_serial() {
        let frame = b"\x0a7600,01-02,150lb\x0d\x03".to_vec();
        let resp = parse_about(&frame).unwrap();
        if let NciResponse::About(a) = resp {
            assert_eq!(a.load_cell_serial, None);
        } else {
            panic!("{resp:?}");
        }
    }

    #[test]
    fn parses_metrology_response() {
        let frame = make_weight_frame(b"   65000");
        // metrology frame has same structure as weight for data extraction
        let resp = parse_metrology(&frame).unwrap();
        if let NciResponse::Metrology(m) = resp {
            assert_eq!(m.raw_counts, 65000);
        } else {
            panic!("{resp:?}");
        }
    }

    #[test]
    fn parses_diagnostic_response() {
        let frame = b"\x0a001,002,003,065000,050000,020000,9.8123,150lb\x0d\x03".to_vec();
        let resp = parse_diagnostic(&frame).unwrap();
        if let NciResponse::Diagnostic(d) = resp {
            assert_eq!(d.power_on_starts, 1);
            assert_eq!(d.calibrations, 2);
            assert_eq!(d.overcapacity_events, 3);
            assert_eq!(d.normalized_counts, 65000);
            assert_eq!(d.span_weight, "150lb");
        } else {
            panic!("{resp:?}");
        }
    }

    #[test]
    fn returns_error_on_malformed_about() {
        let frame = b"\x0ajust_one_field\x0d\x03".to_vec();
        assert!(parse_about(&frame).is_err());
    }

    #[test]
    fn returns_error_on_malformed_diagnostic() {
        let frame = b"\x0aonly,two,fields\x0d\x03".to_vec();
        assert!(parse_diagnostic(&frame).is_err());
    }
}
