#[cfg(test)]
mod tests {
    use crate::Msg;
    use crate::Pact;

    #[tokio::test]
    async fn test_msg_serialization() {
        let msg = Msg {
            body: "Hello, World!".to_string(),
            from_id: "sender123".to_string(),
            id: "msg123".to_string(),
            to_ids: vec!["recipient1".to_string(), "recipient2".to_string()],
            type_: "text".to_string(),
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Msg = serde_json::from_str(&serialized).unwrap();
        assert_eq!(msg, deserialized);
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
}
