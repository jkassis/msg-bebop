use serde::{Deserialize, Serialize};

// Add serialization support to MessagePact and related types
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Pact {
    pub tick_of_last_attempt: u64,
    pub try_count: u32,
}
