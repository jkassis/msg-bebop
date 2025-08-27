use bebop::Record;
use msg_bebop::Msg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let msg = Msg {
        body: "Hello from Rust!",
        from_id: "rust_example",
        id: "example_001",
        to_ids: vec!["user1", "user2"],
        _type: "example",
    };

    println!("Original message: {:?}", msg);

    // Serialize
    let mut bytes = Vec::new();
    msg.serialize(&mut bytes)?;
    println!("Serialized size: {} bytes", bytes.len());

    // Deserialize
    let decoded = Msg::deserialize(&bytes)?;
    println!("Decoded message: {:?}", decoded);

    assert_eq!(msg, decoded);
    println!("✅ Rust serialization test passed!");

    Ok(())
}
