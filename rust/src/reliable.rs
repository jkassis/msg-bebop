use crate::context::Context;
use crate::trx::msg::Msg as TrxMsg;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

pub use crate::courier::Courier;
pub use crate::expiration::{DlqExpirationHook, ExpirationHook, ExpiredEnvelope, NoopExpirationHook};
pub use crate::idempotency::{
    validate_receipt_horizon, IdempotencyResult, IdempotencyStrategy,
    ReceiptIdempotencyStrategy, ReceiptWindowIdempotencyStrategy,
};
pub use crate::observability::{NoopObservabilityRecorder, ObservabilityEvent, ObservabilityRecorder};
pub use crate::pact::Pact;
pub use crate::receipt::Receipt;

fn default_msg_version() -> u16 {
    1
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CourierWireMsg {
    pub body: String,
    pub from_id: String,
    pub id: String,
    pub to_ids: Vec<String>,
    pub type_: String,
    #[serde(default = "default_msg_version")]
    pub version: u16,
    #[serde(default)]
    pub ack_msg_id: Option<String>,
    #[serde(default)]
    pub ack_from_id: Option<String>,
    #[serde(default)]
    pub ack_to_id: Option<String>,
    #[serde(default)]
    pub ack_version: Option<u16>,
}

#[async_trait]
pub trait Tx: Send + Sync {
    async fn tx(&self, ctx: Arc<RwLock<Context>>, msg: &CourierWireMsg) -> Result<(), String>;
}

#[async_trait]
pub trait Rx: Send + Sync {
    async fn rx(&self, ctx: Arc<RwLock<Context>>, msg: &CourierWireMsg) -> Result<(), String>;
}

pub struct SyncTx {
    receiver: RwLock<Option<Arc<dyn Rx + Send + Sync>>>,
}

impl SyncTx {
    pub fn new() -> Self {
        Self {
            receiver: RwLock::new(None),
        }
    }

    pub fn set_receiver(&self, receiver: Arc<dyn Rx + Send + Sync>) {
        let mut receiver_guard = self.receiver.write().unwrap();
        *receiver_guard = Some(receiver);
    }
}

#[async_trait]
impl Tx for SyncTx {
    async fn tx(&self, ctx: Arc<RwLock<Context>>, msg: &CourierWireMsg) -> Result<(), String> {
        let receiver_opt = {
            let receiver_guard = self.receiver.read().unwrap();
            receiver_guard.clone()
        };
        if let Some(receiver) = receiver_opt {
            receiver.rx(ctx, msg).await
        } else {
            Err("Receiver not set".to_string())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierAck {
    pub acked_msg_id: String,
    pub from_id: String,
    pub to_id: String,
    pub version: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierMsg {
    pub msg: TrxMsg,
    #[serde(default)]
    pub ack: Option<CourierAck>,
}

impl CourierMsg {
    pub fn new(msg: TrxMsg) -> Self {
        Self { msg, ack: None }
    }

    pub fn with_ack(mut self, ack: CourierAck) -> Self {
        self.ack = Some(ack);
        self
    }

    pub fn from_wire_msg(msg: &CourierWireMsg) -> Self {
        let ack = match (
            msg.ack_msg_id.as_ref(),
            msg.ack_from_id.as_ref(),
            msg.ack_to_id.as_ref(),
            msg.ack_version,
        ) {
            (Some(acked_msg_id), Some(from_id), Some(to_id), Some(version)) => Some(CourierAck {
                acked_msg_id: acked_msg_id.clone(),
                from_id: from_id.clone(),
                to_id: to_id.clone(),
                version,
            }),
            _ => None,
        };

        Self {
            msg: TrxMsg {
                id: msg.id.clone(),
                from_id: msg.from_id.clone(),
                to_ids: msg.to_ids.clone(),
                type_: msg.type_.clone(),
                version: msg.version,
                body: msg.body.as_bytes().to_vec(),
            },
            ack,
        }
    }

    pub fn try_to_wire_msg(&self) -> Result<CourierWireMsg, String> {
        let body = String::from_utf8(self.msg.body.clone())
            .map_err(|_| "courier legacy wire msg requires utf-8 body".to_string())?;

        Ok(CourierWireMsg {
            body,
            from_id: self.msg.from_id.clone(),
            id: self.msg.id.clone(),
            to_ids: self.msg.to_ids.clone(),
            type_: self.msg.type_.clone(),
            version: self.msg.version,
            ack_msg_id: self.ack.as_ref().map(|ack| ack.acked_msg_id.clone()),
            ack_from_id: self.ack.as_ref().map(|ack| ack.from_id.clone()),
            ack_to_id: self.ack.as_ref().map(|ack| ack.to_id.clone()),
            ack_version: self.ack.as_ref().map(|ack| ack.version),
        })
    }
}
