//! Writes the REAPER render test's projects (task-244, `docs/plugin.md` section 7):
//! `crates/orb-plugin/tests/reaper/<case>.vst3.RPP`, each made from
//! `tests/reaper/template.RPP` with the case's plugin state and MIDI item
//! (`tests/reaper/cases.rs`).
//!
//! `cargo run -p orb-plugin --example reaper_fixtures`; rerun it when the plugin's state format,
//! a case or the template changes (`cargo test -p orb-plugin --test reaper` fails on a stale
//! fixture).

#[path = "../tests/reaper/cases.rs"]
mod cases;

fn main() {
    let template = cases::template();
    for c in &cases::CASES {
        for f in cases::Format::ALL {
            let path = cases::fixture_path(c, f);
            std::fs::write(&path, cases::project(c, f, &template))
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            println!("{}", path.display());
        }
    }
}
