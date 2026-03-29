use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use crate::{ScaleError, transport::Transport};

pub struct TcpTransport {
    stream: TcpStream,
}

impl TcpTransport {
    pub fn connect(host: &str, port: u16) -> Result<Self, ScaleError> {
        let stream = TcpStream::connect((host, port))
            .map_err(ScaleError::Transport)?;
        Ok(Self { stream })
    }
}

impl Read for TcpTransport {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for TcpTransport {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl Transport for TcpTransport {
    fn set_timeout(&mut self, timeout: Duration) -> Result<(), ScaleError> {
        self.stream.set_read_timeout(Some(timeout)).map_err(ScaleError::Transport)?;
        self.stream.set_write_timeout(Some(timeout)).map_err(ScaleError::Transport)
    }
    fn flush_output(&mut self) -> Result<(), ScaleError> {
        self.stream.flush().map_err(ScaleError::Transport)
    }
}
