use std::io::{self, Cursor, Read, Write};
use std::time::Duration;
use crate::ScaleError;
use super::Transport;

pub struct MockTransport {
    reader: Cursor<Vec<u8>>,
    written: Vec<u8>,
}

impl MockTransport {
    pub fn with_response(response: Vec<u8>) -> Self {
        Self {
            reader: Cursor::new(response),
            written: Vec::new(),
        }
    }

    pub fn written(&self) -> &[u8] {
        &self.written
    }
}

impl Read for MockTransport {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl Write for MockTransport {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.written.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Transport for MockTransport {
    fn set_timeout(&mut self, _timeout: Duration) -> Result<(), ScaleError> {
        Ok(())
    }
    fn flush_output(&mut self) -> Result<(), ScaleError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_returns_preset_response_bytes() {
        let response = b"\x0a  1234.56lb\x0d\x0a\xb0\xb0\x0d\x03".to_vec();
        let mut t = MockTransport::with_response(response.clone());
        let mut buf = vec![0u8; response.len()];
        t.read_exact(&mut buf).unwrap();
        assert_eq!(buf, response);
    }

    #[test]
    fn mock_captures_written_bytes() {
        let mut t = MockTransport::with_response(vec![]);
        t.write_all(b"W\r").unwrap();
        assert_eq!(t.written(), b"W\r");
    }

    #[test]
    fn mock_set_timeout_succeeds() {
        let mut t = MockTransport::with_response(vec![]);
        t.set_timeout(std::time::Duration::from_secs(1)).unwrap();
    }
}
