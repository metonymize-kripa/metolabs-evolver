use slow_fibo::{add, fibonacci};

#[test]
fn test_fibonacci_integration() {
    // Test that fibonacci function works correctly
    assert_eq!(fibonacci(10), 55);
    assert_eq!(fibonacci(8), 21);
}

#[test]
fn test_add_integration() {
    assert_eq!(add(5, 7), 12);
}
