#[cfg(test)]
mod tests {
    use crate::trx::msg::Msg;

    #[test]
    fn trx_msg_serializes_with_raw_bytes_body() {
        let msg = Msg {
            id: "m1".to_string(),
            from_id: "sender".to_string(),
            to_ids: vec!["receiver".to_string()],
            type_: "event".to_string(),
            version: 1,
            body: vec![0, 1, 2, 255],
        };

        let serialized = serde_json::to_string(&msg).expect("serialize trx msg");
        let deserialized: Msg = serde_json::from_str(&serialized).expect("deserialize trx msg");

        assert_eq!(msg, deserialized);
        assert!(serialized.contains("\"body\":\"AAEC/w==\""));
    }

    #[test]
    fn trx_msg_defaults_version_to_1() {
        let json =
            r#"{"id":"m1","from_id":"f","to_ids":["t"],"type_":"event","body":"aGk="}"#;
        let msg: Msg = serde_json::from_str(json).expect("deserialize default-version trx msg");
        assert_eq!(msg.version, 1);
        assert_eq!(msg.body, b"hi".to_vec());
    }
}
