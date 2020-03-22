use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parsers/zfs.pest"] // relative to src
pub struct ZfsParser;

#[cfg(test)]
mod test {
    use super::{Rule, ZfsParser};
    use pest::{consumes_to, parses_to, Parser};

    #[test]
    fn test_parse_filesystem_name_root() {
        let line = "z";

        parses_to! {
            parser: ZfsParser,
            input: line,
            rule: Rule::dataset_name,
            tokens: [
                dataset_name(0,1)
            ]
        }

        let pairs = ZfsParser::parse(Rule::dataset_name, line).unwrap();
        assert_eq!("z", pairs.as_str());
    }
    #[test]
    fn test_parse_filesystem_name_nested() {
        let line = "z/foo/bar";

        parses_to! {
            parser: ZfsParser,
            input: line,
            rule: Rule::dataset_name,
            tokens: [
                dataset_name(0,9)
            ]
        }

        let pairs = ZfsParser::parse(Rule::dataset_name, line).unwrap();
        assert_eq!("z/foo/bar", pairs.as_str());
    }
    #[test]
    fn test_parse_filesystem_name_root_snapshot() {
        let line = "z@backup-20190707";

        parses_to! {
            parser: ZfsParser,
            input: line,
            rule: Rule::dataset_name,
            tokens: [
                dataset_name(0,17)
            ]
        }

        let pairs = ZfsParser::parse(Rule::dataset_name, line).unwrap();
        assert_eq!("z@backup-20190707", pairs.as_str());
    }
    #[test]
    fn test_parse_filesystem_name_nexted_snapshot() {
        let line = "z/foo/bar@backup-20190707";

        parses_to! {
            parser: ZfsParser,
            input: line,
            rule: Rule::dataset_name,
            tokens: [
                dataset_name(0,25)
            ]
        }

        let pairs = ZfsParser::parse(Rule::dataset_name, line).unwrap();
        assert_eq!("z/foo/bar@backup-20190707", pairs.as_str());
    }

    #[test]
    fn test_parse_dataset_not_found() {
        let line = "cannot open 's/asd/asd': dataset does not exist";
        let mut pairs = ZfsParser::parse(Rule::error, line).unwrap();
        let dataset_not_found_pair = pairs.next().unwrap().into_inner().next().unwrap();
        assert_eq!(Rule::dataset_not_found, dataset_not_found_pair.as_rule());
        let dataset_name_pair = dataset_not_found_pair.into_inner().next().unwrap();
        assert_eq!("s/asd/asd", dataset_name_pair.as_str());
    }

    #[test]
    fn test_parse_datasets() {
        let lines = "s\ns/s/s/s\ns/d@test";
        let expected = ["s", "s/s/s/s", "s/d@test"];

        let mut pairs = ZfsParser::parse(Rule::datasets, lines).unwrap();
        let datasets_pairs = pairs.next().unwrap().into_inner();
        assert_eq!(3, datasets_pairs.clone().count());

        for (idx, pair) in datasets_pairs.enumerate() {
            assert_eq!(Rule::dataset_name, pair.as_rule());
            assert_eq!(expected[idx], pair.as_str());
        }
    }

    #[test]
    fn test_parse_datasets_with_type() {
        let lines = r#"volume  z/iohyve/rancher/disk0
filesystem      z/var/mail
snapshot        z/var/mail@backup-2019-08-08
bookmark        z/var/mail#backup-2019-08-08
        "#;
        let expected = vec![
            ("volume", "z/iohyve/rancher/disk0"),
            ("filesystem", "z/var/mail"),
            ("snapshot", "z/var/mail@backup-2019-08-08"),
            ("bookmark", "z/var/mail#backup-2019-08-08"),
        ];

        let mut pairs = ZfsParser::parse(Rule::datasets_with_type, lines).unwrap();
        let datasets_pairs = pairs.next().unwrap().into_inner();
        assert_eq!(4, datasets_pairs.clone().count());

        for (idx, pair) in datasets_pairs.enumerate() {
            assert_eq!(Rule::dataset_with_type, pair.as_rule());
            let mut dataset_with_type_pair = pair.into_inner();
            let dataset_type = dataset_with_type_pair.next().unwrap();
            let dataset_name = dataset_with_type_pair.next().unwrap();
            assert_eq!(expected[idx], (dataset_type.as_str(), dataset_name.as_str()));
        }
    }

    #[test]
    fn test_issue_126() {
        let lines = r#"z/ROOT
z/ROOT/default
z/docker
z/iohyve
z/iohyve/Firmware
z/iohyve/ISO
z/iohyve/ISO/rancheros-v1.4.0.iso
z/iohyve/rancher
z/portshaker
z/portshaker/cache
z/portshaker/cache/andoriyu-local
z/portshaker/cache/freebsd-svn
z/portshaker/cache/vscode-git
z/poudriere
z/poudriere/data
z/poudriere/data/.m
z/poudriere/data/cache
z/poudriere/data/logs
z/poudriere/data/packages
z/poudriere/data/wrkdirs
z/poudriere/jails
z/poudriere/jails/live
z/poudriere/ports
z/poudriere/ports/dev
z/poudriere/ports/prestine
z/poudriere/ports/system
z/tmp
z/usr
z/usr/home
z/usr/ports
z/usr/ports/distfiles
z/usr/src
z/var
z/var/ccache
z/var/crash
z/var/log
z/var/mail
z/var/tmp
        "#;


        let mut pairs = ZfsParser::parse(Rule::datasets, lines).unwrap();
        let datasets_pairs = pairs.next().unwrap().into_inner();
        assert_eq!(38, datasets_pairs.clone().count());

    }
}
