use crate::types::{ScaleStatus, WeightRange};
use scale_bridge_core::ScaleError;

/// Parse the raw status byte slice extracted from an NCI response.
///
/// Status byte layout (LSB = bit 0):
///
/// Byte 1: b0=motion, b1=at_zero, b2=RAM_err, b3=EEPROM_err,
///         b4=always1, b5=always1, b6=always0, b7=parity
/// Byte 2: b0=under_cap, b1=over_cap, b2=ROM_err, b3=faulty_cal,
///         b4=always1, b5=always1, b6=byte3_follows, b7=parity
/// Byte 3: b0=range_LSB, b1=net_weight, b2=init_zero_err, b3=reserved,
///         b4=always1, b5=always1, b6=byte4_follows, b7=parity
pub fn parse_status_bytes(bytes: &[u8]) -> Result<ScaleStatus, ScaleError> {
    if bytes.len() < 2 {
        return Err(ScaleError::ParseError(format!(
            "expected at least 2 status bytes, got {}",
            bytes.len()
        )));
    }

    let b1 = bytes[0];
    let b2 = bytes[1];

    let motion = b1 & 0x01 != 0;
    let at_zero = b1 & 0x02 != 0;
    let ram_error = b1 & 0x04 != 0;
    let eeprom_error = b1 & 0x08 != 0;

    let under_capacity = b2 & 0x01 != 0;
    let over_capacity = b2 & 0x02 != 0;
    let rom_error = b2 & 0x04 != 0;
    let faulty_calibration = b2 & 0x08 != 0;
    let more_bytes = b2 & 0x40 != 0;

    let mut net_weight = false;
    let mut initial_zero_error = false;
    let mut range = WeightRange::Low;

    if more_bytes {
        if bytes.len() < 3 {
            return Err(ScaleError::ParseError(
                "byte 2 signals byte 3 follows but frame is too short".into(),
            ));
        }
        let b3 = bytes[2];
        // bits 1:0 encode range: 00=Low, 11=High (per SCP-01 spec)
        range = if b3 & 0x03 == 0x03 {
            WeightRange::High
        } else {
            WeightRange::Low
        };
        net_weight = b3 & 0x02 != 0;
        initial_zero_error = b3 & 0x04 != 0;
    }

    Ok(ScaleStatus {
        motion,
        at_zero,
        under_capacity,
        over_capacity,
        ram_error,
        rom_error,
        eeprom_error,
        faulty_calibration,
        net_weight,
        initial_zero_error,
        range,
    })
}

/// Extract the data bytes and raw status bytes from a complete NCI response frame.
///
/// Frame format: `<LF>[DATA]<CR><LF>[STATUS_BYTES]<CR><ETX>`
///
/// Returns `(data_bytes, status_bytes)` with LF/CR/ETX stripped.
pub fn extract_status_bytes(frame: &[u8]) -> Result<(Vec<u8>, Vec<u8>), ScaleError> {
    // Find second LF (0x0A) — status bytes start right after it
    let mut lf_count = 0usize;
    let mut status_start = None;
    for (i, &b) in frame.iter().enumerate() {
        if b == 0x0A {
            lf_count += 1;
            if lf_count == 2 {
                status_start = Some(i + 1);
                break;
            }
        }
    }

    let start = status_start.ok_or_else(|| {
        ScaleError::ParseError("could not locate status bytes in frame (expected 2 LFs)".into())
    })?;

    // Status bytes end at the CR (0x0D) before ETX
    let end = frame[start..]
        .iter()
        .position(|&b| b == 0x0D)
        .map(|p| start + p)
        .ok_or_else(|| ScaleError::ParseError("no CR after status bytes".into()))?;

    // Data bytes: between first LF and first CR
    let data_end = frame.iter().position(|&b| b == 0x0D).unwrap_or(frame.len());
    let data: Vec<u8> = frame[1..data_end].to_vec(); // skip leading LF

    let status_bytes = frame[start..end].to_vec();
    Ok((data, status_bytes))
}

/// Parse a [`ScaleStatus`] from a complete NCI response frame.
pub fn parse_status_only(frame: &[u8]) -> Result<ScaleStatus, ScaleError> {
    let (_, status_bytes) = extract_status_bytes(frame)?;
    parse_status_bytes(&status_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::WeightRange;

    /// Build a status byte with correct odd parity in bit 7.
    fn with_parity(bits_0_6: u8) -> u8 {
        let count = (bits_0_6 & 0x7F).count_ones();
        if count.is_multiple_of(2) {
            bits_0_6 | 0x80 // add parity bit to make count odd
        } else {
            bits_0_6
        }
    }

    #[test]
    fn parses_stable_not_at_zero_two_bytes() {
        // bits 4,5 always 1 → base = 0x30
        let b1 = with_parity(0x30);
        let b2 = with_parity(0x30);
        let status = parse_status_bytes(&[b1, b2]).unwrap();
        assert!(!status.motion);
        assert!(!status.at_zero);
        assert!(!status.under_capacity);
        assert!(!status.over_capacity);
        assert!(!status.ram_error);
        assert!(!status.has_error());
        assert_eq!(status.range, WeightRange::Low);
    }

    #[test]
    fn parses_motion_flag() {
        let b1 = with_parity(0x30 | 0x01);
        let b2 = with_parity(0x30);
        let status = parse_status_bytes(&[b1, b2]).unwrap();
        assert!(status.motion);
        assert!(!status.at_zero);
    }

    #[test]
    fn parses_at_zero_flag() {
        let b1 = with_parity(0x30 | 0x02);
        let b2 = with_parity(0x30);
        let status = parse_status_bytes(&[b1, b2]).unwrap();
        assert!(status.at_zero);
    }

    #[test]
    fn parses_over_capacity() {
        let b1 = with_parity(0x30);
        let b2 = with_parity(0x30 | 0x02);
        let status = parse_status_bytes(&[b1, b2]).unwrap();
        assert!(status.over_capacity);
        assert!(!status.under_capacity);
    }

    #[test]
    fn parses_under_capacity() {
        let b1 = with_parity(0x30);
        let b2 = with_parity(0x30 | 0x01);
        let status = parse_status_bytes(&[b1, b2]).unwrap();
        assert!(status.under_capacity);
    }

    #[test]
    fn parses_ram_and_eeprom_errors() {
        let b1 = with_parity(0x30 | 0x04 | 0x08);
        let b2 = with_parity(0x30);
        let status = parse_status_bytes(&[b1, b2]).unwrap();
        assert!(status.ram_error);
        assert!(status.eeprom_error);
        assert!(status.has_error());
    }

    #[test]
    fn parses_rom_and_faulty_cal_errors() {
        let b1 = with_parity(0x30);
        let b2 = with_parity(0x30 | 0x04 | 0x08);
        let status = parse_status_bytes(&[b1, b2]).unwrap();
        assert!(status.rom_error);
        assert!(status.faulty_calibration);
        assert!(status.has_error());
    }

    #[test]
    fn parses_three_byte_status_with_net_weight() {
        let b1 = with_parity(0x30);
        let b2 = with_parity(0x30 | 0x40); // bit 6 = more bytes follow
        let b3 = with_parity(0x30 | 0x02); // bit 1 = net_weight
        let status = parse_status_bytes(&[b1, b2, b3]).unwrap();
        assert!(status.net_weight);
        assert_eq!(status.range, WeightRange::Low);
    }

    #[test]
    fn parses_high_range_from_byte_3() {
        let b1 = with_parity(0x30);
        let b2 = with_parity(0x30 | 0x40); // more bytes
        let b3 = with_parity(0x30 | 0x03); // bits 0,1 = 11 → High range
        let status = parse_status_bytes(&[b1, b2, b3]).unwrap();
        assert_eq!(status.range, WeightRange::High);
    }

    #[test]
    fn parses_initial_zero_error_from_byte_3() {
        let b1 = with_parity(0x30);
        let b2 = with_parity(0x30 | 0x40);
        let b3 = with_parity(0x30 | 0x04); // bit 2 = initial_zero_error
        let status = parse_status_bytes(&[b1, b2, b3]).unwrap();
        assert!(status.initial_zero_error);
        assert!(status.has_error());
    }

    #[test]
    fn returns_error_for_empty_input() {
        assert!(parse_status_bytes(&[]).is_err());
    }

    #[test]
    fn returns_error_for_one_byte_only() {
        assert!(parse_status_bytes(&[0xB0]).is_err());
    }

    #[test]
    fn returns_error_when_byte3_signaled_but_missing() {
        let b1 = with_parity(0x30);
        let b2 = with_parity(0x30 | 0x40); // signals byte 3 follows
        assert!(parse_status_bytes(&[b1, b2]).is_err());
    }

    #[test]
    fn extract_status_bytes_splits_frame_correctly() {
        // Frame: LF "  1234.56lb" CR LF b1 b2 CR ETX
        let b1 = with_parity(0x30);
        let b2 = with_parity(0x30);
        let mut frame = vec![0x0A];
        frame.extend_from_slice(b"  1234.56lb");
        frame.push(0x0D);
        frame.push(0x0A);
        frame.push(b1);
        frame.push(b2);
        frame.push(0x0D);
        frame.push(0x03);

        let (data, status) = extract_status_bytes(&frame).unwrap();
        assert_eq!(data, b"  1234.56lb");
        assert_eq!(status, vec![b1, b2]);
    }
}
