//! The plugin's egui has its fonts (task-201): egui 0.37 has no default features, and the
//! plugin links no eframe, whose defaults give the standalone app `default_fonts`. A test or
//! an example of this crate cannot see the gap (its dev-dependency eframe turns the feature on
//! for them), so this asks cargo for the features of the plugin's normal dependencies, the
//! ones the bundles are built with.

use std::path::Path;
use std::process::Command;

#[test]
fn the_plugins_normal_build_has_egui_default_fonts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let out = Command::new(env!("CARGO"))
        .args(["tree", "--offline", "--manifest-path"])
        .arg(&root)
        .args(["-p", "orb-plugin", "-e", "normal,features", "-i", "egui"])
        .output()
        .expect("running cargo tree");
    assert!(
        out.status.success(),
        "cargo tree failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let tree = String::from_utf8_lossy(&out.stdout);
    assert!(
        tree.contains("egui feature \"default_fonts\""),
        "egui's default_fonts is off in the plugin's normal build:\n{tree}"
    );
}

#[test]
fn this_build_has_fonts() {
    assert!(orb_plugin::editor::has_fonts());
}
