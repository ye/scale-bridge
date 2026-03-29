use crate::{transport::Transport, ScaleError};
use std::io::{self, Read, Write};
use std::time::Duration;

pub struct SerialTransport {
    port: Box<dyn serialport::SerialPort>,
}

impl SerialTransport {
    pub fn open(
        port_name: &str,
        baud_rate: u32,
        parity: serialport::Parity,
    ) -> Result<Self, ScaleError> {
        let port = serialport::new(port_name, baud_rate)
            .data_bits(serialport::DataBits::Seven)
            .parity(parity)
            .stop_bits(serialport::StopBits::One)
            .timeout(Duration::from_secs(2))
            .open()
            .map_err(|e| ScaleError::SerialPort(e.to_string()))?;
        Ok(Self { port })
    }
}

impl Read for SerialTransport {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.port.read(buf)
    }
}

impl Write for SerialTransport {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.port.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.port.flush()
    }
}

impl Transport for SerialTransport {
    fn set_timeout(&mut self, timeout: Duration) -> Result<(), ScaleError> {
        self.port
            .set_timeout(timeout)
            .map_err(|e| ScaleError::SerialPort(e.to_string()))
    }
    fn flush_output(&mut self) -> Result<(), ScaleError> {
        self.port.flush().map_err(ScaleError::Transport)
    }
}
