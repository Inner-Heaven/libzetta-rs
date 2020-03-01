use libzetta::Logger;
use slog::{o, Drain, Logger as SlogLogger, OwnedKVList};
use slog_stdlog::StdLog;

#[test]
fn test_default_logger() {
    let logger = Logger::global();
    let pairs = logger.list();
    let expected = String::from("()");
    let actual = format!("{:?}", OwnedKVList::from(pairs.clone()));
    assert_eq!(expected, actual);
}
#[test]
fn test_not_default_logger() {
    let root = SlogLogger::root(StdLog.fuse(), o!("wat" => "wat"));
    Logger::setup(root).unwrap();

    let pairs = Logger::global().list();
    let expected = String::from("(wat)");
    let actual = format!("{:?}", OwnedKVList::from(pairs.clone()));
    assert_eq!(expected, actual);
}
