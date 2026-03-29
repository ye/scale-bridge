mod error;
pub mod codec;
pub mod transport;

pub use error::ScaleError;
pub use codec::{Codec, EtxCodec};
pub use transport::{MockTransport, TcpTransport, Transport};

#[cfg(feature = "serial")]
pub use transport::SerialTransport;
