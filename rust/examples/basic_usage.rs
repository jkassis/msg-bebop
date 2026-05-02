fn main() -> Result<(), Box<dyn std::error::Error>> {
    use trx::trx::msg::Msg;

    let msg = Msg::new(
        "example_001",
        "rust_example",
        vec!["user1".to_string(), "user2".to_string()],
        "example",
        b"Hello from Rust!".to_vec(),
    );

    let json = serde_json::to_string(&msg)?;
    let decoded: Msg = serde_json::from_str(&json)?;

    assert_eq!(msg, decoded);
    println!("Serialized trx msg: {json}");
    println!("Decoded body bytes: {:?}", decoded.body);
    println!("✅ Rust trx serialization test passed!");

    Ok(())
}
