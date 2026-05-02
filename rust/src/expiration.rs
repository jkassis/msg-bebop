use crate::{db::DBTx, reliable::CourierWireMsg, Pact};
use serde::{Deserialize, Serialize};

pub trait ExpirationHook: Send + Sync {
    fn on_expire(
        &self,
        dbtx: &dyn DBTx,
        msg: &CourierWireMsg,
        pact: &Pact,
        tick: u64,
    ) -> Result<(), String>;
}

pub struct NoopExpirationHook;

impl ExpirationHook for NoopExpirationHook {
    fn on_expire(
        &self,
        _dbtx: &dyn DBTx,
        _msg: &CourierWireMsg,
        _pact: &Pact,
        _tick: u64,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpiredEnvelope {
    pub msg: CourierWireMsg,
    pub pact: Pact,
    pub expired_at_tick: u64,
}

pub struct DlqExpirationHook {
    pub key_prefix: String,
}

impl DlqExpirationHook {
    pub fn new(key_prefix: String) -> Self {
        Self { key_prefix }
    }
}

impl ExpirationHook for DlqExpirationHook {
    fn on_expire(
        &self,
        dbtx: &dyn DBTx,
        msg: &CourierWireMsg,
        pact: &Pact,
        tick: u64,
    ) -> Result<(), String> {
        let key = format!("{}:expired:{}:{}", self.key_prefix, tick, msg.id);
        let payload = ExpiredEnvelope {
            msg: msg.clone(),
            pact: pact.clone(),
            expired_at_tick: tick,
        };
        let bytes = serde_json::to_vec(&payload)
            .map_err(|e| format!("failed to serialize DLQ payload: {e}"))?;
        dbtx.obj_put(&key, bytes)
            .map_err(|e| format!("failed to write DLQ payload: {e}"))
    }
}
