//! # Msg Bebop Library
//!
//! High-performance message serialization using Bebop for Rust.
//!
//! ## Example
//!
//! ```rust
//! use msg_bebop::Msg;
//! use bebop::Record;
//!
//! let msg = Msg {
//!     body: "Hello, world!",
//!     from_id: "sender123",
//!     id: "msg456",
//!     to_ids: vec!["recipient1", "recipient2"],
//!     _type: "greeting",
//! };
//!
//! // Serialize
//! let mut bytes = Vec::new();
//! msg.serialize(&mut bytes)?;
//!
//! // Deserialize
//! let decoded_msg = Msg::deserialize(&bytes)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

// Re-export the generated Bebop structures
pub use crate::msg::*;

// Include the generated Bebop code
mod msg;

#[cfg(test)]
mod tests {
    use super::*;
    use bebop::Record;  // Import the trait to use serialize/deserialize methods

    #[test]
    fn test_msg_serialization() {
        let original = Msg {
            body: "Test message",
            from_id: "user1",
            id: "test123",
            to_ids: vec!["user2", "user3"],
            _type: "test",
        };

        // Serialize
        let mut serialized = Vec::new();
        original.serialize(&mut serialized).unwrap();
        assert!(!serialized.is_empty());

        // Deserialize
        let deserialized = Msg::deserialize(&serialized).expect("Deserialization failed");

        // Verify
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_empty_to_ids() {
        let msg = Msg {
            body: "Broadcast message",
            from_id: "system",
            id: "broadcast1",
            to_ids: vec![],
            _type: "broadcast",
        };

        let mut bytes = Vec::new();
        msg.serialize(&mut bytes).unwrap();
        let decoded = Msg::deserialize(&bytes).unwrap();
        assert_eq!(msg.to_ids.len(), 0);
        assert_eq!(decoded.to_ids.len(), 0);
    }

    #[test]
    fn test_performance_benchmark() {
        // Create some user IDs first since we need &str
        let user_ids: Vec<String> = (0..10).map(|i| format!("user{}", i)).collect();
        let user_refs: Vec<&str> = user_ids.iter().map(|s| s.as_str()).collect();
        let body = "x".repeat(1000);

        let msg = Msg {
            body: &body, // 1KB message
            from_id: "perf_test",
            id: "perf123",
            to_ids: user_refs,
            _type: "performance",
        };

        let start = std::time::Instant::now();

        // Serialize/deserialize 1000 times
        for _ in 0..1000 {
            let mut bytes = Vec::new();
            msg.serialize(&mut bytes).unwrap();
            let _decoded = Msg::deserialize(&bytes).unwrap();
        }

        let duration = start.elapsed();
        println!("1000 serialize/deserialize cycles took: {:?}", duration);

        // Should be very fast (adjust threshold as needed)
        assert!(duration.as_millis() < 100, "Performance test failed: took {:?}", duration);
    }
}
