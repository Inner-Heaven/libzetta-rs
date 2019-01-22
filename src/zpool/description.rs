use std::path::PathBuf;

use pest::iterators::Pair;

use parsers::Rule;
use zpool::{CreateZpoolRequest, CreateZpoolRequestBuilder, Health};
use zpool::{CreateVdevRequest, Disk};
use zpool::vdev::VdevType;

#[derive(Getters, Builder, Debug)]
pub struct Zpool {
    name: String,
    /// Only visible during import
    #[builder(default)]
    id: Option<u64>,
    health: Health,
    topology: CreateZpoolRequest,
    #[builder(default)]
    action: Option<String>,
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
                Rule::action => { zpool.action(Some(get_string_from_pair(pair))); }
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
fn get_vdev_type_from_pair(raid_line: Pair<Rule>) -> VdevType {
    let raid_name = raid_line.into_inner().next().unwrap();
    let raid_enum = raid_name.into_inner().next().unwrap();
    VdevType::from_str(raid_enum.as_str())
}

fn get_disk_from_pair(disk_line: Pair<Rule>) -> Disk {
    let path_pair = disk_line.into_inner().next().unwrap();
    let path = PathBuf::from(path_pair.as_str());
    // This sucks, but oh well
    if path.is_relative() {
        Disk::disk(path)
    } else {
        Disk::file(path)
    }
}

#[inline]
fn get_topology_from_pair(vdevs: Pair<Rule>) -> CreateZpoolRequest {
    let mut topo = CreateZpoolRequestBuilder::default();
    for vdev in vdevs.into_inner() {
        match vdev.as_rule() {
            Rule::naked_vdev => {
                // This is very weird way to do it.
                let mut line = vdev.into_inner();
                let disk_line = line.next().unwrap();
                topo.vdev(CreateVdevRequest::SingleDisk(get_disk_from_pair(disk_line)));
            }
            Rule::raided_vdev => {
                // Raid type is always first pair
                let mut disks = Vec::with_capacity(5);
                let mut raidtype = None;
                for inner_pair in vdev.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::raid_line => {
                            raidtype = Some(get_vdev_type_from_pair(inner_pair));
                        },
                        Rule::disk_line => {
                            let disk = get_disk_from_pair(inner_pair);
                            disks.push(disk);
                        },
                        _ => unreachable!()
                    }
                }
                match raidtype {
                    Some(VdevType::Mirror) => topo.vdev(CreateVdevRequest::Mirror(disks)),
                    Some(VdevType::RaidZ) => topo.vdev(CreateVdevRequest::RaidZ(disks)),
                    Some(VdevType::RaidZ2) => topo.vdev(CreateVdevRequest::RaidZ2(disks)),
                    Some(VdevType::RaidZ3) => topo.vdev(CreateVdevRequest::RaidZ3(disks)),
                    _ => unreachable!()
                };
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