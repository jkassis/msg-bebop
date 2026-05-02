use crate::context::Context;
use crate::{
    db::{dbtx_from_ctx, DBTx, DB},
    error::CourierError,
    expiration::{ExpirationHook, NoopExpirationHook},
    idempotency::{IdempotencyResult, IdempotencyStrategy},
    observability::{ObservabilityEvent, ObservabilityRecorder},
    pact::Pact,
    reliable::{CourierWireMsg as Msg, Rx, Tx},
    receipt::Receipt,
};
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{error, info};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::sync::RwLock;
use Result;

lazy_static! {
    pub static ref MSG_ACK_TYPE: String = "Ack".to_string();
    pub static ref MSG_TYPE_TEXT: String = "text".to_string();
    pub static ref MSG_TYPE_COMMAND: String = "command".to_string();
    pub static ref MSG_TYPE_EVENT: String = "event".to_string();
    // Add more message types as needed
}

pub struct DAO {
    db_prefix: String,
}

impl DAO {
    pub fn new(db_prefix: String) -> Self {
        DAO { db_prefix }
    }

    fn key(&self, kind: &str, id: &str) -> String {
        format!("{}:{}:{}", self.db_prefix, kind, id)
    }

    fn prefix(&self, kind: &str) -> String {
        format!("{}:{}:", self.db_prefix, kind)
    }

    fn get_obj<T: serde::de::DeserializeOwned>(
        &self,
        dbtx: &dyn DBTx,
        key: &str,
    ) -> Result<Option<T>, String> {
        let serialized = match dbtx.obj_get(key)? {
            Some(data) => data,
            None => return Ok(None),
        };
        serde_json::from_slice(&serialized)
            .map(Some)
            .map_err(|e| format!("failed to deserialize {}: {}", key, e))
    }
    fn put_obj<T: serde::Serialize>(
        &self,
        dbtx: &dyn DBTx,
        key: &str,
        obj: &T,
    ) -> Result<(), String> {
        let serialized = serde_json::to_vec(obj).map_err(|_| "failed to serialize object")?;
        dbtx.obj_put(key, serialized)
            .map_err(|e| format!("failed to put object: {:?}", e))
    }
    fn del_obj(&self, dbtx: &dyn DBTx, key: &str) -> Result<Option<Vec<u8>>, String> {
        dbtx.obj_del(key)
            .map_err(|e| format!("failed to delete object: {}", e))
    }

    // Key helpers
    pub fn tx_msg_key_make(&self, msg_id: &str) -> String {
        self.key("tm", msg_id)
    }

    pub fn tx_msg_key_parse(&self, key: &str) -> String {
        key.strip_prefix(&format!("{}:tm:", self.db_prefix))
            .unwrap_or(key)
            .to_string()
    }

    pub fn tx_pact_key_make(&self, msg_id: &str) -> String {
        self.key("tp", msg_id)
    }

    pub fn tx_pact_key_parse(&self, key: &str) -> String {
        key.strip_prefix(&format!("{}:tp:", self.db_prefix))
            .unwrap_or(key)
            .to_string()
    }

    pub fn tx_event_key_prefix(&self) -> String {
        self.prefix("te")
    }

    pub fn tx_event_key_make(&self, msg_id: &str, send_at_tick: u64) -> String {
        format!("{}:te:{}:{}", self.db_prefix, send_at_tick, msg_id)
    }

    pub fn tx_event_key_parse(&self, send_event: &str) -> Result<(u64, String), String> {
        let prefix = self.tx_event_key_prefix();
        if !send_event.starts_with(&prefix) {
            return Err(format!(
                "Invalid send_event format: missing prefix '{}': {}",
                prefix, send_event
            ));
        }
        let rest = &send_event[prefix.len()..];
        let mut parts = rest.splitn(2, ':');
        let tick_str = parts.next().ok_or_else(|| "Missing tick".to_string())?;
        let msg_id = parts
            .next()
            .ok_or_else(|| "Missing msg_id".to_string())?
            .to_string();
        let send_at_tick = tick_str
            .parse::<u64>()
            .map_err(|e| format!("failed to parse tick: {}", e))?;
        Ok((send_at_tick, msg_id))
    }

    pub fn rx_msg_key_make(&self, msg_id: &str) -> String {
        self.key("rm", msg_id)
    }

    pub fn rx_msg_key_parse(&self, key: &str) -> String {
        key.strip_prefix(&format!("{}:rm:", self.db_prefix))
            .unwrap_or(key)
            .to_string()
    }

    pub fn rx_receipt_key_prefix(&self) -> String {
        self.prefix("rr")
    }

    pub fn rx_receipt_key_make(&self, msg_id: &str) -> String {
        self.key("rr", msg_id)
    }

    pub fn rx_receipt_key_parse(&self, key: &str) -> String {
        key.strip_prefix(&format!("{}:rr:", self.db_prefix))
            .unwrap_or(key)
            .to_string()
    }

    // Generic get/put/del wrappers for each type
    pub fn rx_msg_get(&self, dbtx: &dyn DBTx, msg_id: &str) -> Result<Option<Msg>, String> {
        self.get_obj(dbtx, &self.rx_msg_key_make(msg_id))
    }

    pub fn rx_msg_put(&self, dbtx: &dyn DBTx, msg: &Msg) -> Result<(), String> {
        self.put_obj(dbtx, &self.rx_msg_key_make(&msg.id), msg)
    }

    pub fn rx_msg_del(&self, dbtx: &dyn DBTx, msg_id: &str) -> Result<Option<Vec<u8>>, String> {
        self.del_obj(dbtx, &self.rx_msg_key_make(msg_id))
    }

    pub fn tx_msg_get(&self, dbtx: &dyn DBTx, msg_id: &str) -> Result<Option<Msg>, String> {
        self.get_obj(dbtx, &self.tx_msg_key_make(msg_id))
    }

    pub fn tx_msg_put(&self, dbtx: &dyn DBTx, msg: &Msg) -> Result<(), String> {
        self.put_obj(dbtx, &self.tx_msg_key_make(&msg.id), msg)
    }

    pub fn tx_msg_del(&self, dbtx: &dyn DBTx, msg_id: &str) -> Result<Option<Vec<u8>>, String> {
        self.del_obj(dbtx, &self.tx_msg_key_make(msg_id))
    }

    pub fn tx_pact_get(&self, dbtx: &dyn DBTx, msg_id: &str) -> Result<Option<Pact>, String> {
        self.get_obj(dbtx, &self.tx_pact_key_make(msg_id))
    }

    pub fn tx_pact_put(&self, dbtx: &dyn DBTx, msg_id: &str, pact: &Pact) -> Result<(), String> {
        self.put_obj(dbtx, &self.tx_pact_key_make(msg_id), pact)
    }

    pub fn tx_pact_del(&self, dbtx: &dyn DBTx, msg_id: &str) -> Result<Option<Vec<u8>>, String> {
        self.del_obj(dbtx, &self.tx_pact_key_make(msg_id))
    }

    pub fn tx_event_del(
        &self,
        dbtx: &dyn DBTx,
        msg_id: &str,
        send_tick: u64,
    ) -> Result<Option<Vec<u8>>, String> {
        self.del_obj(dbtx, &self.tx_event_key_make(msg_id, send_tick))
    }

    pub fn tx_event_put(
        &self,
        dbtx: &dyn DBTx,
        msg_id: &str,
        send_tick: u64,
    ) -> Result<(), String> {
        dbtx.obj_put(&self.tx_event_key_make(msg_id, send_tick), vec![])
    }

    pub fn rx_receipt_get(&self, dbtx: &dyn DBTx, msg_id: &str) -> Result<Option<Receipt>, String> {
        self.get_obj(dbtx, &self.rx_receipt_key_make(msg_id))
    }

    pub fn rx_receipt_put(
        &self,
        dbtx: &dyn DBTx,
        msg_id: &str,
        receipt: &Receipt,
    ) -> Result<(), String> {
        self.put_obj(dbtx, &self.rx_receipt_key_make(msg_id), receipt)
    }

    pub fn rx_receipt_del(&self, dbtx: &dyn DBTx, msg_id: &str) -> Result<Option<Vec<u8>>, String> {
        self.del_obj(dbtx, &self.rx_receipt_key_make(msg_id))
    }
}
pub struct Courier {
    /// The core message delivery state machine for reliable, persistent, and retryable messaging.
    ///
    /// # Construction
    /// To construct a [`Courier`], provide the following:
    ///
    /// - `db`: An implementation of [`crate::db::DB`] for message persistence.
    /// - `sender`: An implementation of [`Tx`] to perform outbound message delivery.
    /// - `handler`: A function or closure to handle inbound messages after receipt.
    /// - `pact_factory`: A closure that produces an initial [`Pact`] for each message during [`Courier::tx`].
    /// - `pact_ticker`: A closure that determines the retry schedule. It is called on each send attempt and returns `Ok(next_tick)` for the next retry, or `Err(reason)` to stop retrying. [`Courier`] will update `Pact.tick_of_last_attempt` automatically.
    ///
    /// This design enables flexible integration with custom storage, retry, and communication strategies.
    id: String,
    db: Arc<dyn DB + Send + Sync>,
    dao: DAO,
    pact_factory: Arc<dyn Fn(&Msg) -> Pact + Send + Sync>,
    pact_ticker: Arc<dyn Fn(&mut Pact, u64) -> Option<u64> + Send + Sync>, // New pact_ticker field
    idempotency_strategy: Arc<dyn IdempotencyStrategy + Send + Sync>,
    recorder: Arc<dyn ObservabilityRecorder + Send + Sync>,
    expiration_hook: Arc<dyn ExpirationHook + Send + Sync>,
    sender: Arc<dyn Tx + Send + Sync>,
    tick_last: AtomicU64,
}

impl Courier {
    /// Public getter for DAO (for testing)
    pub fn dao(&self) -> &DAO {
        &self.dao
    }
}

impl Courier {
    pub fn new(
        id: String,
        db: Arc<dyn DB + Send + Sync>,
        db_prefix: String,
        sender: Arc<dyn Tx + Send + Sync>,
        pact_factory: Arc<dyn Fn(&Msg) -> Pact + Send + Sync>,
        pact_ticker: Arc<dyn Fn(&mut Pact, u64) -> Option<u64> + Send + Sync>,
        idempotency_strategy: Arc<dyn IdempotencyStrategy + Send + Sync>,
        recorder: Arc<dyn ObservabilityRecorder + Send + Sync>,
    ) -> Self {
        Self::new_with_expiration_hook(
            id,
            db,
            db_prefix,
            sender,
            pact_factory,
            pact_ticker,
            idempotency_strategy,
            recorder,
            Arc::new(NoopExpirationHook),
        )
    }

    pub fn new_with_expiration_hook(
        id: String,
        db: Arc<dyn DB + Send + Sync>,
        db_prefix: String,
        sender: Arc<dyn Tx + Send + Sync>,
        pact_factory: Arc<dyn Fn(&Msg) -> Pact + Send + Sync>,
        pact_ticker: Arc<dyn Fn(&mut Pact, u64) -> Option<u64> + Send + Sync>,
        idempotency_strategy: Arc<dyn IdempotencyStrategy + Send + Sync>,
        recorder: Arc<dyn ObservabilityRecorder + Send + Sync>,
        expiration_hook: Arc<dyn ExpirationHook + Send + Sync>,
    ) -> Self {
        Courier {
            id,
            db,
            dao: DAO::new(db_prefix),
            sender,
            pact_factory,
            pact_ticker,
            idempotency_strategy,
            recorder,
            expiration_hook,
            tick_last: AtomicU64::new(0),
        }
    }

    /// Advances the Courier state machine for a given logical tick.
    ///
    /// This method processes all scheduled send events whose tick is due, attempts message delivery,
    /// manages retry scheduling, and expires receipts as needed. It is intended to be called periodically
    /// (e.g., by a timer or event loop) to drive message delivery and cleanup.
    ///
    /// # Arguments
    /// * `ctx` - Shared context for the operation (used for async message sending).
    /// * `dbtx` - An open database transaction for all reads and writes.
    /// * `tick` - The current logical tick (time step) for scheduling.
    ///
    /// # Returns
    /// * `Ok(())` on success, or an error string if any operation fails.
    ///
    /// # Transaction Management
    /// All database operations are performed within the provided transaction. The caller is responsible
    /// for committing or rolling back the transaction as appropriate.
    pub async fn tick(
        &self,
        ctx: Arc<RwLock<Context>>,
        dbtx: &dyn DBTx,
        tick: u64,
    ) -> Result<(), String> {
        info!("Tick started.");
        self.tick_last.store(tick, Relaxed);
        self.recorder.record(ObservabilityEvent {
            tick: Some(tick),
            ..ObservabilityEvent::new("courier.tick.start")
        });

        // handle send events
        let tx_event_keys = dbtx
            .seq_get(&self.dao.tx_event_key_prefix())
            .inspect_err(|e| {
                error!("failed to retrieve tick sequence: {}", e);
            })?;
        for tx_event_key in tx_event_keys {
            info!("Scanning key: {}", tx_event_key); // Debugging log

            // parse the tx_event_key
            let (send_tick, msg_id) = self.dao.tx_event_key_parse(&tx_event_key)?;

            // if send_tick > tick, the send_event is in the future
            if send_tick > tick {
                // we assume the scan happens in order, so we are done ticking
                break;
            }

            // delete the send_event from the tick sequence
            self.dao.tx_event_del(dbtx, &msg_id, send_tick)?;

            // retrieve the msg from the db
            let msg = match self.dao.tx_msg_get(dbtx, &msg_id)? {
                Some(msg) => msg,
                None => {
                    // should never happen...
                    // but what if our db allows a delete within this dbtx?
                    error!("msg_id {} deleted while processing send_event", msg_id);
                    continue;
                }
            };

            // get the pact from the db
            let mut pact: Pact = match self.dao.tx_pact_get(dbtx, &msg_id)? {
                Some(pact) => pact,
                None => {
                    // should never happen...
                    // but what if our db allows a delete within this dbtx?
                    error!(
                        "pact for msg_id {} deleted while processing send_event",
                        msg_id
                    );
                    continue;
                }
            };

            self.recorder.record(ObservabilityEvent {
                msg_id: Some(msg_id.clone()),
                tick: Some(tick),
                try_count: Some(pact.try_count),
                ..ObservabilityEvent::new("courier.tick.send_attempt")
            });

            // should we schedule a next send_event?
            if let Some(next_send_tick) = (self.pact_ticker)(&mut pact, tick) {
                // yes. save a new send event
                self.dao.tx_event_put(dbtx, &msg_id, next_send_tick)?;

                // update and save the pact
                pact.tick_of_last_attempt = tick;
                self.dao.tx_pact_put(dbtx, &msg_id, &pact)?;
            } else {
                // no more retries
                self.expiration_hook.on_expire(dbtx, &msg, &pact, tick)?;
                self.recorder.record(ObservabilityEvent {
                    msg_id: Some(msg.id.clone()),
                    tick: Some(tick),
                    try_count: Some(pact.try_count),
                    ..ObservabilityEvent::new("courier.msg.expired")
                });
                self.dao.tx_msg_del(dbtx, &msg.id)?;
                self.dao.tx_pact_del(dbtx, &msg.id)?;
            }

            // send and await
            if let Err(e) = self.sender.tx(ctx.clone(), &msg).await {
                error!("failed to send message: {}", e);
                self.recorder.record(ObservabilityEvent {
                    msg_id: Some(msg_id.clone()),
                    tick: Some(tick),
                    detail: Some(e),
                    ..ObservabilityEvent::new("courier.tick.send_failure")
                });
            } else {
                self.recorder.record(ObservabilityEvent {
                    msg_id: Some(msg_id),
                    tick: Some(tick),
                    ..ObservabilityEvent::new("courier.tick.send_success")
                });
            }
        }

        self.idempotency_strategy.on_tick(&self.dao, dbtx, tick)?;

        Ok(())
    }
}

#[async_trait]
impl Rx for Courier {
    /// Handles inbound messages and manages their persistence and acknowledgment.
    ///
    /// This method is called when a message is received. It creates and commits a new database transaction
    /// for each inbound message. If the message is an ACK, it deletes the corresponding outbound message and pact.
    /// For standard messages, it stores the message and a receipt, or updates the receipt if already present.
    /// An ACK is always sent in response to a standard message.
    ///
    /// # Arguments
    /// * `ctx` - Shared context for the operation (used for async message sending).
    /// * `msg` - The inbound message to process.
    ///
    /// # Returns
    /// * `Ok(())` on success, or an error string if any operation fails.
    ///
    /// # Transaction Management
    /// A new transaction is created and committed for each call.
    async fn rx(&self, ctx: Arc<RwLock<Context>>, msg: &Msg) -> Result<(), String> {
        let dbtx_arc = self.db.dbtx_create().map_err(|e| e.to_string())?;
        let dbtx = dbtx_arc.as_ref();
        let tick = self.tick_last.load(Relaxed);

        // is the incoming message an ack?
        if msg.type_ == *MSG_ACK_TYPE {
            let acked_msg_id = msg.ack_msg_id.clone().ok_or_else(|| {
                CourierError::caller_bug("invalid ack: missing ack_msg_id").to_string()
            })?;
            let ack_from_id = msg.ack_from_id.clone().ok_or_else(|| {
                CourierError::caller_bug("invalid ack: missing ack_from_id").to_string()
            })?;
            let ack_to_id = msg.ack_to_id.clone().ok_or_else(|| {
                CourierError::caller_bug("invalid ack: missing ack_to_id").to_string()
            })?;
            let ack_version = msg.ack_version.ok_or_else(|| {
                CourierError::caller_bug("invalid ack: missing ack_version").to_string()
            })?;
            if acked_msg_id.is_empty() {
                return Err(CourierError::caller_bug("ack msg_id cannot be empty").to_string());
            }
            if ack_from_id != msg.from_id {
                return Err(CourierError::caller_bug(
                    "invalid ack: ack_from_id does not match envelope from_id",
                )
                .to_string());
            }
            if !msg.to_ids.iter().any(|to_id| to_id == &ack_to_id) {
                return Err(CourierError::caller_bug(
                    "invalid ack: ack_to_id must be in envelope to_ids",
                )
                .to_string());
            }
            if ack_to_id != self.id {
                return Err(CourierError::caller_bug(format!(
                    "invalid ack: ack_to_id {} does not target this courier {}",
                    ack_to_id, self.id
                ))
                .to_string());
            }
            if msg.version != ack_version {
                return Err(CourierError::caller_bug(
                    "invalid ack: envelope version must match ack_version",
                )
                .to_string());
            }
            let mut tx_msg = match self.dao.tx_msg_get(dbtx, &acked_msg_id)? {
                Some(tx_msg) => tx_msg,
                None => return Ok(()),
            };
            if ack_version != tx_msg.version {
                return Err(CourierError::caller_bug(format!(
                    "invalid ack: ack_version {} does not match msg version {}",
                    ack_version, tx_msg.version
                ))
                .to_string());
            }

            let before_len = tx_msg.to_ids.len();
            tx_msg.to_ids.retain(|to_id| to_id != &ack_from_id);
            if tx_msg.to_ids.len() == before_len {
                return Err(CourierError::caller_bug(format!(
                    "invalid ack: {} is not a pending recipient for msg {}",
                    ack_from_id, acked_msg_id
                ))
                .to_string());
            }

            if tx_msg.to_ids.is_empty() {
                self.dao.tx_msg_del(dbtx, &acked_msg_id)?;
                self.dao.tx_pact_del(dbtx, &acked_msg_id)?;
            } else {
                self.dao.tx_msg_put(dbtx, &tx_msg)?;
            }

            self.recorder.record(ObservabilityEvent {
                msg_id: Some(acked_msg_id),
                tick: Some(tick),
                ..ObservabilityEvent::new("courier.ack.processed")
            });
        } else {
            if msg.ack_msg_id.is_some()
                || msg.ack_from_id.is_some()
                || msg.ack_to_id.is_some()
                || msg.ack_version.is_some()
            {
                return Err(CourierError::caller_bug(
                    "invalid message: non-ack message must not include ack correlation fields",
                )
                .to_string());
            }
            match self
                .idempotency_strategy
                .on_rx(&self.dao, dbtx, msg, tick)?
            {
                IdempotencyResult::Duplicate => {
                    self.recorder.record(ObservabilityEvent {
                        msg_id: Some(msg.id.clone()),
                        tick: Some(tick),
                        ..ObservabilityEvent::new("courier.rx.duplicate")
                    });
                }
                IdempotencyResult::New => {
                    // save the inbound message for pickup later
                    self.dao.rx_msg_put(dbtx, &msg)?;
                    self.recorder.record(ObservabilityEvent {
                        msg_id: Some(msg.id.clone()),
                        tick: Some(tick),
                        ..ObservabilityEvent::new("courier.rx.handler_success")
                    });
                }
            }
        }

        // commit the tx (before sending the ack)
        dbtx.commit().map_err(|e| {
            error!("failed to commit rx transaction {}", e);
            e
        })?;

        if msg.type_ != *MSG_ACK_TYPE {
            // send an ack
            let ack_msg = Msg {
                id: uuid::Uuid::new_v4().to_string(),
                from_id: self.id.clone(),
                to_ids: vec![msg.from_id.clone()],
                type_: MSG_ACK_TYPE.clone(),
                body: msg.id.clone(),
                version: msg.version,
                ack_msg_id: Some(msg.id.clone()),
                ack_from_id: Some(self.id.clone()),
                ack_to_id: Some(msg.from_id.clone()),
                ack_version: Some(msg.version),
            };

            // await the send and log errors
            let send_result = self.sender.tx(ctx, &ack_msg).await;
            if let Err(e) = send_result {
                error!("failed to send ack: {}", e);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Tx for Courier {
    /// Schedules a message for delivery and persists its state.
    ///
    /// This method is called to send a message. It writes the message and its initial pact to the database,
    /// and schedules an event for immediate delivery. All operations are performed within the provided transaction.
    /// The transaction is not committed here; the caller is responsible for committing or rolling back as needed.
    ///
    /// # Arguments
    /// * `ctx` - Shared context for the operation (must contain a valid database transaction).
    /// * `msg` - The message to send.
    ///
    /// # Returns
    /// * `Ok(())` on success, or an error string if any operation fails.
    ///
    /// # Transaction Management
    /// All database operations are performed within the transaction from the context.
    async fn tx(&self, ctx: Arc<RwLock<Context>>, msg: &Msg) -> Result<(), String> {
        if msg.version == 0 {
            return Err(
                CourierError::caller_bug("invalid message: version must be >= 1").to_string(),
            );
        }
        if msg.type_ != *MSG_ACK_TYPE
            && (msg.ack_msg_id.is_some()
                || msg.ack_from_id.is_some()
                || msg.ack_to_id.is_some()
                || msg.ack_version.is_some())
        {
            return Err(CourierError::caller_bug(
                "invalid message: non-ack message must not include ack correlation fields",
            )
            .to_string());
        }

        // Retrieve dbtx from context
        let ctx_guard = ctx.read().unwrap();
        let dbtx = dbtx_from_ctx(&ctx_guard).unwrap();
        let dbtx = dbtx.as_ref();

        // save the message to the db
        self.dao.tx_msg_put(dbtx, &msg)?;

        // save a pact
        let pact = (self.pact_factory)(msg);
        self.dao.tx_pact_put(dbtx, &msg.id, &pact)?;

        // schedule an event to send asap
        self.dao.tx_event_put(dbtx, &msg.id, 0)?;
        self.recorder.record(ObservabilityEvent {
            msg_id: Some(msg.id.clone()),
            tick: Some(0),
            ..ObservabilityEvent::new("courier.tx.persisted")
        });

        Ok(())
    }
}

// Manually implement Debug for Courier
impl std::fmt::Debug for Courier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Courier")
            .field("db", &"<DB trait object>")
            .field("comm", &"<Comm trait object>")
            .finish()
    }
}
