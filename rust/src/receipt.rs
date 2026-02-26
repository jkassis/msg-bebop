use serde::{Deserialize, Serialize};

// Add serialization support to MessagePact and related types
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Receipt {
    pub tick_acked_first: u64,
    pub tick_acked_last: u64,
}
