use crate::{Codec, Protocol, ScaleError, Transport};

pub struct Scale<T: Transport, C: Codec, P: Protocol> {
    pub transport: T,
    codec: C,
    protocol: P,
}

impl<T: Transport, C: Codec, P: Protocol> Scale<T, C, P> {
    pub fn new(transport: T, codec: C, protocol: P) -> Self {
        Self {
            transport,
            codec,
            protocol,
        }
    }

    pub fn send(&mut self, cmd: P::Command) -> Result<P::Response, ScaleError> {
        let bytes = self.protocol.encode_command(&cmd);
        let frame_out = self.codec.encode(&bytes);
        std::io::Write::write_all(&mut self.transport, &frame_out)?;
        self.transport.flush_output()?;

        let frame_in = self.read_frame()?;
        self.protocol.decode_response(&cmd, &frame_in)
    }

    fn read_frame(&mut self) -> Result<Vec<u8>, ScaleError> {
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            match self.transport.read(&mut byte) {
                Ok(0) => {
                    return Err(ScaleError::FramingError(
                        "connection closed before ETX".into(),
                    ))
                }
                Ok(_) => {
                    buf.push(byte[0]);
                    if let Some(frame) = self.codec.decode(&mut buf)? {
                        return Ok(frame);
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EtxCodec, MockTransport};

    struct EchoProtocol;

    #[derive(Clone)]
    struct EchoCommand(u8);

    impl crate::Command for EchoCommand {
        fn command_byte(&self) -> u8 {
            self.0
        }
    }

    impl crate::Protocol for EchoProtocol {
        type Command = EchoCommand;
        type Response = Vec<u8>;

        fn encode_command(&self, cmd: &EchoCommand) -> Vec<u8> {
            vec![cmd.0]
        }
        fn decode_response(&self, _cmd: &EchoCommand, frame: &[u8]) -> Result<Vec<u8>, ScaleError> {
            Ok(frame.to_vec())
        }
    }

    #[test]
    fn send_writes_encoded_command_and_returns_decoded_response() {
        let response = b"hello\x03".to_vec();
        let transport = MockTransport::with_response(response.clone());
        let mut scale = Scale::new(transport, EtxCodec::new(), EchoProtocol);
        let result = scale.send(EchoCommand(b'W')).unwrap();
        assert_eq!(result, b"hello\x03");
        // codec wraps command with CR
        assert_eq!(scale.transport.written(), b"W\r");
    }

    #[test]
    fn send_returns_framing_error_when_transport_closes_early() {
        let transport = MockTransport::with_response(vec![0x0A, 0x41]); // no ETX
        let mut scale = Scale::new(transport, EtxCodec::new(), EchoProtocol);
        let err = scale.send(EchoCommand(b'W')).unwrap_err();
        assert!(matches!(err, ScaleError::FramingError(_)));
    }
}
