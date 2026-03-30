use crate::args::{Cli, SerialParity};
use scale_bridge_core::{ScaleError, Transport};
use std::io::{Read, Write};
use std::time::Duration;

#[cfg(feature = "serial")]
fn to_serialport_parity(parity: &SerialParity) -> serialport::Parity {
    match parity {
        SerialParity::None => serialport::Parity::None,
        SerialParity::Odd => serialport::Parity::Odd,
        SerialParity::Even => serialport::Parity::Even,
    }
}

/// Type-erased transport wrapping any concrete transport implementation.
pub enum AnyTransport {
    #[cfg(feature = "serial")]
    Serial(scale_bridge_core::SerialTransport),
    Tcp(scale_bridge_core::TcpTransport),
    Mock(scale_bridge_core::MockTransport),
}

impl Read for AnyTransport {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "serial")]
            AnyTransport::Serial(t) => t.read(buf),
            AnyTransport::Tcp(t) => t.read(buf),
            AnyTransport::Mock(t) => t.read(buf),
        }
    }
}

impl Write for AnyTransport {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "serial")]
            AnyTransport::Serial(t) => t.write(buf),
            AnyTransport::Tcp(t) => t.write(buf),
            AnyTransport::Mock(t) => t.write(buf),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(feature = "serial")]
            AnyTransport::Serial(t) => t.flush(),
            AnyTransport::Tcp(t) => t.flush(),
            AnyTransport::Mock(t) => t.flush(),
        }
    }
}

impl Transport for AnyTransport {
    fn set_timeout(&mut self, d: Duration) -> Result<(), ScaleError> {
        match self {
            #[cfg(feature = "serial")]
            AnyTransport::Serial(t) => t.set_timeout(d),
            AnyTransport::Tcp(t) => t.set_timeout(d),
            AnyTransport::Mock(t) => t.set_timeout(d),
        }
    }
    fn flush_output(&mut self) -> Result<(), ScaleError> {
        match self {
            #[cfg(feature = "serial")]
            AnyTransport::Serial(t) => t.flush_output(),
            AnyTransport::Tcp(t) => t.flush_output(),
            AnyTransport::Mock(t) => t.flush_output(),
        }
    }
}

/// Build the appropriate transport from CLI arguments.
///
/// If the `SCALE_BRIDGE_MOCK` environment variable is set, returns a
/// `MockTransport` pre-loaded with a stable 1234.56 lb response — useful
/// for testing and CI without real hardware.
pub fn build_transport(cli: &Cli) -> Result<AnyTransport, ScaleError> {
    if std::env::var("SCALE_BRIDGE_MOCK").is_ok() {
        // Stable 1234.56 lb, no motion, no errors
        let resp = b"\x0a  1234.56lb\x0d\x0a\xb0\xb0\x0d\x03".to_vec();
        return Ok(AnyTransport::Mock(
            scale_bridge_core::MockTransport::with_response(resp),
        ));
    }

    if let Some(host) = &cli.host {
        return Ok(AnyTransport::Tcp(scale_bridge_core::TcpTransport::connect(
            host,
            cli.tcp_port,
        )?));
    }

    #[cfg(feature = "serial")]
    if let Some(port) = &cli.serial_port {
        return Ok(AnyTransport::Serial(
            scale_bridge_core::SerialTransport::open(
                port,
                cli.baud,
                to_serialport_parity(&cli.parity),
            )?,
        ));
    }

    Err(ScaleError::Transport(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "specify --serial-port (serial) or --host (TCP/Ethernet)",
    )))
}
