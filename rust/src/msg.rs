use serde::{Serialize, Deserialize};

// Define the Msg structure
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Msg {
    pub body: String,       // The actual message payload
    pub from_id: String,     // Sender ID
    pub id: String,         // ID for idempotency
    pub to_ids: Vec<String>, // Array of recipient IDs
    pub type_: String,       // Type of the message
}