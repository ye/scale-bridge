mod args;
mod output;
mod runner;
mod transport_builder;

use args::Cli;
use clap::Parser;
use scale_bridge_core::ScaleError;
use transport_builder::build_transport;

fn main() {
    let cli = Cli::parse();

    let subscriber = tracing_subscriber::fmt()
        .with_ansi(!cli.systemd)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();

    let transport = match build_transport(&cli) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };

    match runner::run(transport, &cli.command) {
        Ok(()) => {}
        Err(ScaleError::UnrecognizedCommand) => {
            eprintln!("error: scale did not recognize the command");
            std::process::exit(1);
        }
        Err(ScaleError::Timeout)
        | Err(ScaleError::Transport(_))
        | Err(ScaleError::SerialPort(_)) => {
            eprintln!("error: transport failure");
            std::process::exit(2);
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(3);
        }
    }
}
