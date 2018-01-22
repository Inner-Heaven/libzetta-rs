
#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("stdout.pest");

#[derive(Parser)]
#[grammar = "parsers/stdout.pest"] // relative to src
pub struct StdoutParser;

#[cfg(test)]
mod test {
    use pest::Parser;
    use parsers::*;
    use zpool::Zpool;
    #[test]
    fn test_action() {
        let one_line = " action: The pool can be imported using its name or numeric identifier.\n";
        parses_to! {
            parser: StdoutParser,
            input: one_line,
            rule: Rule::action,
            tokens: [
                action(0, 72, [
                       action_msg(9,72, [action_good_msg(9,72)])
                ])
            ]
        }

        let two_lines =" action: The pool cannot be imported. Attach the missing\n        devices and try again.\n";
        parses_to! {
            parser: StdoutParser,
            input: two_lines,
            rule: Rule::action,
            tokens: [
                action(0, 88, [
                       action_msg(9,88,[action_bad_msg(9,88)])
                ])
            ]
        }
    }

    #[test]
    fn test_naked() {
        let stdout_valid_two_disks = r#"pool: naked_test
     id: 3364973538352047455
  state: ONLINE
 action: The pool can be imported using its name or numeric identifier.
 config:

        naked_test             ONLINE
          /vdevs/import/vdev0  ONLINE
          /vdevs/import/vdev1  ONLINE
          "#;

        parses_to! {
            parser: StdoutParser,
            input: stdout_valid_two_disks,
            rule: Rule::header,
            tokens: [
                header(0, 268, [
                    pool_name(0, 17, [name(6,16)]),
                    pool_id(22, 46, [digits(26,45)]),
                    state(48, 62, [state_enum(55, 61)]),
                    action(63, 134, [action_msg(71, 134)]),
                    config(135, 143),
                    pool_line(152, 182, [name(152, 162), state_enum(175, 181)]),
                    vdevs(192, 268, [
                        vdev(192,220, [
                            naked_vdev(192, 220, [
                                disk_line(192, 220, [
                                    path(192, 211),
                                    state_enum(213, 219)
                                ])
                            ])
                        ]),
                        vdev(230,258, [
                            naked_vdev(230, 258, [
                                disk_line(230, 258, [
                                    path(230, 249),
                                    state_enum(251, 257)
                                ])
                            ])
                        ])
                    ])
                ])
            ]
        }
        let mut  pairs = StdoutParser::parse(Rule::header, stdout_valid_two_disks).unwrap_or_else(|e| panic!("{}", e));
        let pair = pairs.next().unwrap();
        Zpool::from_pest_pair(pair);
        /*
    let stdout_invalid_two_disks = r#"pool: naked_test
     id: 3364973538352047455
  state: UNAVAIL
 status: One or more devices are missing from the system.
 action: The pool cannot be imported. Attach the missing
        devices and try again.
   see: http://illumos.org/msg/ZFS-8000-6X
 config:

        naked_test             UNAVAIL  missing device
          /vdevs/import/vdev0  ONLINE

        Additional devices are known to be part of this pool, though their
        exact configuration cannot be determined.
        "#;
        let pairs = StdoutParser::parse(Rule::header, stdout_invalid_two_disks).unwrap_or_else(|e| panic!("{}", e));
        for pair in pairs.clone() {
            // A pair is a combination of the rule which matched and a span of input
            println!("Rule:    {:?}", pair.as_rule());
            println!("Span:    {:?}", pair.clone().into_span());
            println!("Text:    {}", pair.clone().into_span().as_str());
            for inner_pair in pair.into_inner() {
                println!("Rule:    {:?}", inner_pair.as_rule());
                println!("Span:    {:?}", inner_pair.clone().into_span());
                println!("Text:    {}", inner_pair.clone().into_span().as_str());
            }
        }

        println!("{:#?}", pairs);*/
    }
}

