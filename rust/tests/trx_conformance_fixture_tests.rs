use base64::Engine;
use courier::trx::msg::Msg;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Suite {
    message_decode: Vec<MessageDecodeCase>,
}

#[derive(Debug, Deserialize)]
struct MessageDecodeCase {
    name: String,
    input: serde_json::Value,
    expected: MessageDecodeExpected,
}

#[derive(Debug, Deserialize)]
struct MessageDecodeExpected {
    version: u16,
    body_base64: String,
}

#[test]
fn trx_conformance_message_decode_cases() {
    let raw = std::fs::read_to_string("../conformance/fixtures/trx_suite.v1.json")
        .expect("read trx conformance fixture");
    let suite: Suite = serde_json::from_str(&raw).expect("parse trx conformance fixture");

    for case in suite.message_decode {
        let msg: Msg = serde_json::from_value(case.input).expect("decode trx message");
        assert_eq!(msg.version, case.expected.version, "case: {}", case.name);
        assert_eq!(
            base64::engine::general_purpose::STANDARD.encode(msg.body),
            case.expected.body_base64,
            "case: {}",
            case.name
        );
    }
}
