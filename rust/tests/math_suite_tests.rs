// rust/tests/integration_test.rs

use courier::add;

#[test]
fn test_add_from_integration() {
    assert_eq!(add(1, 2), 3);
}

