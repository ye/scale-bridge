use scale_bridge_core::ScaleError;
use scale_bridge_scp01::{DisplayState, NciResponse, ScaleStatus};

use crate::output::weight_status_display;

pub fn print(response: &NciResponse) -> Result<(), ScaleError> {
    match response {
        NciResponse::Weight(w) | NciResponse::HighResolution(w) => {
            match w.display {
                DisplayState::Normal => {
                    println!("{} {}", w.value, w.unit.as_str());
                }
                DisplayState::OverCapacity => println!("OVER CAPACITY"),
                DisplayState::UnderCapacity => println!("UNDER CAPACITY"),
                DisplayState::ZeroError => println!("ZERO ERROR"),
            }
            if w.status.motion {
                eprintln!("[motion — reading may be unstable]");
            }
            if w.status.has_error() {
                eprintln!(
                    "[scale error: ram={} rom={} eeprom={} cal={} zero={}]",
                    w.status.ram_error,
                    w.status.rom_error,
                    w.status.eeprom_error,
                    w.status.faulty_calibration,
                    w.status.initial_zero_error,
                );
            }
        }
        NciResponse::Status(s) => {
            println!(
                "motion={} at_zero={} over_cap={} under_cap={}",
                s.motion, s.at_zero, s.over_capacity, s.under_capacity
            );
            if s.has_error() {
                eprintln!(
                    "[scale error: ram={} rom={} eeprom={} cal={} zero={}]",
                    s.ram_error,
                    s.rom_error,
                    s.eeprom_error,
                    s.faulty_calibration,
                    s.initial_zero_error,
                );
            }
        }
        NciResponse::Acknowledged => println!("OK"),
        NciResponse::Metrology(m) => println!("raw_counts={}", m.raw_counts),
        NciResponse::About(a) => {
            println!(
                "model={} version={} capacity={}",
                a.model, a.version, a.capacity
            );
            if let Some(sn) = &a.load_cell_serial {
                println!("load_cell_serial={sn}");
            }
        }
        NciResponse::Diagnostic(d) => {
            println!(
                "power_on_starts={} calibrations={} overcapacity={}",
                d.power_on_starts, d.calibrations, d.overcapacity_events
            );
            println!(
                "normalized_counts={} span_counts={} zero_counts={}",
                d.normalized_counts, d.span_counts, d.zero_counts
            );
            println!(
                "cal_gravity={} span_weight={}",
                d.cal_gravity, d.span_weight
            );
        }
        NciResponse::UnrecognizedCommand => {
            eprintln!("scale did not recognize the command");
        }
    }
    Ok(())
}

pub fn print_weight_conflict(status: &ScaleStatus) -> Result<(), ScaleError> {
    let display = weight_status_display(status);
    println!("{}", display.error);
    Ok(())
}
