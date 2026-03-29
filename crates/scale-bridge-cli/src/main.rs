mod args;
mod output;
mod runner;
mod transport_builder;

use args::Cli;
use clap::Parser;
use crate::args::Commands;
use scale_bridge_core::ScaleError;
use transport_builder::build_transport;

fn main() {
    let cli = Cli::parse();
    let max_level = match cli.verbose {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    let subscriber = tracing_subscriber::fmt()
        .with_ansi(!cli.systemd)
        .with_target(false)
        .with_max_level(max_level)
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();

    if let Commands::Serve {
        https_port,
        bind,
        cert,
        key,
    } = &cli.command
    {
        let config = scale_bridge_server::ServerConfig {
            https_port: *https_port,
            bind_addr: bind.clone(),
            scale_serial_port: cli.serial_port.clone(),
            scale_host: cli.host.clone(),
            scale_tcp_port: cli.tcp_port,
            cert_path: cert.clone(),
            key_path: key.clone(),
        };

        if let Err(e) = scale_bridge_server::serve(config) {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
        return;
    }

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
