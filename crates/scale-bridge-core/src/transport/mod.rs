use crate::ScaleError;
use std::io::{Read, Write};
use std::time::Duration;

pub trait Transport: Read + Write {
    fn set_timeout(&mut self, timeout: Duration) -> Result<(), ScaleError>;
    fn flush_output(&mut self) -> Result<(), ScaleError>;
}

pub mod mock;
pub mod tcp;
pub use mock::MockTransport;
pub use tcp::TcpTransport;

#[cfg(feature = "serial")]
pub mod serial;
#[cfg(feature = "serial")]
pub use serial::SerialTransport;
