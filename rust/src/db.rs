use crate::context::Context;
use std::sync::{Arc, RwLock};

pub type SeqPos = String;

const DBTX_KEY: &str = "dbtx";

pub fn dbtx_to_ctx(ctx: &mut Context, dbtx: Arc<dyn DBTx>) -> Arc<RwLock<Context>> {
    ctx.with_value(DBTX_KEY.to_string(), dbtx)
}

pub fn dbtx_from_ctx(ctx: &Context) -> Option<Arc<dyn DBTx>> {
    ctx.value(DBTX_KEY)
        .and_then(|option_box| option_box.downcast_ref::<Arc<dyn DBTx>>().cloned())
}
pub trait DBTx: Send + Sync {
    fn obj_put(&self, key: &str, val: Vec<u8>) -> Result<(), String>; // Store object as a byte array
    fn obj_get(&self, key: &str) -> Result<Option<Vec<u8>>, String>; // Retrieve object as a byte array
    fn obj_del(&self, key: &str) -> Result<Option<Vec<u8>>, String>; // Delete an object by its key (true if deleted, false if didn't exist)
    fn tail_push(&self, prefix: &str, val: Vec<u8>) -> Result<SeqPos, String>; // Add to the tail and return the position ID
    fn tail_pop(&self, prefix: &str) -> Result<Option<(SeqPos, Vec<u8>)>, String>; // Remove from the tail and return the position ID and object data
    fn head_push(&self, prefix: &str, val: Vec<u8>) -> Result<SeqPos, String>; // Add to the head and return the position ID
    fn head_pop(&self, prefix: &str) -> Result<Option<(SeqPos, Vec<u8>)>, String>; // Remove from the head and return the position ID and object data
    fn seq_get(&self, prefix: &str) -> Result<Box<dyn Iterator<Item = String> + Send>, String>; // Get a sequence of items
    fn commit(&self) -> Result<(), String>; // Commit the transaction
    fn cancel(&self) -> Result<(), String>; // Cancel the transaction
}

pub trait DB {
    fn dbtx_create(&self) -> Result<Arc<dyn DBTx>, String>; // Return an abstract DBTx type
    fn flush(&self) -> Result<(), String>; // Close the database and release resources
}
