use parsers::Rule;
use std::path::PathBuf;
use zpool::Vdev;
use zpool::{Health, Topology, TopologyBuilder};

use pest::iterators::Pair;

#[derive(Getters, Builder, Debug)]
pub struct Zpool {
    name: String,
    /// Only visible during import
    #[builder(default)]
    id: Option<u64>,
    health: Health,
    topology: Topology,
    action: String,
    #[builder(default)]
    errors: Option<String>,
}

impl Zpool {
    pub fn from_pest_pair(pair: Pair<Rule>) -> Zpool {
        debug_assert!(pair.as_rule() == Rule::zpool);
        let pairs = pair.into_inner();
        let mut zpool = ZpoolBuilder::default();
        for pair in pairs {
            match pair.as_rule() {
                Rule::pool_name => { zpool.name(get_string_from_pair(pair)); }
                Rule::pool_id => { zpool.id(Some(get_u64_from_pair(pair))); }
                Rule::state => { zpool.health(get_health_from_pair(pair)); }
                Rule::vdevs => { zpool.topology(get_topology_from_pair(pair)); }
                Rule::action => { zpool.action(get_string_from_pair(pair)); }
                Rule::errors => { zpool.errors(get_error_from_pair(pair)); }
                Rule::config | Rule::pool_line | Rule::status | Rule::see | Rule::pool_headers => {}
                Rule::scan_line => {}
                _ => unreachable!(),
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
                // This is very weird way to do it.
                let mut line = vdev.into_inner();
                let disk_line = line.next().unwrap();
                let path_pair = disk_line.into_inner().next().unwrap();
                let path = PathBuf::from(path_pair.as_str());
                topo.vdev(Vdev::disk(path));
            }
            Rule::raided_vdev => {
                unimplemented!();
            }
            _ => unreachable!(),
        }
    }
    topo.build().unwrap()
}

#[inline]
fn get_health_from_pair(pair: Pair<Rule>) -> Health {
    let health = get_string_from_pair(pair);
    Health::try_from_str(Some(&health)).expect("Failed to unwrap health")
}

#[inline]
fn get_u64_from_pair(pair: Pair<Rule>) -> u64 {
    get_value_from_pair(pair).as_str().parse().expect("Failed to unwrap u64")
}

#[inline]
fn get_string_from_pair(pair: Pair<Rule>) -> String {
    String::from(get_value_from_pair(pair).as_str())
}

#[inline]
fn get_value_from_pair(pair: Pair<Rule>) -> Pair<Rule> {
    let mut pairs = pair.into_inner();
    pairs.next().expect("Failed to unwrap value")
}

#[inline]
fn get_error_from_pair(pair: Pair<Rule>) -> Option<String> {
    let mut pairs = pair.into_inner();
    let error_pair = pairs.next().expect("Failed to unwrap error");
    match error_pair.as_rule() {
        Rule::no_errors => None,
        _ => Some(String::from(error_pair.as_str()))
    }
}