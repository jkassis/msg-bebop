use crate::context::Context;
/// An implementation of the `Tx` trait intended primarily for testing purposes.
///
/// When a receiver is set using [`set_receiver`], calls to [`tx`] will immediately and synchronously
/// invoke the `rx` method of the receiver with the provided message. This allows for direct, in-process
/// message delivery without any asynchronous or networked transport.
///
/// # Note
/// - This implementation is not intended for production use.
/// - If no receiver is set, [`tx`] will return an error.
use crate::msg::Msg;
use crate::txrx::{Rx, Tx};
use async_trait::async_trait;
use std::sync::{Arc, RwLock};

pub struct SyncTx {
    receiver: RwLock<Option<Arc<dyn Rx + Send + Sync>>>,
}

impl SyncTx {
    pub fn new() -> Self {
        SyncTx {
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
    async fn tx(&self, ctx: Arc<RwLock<Context>>, msg: &Msg) -> Result<(), String> {
        print!("SyncTxRx::rx");
        let receiver_opt = {
            let receiver_guard = self.receiver.read().unwrap();
            receiver_guard.clone()
        };
        if let Some(receiver) = receiver_opt {
            receiver.rx(ctx, &msg).await // Pass the Context instance and a reference to Msg
        } else {
            Err("Receiver not set".to_string())
        }
    }
}
