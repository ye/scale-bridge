use clap::{Parser, Subcommand, ValueEnum};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = "scale-bridge",
    version,
    author = "Ye Wang <ye@users.noreply.github.com>",
    about = "Avery Weigh-Tronix / NCI / Brecknell scale CLI — SCP-01/NCI protocol",
    long_about = "scale-bridge communicates with Avery Weigh-Tronix digital bench scales\n\
        over serial (RS-232/USB) or Ethernet using the SCP-01/NCI protocol.\n\n\
        Observed on tested NCI 6720-15 hardware:\n  \
        Serial parity defaults to even.\n  \
        Status replies may be standalone ASCII frames like S00.\n  \
        Weight units may be uppercase (for example LB).\n  \
        Unsupported commands may reply with framed '?'.\n\n\
        One-shot mode: query the scale once and exit.\n\
        Watch mode (--watch): stream readings until Ctrl-C.\n\n\
        Connection:\n  \
        Serial:   --serial-port /dev/ttyUSB0 --baud 9600 --parity even\n  \
        Ethernet: --host 192.168.1.50 --tcp-port 3001\n\n\
        HTTPS server:\n  \
        scale-bridge --serial-port /dev/ttyUSB0 serve --https-port 443 --bind 127.0.0.1 --cert cert.pem --key key.pem\n\n\
        Set SCALE_BRIDGE_MOCK=1 to use built-in mock transport for testing.",
    after_help = "Author:\n  Ye Wang <ye@users.noreply.github.com>\n\n\
        Help levels:\n  \
        `scale-bridge --help` shows top-level connection flags.\n  \
        `scale-bridge <subcommand> --help` shows options for that subcommand.\n\n\
        Server help:\n  \
        Use `scale-bridge serve --help` to see HTTPS listener options such as `--https-port` and `--bind`."
)]
pub struct Cli {
    /// Serial port path (e.g. /dev/ttyUSB0 or COM3)
    #[arg(long = "serial-port", alias = "port", conflicts_with = "host")]
    pub serial_port: Option<String>,

    /// Baud rate for serial connection
    #[arg(long, default_value = "9600")]
    pub baud: u32,

    /// Serial parity
    #[arg(long, default_value = "even")]
    pub parity: SerialParity,

    /// TCP hostname of the scale for Ethernet-connected devices
    #[arg(long, conflicts_with = "serial_port")]
    pub host: Option<String>,

    /// TCP port number of the scale for Ethernet-connected devices
    #[arg(long = "tcp-port", default_value = "3001")]
    pub tcp_port: u16,

    /// Suppress timestamps and ANSI color (for systemd/journald)
    #[arg(long)]
    pub systemd: bool,

    /// Verbosity level: 0=quiet, 1=debug wire logs, 2=trace
    #[arg(long, default_value_t = 0)]
    pub verbose: u8,

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
    #[command(
        long_about = "Start the HTTPS REST server.\n\n\
            The HTTPS listener is controlled by --https-port and --bind.\n\
            The scale connection is separate: use --serial-port for a local serial device, or --host/--tcp-port for an Ethernet-connected scale.\n\n\
            Example:\n  \
            scale-bridge --serial-port /dev/ttyUSB0 serve --https-port 443 --bind 127.0.0.1 --cert cert.pem --key key.pem"
    )]
    /// Start HTTPS REST server
    Serve {
        /// HTTPS port
        #[arg(long = "https-port", default_value = "443")]
        https_port: u16,
        /// Bind host or IP address for the HTTPS listener
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
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

#[derive(Debug, Clone, ValueEnum)]
pub enum SerialParity {
    None,
    Odd,
    Even,
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
