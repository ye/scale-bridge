use std::fmt;

#[derive(Debug)]
pub enum ScaleError {
    Transport(std::io::Error),
    Timeout,
    FramingError(String),
    ParseError(String),
    UnrecognizedCommand,
    SerialPort(String),
}

impl fmt::Display for ScaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScaleError::Transport(e)      => write!(f, "transport error: {e}"),
            ScaleError::Timeout           => write!(f, "scale communication timeout"),
            ScaleError::FramingError(msg) => write!(f, "framing error: {msg}"),
            ScaleError::ParseError(msg)   => write!(f, "parse error: {msg}"),
            ScaleError::UnrecognizedCommand => write!(f, "scale did not recognize command"),
            ScaleError::SerialPort(msg)   => write!(f, "serial port error: {msg}"),
        }
    }
}

impl std::error::Error for ScaleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ScaleError::Transport(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ScaleError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::TimedOut {
            ScaleError::Timeout
        } else {
            ScaleError::Transport(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_io_error_converts_to_timeout_variant() {
        let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "timed out");
        let scale_err: ScaleError = io_err.into();
        assert!(matches!(scale_err, ScaleError::Timeout));
    }

    #[test]
    fn non_timeout_io_error_converts_to_transport_variant() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken pipe");
        let scale_err: ScaleError = io_err.into();
        assert!(matches!(scale_err, ScaleError::Transport(_)));
    }

    #[test]
    fn display_formats_all_variants() {
        assert!(ScaleError::Timeout.to_string().contains("timeout"));
        assert!(ScaleError::UnrecognizedCommand.to_string().contains("recognize"));
        assert!(ScaleError::FramingError("bad".into()).to_string().contains("bad"));
        assert!(ScaleError::ParseError("oops".into()).to_string().contains("oops"));
        assert!(ScaleError::SerialPort("port gone".into()).to_string().contains("port gone"));
    }
}
