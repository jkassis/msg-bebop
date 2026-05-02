use courier::Msg;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Suite {
    message_decode: Vec<MessageDecodeCase>,
    tx_validate: Vec<TxValidateCase>,
    ack_apply: Vec<AckApplyCase>,
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
}

#[derive(Debug, Deserialize)]
struct TxValidateCase {
    name: String,
    msg: Msg,
    expected_error_code: String,
}

#[derive(Debug, Deserialize)]
struct AckApplyCase {
    name: String,
    courier_id: String,
    tx_msg: Msg,
    ack_msg: Msg,
    expected: Option<AckExpected>,
    expected_error_code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AckExpected {
    deleted: bool,
    remaining_to_ids: Vec<String>,
}

struct AckApplyResult {
    deleted: bool,
    remaining_to_ids: Vec<String>,
}

fn load_suite() -> Suite {
    let raw = std::fs::read_to_string("../conformance/fixtures/suite.v1.json")
        .expect("read conformance fixture");
    serde_json::from_str(&raw).expect("parse conformance fixture")
}

fn validate_tx_msg(msg: &Msg) -> Result<(), String> {
    if msg.version == 0 {
        return Err("invalid_msg_version".to_string());
    }
    if msg.type_ != "Ack"
        && (msg.ack_msg_id.is_some()
            || msg.ack_from_id.is_some()
            || msg.ack_to_id.is_some()
            || msg.ack_version.is_some())
    {
        return Err("non_ack_has_ack_fields".to_string());
    }
    Ok(())
}

fn apply_ack(courier_id: &str, tx_msg: &Msg, ack_msg: &Msg) -> Result<AckApplyResult, String> {
    let ack_msg_id = ack_msg
        .ack_msg_id
        .as_ref()
        .ok_or_else(|| "missing_ack_msg_id".to_string())?;
    let ack_from_id = ack_msg
        .ack_from_id
        .as_ref()
        .ok_or_else(|| "missing_ack_from_id".to_string())?;
    let ack_to_id = ack_msg
        .ack_to_id
        .as_ref()
        .ok_or_else(|| "missing_ack_to_id".to_string())?;
    let ack_version = ack_msg
        .ack_version
        .ok_or_else(|| "missing_ack_version".to_string())?;

    if ack_msg.from_id != *ack_from_id {
        return Err("ack_from_mismatch".to_string());
    }
    if !ack_msg.to_ids.iter().any(|to_id| to_id == ack_to_id) {
        return Err("ack_to_missing_in_envelope".to_string());
    }
    if ack_to_id != courier_id {
        return Err("ack_to_wrong_courier".to_string());
    }
    if ack_msg.version != ack_version {
        return Err("ack_envelope_version_mismatch".to_string());
    }
    if ack_msg_id != &tx_msg.id {
        return Err("ack_msg_id_mismatch".to_string());
    }
    if ack_version != tx_msg.version {
        return Err("ack_version_mismatch".to_string());
    }

    let mut remaining: Vec<String> = tx_msg
        .to_ids
        .iter()
        .filter(|to_id| *to_id != ack_from_id)
        .cloned()
        .collect();
    if remaining.len() == tx_msg.to_ids.len() {
        return Err("ack_from_not_pending".to_string());
    }
    remaining.sort();

    Ok(AckApplyResult {
        deleted: remaining.is_empty(),
        remaining_to_ids: remaining,
    })
}

#[test]
fn conformance_message_decode_cases() {
    let suite = load_suite();
    for case in suite.message_decode {
        let msg: Msg = serde_json::from_value(case.input).expect("decode message");
        assert_eq!(msg.version, case.expected.version, "case: {}", case.name);
    }
}

#[test]
fn conformance_tx_validate_cases() {
    let suite = load_suite();
    for case in suite.tx_validate {
        let err = validate_tx_msg(&case.msg).expect_err("expected tx validation failure");
        assert_eq!(err, case.expected_error_code, "case: {}", case.name);
    }
}

#[test]
fn conformance_ack_apply_cases() {
    let suite = load_suite();
    for case in suite.ack_apply {
        let actual = apply_ack(&case.courier_id, &case.tx_msg, &case.ack_msg);
        match (actual, case.expected, case.expected_error_code) {
            (Ok(result), Some(expected), None) => {
                let mut got = result.remaining_to_ids;
                let mut want = expected.remaining_to_ids;
                got.sort();
                want.sort();
                assert_eq!(result.deleted, expected.deleted, "case: {}", case.name);
                assert_eq!(got, want, "case: {}", case.name);
            }
            (Err(err), None, Some(expected_error_code)) => {
                assert_eq!(err, expected_error_code, "case: {}", case.name);
            }
            _ => panic!("invalid fixture expectation shape for case {}", case.name),
        }
    }
}
