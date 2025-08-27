#[cfg(test)]
mod courier_tests {
    use courier::{
        context::Context,
        db::{dbtx_to_ctx, DB},
        db_sled::SledDB,
        pact::Pact,
        receipt::Receipt,
        tx_sync::SyncTx,
        txrx::*,
        Courier, Msg,
    };
    use std::sync::{Arc, RwLock};

    #[tokio::test]
    async fn test_tx_rx() {
        // Initialize logging
        let _ = env_logger::builder().is_test(true).try_init();

        // Use unique paths for each db
        let tx_db_path = "/tmp/test_tx_rx_tx_db";
        std::fs::remove_dir_all(tx_db_path).ok();
        let rx_db_path = "/tmp/test_tx_rx_rx_db";
        std::fs::remove_dir_all(rx_db_path).ok();

        // use same pact_factory and pact_ticker for tx / rx
        let pact_factory = Arc::new(|_: &Msg| Pact {
            tick_of_last_attempt: 0,
            try_count: 0,
        });
        let pact_ticker = Arc::new(|pact: &mut Pact, tick: u64| -> Option<u64> {
            if pact.try_count > 10 {
                return None;
            }
            pact.try_count = pact.try_count + 1;
            Some(tick + 1)
        });
        let keep_receipt = Arc::new(|_receipt: &Receipt| true);

        // init tx_courier
        let tx_db: Arc<dyn DB + Send + Sync> =
            Arc::new(SledDB::new(tx_db_path).expect("Failed to initialize DbSled for tx_courier"));
        let tx_comm = Arc::new(SyncTx::new());
        let tx_courier = Arc::new(Courier::new(
            "tx".to_string(),
            tx_db.clone(),
            "tx".to_string(),
            tx_comm.clone(),
            pact_factory.clone(),
            pact_ticker.clone(),
            keep_receipt.clone(),
        ));

        // init rx_courier
        let rx_db: Arc<dyn DB + Send + Sync> =
            Arc::new(SledDB::new(rx_db_path).expect("Failed to initialize DbSled for rx_courier"));
        let rx_comm = Arc::new(SyncTx::new());
        let rx_courier = Arc::new(Courier::new(
            "rx".to_string(),
            rx_db.clone(),
            "rx".to_string(),
            rx_comm.clone(),
            pact_factory.clone(),
            pact_ticker.clone(),
            keep_receipt.clone(),
        ));

        // Register each Courier as a receiver on the comm of the other
        tx_comm.set_receiver(rx_courier.clone());
        rx_comm.set_receiver(tx_courier.clone());

        // TX
        let msg = Msg {
            body: "Courier test message".to_string(),
            from_id: "courier_sender".to_string(),
            id: "courier_msg_id".to_string(),
            to_ids: vec!["courier_recipient".to_string()],
            type_: "text".to_string(),
        };
        let tx_txn = tx_db
            .dbtx_create()
            .expect("Failed to create DB transaction for Courier 1");
        let mut tx_ctx = Context::new();
        let tx_ctx_arc = dbtx_to_ctx(&mut tx_ctx, tx_txn.clone());
        tx_courier
            .tx(tx_ctx_arc.clone(), &msg)
            .await
            .expect("Failed to send message from Courier 1");
        tx_txn.commit().expect("tx_txn to commit");

        println!("Message sent via tx_courier");

        // Tick
        let tx_tick = tx_db
            .dbtx_create()
            .expect("Failed to create DB transaction for tx_courier");
        tx_courier
            .tick(tx_ctx_arc.clone(), tx_tick.as_ref(), 0)
            .await
            .expect("Failed to process message on Courier 2");
        tx_tick.commit().expect("tx_tick to commit");

        println!("Message successfully processed via tick on tx_courier");

        // RX
        let rx_txn = rx_db
            .dbtx_create()
            .expect("Failed to create DB transaction for rx_courier");
        let dao = rx_courier.dao();
        let rx_msg = dao
            .rx_msg_get(rx_txn.as_ref(), &msg.id)
            .expect("rx_msg_get success")
            .unwrap();

        assert_eq!(msg, rx_msg);

        println!("Message successfully stored in database of rx_courier");
    }

    #[tokio::test]
    async fn test_durability() {
        use std::time::Duration;

        // TX
        let msg = Msg {
            body: "Courier test message".to_string(),
            from_id: "courier_sender".to_string(),
            id: "courier_msg_id".to_string(),
            to_ids: vec!["courier_recipient".to_string()],
            type_: "text".to_string(),
        };

        // Initialize logging
        let _ = env_logger::builder().is_test(true).try_init();

        // Use unique paths
        let tx_db_path = "/tmp/test_durability_tx_db";
        std::fs::remove_dir_all(tx_db_path).ok();
        let tx_db: Arc<dyn DB + Send + Sync> =
            Arc::new(SledDB::new(tx_db_path).expect("Failed to initialize DbSled for tx_courier"));

        let rx_db_path = "/tmp/test_durability_rx_db";
        let rx_db: Arc<dyn DB + Send + Sync> =
            Arc::new(SledDB::new(rx_db_path).expect("Failed to initialize DbSled for rx_courier"));
        std::fs::remove_dir_all(tx_db_path).ok();

        // use same pact_factory and pact_ticker for tx / rx
        let pact_factory = Arc::new(|_: &Msg| Pact {
            tick_of_last_attempt: 0,
            try_count: 0,
        });
        let pact_ticker = Arc::new(|pact: &mut Pact, tick: u64| -> Option<u64> {
            if pact.try_count > 10 {
                return None;
            }
            pact.tick_of_last_attempt = tick;
            pact.try_count += 1;
            Some(tick + 1)
        });
        let keep_receipt = Arc::new(|_receipt: &Receipt| true);

        {
            // Initialize tx_courier
            let tx_comm = Arc::new(SyncTx::new());
            let tx_courier = Arc::new(Courier::new(
                "tx".to_string(),
                tx_db.clone(),
                "tx".to_string(),
                tx_comm.clone(),
                pact_factory.clone(),
                pact_ticker.clone(),
                keep_receipt.clone(),
            ));

            // Initialize rx_courier
            let rx_comm = Arc::new(SyncTx::new());
            let rx_courier = Arc::new(Courier::new(
                "rx".to_string(),
                rx_db.clone(),
                "rx".to_string(),
                rx_comm.clone(),
                pact_factory.clone(),
                pact_ticker.clone(),
                keep_receipt.clone(),
            ));

            // Register each Courier as a receiver on the comm of the other
            tx_comm.set_receiver(rx_courier.clone());
            rx_comm.set_receiver(tx_courier.clone());

            let tx_txn = tx_db
                .dbtx_create()
                .expect("Failed to create DB transaction for Courier 1");

            let mut tx_ctx = Context::new();
            let tx_ctx_arc = dbtx_to_ctx(&mut tx_ctx, tx_txn.clone());
            tx_courier
                .tx(tx_ctx_arc.clone(), &msg)
                .await
                .expect("Failed to send message from Courier 1");

            println!("Message sent via tx_courier");

            // Close tx_courier and wait for lock release
            tx_db.flush().expect("Failed to close tx_courier database");

            drop(tx_courier);
            drop(tx_txn);
            drop(tx_comm);

            drop(rx_courier);
            drop(rx_comm);

            // Debug print to confirm all handles dropped
            println!("All DB, courier, and comm handles dropped. Waiting for OS lock release...");
        }

        // Check for background threads (debug)
        let thread_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(0);
        println!(
            "[DEBUG] Number of available parallel threads after drop: {}",
            thread_count
        );

        // let lock_file = File::open("/tmp/test_durable_tx_db/ai").expect("Failed to open lock file");
        // lock_file
        //     .lock_exclusive()
        //     .expect("Failed to acquire exclusive lock");
        // lock_file
        //     .unlock()
        //     .expect("Failed to release exclusive lock");

        // Reinitialize tx_courier
        let tx_comm = Arc::new(SyncTx::new());
        let tx_courier = Arc::new(Courier::new(
            "tx".to_string(),
            tx_db.clone(),
            "tx".to_string(),
            tx_comm.clone(),
            pact_factory.clone(),
            pact_ticker.clone(),
            keep_receipt.clone(),
        ));
        let tx_ctx = Arc::new(RwLock::new(Context::new()));

        // Initialize rx_courier
        let rx_comm = Arc::new(SyncTx::new());
        let rx_courier = Arc::new(Courier::new(
            "rx".to_string(),
            rx_db.clone(),
            "rx".to_string(),
            rx_comm.clone(),
            pact_factory.clone(),
            pact_ticker.clone(),
            keep_receipt.clone(),
        ));

        // Register each Courier as a receiver on the comm of the other
        tx_comm.set_receiver(rx_courier.clone());
        rx_comm.set_receiver(tx_courier.clone());

        // Tick
        let tx_tick = tx_db
            .dbtx_create()
            .expect("Failed to create DB transaction for tx_courier");
        tx_courier
            .tick(tx_ctx.clone(), tx_tick.as_ref(), 0)
            .await
            .expect("tick to succeed");

        // RX
        let rx_txn = rx_db
            .dbtx_create()
            .expect("Failed to create DB transaction for rx_courier");
        let dao = rx_courier.dao();

        let key = dao.rx_msg_key_make(&msg.id);
        let rx_msg: Msg = serde_json::from_slice(&rx_txn.obj_get(&key).unwrap().unwrap()).unwrap();
        assert_eq!(msg, rx_msg);

        println!("Message successfully stored in database of rx_courier");

        // Cleanup
        rx_db.flush().expect("Failed to close rx_courier database");
    }
}
