use zpool::{Health, Topology, TopologyBuilder};
use ::parsers::Rule;
use std::path::PathBuf;
use zpool::{Vdev};

use pest::iterators::Pair;

#[derive(Getters, Builder)]
pub struct Zpool {
    name: String,
    id: u64,
    health: Health,
    topology: Topology
}

impl Zpool {
    pub fn from_pest_pair(pair: Pair<Rule>) -> Zpool {
        debug_assert!(pair.as_rule() == Rule::header);
        let pairs = pair.into_inner();
        let mut zpool = ZpoolBuilder::default();
        for pair in pairs {
            match pair.as_rule() {
                Rule::pool_name => { zpool.name(get_string_from_pair(pair));},
                Rule::pool_id => { zpool.id(get_u64_from_pair(pair));},
                Rule::state => { zpool.health(get_health_from_pair(pair));},
                Rule::vdevs => { zpool.topology(get_topology_from_pair(pair));},
                Rule::config | Rule::action | Rule::pool_line  | Rule::status | Rule::see => {},
                _ => unreachable!()
            }
        }

        zpool.build().unwrap()
    }
}

#[inline]
fn get_topology_from_pair(pair: Pair<Rule>) -> Topology {
    let mut topo = TopologyBuilder::default();
    let vdevs = pair.into_inner().next().unwrap().into_inner();
    for vdev in vdevs {
        match vdev.as_rule() {
            Rule::naked_vdev => {
                let mut line = vdev.into_inner();
                let path = PathBuf::from(line.next().unwrap().as_str());
                topo.vdev(Vdev::disk(path));
            },
            Rule::raided_vdev => { unimplemented!(); },
            _ => unreachable!()
        }
    }
    topo.build().unwrap()
}
#[inline]
fn get_health_from_pair(pair: Pair<Rule>) -> Health {
    let health = get_string_from_pair(pair);
    Health::try_from_str(Some(&health)).unwrap()
}

#[inline]
fn get_u64_from_pair(pair: Pair<Rule>) -> u64 {
    get_value_from_pair(pair).as_str().parse().unwrap()
}

#[inline]
fn get_string_from_pair(pair: Pair<Rule>) -> String {
    String::from(get_value_from_pair(pair).as_str())
}
#[inline]
fn get_value_from_pair(pair: Pair<Rule>) -> Pair<Rule> {
    let mut pairs = pair.into_inner();
    pairs.next().unwrap()
}
