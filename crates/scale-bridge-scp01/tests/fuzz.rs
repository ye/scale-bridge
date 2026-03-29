use proptest::prelude::*;
use scale_bridge_core::Protocol;
use scale_bridge_scp01::{NciCommand, NciProtocol};

proptest! {
    #[test]
    fn weight_parser_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..256)) {
        let p = NciProtocol;
        let _ = p.decode_response(&NciCommand::Weight, &bytes);
    }

    #[test]
    fn high_resolution_parser_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..256)) {
        let p = NciProtocol;
        let _ = p.decode_response(&NciCommand::HighResolution, &bytes);
    }

    #[test]
    fn status_parser_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..32)) {
        let _ = scale_bridge_scp01::parser::status::parse_status_bytes(&bytes);
    }

    #[test]
    fn about_parser_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..128)) {
        let p = NciProtocol;
        let _ = p.decode_response(&NciCommand::About, &bytes);
    }

    #[test]
    fn diagnostic_parser_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..256)) {
        let p = NciProtocol;
        let _ = p.decode_response(&NciCommand::Diagnostic, &bytes);
    }

    #[test]
    fn metrology_parser_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..128)) {
        let p = NciProtocol;
        let _ = p.decode_response(&NciCommand::Metrology, &bytes);
    }

    #[test]
    fn unrecognized_command_response_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..64)) {
        // Any frame starting with '?' should return UnrecognizedCommand, not panic
        let p = NciProtocol;
        let mut frame = vec![b'?'];
        frame.extend_from_slice(&bytes);
        let result = p.decode_response(&NciCommand::Weight, &frame);
        if let Ok(resp) = result {
            assert!(matches!(resp, scale_bridge_scp01::NciResponse::UnrecognizedCommand));
        }
    }
}
