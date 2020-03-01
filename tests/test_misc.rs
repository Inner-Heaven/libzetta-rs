use libzetta::GlobalLogger;
use slog::{o, Drain, Logger};
use slog_stdlog::StdLog;

#[test]
fn test_not_default_logger() {
    let root = Logger::root(StdLog.fuse(), o!("wat" => "wat"));
    GlobalLogger::setup(&root).unwrap();
    let pairs = GlobalLogger::get().list();
    let expected = String::from("(zetta_version, wat)");
    let actual = format!("{:?}", pairs);
    assert_eq!(expected, actual);
}
