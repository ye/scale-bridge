use super::Codec;
use crate::ScaleError;

const ETX: u8 = 0x03;
const CR: u8 = 0x0D;

pub struct EtxCodec {
    internal: Vec<u8>,
}

impl EtxCodec {
    pub fn new() -> Self {
        Self { internal: Vec::new() }
    }
}

impl Default for EtxCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Codec for EtxCodec {
    fn encode(&self, raw: &[u8]) -> Vec<u8> {
        let mut out = raw.to_vec();
        out.push(CR);
        out
    }

    fn decode(&mut self, buf: &mut Vec<u8>) -> Result<Option<Vec<u8>>, ScaleError> {
        self.internal.append(buf);

        if let Some(pos) = self.internal.iter().position(|&b| b == ETX) {
            let frame: Vec<u8> = self.internal.drain(..=pos).collect();
            // remaining bytes (after this frame) go back to buf for the next call
            *buf = std::mem::take(&mut self.internal);
            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::Codec;

    fn codec() -> EtxCodec {
        EtxCodec::new()
    }

    #[test]
    fn returns_none_when_no_etx_yet() {
        let mut c = codec();
        let mut buf = b"hello".to_vec();
        assert!(c.decode(&mut buf).unwrap().is_none());
        assert!(buf.is_empty()); // consumed into internal buffer
    }

    #[test]
    fn returns_frame_when_etx_received() {
        let mut c = codec();
        let mut buf = b"\x0ahello\x0d\x03".to_vec();
        let frame = c.decode(&mut buf).unwrap().unwrap();
        assert_eq!(frame, b"\x0ahello\x0d\x03");
        assert!(buf.is_empty());
    }

    #[test]
    fn handles_data_split_across_two_decode_calls() {
        let mut c = codec();
        let mut part1 = b"\x0ahel".to_vec();
        assert!(c.decode(&mut part1).unwrap().is_none());
        let mut part2 = b"lo\x0d\x03".to_vec();
        let frame = c.decode(&mut part2).unwrap().unwrap();
        assert_eq!(frame, b"\x0ahello\x0d\x03");
    }

    #[test]
    fn handles_two_frames_in_one_buffer() {
        let mut c = codec();
        let mut buf = b"\x0afirst\x03\x0asecond\x03".to_vec();
        let frame1 = c.decode(&mut buf).unwrap().unwrap();
        assert_eq!(frame1, b"\x0afirst\x03");
        // remaining bytes returned in buf for second frame
        let frame2 = c.decode(&mut buf).unwrap().unwrap();
        assert_eq!(frame2, b"\x0asecond\x03");
    }

    #[test]
    fn encode_appends_cr() {
        let c = codec();
        assert_eq!(c.encode(b"W"), b"W\r");
    }

    #[test]
    fn empty_buffer_returns_none() {
        let mut c = codec();
        let mut buf = vec![];
        assert!(c.decode(&mut buf).unwrap().is_none());
    }
}
