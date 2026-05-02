#[cfg(test)]
mod tests {
    use crate::reliable::{CourierAck, CourierMsg};
    use crate::trx::msg::Msg;

    #[test]
    fn courier_msg_wraps_trx_msg_and_serializes() {
        let trx_msg = Msg::new(
            "m1",
            "sender",
            vec!["receiver".to_string()],
            "event",
            b"hello".to_vec(),
        );
        let msg = CourierMsg {
            msg: trx_msg,
            ack: Some(CourierAck {
                acked_msg_id: "m1".to_string(),
                from_id: "receiver".to_string(),
                to_id: "sender".to_string(),
                version: 1,
            }),
        };

        let serialized = serde_json::to_string(&msg).expect("serialize courier msg");
        let deserialized: CourierMsg =
            serde_json::from_str(&serialized).expect("deserialize courier msg");

        assert_eq!(msg, deserialized);
    }
}
