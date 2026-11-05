//! The baseline of the accessibility tree (task-183, docs/accessibility.md section 1.1): what a
//! screen reader gets from the panel, with the menu bar shown and no song loaded. The numbers
//! started milestone m-7: each task that lands changes some of them and updates this test
//! (never deletes it), so the improvement is on record.
//!
//! Measured headless (egui 0.37, panel size, menu shown). The real window's numbers differ a
//! little (UI Automation, docs/accessibility.md 6.3).
//!
//! | Task | Nodes | Named | Unknown role | Valued | Focusable |
//! |---|---|---|---|---|---|
//! | task-183 (main at 4073a58) | 261 | 4 | 250 | 0 | 254 |
//! | task-186 (names, roles, values) | 267 | 260 | 0 | 108 | 248 |
//!
//! Task-186: the 244 controls of `layout.tsv` named, with roles; 12 display labels added (not
//! focusable); the six click areas of the drum machines (two step strips, four instrument
//! selection areas) no longer focusable, so no longer in the tree. Valued: 86 knobs and faders,
//! 10 switches, 12 displays (a label's text is its value in UI Automation).

#[path = "common/a11y.rs"]
mod a11y;

use std::collections::BTreeMap;

use a11y::AppTree;
use eframe::egui::accesskit::{Action, Role};

const BASELINE_NODES: usize = 267;
const BASELINE_NAMED: usize = 260;
const BASELINE_UNKNOWN_ROLE: usize = 0;
const BASELINE_VALUED: usize = 108;
const BASELINE_FOCUSABLE: usize = 248;
const BASELINE_NAMES: [&str; 8] = [
    "File",
    "Mods",
    "Options",
    "Help",
    "Cutoff",
    "Play",
    "Channel 3 on",
    "Tempo 140 BPM",
];

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
    let focusable = t
        .nodes()
        .filter(|(_, n)| a11y::supports(n, Action::Focus))
        .count();
    println!(
        "nodes {} named {} valued {valued} focusable {focusable}",
        t.len(),
        named.len()
    );
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
    assert_eq!(valued, BASELINE_VALUED, "nodes with a value");
    assert_eq!(
        focusable, BASELINE_FOCUSABLE,
        "nodes that can take the focus"
    );
}
