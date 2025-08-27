use crate::context::Context;
use crate::msg::Msg;
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::RwLock;

#[async_trait]
pub trait Tx {
    async fn tx(&self, ctx: Arc<RwLock<Context>>, msg: &Msg) -> Result<(), String>;
}

#[async_trait]
pub trait Rx {
    async fn rx(&self, ctx: Arc<RwLock<Context>>, msg: &Msg) -> Result<(), String>;
}
