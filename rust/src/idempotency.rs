use crate::{
    courier::DAO, db::DBTx, error::CourierError, receipt::Receipt, reliable::CourierWireMsg,
};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdempotencyResult {
    New,
    Duplicate,
}

pub trait IdempotencyStrategy: Send + Sync {
    fn on_rx(
        &self,
        dao: &DAO,
        dbtx: &dyn DBTx,
        msg: &CourierWireMsg,
        tick: u64,
    ) -> Result<IdempotencyResult, String>;
    fn on_tick(&self, dao: &DAO, dbtx: &dyn DBTx, tick: u64) -> Result<(), String>;
}

pub struct ReceiptIdempotencyStrategy {
    keep_receipt: Arc<dyn Fn(&Receipt) -> bool + Send + Sync>,
}

impl ReceiptIdempotencyStrategy {
    pub fn new(keep_receipt: Arc<dyn Fn(&Receipt) -> bool + Send + Sync>) -> Self {
        Self { keep_receipt }
    }
}

pub fn validate_receipt_horizon(
    keep_receipt_ticks: u64,
    max_retry_horizon_ticks: u64,
) -> Result<(), String> {
    if keep_receipt_ticks < max_retry_horizon_ticks {
        return Err(CourierError::caller_bug(format!(
            "invalid receipt strategy: keep_receipt_ticks ({keep_receipt_ticks}) < max_retry_horizon_ticks ({max_retry_horizon_ticks})"
        ))
        .to_string());
    }
    Ok(())
}

pub struct ReceiptWindowIdempotencyStrategy {
    keep_receipt_ticks: u64,
}

impl ReceiptWindowIdempotencyStrategy {
    pub fn new(
        keep_receipt_ticks: u64,
        max_retry_horizon_ticks: Option<u64>,
    ) -> Result<Self, String> {
        if let Some(max_retry_horizon_ticks) = max_retry_horizon_ticks {
            validate_receipt_horizon(keep_receipt_ticks, max_retry_horizon_ticks)?;
        }
        Ok(Self { keep_receipt_ticks })
    }
}

impl IdempotencyStrategy for ReceiptIdempotencyStrategy {
    fn on_rx(
        &self,
        dao: &DAO,
        dbtx: &dyn DBTx,
        msg: &CourierWireMsg,
        tick: u64,
    ) -> Result<IdempotencyResult, String> {
        if let Some(mut receipt) = dao.rx_receipt_get(dbtx, &msg.id)? {
            receipt.tick_acked_last = tick;
            dao.rx_receipt_put(dbtx, &msg.id, &receipt)?;
            return Ok(IdempotencyResult::Duplicate);
        }

        let receipt = Receipt {
            tick_acked_first: tick,
            tick_acked_last: tick,
        };
        dao.rx_receipt_put(dbtx, &msg.id, &receipt)?;
        Ok(IdempotencyResult::New)
    }

    fn on_tick(&self, dao: &DAO, dbtx: &dyn DBTx, tick: u64) -> Result<(), String> {
        let rx_receipt_keys = dbtx
            .seq_get(&dao.rx_receipt_key_prefix())
            .map_err(|e| format!("failed to retrieve receipt sequence: {}", e))?;

        for rx_receipt_key in rx_receipt_keys {
            let msg_id = dao.rx_receipt_key_parse(&rx_receipt_key);
            let mut receipt = match dao.rx_receipt_get(dbtx, &msg_id)? {
                Some(r) => r,
                None => continue,
            };

            if (self.keep_receipt)(&receipt) {
                receipt.tick_acked_last = tick;
                dao.rx_receipt_put(dbtx, &msg_id, &receipt)?;
            } else {
                dao.rx_receipt_del(dbtx, &msg_id)?;
            }
        }

        Ok(())
    }
}

impl IdempotencyStrategy for ReceiptWindowIdempotencyStrategy {
    fn on_rx(
        &self,
        dao: &DAO,
        dbtx: &dyn DBTx,
        msg: &CourierWireMsg,
        tick: u64,
    ) -> Result<IdempotencyResult, String> {
        if let Some(mut receipt) = dao.rx_receipt_get(dbtx, &msg.id)? {
            receipt.tick_acked_last = tick;
            dao.rx_receipt_put(dbtx, &msg.id, &receipt)?;
            return Ok(IdempotencyResult::Duplicate);
        }

        let receipt = Receipt {
            tick_acked_first: tick,
            tick_acked_last: tick,
        };
        dao.rx_receipt_put(dbtx, &msg.id, &receipt)?;
        Ok(IdempotencyResult::New)
    }

    fn on_tick(&self, dao: &DAO, dbtx: &dyn DBTx, tick: u64) -> Result<(), String> {
        let rx_receipt_keys = dbtx
            .seq_get(&dao.rx_receipt_key_prefix())
            .map_err(|e| format!("failed to retrieve receipt sequence: {}", e))?;

        for rx_receipt_key in rx_receipt_keys {
            let msg_id = dao.rx_receipt_key_parse(&rx_receipt_key);
            let mut receipt = match dao.rx_receipt_get(dbtx, &msg_id)? {
                Some(r) => r,
                None => continue,
            };

            let age = tick.saturating_sub(receipt.tick_acked_first);
            if age <= self.keep_receipt_ticks {
                receipt.tick_acked_last = tick;
                dao.rx_receipt_put(dbtx, &msg_id, &receipt)?;
            } else {
                dao.rx_receipt_del(dbtx, &msg_id)?;
            }
        }

        Ok(())
    }
}
