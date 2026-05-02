#[cfg(test)]
mod tests {
    use crate::db::DBTx;
    use crate::db_sled::{SledDB, SledDBTx};
    use crate::DB;
    use crate::reliable::CourierWireMsg;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_db_operations() {
        // Clean up the temporary database directory
        let db_path = "/tmp/test_db";
        std::fs::remove_dir_all(db_path).ok();

        // Initialize SledDB with a temporary path
        let db = SledDB::new(db_path).expect("Failed to initialize SledDB");
        let db = Arc::new(db);

        // Test storing and retrieving a message in the database
        let msg = CourierWireMsg {
            body: "Test message".to_string(),
            from_id: "sender123".to_string(),
            id: "msg123".to_string(),
            to_ids: vec!["recipient1".to_string()],
            type_: "text".to_string(),
            version: 1,
            ack_msg_id: None,
            ack_from_id: None,
            ack_to_id: None,
            ack_version: None,
        };

        let dbtx = db.dbtx_create().expect("Failed to create DB transaction");
        dbtx.obj_put(&msg.id, serde_json::to_vec(&msg).unwrap())
            .expect("Failed to store message");
        dbtx.commit().expect("Failed to commit transaction");

        println!("Message stored in database");

        let dbtx = db.dbtx_create().expect("Failed to create DB transaction");
        let retrieved_msg: CourierWireMsg =
            serde_json::from_slice(&dbtx.obj_get(&msg.id).unwrap().unwrap()).unwrap();
        assert_eq!(msg, retrieved_msg);

        println!("Message retrieved from database");
    }

    #[test]
    fn test_seq_get() {
        let db_path = "/tmp/test_sled_db";
        std::fs::remove_dir_all(db_path).ok();

        let db = sled::open(db_path).expect("Failed to open sled database");
        let tree = db.open_tree("default").expect("Failed to open tree");

        // Insert test data
        tree.insert("PREFIX_TICK:1", b"value1")
            .expect("Failed to insert key 1");
        tree.insert("PREFIX_TICK:2", b"value2")
            .expect("Failed to insert key 2");
        tree.insert("PREFIX_TICK:3", b"value3")
            .expect("Failed to insert key 3");

        // Test seq_get
        let sled_db_tx = SledDBTx::new(tree);
        let keys: Vec<String> = sled_db_tx
            .seq_get("PREFIX_TICK")
            .expect("Failed to get sequence")
            .collect();

        assert_eq!(
            keys,
            vec!["PREFIX_TICK:1", "PREFIX_TICK:2", "PREFIX_TICK:3"]
        );

        println!("seq_get test passed");
    }
}
