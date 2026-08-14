use cart::lockfile::{CartLock, LockedPackage, LockedSource};
use cart::resolver::{ResolvedGraph, ResolvedPackage};

fn make_package(name: &str, version: &str, checksum: &str) -> ResolvedPackage {
    ResolvedPackage {
        name: name.to_string(),
        version: version.to_string(),
        source: LockedSource::Path {
            dir: format!("~/.carts/{name}"),
        },
        checksum: checksum.to_string(),
    }
}

#[test]
fn lockfile_roundtrip() {
    let mut lock = CartLock::new();
    lock.package.push(LockedPackage {
        name: "std".to_string(),
        version: "0.1.0".to_string(),
        source: LockedSource::Git {
            url: "https://github.com/op/std".to_string(),
            sha: "abc123".to_string(),
        },
        checksum: "deadbeef".to_string(),
    });
    let text = lock.to_toml().expect("serialize");
    let reparsed = CartLock::from_toml(&text).expect("reparse");
    assert_eq!(reparsed.version, 1);
    assert_eq!(reparsed.package.len(), 1);
    assert_eq!(reparsed.package[0].name, "std");
}

#[test]
fn lockfile_is_fresh_when_matching() {
    let mut graph = ResolvedGraph::default();
    graph.packages.push(make_package("std", "0.1.0", "abc"));

    let mut lock = CartLock::new();
    lock.update_from_graph(&graph);

    assert!(lock.is_fresh(&graph));
}

#[test]
fn lockfile_is_stale_when_version_differs() {
    let mut graph = ResolvedGraph::default();
    graph.packages.push(make_package("std", "0.1.0", "abc"));

    let mut lock = CartLock::new();
    lock.package.push(LockedPackage {
        name: "std".to_string(),
        version: "0.2.0".to_string(),
        source: LockedSource::Path {
            dir: "~/.carts/std".to_string(),
        },
        checksum: "abc".to_string(),
    });

    assert!(!lock.is_fresh(&graph));
}

#[test]
fn lockfile_is_stale_when_package_missing() {
    let mut graph = ResolvedGraph::default();
    graph.packages.push(make_package("std", "0.1.0", "abc"));

    let lock = CartLock::new();

    assert!(!lock.is_fresh(&graph));
}

#[test]
fn lockfile_is_stale_when_checksum_differs() {
    let mut graph = ResolvedGraph::default();
    graph.packages.push(make_package("std", "0.1.0", "abc"));

    let mut lock = CartLock::new();
    lock.package.push(LockedPackage {
        name: "std".to_string(),
        version: "0.1.0".to_string(),
        source: LockedSource::Path {
            dir: "~/.carts/std".to_string(),
        },
        checksum: "different".to_string(),
    });

    assert!(!lock.is_fresh(&graph));
}

#[test]
fn lockfile_update_from_graph() {
    let mut graph = ResolvedGraph::default();
    graph.packages.push(make_package("std", "0.1.0", "abc"));
    graph
        .packages
        .push(make_package("nes-bank", "1.0.0", "def"));

    let mut lock = CartLock::new();
    lock.update_from_graph(&graph);

    assert_eq!(lock.package.len(), 2);
    assert_eq!(lock.package[0].name, "std");
    assert_eq!(lock.package[1].name, "nes-bank");
}
