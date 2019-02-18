use std::path::PathBuf;
use std::str::FromStr;

use pest::iterators::Pair;
use pest::iterators::Pairs;

use parsers::Rule;
use zpool::{CreateZpoolRequest, Disk, Health};
use zpool::vdev::{ErrorStatistics, Vdev, VdevType};

/// Reason why zpool in this state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reason {
    /// Not yet classified reason.
    Other(String),
}

#[derive(Getters, Builder, Debug, Eq, PartialEq)]
pub struct Zpool {
    /// Name of the pool
    name: String,
    /// UID of the pool. Only visible during import
    #[builder(default)]
    id: Option<u64>,
    /// Current Health status of the pool.
    health: Health,
    /// List of VDEVs
    vdevs: Vec<Vdev>,
    /// List of cache devices.
    #[builder(default)]
    caches: Vec<Disk>,
    /// Zfs Intent Log device
    #[builder(default)]
    zil: Option<Vdev>,
    /// Value of action field what ever it is.
    #[builder(default)]
    action: Option<String>,
    /// Errors?
    #[builder(default)]
    errors: Option<String>,
    /// Reason why this Zpool is not healthy.
    #[builder(default)]
    reason: Option<Reason>,
    /// Error statistics
    #[builder(default)]
    error_statistics: ErrorStatistics,
}

impl Zpool {
    pub fn from_pest_pair(pair: Pair<Rule>) -> Zpool {
        debug_assert!(pair.as_rule() == Rule::zpool);
        let pairs = pair.into_inner();
        let mut zpool = ZpoolBuilder::default();
        for pair in pairs {
            match pair.as_rule() {
                Rule::pool_name => {
                    zpool.name(get_string_from_pair(pair));
                }
                Rule::pool_id => {
                    zpool.id(Some(get_u64_from_pair(pair)));
                }
                Rule::state => {
                    zpool.health(get_health_from_pair(pair));
                }
                Rule::action => {
                    zpool.action(Some(get_string_from_pair(pair)));
                }
                Rule::errors => {
                    zpool.errors(get_error_from_pair(pair));
                }
                Rule::vdevs => {
                    zpool.vdevs(get_vdevs_from_pair(pair));
                }
                Rule::pool_line => {
                    set_stats_and_reason_from_pool_line(pair, &mut zpool);
                }
                Rule::config | Rule::status | Rule::see | Rule::pool_headers => {}
                Rule::scan_line => {}
                _ => {
                    dbg!(pair);
                    unreachable!()
                }
            }
        }
        zpool.build().unwrap()
    }
}

impl PartialEq<CreateZpoolRequest> for Zpool {
    fn eq(&self, other: &CreateZpoolRequest) -> bool {
        let the_same_zpool = match (&self.zil, other.zil()) {
            (None, None) => true,
            (Some(ref a), Some(ref b)) => a == b,
            _ => false,
        };
        the_same_zpool
            && &self.name == other.name()
            && &self.caches == other.caches()
            && &self.vdevs == other.vdevs()
    }
}

impl PartialEq<Zpool> for CreateZpoolRequest {
    fn eq(&self, other: &Zpool) -> bool {
        other == self
    }
}

#[inline]
fn get_error_statistics_from_pair(pair: Pair<Rule>) -> ErrorStatistics {
    debug_assert_eq!(Rule::error_statistics, pair.as_rule());
    let mut inner = pair.into_inner();
    ErrorStatistics {
        read: inner.next().unwrap().into_span().as_str().parse().unwrap(),
        write: inner.next().unwrap().into_span().as_str().parse().unwrap(),
        checksum: inner.next().unwrap().into_span().as_str().parse().unwrap(),
    }
}

#[inline]
fn set_stats_and_reason_from_pool_line(pool_line: Pair<Rule>, zpool: &mut ZpoolBuilder) {
    debug_assert_eq!(pool_line.as_rule(), Rule::pool_line);

    for pair in pool_line.into_inner() {
        match pair.as_rule() {
            Rule::reason => {
                zpool.reason(Some(Reason::Other(String::from(pair.into_span().as_str()))));
            }
            Rule::error_statistics => {
                zpool.error_statistics(get_error_statistics_from_pair(pair));
            }
            _ => {}
        };
    }
}

#[inline]
fn get_vdev_type(raid_name: Pair<Rule>) -> VdevType {
    let raid_enum = raid_name
        .into_inner()
        .next()
        .expect("Failed to parse raid_enum");
    debug_assert!(raid_enum.as_rule() == Rule::raid_enum);
    VdevType::from_str(raid_enum.as_str()).expect("Failed to parse raid type")
}

#[inline]
fn get_path_from_path(path: Option<Pair<Rule>>) -> PathBuf {
    let path = path.expect("Missing path from disk line");
    debug_assert!(path.as_rule() == Rule::path);
    PathBuf::from(path.into_span().as_str())
}

#[inline]
fn get_health_from_health(health: Option<Pair<Rule>>) -> Health {
    let health = health.expect("Missing health from disk line");
    debug_assert!(health.as_rule() == Rule::state_enum);
    Health::try_from_str(Some(health.into_span().as_str())).expect("Failed to parse Health")
}

#[inline]
fn get_disk_from_disk_line(disk_line: Pair<Rule>) -> Disk {
    debug_assert!(disk_line.as_rule() == Rule::disk_line);

    let mut inner = disk_line.into_inner();

    let path = get_path_from_path(inner.next());
    let health = get_health_from_health(inner.next());

    let (error_statics, reason) = get_stats_and_reason_from_pairs(inner);
    Disk::builder()
        .path(path)
        .health(health)
        .error_statistics(error_statics)
        .reason(reason)
        .build()
        .expect("Failed to build disk")
}

#[inline]
fn get_stats_and_reason_from_pairs(pairs: Pairs<Rule>) -> (ErrorStatistics, Option<Reason>) {
    let mut stats = None;
    let mut reason = None;
    for pair in pairs {
        match pair.as_rule() {
            Rule::error_statistics => stats = Some(get_error_statistics_from_pair(pair)),
            Rule::reason => reason = Some(Reason::Other(String::from(pair.into_span().as_str()))),
            _ => {
                unreachable!();
            }
        }
    }
    (stats.unwrap_or_default(), reason)
}

#[inline]
fn get_vdevs_from_pair(pair: Pair<Rule>) -> Vec<Vdev> {
    debug_assert!(pair.as_rule() == Rule::vdevs);

    pair.into_inner()
        .map(|vdev| match vdev.as_rule() {
            Rule::naked_vdev => {
                let disk_line = vdev.into_inner().next().unwrap();

                let disk = get_disk_from_disk_line(disk_line);

                Vdev::builder()
                    .kind(VdevType::SingleDisk)
                    .health(disk.health().clone())
                    .reason(None)
                    .disks(vec![disk])
                    .build()
                    .expect("Failed to build Vdev")
            }
            Rule::raided_vdev => {
                let mut inner = vdev.into_inner();
                let raid_line = inner.next().unwrap();
                debug_assert!(raid_line.as_rule() == Rule::raid_line);
                let mut raid_line = raid_line.into_inner();
                let raid_name = raid_line.next().unwrap();

                let health = get_health_from_health(raid_line.next());

                let (error_statics, reason) = get_stats_and_reason_from_pairs(raid_line);

                Vdev::builder()
                    .kind(get_vdev_type(raid_name))
                    .health(health)
                    .disks(inner.map(get_disk_from_disk_line).collect())
                    .error_statistics(error_statics)
                    .reason(reason)
                    .build()
                    .expect("Failed to build vdev")
            }
            _ => {
                unreachable!();
            }
        })
        .collect()
}

#[inline]
fn get_health_from_pair(pair: Pair<Rule>) -> Health {
    let health = get_string_from_pair(pair);
    Health::try_from_str(Some(&health)).expect("Failed to unwrap health")
}

#[inline]
fn get_u64_from_pair(pair: Pair<Rule>) -> u64 {
    get_value_from_pair(pair)
        .as_str()
        .parse()
        .expect("Failed to unwrap u64")
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
        _ => Some(String::from(error_pair.as_str())),
    }
}
