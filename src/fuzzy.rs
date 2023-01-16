use crate::{
    parsers::{Rule, StdoutParser},
    pest::Parser,
    zpool::Zpool,
};

pub fn fuzzy_target_1(data: &[u8]) {
    if let Ok(s) = std::str::from_utf8(data) {
        let _: Vec<_> = StdoutParser::parse(Rule::zpools, s)
            .map(|pairs| pairs.map(Zpool::from_pest_pair).collect())
            .unwrap();
    }
}
