//! The baseline of the accessibility tree (task-183, docs/accessibility.md section 1.1): what a
//! screen reader gets from the panel today, with the menu bar shown and no song loaded. The
//! numbers are the starting point of milestone m-7: each task that lands changes some of them
//! and updates this test (never deletes it), so the improvement is on record.
//!
//! Measured headless (egui 0.37, panel size, menu shown). The real window's numbers differ a
//! little (256 elements in UI Automation, docs/accessibility.md 6.3).

#[path = "common/a11y.rs"]
mod a11y;

use std::collections::BTreeMap;

use a11y::AppTree;
use eframe::egui::accesskit::Role;

const BASELINE_NODES: usize = 261;
const BASELINE_NAMED: usize = 4;
const BASELINE_UNKNOWN_ROLE: usize = 250;
const BASELINE_NAMES: [&str; 4] = ["File", "Mods", "Options", "Help"];

#[test]
fn the_tree_today() {
    let h = AppTree::new();
    let t = h.tree();

    let mut roles: BTreeMap<String, usize> = BTreeMap::new();
    for (_, n) in t.nodes() {
        *roles.entry(format!("{:?}", a11y::role(n))).or_default() += 1;
    }
    let named = t.named();
    let valued = t
        .nodes()
        .filter(|(_, n)| a11y::value(n).is_some() || a11y::numeric_value(n).is_some())
        .count();
    println!("nodes {} named {} valued {valued}", t.len(), named.len());
    println!("roles {roles:?}");
    println!("names {named:?}");

    assert_eq!(t.len(), BASELINE_NODES, "node count");
    assert_eq!(named.len(), BASELINE_NAMED, "named nodes");
    for n in BASELINE_NAMES {
        assert!(named.iter().any(|s| s == n), "{n} is named");
    }
    assert_eq!(
        t.by_role(Role::Unknown).count(),
        BASELINE_UNKNOWN_ROLE,
        "role Unknown"
    );
    assert_eq!(valued, 0, "no node has a value");
}
