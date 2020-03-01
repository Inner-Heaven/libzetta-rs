use libzetta::GlobalLogger;

#[test]
fn test_default_logger() {
    let logger = GlobalLogger::get();
    let pairs = logger.list();
    let expected = String::from("(zetta_version)");
    let actual = format!("{:?}", pairs);
    assert_eq!(expected, actual);
}
