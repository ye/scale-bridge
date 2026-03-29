use crate::args::{Commands, OutputFormat};
use crate::output::print_response;
use crate::transport_builder::AnyTransport;
use scale_bridge_core::{Codec, EtxCodec, Protocol, Scale, ScaleError, Transport};
use scale_bridge_scp01::{NciCommand, NciProtocol, NciResponse};
use std::time::Duration;

pub fn run(transport: AnyTransport, command: &Commands) -> Result<(), ScaleError> {
    let mut scale = Scale::new(transport, EtxCodec::new(), NciProtocol);

    match command {
        Commands::Weight {
            watch,
            interval,
            output,
        } => run_maybe_watch(&mut scale, NciCommand::Weight, output, *watch, *interval),
        Commands::Status { output } => {
            let resp = scale.send(NciCommand::Status)?;
            print_response(&resp, output)
        }
        Commands::Zero => {
            let resp = scale.send(NciCommand::Zero)?;
            match resp {
                NciResponse::Status(_) => {
                    print_response(&resp, &OutputFormat::Text)?;
                    let weight = scale.send(NciCommand::Weight)?;
                    print_response(&weight, &OutputFormat::Text)
                }
                _ => print_response(&resp, &OutputFormat::Text),
            }
        }
        Commands::Tare => {
            let resp = scale.send(NciCommand::Tare)?;
            print_response(&resp, &OutputFormat::Text)
        }
        Commands::Units => {
            let resp = scale.send(NciCommand::Units)?;
            print_response(&resp, &OutputFormat::Text)
        }
        Commands::HighResolution { output } => run_maybe_watch(
            &mut scale,
            NciCommand::HighResolution,
            output,
            false,
            Duration::ZERO,
        ),
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
        Commands::Serve { .. } => unreachable!("serve is handled in main before transport setup"),
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
