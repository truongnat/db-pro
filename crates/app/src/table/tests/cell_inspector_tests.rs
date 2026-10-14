//! Tests for `../cell_inspector.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn pretty_and_tree_json() {
        let pretty = pretty_json(r#"{"a":1,"b":[true]}"#).unwrap();
        assert!(pretty.contains('\n'));
        let lines = json_tree_lines(r#"{"a":1,"b":[true]}"#, 20);
        assert!(lines.iter().any(|l| l.contains("a = 1")));
        assert!(lines.iter().any(|l| l.contains("b[0] = true")));
    }

    #[test]
    fn hex_round_trip_metadata() {
        let raw = "\\xdeadbeef";
        let bytes = decode_bytes_payload(raw).unwrap();
        assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(encode_hex(&bytes), raw);
        assert!(bytes_metadata(raw).contains("4 byte"));
        let b64 = encode_base64(&bytes);
        assert_eq!(decode_bytes_payload(&b64).unwrap(), bytes);
    }

    #[test]
    fn classify_json_and_bytes() {
        assert_eq!(
            classify_cell(&UiCell::Json("{}".into()), "text"),
            CellInspectorKind::Json
        );
        assert_eq!(
            classify_cell(&UiCell::Bytes("\\x00".into()), "bytea"),
            CellInspectorKind::Bytes
        );
    }