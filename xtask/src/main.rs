//! `cargo xtask bundle orb-plugin [--release | --profile P] [--target T]`: the plugin's
//! OpenRB-338.vst3 and OpenRB-338.clap under `target/bundled/` (task-197; `docs/plugin.md`
//! section 7).
//!
//! nice-plug-xtask's own entry point looks for the workspace root as the topmost ancestor
//! directory that has a `Cargo.toml`, which is the wrong one in a checkout inside another (the
//! git worktrees under `.claude/worktrees/`); this one starts at this crate's parent and calls
//! its `build` and `bundle` from there.

use std::path::{Path, PathBuf};

use nice_plug_xtask::Result;

const USAGE: &str = "usage: cargo xtask bundle <package> [--no-build] [cargo build options]";

fn main() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is in the workspace")
        .to_path_buf();
    std::env::set_current_dir(&root)?;
    let mut args = std::env::args().skip(1);
    let usage = || std::io::Error::other(USAGE);
    if args.next().as_deref() != Some("bundle") {
        Err(usage())?;
    }
    let package = args.next().ok_or_else(usage)?;
    let mut rest: Vec<String> = args.collect();
    // `--no-build`: the library is already built (scripts/build.sh builds it with cargo-xwin or
    // in Docker with the app); only lay out the bundles.
    let no_build = rest.iter().any(|a| a == "--no-build");
    rest.retain(|a| a != "--no-build");
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"));
    if !no_build {
        nice_plug_xtask::build(std::slice::from_ref(&package), &rest)?;
    }
    nice_plug_xtask::bundle(&target_dir, &package, &rest, false)
}
