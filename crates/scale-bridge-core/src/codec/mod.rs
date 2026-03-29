use crate::ScaleError;

pub trait Codec {
    fn encode(&self, raw: &[u8]) -> Vec<u8>;
    fn decode(&mut self, buf: &mut Vec<u8>) -> Result<Option<Vec<u8>>, ScaleError>;
}

pub mod etx;
pub use etx::EtxCodec;
