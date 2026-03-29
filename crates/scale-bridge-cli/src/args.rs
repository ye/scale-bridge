use clap::{Parser, Subcommand, ValueEnum};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = "scale-bridge",
    version,
    about = "Avery WeighTronix scale CLI — SCP-01/NCI protocol",
    long_about = "scale-bridge communicates with Avery WeighTronix digital bench scales\n\
        over serial (RS-232/USB) or Ethernet using the SCP-01/NCI protocol.\n\n\
        One-shot mode: query the scale once and exit.\n\
        Watch mode (--watch): stream readings until Ctrl-C.\n\n\
        Connection:\n  \
        Serial:   --port /dev/ttyUSB0 --baud 9600\n  \
        Ethernet: --host 192.168.1.50 --tcp-port 3001\n\n\
        Set SCALE_BRIDGE_MOCK=1 to use built-in mock transport for testing."
)]
pub struct Cli {
    /// Serial port path (e.g. /dev/ttyUSB0 or COM3)
    #[arg(long, conflicts_with = "host")]
    pub port: Option<String>,

    /// Baud rate for serial connection
    #[arg(long, default_value = "9600")]
    pub baud: u32,

    /// TCP hostname for scales with built-in Ethernet
    #[arg(long, conflicts_with = "port")]
    pub host: Option<String>,

    /// TCP port number
    #[arg(long = "tcp-port", default_value = "3001")]
    pub tcp_port: u16,

    /// Suppress timestamps and ANSI color (for systemd/journald)
    #[arg(long)]
    pub systemd: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Read current weight
    Weight {
        /// Stream weight continuously until Ctrl-C
        #[arg(long, short)]
        watch: bool,
        /// Polling interval for --watch mode (e.g. 500ms, 2s)
        #[arg(long, default_value = "1s", value_parser = parse_duration)]
        interval: Duration,
        /// Output format
        #[arg(long, short, default_value = "text")]
        output: OutputFormat,
    },
    /// Read scale status
    Status {
        #[arg(long, short, default_value = "text")]
        output: OutputFormat,
    },
    /// Zero the scale
    Zero,
    /// Tare the scale
    Tare,
    /// Switch units of measure
    Units,
    /// Read high-resolution weight (10x normal resolution)
    HighResolution {
        #[arg(long, short, default_value = "text")]
        output: OutputFormat,
    },
    /// Read raw metrology counts
    Metrology {
        #[arg(long, short, default_value = "text")]
        output: OutputFormat,
    },
    /// Read model and version info (7600 series)
    About {
        #[arg(long, short, default_value = "text")]
        output: OutputFormat,
    },
    /// Read diagnostic data (7600 series)
    Diagnostic {
        #[arg(long, short, default_value = "text")]
        output: OutputFormat,
    },
    /// Start HTTPS REST server (not yet implemented)
    Serve {
        /// HTTPS port
        #[arg(long, default_value = "8443")]
        port: u16,
        /// Serial port for scale connection
        #[arg(long)]
        scale_port: Option<String>,
        /// TLS certificate file
        #[arg(long)]
        cert: Option<String>,
        /// TLS key file
        #[arg(long)]
        key: Option<String>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    if let Some(ms) = s.strip_suffix("ms") {
        ms.parse::<u64>()
            .map(Duration::from_millis)
            .map_err(|e| e.to_string())
    } else if let Some(secs) = s.strip_suffix('s') {
        secs.parse::<u64>()
            .map(Duration::from_secs)
            .map_err(|e| e.to_string())
    } else {
        s.parse::<u64>()
            .map(Duration::from_secs)
            .map_err(|e| e.to_string())
    }
}
