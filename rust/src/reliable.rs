use crate::trx::msg::Msg as TrxMsg;
use serde::{Deserialize, Serialize};

pub use crate::courier::Courier;
pub use crate::expiration::{DlqExpirationHook, ExpirationHook, ExpiredEnvelope, NoopExpirationHook};
pub use crate::idempotency::{
    validate_receipt_horizon, IdempotencyResult, IdempotencyStrategy,
    ReceiptIdempotencyStrategy, ReceiptWindowIdempotencyStrategy,
};
pub use crate::observability::{NoopObservabilityRecorder, ObservabilityEvent, ObservabilityRecorder};
pub use crate::pact::Pact;
pub use crate::receipt::Receipt;

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
}
