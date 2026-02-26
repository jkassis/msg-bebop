#[cfg(test)]
mod tests {
    use crate::validate_receipt_horizon;
    use crate::Msg;
    use crate::Pact;
    use crate::{CourierError, ErrorCategory};

    #[tokio::test]
    async fn test_msg_serialization() {
        let msg = Msg {
            body: "Hello, World!".to_string(),
            from_id: "sender123".to_string(),
            id: "msg123".to_string(),
            to_ids: vec!["recipient1".to_string(), "recipient2".to_string()],
            type_: "text".to_string(),
            version: 1,
            ack_msg_id: None,
            ack_from_id: None,
            ack_to_id: None,
            ack_version: None,
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Msg = serde_json::from_str(&serialized).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[tokio::test]
    async fn test_msg_deserialize_defaults_version_to_1() {
        let json = r#"{"id":"m1","from_id":"f","to_ids":["t"],"type_":"text","body":"b"}"#;
        let msg: Msg = serde_json::from_str(json).unwrap();
        assert_eq!(msg.version, 1);
        assert!(msg.ack_msg_id.is_none());
        assert!(msg.ack_from_id.is_none());
        assert!(msg.ack_to_id.is_none());
        assert!(msg.ack_version.is_none());
    }

    #[tokio::test]
    async fn test_pact_serialization() {
        let pact = Pact {
            tick_of_last_attempt: 100,
            try_count: 3,
        };
        let serialized = serde_json::to_string(&pact).unwrap();
        let deserialized: Pact = serde_json::from_str(&serialized).unwrap();
        assert_eq!(pact, deserialized);
    }

    #[tokio::test]
    async fn test_receipt_horizon_guardrail() {
        assert!(validate_receipt_horizon(100, 50).is_ok());
        assert!(validate_receipt_horizon(50, 50).is_ok());
        let err = validate_receipt_horizon(49, 50).expect_err("must reject invalid horizon");
        assert!(err.contains("caller_bug"));
    }

    #[tokio::test]
    async fn test_typed_error_display_prefix() {
        let err = CourierError::retryable("temporary db timeout");
        assert_eq!(err.category, ErrorCategory::Retryable);
        assert_eq!(err.to_string(), "retryable: temporary db timeout");
    }
}
