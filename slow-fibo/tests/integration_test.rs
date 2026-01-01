use slow_fibo::add;

#[test_log::test]
fn test_basic_math() {
    tracing::info!("Running integration test...");
    let result = add(2, 2);
    assert_eq!(result, 4);
    tracing::info!("Math still works!");
}
