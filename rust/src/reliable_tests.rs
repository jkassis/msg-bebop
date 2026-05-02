#[cfg(test)]
mod tests {
    use crate::reliable::{CourierAck, CourierMsg};
    use crate::rustie::msg::msg::Msg as LegacyCourierWireMsg;
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

    #[test]
    fn courier_msg_round_trips_through_legacy_wire_shape() {
        let legacy = LegacyCourierWireMsg {
            body: "hello".to_string(),
            from_id: "sender".to_string(),
            id: "m1".to_string(),
            to_ids: vec!["receiver".to_string()],
            type_: "Ack".to_string(),
            version: 1,
            ack_msg_id: Some("m0".to_string()),
            ack_from_id: Some("receiver".to_string()),
            ack_to_id: Some("sender".to_string()),
            ack_version: Some(1),
        };

        let courier = CourierMsg::from_legacy_wire_msg(&legacy);
        let encoded = courier
            .try_to_legacy_wire_msg()
            .expect("encode legacy courier wire msg");

        assert_eq!(legacy, encoded);
    }

    #[test]
    fn courier_msg_rejects_non_utf8_when_encoding_legacy_wire_shape() {
        let courier = CourierMsg::new(Msg::new(
            "m1",
            "sender",
            vec!["receiver".to_string()],
            "event",
            vec![0xff, 0xfe],
        ))
        .with_ack(CourierAck {
            acked_msg_id: "m0".to_string(),
            from_id: "receiver".to_string(),
            to_id: "sender".to_string(),
            version: 1,
        });

        let err = courier
            .try_to_legacy_wire_msg()
            .expect_err("non-utf8 body must fail for legacy wire shape");
        assert!(err.contains("utf-8"));
    }
}
