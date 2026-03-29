mod error;
pub mod transport;

pub use error::ScaleError;
pub use transport::{MockTransport, TcpTransport, Transport};

#[cfg(feature = "serial")]
pub use transport::SerialTransport;
