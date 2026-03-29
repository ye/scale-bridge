use std::time::Duration;
use scale_bridge_core::{Codec, EtxCodec, Protocol, Scale, ScaleError, Transport};
use scale_bridge_scp01::{NciCommand, NciProtocol, NciResponse};
use crate::args::{Commands, OutputFormat};
use crate::output::print_response;
use crate::transport_builder::AnyTransport;

pub fn run(transport: AnyTransport, command: &Commands) -> Result<(), ScaleError> {
    let mut scale = Scale::new(transport, EtxCodec::new(), NciProtocol);

    match command {
        Commands::Weight { watch, interval, output } => {
            run_maybe_watch(&mut scale, NciCommand::Weight, output, *watch, *interval)
        }
        Commands::Status { output } => {
            let resp = scale.send(NciCommand::Status)?;
            print_response(&resp, output)
        }
        Commands::Zero => {
            scale.send(NciCommand::Zero)?;
            println!("OK");
            Ok(())
        }
        Commands::Tare => {
            scale.send(NciCommand::Tare)?;
            println!("OK");
            Ok(())
        }
        Commands::Units => {
            scale.send(NciCommand::Units)?;
            println!("OK");
            Ok(())
        }
        Commands::HighResolution { output } => {
            run_maybe_watch(&mut scale, NciCommand::HighResolution, output, false, Duration::ZERO)
        }
        Commands::Metrology { output } => {
            let resp = scale.send(NciCommand::Metrology)?;
            print_response(&resp, output)
        }
        Commands::About { output } => {
            let resp = scale.send(NciCommand::About)?;
            print_response(&resp, output)
        }
        Commands::Diagnostic { output } => {
            let resp = scale.send(NciCommand::Diagnostic)?;
            print_response(&resp, output)
        }
        Commands::Serve { port, .. } => {
            eprintln!("Server mode not yet implemented (would listen on port {port})");
            eprintln!("See crates/scale-bridge-server/src/lib.rs for planned API.");
            Ok(())
        }
    }
}

fn run_maybe_watch<T, C, P>(
    scale: &mut Scale<T, C, P>,
    cmd: NciCommand,
    output: &OutputFormat,
    watch: bool,
    interval: Duration,
) -> Result<(), ScaleError>
where
    T: Transport,
    C: Codec,
    P: Protocol<Command = NciCommand, Response = NciResponse>,
{
    loop {
        let resp = scale.send(cmd.clone())?;
        print_response(&resp, output)?;
        if !watch {
            break;
        }
        std::thread::sleep(interval);
    }
    Ok(())
}
